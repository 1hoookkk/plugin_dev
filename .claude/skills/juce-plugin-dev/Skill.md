---
name: JUCE Audio Plugin Development
description: Real-time safe JUCE 8+ plugin patterns for VST3/Standalone, covering RT-safe DSP, lock-free UI communication, CMake integration, and production-ready workflows
version: 1.0.0
dependencies: JUCE 8.0.10+, CMake 3.24+, C++20
---

# JUCE Audio Plugin Development Skill

## Overview
This Skill provides production-ready patterns for building JUCE 8+ audio plugins (VST3/Standalone) with real-time safety, efficient DSP, and robust build systems. Apply these patterns when creating audio effects, synthesizers, or processors that must run without glitches in professional DAW environments.

**When to use this Skill:**
- Creating new JUCE-based audio plugins
- Refactoring existing plugins for RT safety
- Setting up CMake build systems for JUCE projects
- Implementing lock-free audio↔UI communication
- Optimizing DSP for real-time performance

## Core Principles

### 1. Real-Time Safety (Non-Negotiable)
**Rule:** ZERO allocations, locks, or blocking operations in `processBlock()`

**Why:** Audio thread runs at high priority. Any blocking causes audible glitches (dropouts, clicks, pops).

**Patterns:**

```cpp
// ✅ CORRECT: Cache parameter pointers in constructor
class MyProcessor : public AudioProcessor {
    std::atomic<float>* mixParam_ = nullptr;

    MyProcessor() {
        mixParam_ = apvts_.getRawParameterValue("mix");
    }

    void processBlock(AudioBuffer<float>& buffer, MidiBuffer&) override {
        ScopedNoDenormals noDenormals;  // Prevent CPU slowdown

        const float mix = mixParam_->load();  // Atomic read, RT-safe

        // Process audio...
    }
};

// ❌ WRONG: Allocations in processBlock
void processBlock(AudioBuffer<float>& buffer, MidiBuffer&) override {
    std::vector<float> temp(buffer.getNumSamples());  // ❌ Allocation!
    auto mix = apvts_.getParameter("mix")->getValue();  // ❌ Tree traversal!
}
```

