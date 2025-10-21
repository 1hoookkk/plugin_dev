#include "FieldProcessor.h"
#include "ui/FieldWaveformUI.h"

#include <algorithm>
#include <cstring>
#include <cmath>

FieldProcessor::FieldProcessor()
    : juce::AudioProcessor (BusesProperties().withInput("Input", juce::AudioChannelSet::stereo(), true)
                                               .withOutput("Output", juce::AudioChannelSet::stereo(), true))
{
    // Lock DSP shapes to Vowel pair by default (constant, set once)
    zf_.setShapePair(emu::VOWEL_A, emu::VOWEL_B);
    zf_.setIntensity(kIntensity);
    zf_.setDrive(kDrive);
    zf_.setSectionSaturation(kSat);

    // Cache parameter pointers (avoid repeated APVTS lookups on audio thread)
    characterParam_ = apvts_.getRawParameterValue(enginefield::params::characterId);
    mixParam_ = apvts_.getRawParameterValue(enginefield::params::mixId);
    gainParam_ = apvts_.getRawParameterValue(enginefield::params::gainId);
    bypassParam_ = apvts_.getRawParameterValue(enginefield::params::bypassId);
    effectModeParam_ = apvts_.getRawParameterValue(enginefield::params::effectModeId);

    // Initialize smoothing
    bypassSmooth_.reset(48000.0, 0.01);
}

void FieldProcessor::prepareToPlay(double sampleRate, int samplesPerBlock)
{
    juce::ignoreUnused(samplesPerBlock);
    juce::dsp::ProcessSpec spec{ sampleRate, static_cast<juce::uint32>(samplesPerBlock), static_cast<juce::uint32>(getTotalNumOutputChannels()) };
    outGain_.prepare(spec);
    outGain_.setRampDurationSeconds(0.02);  // 20ms smoothing for gain changes

    dryBuffer_.setSize(getTotalNumOutputChannels(), samplesPerBlock, false, false, true);

    zf_.prepare(sampleRate, samplesPerBlock);
    zf_.setSectionSaturation(kSat);

    env_.prepare(sampleRate);
    env_.setAttackMs(0.489f);
    env_.setReleaseMs(80.0f);
    env_.setDepth(0.75f);  // v1.0.1: Reduced from 0.945 for balanced modulation (±15% vs ±18.9%)

    // Initialize UI envelope follower
    const float sr = static_cast<float>(sampleRate);
    constexpr float attackTime = 0.010f;   // 10 ms
    constexpr float releaseTime = 0.150f;  // 150 ms

    // smoothing coefficients (use 1 - exp(-1/(tau*sr)) so update is: state += coef * (input - state))
    uiEnvelopeAttackCoef_  = 1.0f - std::expf(-1.0f / (attackTime * sr));
    uiEnvelopeReleaseCoef_ = 1.0f - std::expf(-1.0f / (releaseTime * sr));

    // Reset smoothers with correct sample rate and initialize to current bypass state
    bypassSmooth_.reset(sampleRate, 0.01);
    const bool currentBypass = bypassParam_->load() > 0.5f;
    bypassSmooth_.setCurrentAndTargetValue(currentBypass ? 0.0f : 1.0f);

    uiWaveformFifo_.reset();
    uiWaveformRingBuffer_.assign(kWaveformDepth, 0.0f);
}

void FieldProcessor::releaseResources()
{
}

bool FieldProcessor::isBusesLayoutSupported(const BusesLayout& layouts) const
{
    const auto& in  = layouts.getChannelSet(true, 0);
    const auto& out = layouts.getChannelSet(false, 0);
    return in == juce::AudioChannelSet::stereo() && out == juce::AudioChannelSet::stereo();
}

