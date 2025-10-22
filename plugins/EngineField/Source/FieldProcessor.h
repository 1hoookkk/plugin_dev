#pragma once

#include <juce_audio_processors/juce_audio_processors.h>
#include <juce_audio_basics/juce_audio_basics.h>

#include "parameters.h"
#include "dsp/ZPlaneFilter.h"
#include "dsp/EMUAuthenticTables.h"
#include "dsp/EnvelopeFollower.h"
#include "ui/VisualisationConfig.h"

#include <atomic>
#include <vector>

class FieldProcessor : public juce::AudioProcessor
{
public:
    FieldProcessor();
    ~FieldProcessor() override = default;

    //==============================================================================
    const juce::String getName() const override { return "EngineField"; }
    bool acceptsMidi() const override { return false; }
    bool producesMidi() const override { return false; }
    double getTailLengthSeconds() const override { return 0.0; }

    //==============================================================================
    void prepareToPlay(double sampleRate, int samplesPerBlockExpected) override;
    void releaseResources() override;
    bool isBusesLayoutSupported(const BusesLayout& layouts) const override;
    void processBlock(juce::AudioBuffer<float>&, juce::MidiBuffer&) override;

    //==============================================================================
    juce::AudioProcessorEditor* createEditor() override;
    bool hasEditor() const override { return true; }

    //==============================================================================
    int getNumPrograms() override { return 1; }
    int getCurrentProgram() override { return 0; }
    void setCurrentProgram (int) override {}
    const juce::String getProgramName (int) override { return {}; }
    void changeProgramName (int, const juce::String&) override {}

    //==============================================================================
    void getStateInformation (juce::MemoryBlock& destData) override;
    void setStateInformation (const void* data, int sizeInBytes) override;

    juce::AudioProcessorValueTreeState& getAPVTS() noexcept { return apvts_; }

    // UI pole data (lock-free: audio thread writes, UI reads)
    const std::array<std::atomic<float>, 12>& getUIPoles() const noexcept { return uiPoles_; }

    // Audio level for UI visualization (lock-free: audio thread writes, UI reads)
    float getAudioLevel() const noexcept { return currentLevel.load(); }

    // Waveform visualization constants
    static constexpr int NUM_WAVEFORM_BARS = engine::viz::kWaveformBarCount;

    // Waveform peaks for UI visualization (lock-free: audio thread writes, UI reads)
    void getWaveformPeaks(std::array<float, NUM_WAVEFORM_BARS>& dest) const noexcept
    {
        int currentIndex = waveformIndex_.load(std::memory_order_relaxed);
        for (int i = 0; i < NUM_WAVEFORM_BARS; ++i)
        {
            // Read from circular buffer in display order (oldest to newest)
            int readIndex = (currentIndex + i + 1) % NUM_WAVEFORM_BARS;
            dest[i] = waveformPeaks_[readIndex].load(std::memory_order_relaxed);
        }
    }

    // Visualiser FIFO (single-producer: audio thread, single-consumer: GUI)

    // UI access (RT-safe)
    static constexpr int kWaveformDepth = 512; // tuneable
    int getWaveformSamples(float* destBuffer, int maxSamples) noexcept;
    float getCurrentLevel() const noexcept { return uiCurrentLevel_.load(std::memory_order_relaxed); }

private:
    // Parameters
    juce::AudioProcessorValueTreeState apvts_{
        *this, nullptr, "PARAMS", enginefield::params::createLayout()
    };

    // DSP
    emu::ZPlaneFilter zf_;
    emu::EnvelopeFollower env_;
    juce::dsp::Gain<float> outGain_;

    // Fixed authentic values (locked)
    static constexpr float kIntensity = emu::AUTHENTIC_INTENSITY;
    static constexpr float kDrive     = emu::AUTHENTIC_DRIVE;
    static constexpr float kSat       = emu::AUTHENTIC_SATURATION;

    // Smoothing (gain smoothing handled by juce::dsp::Gain internally)
    juce::LinearSmoothedValue<float> bypassSmooth_;
    juce::UndoManager undo_;

    // Cached parameter pointers (avoid repeated atomic lookups in processBlock)
    std::atomic<float>* characterParam_ = nullptr;
    std::atomic<float>* mixParam_ = nullptr;
    std::atomic<float>* gainParam_ = nullptr;
    std::atomic<float>* bypassParam_ = nullptr;
    std::atomic<float>* effectModeParam_ = nullptr;
    std::atomic<float>* testToneParam_ = nullptr;  // Patch 1: avoid APVTS tree traversal

    // Dry buffer (pre-allocated)
    juce::AudioBuffer<float> dryBuffer_;

    // Visualiser ring buffer (mono)

    // UI pole data (6 poles × 2 values = 12 floats: r0,theta0, r1,theta1, ...)
    std::array<std::atomic<float>, 12> uiPoles_{};

    // UI audio level (envelope follower for pad visualization)
    std::atomic<float> currentLevel { 0.0f };
    float envelopeAttack = 0.0f;
    float envelopeRelease = 0.0f;
    float envelopeState = 0.0f;
    float deltaEnvelopeState = 0.0f; // smoothed delta (wet - dry) for waveform bars

    // Waveform visualization circular buffer (60 bars for UI)
    std::array<std::atomic<float>, NUM_WAVEFORM_BARS> waveformPeaks_{};
    std::atomic<int> waveformIndex_ { 0 };

    // Test tone phase (instance member, not static - thread-safe)
    double testTonePhase_ { 0.0 };

    // Lock-free SPSC for waveform peaks (producer: audio thread, consumer: UI thread)
    juce::AbstractFifo uiWaveformFifo_{ kWaveformDepth };
    std::vector<float> uiWaveformRingBuffer_{};

    // level envelope state (audio thread) and atomic snapshot for UI
    float uiEnvelopeState_ = 0.0f;
    float uiEnvelopeAttackCoef_ = 0.0f;
    float uiEnvelopeReleaseCoef_ = 0.0f;
    std::atomic<float> uiCurrentLevel_{ 0.0f };

    JUCE_DECLARE_NON_COPYABLE_WITH_LEAK_DETECTOR(FieldProcessor)
};