**Checklist:**
- [ ] Use `ScopedNoDenormals` at start of `processBlock()`
- [ ] Pre-allocate all buffers in constructor/`prepareToPlay()`
- [ ] Cache APVTS parameter pointers (don't traverse tree)
- [ ] Use atomics for audio↔UI communication (no mutexes)
- [ ] Update expensive coefficients once per block, not per sample

---

### 2. Per-Block Coefficient Caching
**Pattern:** Separate expensive setup from cheap per-sample processing

```cpp
class ZPlaneFilter {
    // Expensive: bilinear transform, pole-to-biquad conversion
    void updateCoeffsBlock() {
        // Called ONCE per audio block
        for (int i = 0; i < 6; ++i) {
            auto pole = interpolatePole(shapeA[i], shapeB[i], morph);
            pole = remapPole48kToFs(pole, sampleRate);  // Bilinear transform
            biquads_[i].setCoefficients(poleToCoeffs(pole));
        }
    }

    // Cheap: just filtering + gain
    void process(float* L, float* R, int numSamples) {
        // Called for EACH sample
        for (int n = 0; n < numSamples; ++n) {
            float sample = L[n];
            for (auto& biquad : biquads_)
                sample = biquad.processSample(sample);
            L[n] = sample;
        }
    }
};

// In processBlock:
void processBlock(AudioBuffer<float>& buffer, MidiBuffer&) override {
    filter_.updateCoeffsBlock();  // Once
    filter_.process(buffer.getWritePointer(0),
                    buffer.getWritePointer(1),
                    buffer.getNumSamples());  // Per-sample loop
}
```

**When to use:**
- IIR filters with modulating coefficients
- Complex DSP with state-dependent calculations
- Any operation involving transcendental functions (sin, log, exp)

---

### 3. Lock-Free Audio↔UI Communication
**Pattern:** Atomics for simple values, FIFOs for streams

```cpp
// Processor.h
class MyProcessor : public AudioProcessor {
public:
    // UI reads these (RT-safe, no blocking)
    float getCurrentLevel() const {
        return currentLevel_.load(std::memory_order_relaxed);
    }

    void getWaveformPeaks(std::array<std::atomic<float>, 60>& out) {
        for (size_t i = 0; i < 60; ++i)
            out[i].store(waveformPeaks_[i].load());
    }

private:
    // Audio thread writes
    std::atomic<float> currentLevel_{0.0f};
    std::array<std::atomic<float>, 60> waveformPeaks_;

    void processBlock(AudioBuffer<float>& buffer, MidiBuffer&) override {
        auto level = buffer.getRMSLevel(0, 0, buffer.getNumSamples());
        currentLevel_.store(level, std::memory_order_relaxed);

        // Update waveform peaks...
    }
};

// Editor.cpp (UI thread)
void MyEditor::timerCallback() {
    float level = processor_.getCurrentLevel();  // No blocking!
    levelMeter_.setLevel(level);

    processor_.getWaveformPeaks(waveformData_);
    waveformDisplay_.repaint();
}
```

**Key Rules:**
- Audio thread: **Write** to atomics (store)
- UI thread: **Read** from atomics (load)
- Use `std::memory_order_relaxed` for performance (ordering not critical)
- Never use mutexes/locks in `processBlock()`

**For larger data (FFT, spectrograms):**
```cpp
juce::AbstractFifo fifo_{32768};  // Lock-free ring buffer
```

---

### 4. APVTS (AudioProcessorValueTreeState) Best Practices

```cpp
// parameters.h
namespace params {
    constexpr auto characterId = "character";
    constexpr auto mixId = "mix";
    constexpr auto outputId = "output";
}

AudioProcessorValueTreeState::ParameterLayout createParameterLayout() {
    std::vector<std::unique_ptr<RangedAudioParameter>> params;

    params.push_back(std::make_unique<AudioParameterFloat>(
        ParameterID{params::characterId, 1},
        "CHARACTER",
        NormalisableRange<float>(0.0f, 100.0f, 0.1f),
        50.0f,  // Default
        AudioParameterFloatAttributes()
            .withLabel("%")
            .withStringFromValueFunction([](float v, int) {
                return String(v, 1) + "%";
            })
    ));

    return {params.begin(), params.end()};
}

// Processor constructor
MyProcessor()
    : apvts_(*this, nullptr, "PARAMETERS", createParameterLayout())
{
    // Cache pointers for RT-safe access
    characterParam_ = apvts_.getRawParameterValue(params::characterId);
}
```

**UI Attachments:**
```cpp
// Editor constructor
MyEditor(MyProcessor& p)
    : AudioProcessorEditor(p)
{
    characterSlider_.setSliderStyle(Slider::LinearHorizontal);
    addAndMakeVisible(characterSlider_);

    // Automatic sync between slider and parameter
    characterAttachment_ = std::make_unique<SliderAttachment>(
        p.apvts_, params::characterId, characterSlider_
    );
}

private:
    Slider characterSlider_;
    std::unique_ptr<AudioProcessorValueTreeState::SliderAttachment> characterAttachment_;
```

---

### 5. CMake + JUCE Integration

**Root CMakeLists.txt:**
```cmake
cmake_minimum_required(VERSION 3.24)
project(MyPlugin VERSION 1.0.0 LANGUAGES C CXX)

set(CMAKE_CXX_STANDARD 20)
set(CMAKE_CXX_STANDARD_REQUIRED YES)

# Find JUCE (assumes JUCE_SOURCE_DIR set or in system paths)
find_package(JUCE CONFIG REQUIRED)

add_subdirectory(plugins/MyPlugin)
```

**Plugin CMakeLists.txt:**
```cmake
juce_add_plugin(MyPlugin
    COMPANY_NAME "YourCompany"
    PLUGIN_MANUFACTURER_CODE Yoco
    PLUGIN_CODE MyPl
    FORMATS VST3 Standalone
    PRODUCT_NAME "My Plugin"
    IS_SYNTH FALSE
    NEEDS_MIDI_INPUT FALSE
    EDITOR_WANTS_KEYBOARD_FOCUS FALSE
)

target_sources(MyPlugin PRIVATE
    Source/PluginProcessor.cpp
    Source/PluginProcessor.h
    Source/PluginEditor.cpp
    Source/PluginEditor.h
)

target_compile_definitions(MyPlugin PUBLIC
    JUCE_VST3_CAN_REPLACE_VST2=0
    JUCE_WEB_BROWSER=0
    JUCE_USE_CURL=0
)

target_link_libraries(MyPlugin PRIVATE
    juce::juce_audio_processors
    juce::juce_dsp
)

# Warnings as errors (strict build)
if(MSVC)
    target_compile_options(MyPlugin PRIVATE /W4 /WX)
else()
    target_compile_options(MyPlugin PRIVATE -Wall -Wextra -Werror)
endif()
```

**Build Commands:**
```bash
# Configure
cmake -S . -B build -DCMAKE_BUILD_TYPE=Release -DJUCE_SOURCE_DIR="C:/JUCE"

# Build
cmake --build build --config Release

# Run pluginval (validation)
pluginval --validate "build/MyPlugin_artefacts/Release/VST3/MyPlugin.vst3"
```

---

### 6. DSP Best Practices

#### Equal-Power Crossfade (Dry/Wet Mixing)
```cpp
// ❌ WRONG: Linear mix (loudness dip at 50%)
float out = wet * mix + dry * (1.0f - mix);

// ✅ CORRECT: Equal-power (constant perceived loudness)
float wetGain = std::sqrt(mix);
float dryGain = std::sqrt(1.0f - mix);
float out = wet * wetGain + dry * dryGain;
```

#### Bilinear Transform for Sample Rate Conversion
```cpp
// Convert pole from 48kHz reference to target sample rate
PolePair remapPole48kToFs(PolePair p48k, double targetFs) {
    constexpr double refFs = 48000.0;

    // Fast path: skip math if already at reference rate
    if (std::abs(targetFs - refFs) < 0.1)
        return p48k;

    // Inverse bilinear: digital (48k) → analog
    double omega48k = p48k.theta;
    double omegaAnalog = (2.0 * refFs) * std::tan(omega48k / 2.0);

    // Forward bilinear: analog → digital (target)
    double omegaNew = 2.0 * std::atan(omegaAnalog / (2.0 * targetFs));

    return {p48k.radius, omegaNew};
}
```

#### Geodesic Pole Interpolation (Log-Space)
```cpp
// ❌ WRONG: Linear radius (sounds digital)
float r = rA * (1.0f - t) + rB * t;

// ✅ CORRECT: Geodesic (log-space, smoother morphing)
float logRA = std::log(rA);
float logRB = std::log(rB);
float r = std::exp(logRA * (1.0f - t) + logRB * t);

// Angle: shortest path (wrap at 2π)
float thetaDiff = thetaB - thetaA;
if (thetaDiff > M_PI) thetaDiff -= 2.0f * M_PI;
if (thetaDiff < -M_PI) thetaDiff += 2.0f * M_PI;
float theta = thetaA + thetaDiff * t;
```

---

### 7. UI Patterns

#### Custom Look and Feel
```cpp
class MyLookAndFeel : public juce::LookAndFeel_V4 {
public:
    void drawRotarySlider(Graphics& g, int x, int y, int width, int height,
                          float sliderPos, float rotaryStartAngle,
                          float rotaryEndAngle, Slider& slider) override {
        auto bounds = Rectangle<int>(x, y, width, height).toFloat();
        auto radius = jmin(bounds.getWidth(), bounds.getHeight()) / 2.0f;
        auto centreX = bounds.getCentreX();
        auto centreY = bounds.getCentreY();
        auto angle = rotaryStartAngle + sliderPos * (rotaryEndAngle - rotaryStartAngle);

        // Draw track
        g.setColour(Colours::darkgrey);
        g.fillEllipse(bounds.reduced(10.0f));

        // Draw indicator
        Path p;
        p.startNewSubPath(centreX, centreY);
        p.lineTo(centreX + radius * std::cos(angle - MathConstants<float>::halfPi),
                 centreY + radius * std::sin(angle - MathConstants<float>::halfPi));
        g.setColour(Colours::yellow);
        g.strokePath(p, PathStrokeType(3.0f));
    }
};
```

#### Efficient Repaint (Timer-Based)
```cpp
class WaveformDisplay : public Component, private Timer {
public:
    WaveformDisplay() {
        startTimerHz(30);  // 30 FPS (not 60, saves CPU)
    }

private:
    void timerCallback() override {
        // Only repaint if data changed
        if (processor_.hasNewWaveformData()) {
            processor_.getWaveformPeaks(waveformData_);
            repaint();
        }
    }

    void paint(Graphics& g) override {
        // Draw waveform from cached data
        for (size_t i = 0; i < waveformData_.size(); ++i) {
            float barHeight = waveformData_[i].load() * getHeight();
            g.fillRect(i * barWidth, getHeight() - barHeight, barWidth, barHeight);
        }
    }
};
```

---

## Common Pitfalls & Solutions

### Pitfall 1: Smoothing Parameters Without LinearSmoothedValue
**Problem:** Direct parameter changes cause zipper noise

```cpp
// ❌ WRONG: Instant parameter changes
void processBlock(AudioBuffer<float>& buffer, MidiBuffer&) {
    float mix = mixParam_->load();  // Sudden jumps!
    // Apply mix...
}

// ✅ CORRECT: Smoothed parameter changes
class MyProcessor {
    juce::LinearSmoothedValue<float> mixSmooth_;

    void prepareToPlay(double sampleRate, int samplesPerBlock) override {
        mixSmooth_.reset(sampleRate, 0.05);  // 50ms ramp
    }

    void processBlock(AudioBuffer<float>& buffer, MidiBuffer&) {
        mixSmooth_.setTargetValue(mixParam_->load());

        for (int i = 0; i < buffer.getNumSamples(); ++i) {
            float mix = mixSmooth_.getNextValue();  // Smooth!
            // Apply mix...
        }
    }
};
```

### Pitfall 2: Not Handling Sample Rate Changes
**Problem:** Coefficients become invalid when DAW changes sample rate

```cpp
void prepareToPlay(double sampleRate, int samplesPerBlock) override {
    sampleRate_ = sampleRate;

    // Recalculate ALL sample-rate-dependent coefficients
    filter_.setSampleRate(sampleRate);
    envelopeFollower_.setAttackMs(5.0, sampleRate);
    envelopeFollower_.setReleaseMs(100.0, sampleRate);

    // Resize buffers if needed
    dryBuffer_.setSize(2, samplesPerBlock, false, true, true);
}
```

### Pitfall 3: Unsafe Dynamic Casts in Editor
**Problem:** `getAudioProcessor()` returns generic `AudioProcessor*`

```cpp
// ❌ RISKY: dynamic_cast can fail
MyEditor::MyEditor(AudioProcessor& p)
    : AudioProcessorEditor(p)
{
    auto* myProc = dynamic_cast<MyProcessor*>(&p);  // Could be nullptr!
}

// ✅ SAFE: Pass typed reference
class MyProcessor : public AudioProcessor {
    AudioProcessorEditor* createEditor() override {
        return new MyEditor(*this);  // Pass MyProcessor&
    }
};

MyEditor::MyEditor(MyProcessor& p)  // Typed reference
    : AudioProcessorEditor(p), processor_(p) {}
```

---

## Verification Checklist

Before releasing a plugin, verify:

**RT Safety:**
- [ ] Run with JUCE's `JUCE_CHECK_MEMORY_LEAKS=1` (no allocations in processBlock)
- [ ] Use address sanitizer: `-fsanitize=address` (detect memory errors)
- [ ] Profile with DAW's performance meter (< 5% CPU target)

**Validation:**
- [ ] Pass pluginval: `pluginval --validate plugin.vst3`
- [ ] Test in multiple DAWs (Reaper, Ableton, FL Studio)
- [ ] Test at different sample rates (44.1k, 48k, 96k, 192k)
- [ ] Test with different buffer sizes (32, 64, 512, 2048 samples)

**State Management:**
- [ ] Save/recall state correctly (APVTS handles this automatically)
- [ ] Presets load without glitches
- [ ] No crashes when changing presets mid-playback

**Cross-Platform:**
- [ ] Build on Windows (MSVC), macOS (Clang), Linux (GCC)
- [ ] Test on different CPU architectures (x86, ARM if applicable)

---

## Reference Implementation

See `plugins/EngineField/` for a complete production example:
- **ZPlaneFilter.h** - RT-safe 12th-order IIR with bilinear warping
- **FieldProcessor.cpp** - Lock-free audio↔UI, APVTS integration
- **FieldWaveformUI.cpp** - Efficient 30Hz timer-based visualization
- **CMakeLists.txt** - JUCE 8 integration with strict warnings

---

## When NOT to Use This Skill

- Building non-audio applications (use general C++ patterns)
- Prototyping (RT safety can be relaxed for experiments)
- Offline processing tools (no real-time constraints)
- Non-JUCE frameworks (these patterns are JUCE-specific)

---

## Additional Resources

- JUCE Docs: https://docs.juce.com/
- JUCE Forum: https://forum.juce.com/
- Pluginval: https://github.com/Tracktion/pluginval
- Engine:Field reference: `plugins/EngineField/` (this codebase)