void FieldProcessor::processBlock (juce::AudioBuffer<float>& buffer, juce::MidiBuffer& midi)
{
    juce::ignoreUnused(midi);
    juce::ScopedNoDenormals noDenormals;

    const auto numSamples = buffer.getNumSamples();
    const auto numCh      = buffer.getNumChannels();

    // Test tone (off by default)
    const bool testTone = apvts_.getRawParameterValue(enginefield::params::testToneId)->load() > 0.5f;
    if (testTone)
    {
        const double fs = getSampleRate();
        const double inc = 440.0 * juce::MathConstants<double>::twoPi / fs;
        for (int ch = 0; ch < numCh; ++ch)
        {
            auto* data = buffer.getWritePointer(ch);
            double p = testTonePhase_;
            for (int i = 0; i < numSamples; ++i)
            {
                data[i] = static_cast<float>(std::sin(p)) * 0.05f;
                p += inc;
                if (p >= juce::MathConstants<double>::twoPi)
                    p -= juce::MathConstants<double>::twoPi;
            }
            testTonePhase_ = p;
        }
    }

    // Pre-copy for dry/wet
    for (int ch = 0; ch < numCh; ++ch)
        dryBuffer_.copyFrom(ch, 0, buffer, ch, 0, numSamples);

    // Parameter reads (use cached pointers, avoid repeated APVTS lookups)
    const auto character = characterParam_->load();
    const auto mixPct    = mixParam_->load();
    const auto outDb     = gainParam_->load();
    const bool bypass    = bypassParam_->load() > 0.5f;
    const bool effectOn  = effectModeParam_->load() > 0.5f;

    // Targets (mix smoothing happens inside ZPlaneFilter)
    const float mixTarget = juce::jlimit(0.0f, 1.0f, mixPct * 0.01f);
    bypassSmooth_.setTargetValue(bypass ? 0.0f : 1.0f); // 1 = active, 0 = bypassed

    // Envelope follower on left channel (authentic)
    float envValue = 0.0f;
    {
        const float* L = buffer.getReadPointer(0);
        for (int i = 0; i < numSamples; ++i) envValue = env_.process(L[i]);
    }

    // Modulate morph by envelope depth (20% scale as per spec example)
    const float baseMorph = character * 0.01f;
    const float modulatedMorph = juce::jlimit(0.0f, 1.0f, baseMorph + envValue * 0.2f);

    // EFFECT mode: SOLO THE WET SIGNAL (100% wet) so you can hear what the Engine is doing
    // Off = normal (respects MIX knob target)
    // On  = 100% wet (ignores MIX, pure filtered signal)
    const float effectiveMix = effectOn ? 1.0f : mixTarget;

    // Update filter parameters (skip constant setters - already set in constructor)
    zf_.setMorph(modulatedMorph);
        zf_.setMix(effectiveMix);

    // Update coefficients once per block (expensive)
    zf_.updateCoeffsBlock(numSamples);

    // Copy pole data to UI atomics (lock-free, cheap)
    const auto& poles = zf_.getLastPoles();
    for (size_t i = 0; i < 6; ++i)
    {
        uiPoles_[i * 2].store(poles[i].r, std::memory_order_relaxed);
        uiPoles_[i * 2 + 1].store(poles[i].theta, std::memory_order_relaxed);
    }

    // Process (cheap)
    float* L = buffer.getWritePointer(0);
    float* R = buffer.getWritePointer(1);
    zf_.process(L, R, numSamples);

    // Compute wet/dry block peaks (pre-bypass mix) for delta visualization
    // Cache channel pointers once before loops
    const float* wetL = buffer.getReadPointer(0);
    const float* wetR = numCh > 1 ? buffer.getReadPointer(1) : wetL;
    const float* dryL = dryBuffer_.getReadPointer(0);
    const float* dryR = numCh > 1 ? dryBuffer_.getReadPointer(1) : dryL;

    float wetBlockMax = 0.0f;
    float dryBlockMax = 0.0f;
    for (int i = 0; i < numSamples; ++i)
    {
        wetBlockMax = std::max(wetBlockMax, std::max(std::abs(wetL[i]), std::abs(wetR[i])));
        dryBlockMax = std::max(dryBlockMax, std::max(std::abs(dryL[i]), std::abs(dryR[i])));
    }

    // Update waveform circular buffer using smoothed delta (wet - dry)
    {
        const float delta = std::max(0.0f, wetBlockMax - dryBlockMax);
        const float coef = (delta > deltaEnvelopeState) ? uiEnvelopeAttackCoef_ : uiEnvelopeReleaseCoef_;
        deltaEnvelopeState += coef * (delta - deltaEnvelopeState);

        int idx = waveformIndex_.load(std::memory_order_relaxed);
        waveformPeaks_[idx].store(deltaEnvelopeState, std::memory_order_relaxed);
        waveformIndex_.store((idx + 1) % NUM_WAVEFORM_BARS, std::memory_order_relaxed);
    }

    // Bypass crossfade (per-sample smoothing to avoid zipper noise)
    // Cache write pointers once (already have read pointers from above)
    float* wetLW = buffer.getWritePointer(0);
    float* wetRW = numCh > 1 ? buffer.getWritePointer(1) : wetLW;

    for (int i = 0; i < numSamples; ++i)
    {
        const float bypassAmt = bypassSmooth_.getNextValue();
        wetLW[i] = wetLW[i] * bypassAmt + dryL[i] * (1.0f - bypassAmt);
        if (numCh > 1)
            wetRW[i] = wetRW[i] * bypassAmt + dryR[i] * (1.0f - bypassAmt);
    }

    // Output gain (juce::dsp::Gain smooths internally via setRampDurationSeconds)
    const float gainLinear = juce::Decibels::decibelsToGain(outDb);
    outGain_.setGainLinear(gainLinear);
    juce::dsp::AudioBlock<float> blk (buffer);
    outGain_.process(juce::dsp::ProcessContextReplacing<float>(blk));

    // --- UI: compute overall output level AFTER all DSP (for meters) ---
    // Use already-cached pointers (wetLW/wetRW now contain final output)
    {
        float blockMax = 0.0f;
        for (int i = 0; i < numSamples; ++i)
        {
            blockMax = std::max(blockMax, std::max(std::abs(wetLW[i]), std::abs(wetRW[i])));
        }

        const float coeff = (blockMax > uiEnvelopeState_) ? uiEnvelopeAttackCoef_ : uiEnvelopeReleaseCoef_;
        uiEnvelopeState_ += coeff * (blockMax - uiEnvelopeState_);
        uiCurrentLevel_.store(uiEnvelopeState_, std::memory_order_relaxed);
    }

    // Push waveform data to UI ring buffer (reuse already-computed envelope)
    {
        // Write to SPSC FIFO for waveform visualization
        int start1 = 0, size1 = 0, start2 = 0, size2 = 0;
        uiWaveformFifo_.prepareToWrite(1, start1, size1, start2, size2);
        if (size1 > 0) uiWaveformRingBuffer_[start1] = uiEnvelopeState_;
        if (size2 > 0) uiWaveformRingBuffer_[start2] = uiEnvelopeState_;
        const int wrote = size1 + size2;
        if (wrote > 0) uiWaveformFifo_.finishedWrite(wrote);
    }

}

juce::AudioProcessorEditor* FieldProcessor::createEditor() { return new engine::ui::FieldWaveformEditor(*this, apvts_); }

void FieldProcessor::getStateInformation (juce::MemoryBlock& destData)
{
    auto state = apvts_.copyState();
    std::unique_ptr<juce::XmlElement> xml (state.createXml());
    copyXmlToBinary(*xml, destData);
}

void FieldProcessor::setStateInformation (const void* data, int sizeInBytes)
{
    std::unique_ptr<juce::XmlElement> xml (getXmlFromBinary(data, sizeInBytes));
    if (xml.get() != nullptr)
        if (xml->hasTagName(apvts_.state.getType()))
            apvts_.replaceState(juce::ValueTree::fromXml(*xml));
}

// RT-safe reader for the editor (no allocations)
int FieldProcessor::getWaveformSamples(float* destBuffer, int maxSamples) noexcept
{
    if (maxSamples <= 0 || destBuffer == nullptr)
        return 0;

    const int available = uiWaveformFifo_.getNumReady();
    const int toRead = std::min(available, maxSamples);
    if (toRead == 0)
        return 0;

    int start1 = 0, size1 = 0, start2 = 0, size2 = 0;
    uiWaveformFifo_.prepareToRead(toRead, start1, size1, start2, size2);

    if (size1 > 0)
        std::memcpy(destBuffer, uiWaveformRingBuffer_.data() + start1, sizeof(float) * size1);
    if (size2 > 0)
        std::memcpy(destBuffer + size1, uiWaveformRingBuffer_.data() + start2, sizeof(float) * size2);

    uiWaveformFifo_.finishedRead(size1 + size2);
    return (size1 + size2);
}
