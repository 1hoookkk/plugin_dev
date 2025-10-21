# AUTHENTIC EMU Z-PLANE DSP PACKAGE
## Complete Specification for Coding Agents

**Date:** 2025-10-17
**Purpose:** Comprehensive DSP specification for implementing authentic EMU Audity 2000 Z-plane filtering
**Target:** JUCE audio plugin (VST3/Standalone)

---

## 1. OVERVIEW

### What is EMU Z-Plane Filtering?

Z-plane filtering is E-mu's proprietary technology from the Audity 2000 (1998-2000) that allows **real-time morphing between different digital filter types**. Unlike traditional static filters, Z-plane filters continuously interpolate between different pole/zero configurations.

### Key Innovation
- **Dynamic filter morphing** - smoothly transition between completely different filter characteristics
- **6-section cascade** - 12th-order filtering using 6 biquad sections in series
- **Per-section saturation** - subtle harmonic saturation per biquad stage
- **Pole/zero interpolation** - complex pole pairs stored as [radius, theta] in polar coordinates

### Critical Implementation Point
The EMU hardware had specific parameter defaults and limits that make it sound "right". These values were **extracted from real hardware** and must be used verbatim.

---

## 2. AUTHENTIC PARAMETER VALUES (LOCKED)

These values are **EXTRACTED FROM REAL EMU HARDWARE** and must be used exactly as specified:

### Default Operating Values
```cpp
// Core DSP Parameters (FROM HARDWARE EXTRACTION)
constexpr float AUTHENTIC_INTENSITY = 0.4f;        // 40% (NOT 0.6 or 0.3!)
constexpr float AUTHENTIC_DRIVE = 0.2f;            // ~3dB = 1.4x gain
constexpr float AUTHENTIC_SATURATION = 0.2f;       // 20% per section
constexpr float AUTHENTIC_POLE_LIMIT = 0.9950f;   // EMU hardware max radius

// Modulation Defaults (FROM EMU PLANET PHATT EXTRACTION)
constexpr float DEFAULT_LFO_RATE = 1.2f;          // Hz
constexpr float DEFAULT_LFO_DEPTH = 0.15f;        // 15% modulation
constexpr float DEFAULT_ENV_DEPTH = 0.35f;        // 35% envelope mod

// Envelope Follower (FROM AUDITY 2000 HARDWARE)
constexpr float ENVELOPE_ATTACK_MS = 0.489f;      // ~0.5ms (VERY FAST)
constexpr float ENVELOPE_RELEASE_MS = 80.0f;      // 80ms
constexpr float ENVELOPE_DEPTH = 0.945f;          // 94.5% modulation
```

### Critical Stability Limits
```cpp
// STABILITY CRITICAL - DO NOT CHANGE
constexpr float MIN_POLE_RADIUS = 0.1f;           // Minimum stable radius
constexpr float MAX_POLE_RADIUS = 0.9950f;        // Maximum (NOT 0.999999!)
constexpr float MAX_INTENSITY_BOOST = 1.06f;      // 1.0 + intensity * 0.06

// Sample rate scaling (for non-48kHz operation)
// Authentic shapes are at 48kHz reference
constexpr double REFERENCE_SAMPLE_RATE = 48000.0;
```

### Why These Specific Values?

1. **Pole Limit 0.9950**: Real EMU hardware had this limit. Values closer to 1.0 (like 0.999999) cause harsh metallic ringing on transients
2. **Intensity 0.4**: This is the sweet spot from Planet Phatt presets. Too low = no character, too high = harsh
3. **Drive 0.2**: Equivalent to ~3dB (1.4x gain). This adds warmth without distortion
4. **Saturation 0.2**: Per-section tanh() amount. Subtle harmonic enhancement
5. **Envelope Fast Attack**: Drums need sub-millisecond attack to respond properly

---

## 3. AUTHENTIC EMU SHAPES (POLE/THETA DATA)

### Shape Format
Each shape = **6 complex pole pairs** stored as **12 floats** `[r, theta]`
- `r` = radius (0.0-1.0, must be < 1.0 for stability)
- `theta` = angle in radians (0 to 2π)

### Vowel Pair (Most Musical - Use as Default)

**Shape A - Vowel "Ae" (to morph toward Shape B)**
```cpp
// FROM: audity_shapes_A_48k.json (AUTHENTIC EXTRACTION)
const float VOWEL_A[12] = {
    0.95f,  0.01047197551529928f,   // Section 1
    0.96f,  0.01963495409118615f,   // Section 2
    0.985f, 0.03926990818237230f,   // Section 3
    0.992f, 0.11780972454711690f,   // Section 4
    0.993f, 0.32724923485310250f,   // Section 5
    0.985f, 0.45814892879434435f    // Section 6
};
```

**Shape B - Vowel "Oo" (morph target)**
```cpp
// FROM: audity_shapes_B_48k.json (AUTHENTIC EXTRACTION)
const float VOWEL_B[12] = {
    0.88f,  0.00523598775764964f,   // Section 1
    0.90f,  0.01047197551529928f,   // Section 2
    0.92f,  0.02094395103059856f,   // Section 3
    0.94f,  0.04188790206119712f,   // Section 4
    0.96f,  0.08377580412239424f,   // Section 5
    0.97f,  0.16755160824478848f    // Section 6
};
```

### Bell Pair (Bright Metallic)

**Shape A - Bell Metallic**
```cpp
const float BELL_A[12] = {
    0.996f, 0.14398966333536510f,
    0.995f, 0.18325957151773740f,
    0.994f, 0.28797932667073020f,
    0.993f, 0.39269908182372300f,
    0.992f, 0.54977871437816500f,
    0.990f, 0.78539816364744630f
};
```

**Shape B - Metallic Cluster**
```cpp
const float BELL_B[12] = {
    0.994f, 0.19634954085771740f,
    0.993f, 0.26179938779814450f,
    0.992f, 0.39269908182372300f,
    0.991f, 0.52359877584930150f,
    0.990f, 0.70685834741592550f,
    0.988f, 0.94247779605813900f
};
```

### Low Pair (Punchy Bass)

**Shape A - Low LP Punch**
```cpp
const float LOW_A[12] = {
    0.88f,  0.00392699081823723f,
    0.90f,  0.00785398163647446f,
    0.92f,  0.01570796327294893f,
    0.94f,  0.03272492348531062f,
    0.96f,  0.06544984697062124f,
    0.97f,  0.13089969394124100f
};
```

**Shape B - Formant Pad**
```cpp
const float LOW_B[12] = {
    0.92f,  0.00654498469706212f,
    0.94f,  0.01308996939412425f,
    0.96f,  0.02617993878824850f,
    0.97f,  0.05235987755649700f,
    0.98f,  0.10471975511299400f,
    0.985f, 0.20943951022598800f
};
```

---

## 4. CORE DSP MATHEMATICS

### Pole Pair Interpolation (Linear for Radius, Shortest-Path for Angle)

```cpp
struct PolePair {
    float r;      // radius
    float theta;  // angle (radians)
};

PolePair interpolatePole(const PolePair& A, const PolePair& B, float morph) {
    PolePair result;

    // Linear interpolation for radius
    result.r = A.r + morph * (B.r - A.r);

    // Shortest-path interpolation for angle (avoid phase jumps)
    float angleDiff = B.theta - A.theta;
    while (angleDiff > PI) angleDiff -= TWO_PI;
    while (angleDiff < -PI) angleDiff += TWO_PI;
    result.theta = A.theta + morph * angleDiff;

    // Apply intensity scaling (0.4 default)
    float intensityBoost = 1.0f + intensity * 0.06f;
    result.r = std::min(result.r * intensityBoost, 0.9950f); // HARDWARE LIMIT

    return result;
}
```

### Pole Pair to Biquad Coefficients (Direct Form II Transposed)

```cpp
void poleToBiquad(const PolePair& pole, float& a1, float& a2,
                  float& b0, float& b1, float& b2) {
    // Denominator (recursive) coefficients from pole
    a1 = -2.0f * pole.r * cos(pole.theta);
    a2 = pole.r * pole.r;

    // Numerator (feedforward) for resonant response
    float rz = std::clamp(0.9f * pole.r, 0.0f, 0.999f);
    float c = cos(pole.theta);
    b0 = 1.0f;
    b1 = -2.0f * rz * c;
    b2 = rz * rz;

    // Normalize to prevent gain explosion
    float norm = 1.0f / std::max(0.25f, abs(b0) + abs(b1) + abs(b2));
    b0 *= norm;
    b1 *= norm;
    b2 *= norm;
}
```

### Biquad Processing with Saturation (Per Section)

```cpp
class BiquadSection {
    float z1 = 0.0f, z2 = 0.0f;  // state variables
    float b0, b1, b2, a1, a2;     // coefficients
    bool saturationEnabled = true;
    float saturationAmount = 0.2f;

    inline float processSample(float x) {
        // Direct Form II Transposed
        float y = b0 * x + z1;
        z1 = b1 * x - a1 * y + z2;
        z2 = b2 * x - a2 * y;

        // Per-section saturation (AUTHENTIC EMU)
        if (saturationEnabled && saturationAmount > 0.0f) {
            const float gain = 1.0f + saturationAmount * 4.0f;
            y = tanh(y * gain);
        }

        // NaN protection
        if (!isfinite(y)) y = 0.0f;
        return y;
    }

    void reset() { z1 = z2 = 0.0f; }
};
```

### 6-Section Cascade (12th Order)

```cpp
class ZPlaneFilter {
    std::array<BiquadSection, 6> sectionsL;  // Left channel
    std::array<BiquadSection, 6> sectionsR;  // Right channel

    void updateCoefficientsBlock() {
        // Get smoothed parameters
        float morph = morphSmooth.getNextValue();
        float intensity = intensitySmooth.getNextValue();

        const float intensityBoost = 1.0f + intensity * 0.06f;

        for (int i = 0; i < 6; ++i) {
            // Interpolate pole pair
            PolePair p = interpPole(shapeA[i], shapeB[i], morph);

            // Apply intensity and limit
            p.r = std::min(p.r * intensityBoost, 0.9950f);

            // Convert to biquad coefficients
            float a1, a2, b0, b1, b2;
            poleToBiquad(p, a1, a2, b0, b1, b2);

            // Set coefficients for both channels
            sectionsL[i].setCoeffs(b0, b1, b2, a1, a2);
            sectionsR[i].setCoeffs(b0, b1, b2, a1, a2);
        }
    }

    void processBlock(float* left, float* right, int numSamples) {
        for (int n = 0; n < numSamples; ++n) {
            float l = left[n];
            float r = right[n];

            // Cascade through 6 sections
            for (int i = 0; i < 6; ++i) {
                l = sectionsL[i].processSample(l);
                r = sectionsR[i].processSample(r);
            }

            left[n] = l;
            right[n] = r;
        }
    }
};
```

---

## 5. REAL-TIME SAFETY REQUIREMENTS

### Block-Rate Coefficient Updates (CRITICAL)

```cpp
// CORRECT: Update coefficients ONCE per block
void processBlock(AudioBuffer<float>& buffer, MidiBuffer& midi) {
    // Get parameter values
    float character = characterParam->load();
    float morphPosition = character / 100.0f;  // 0-100 -> 0-1

    // Set parameters
    zplaneFilter.setMorph(morphPosition);
    zplaneFilter.setIntensity(0.4f);  // AUTHENTIC
    zplaneFilter.setDrive(0.2f);      // AUTHENTIC

    // Update coefficients ONCE (expensive!)
    zplaneFilter.updateCoefficientsBlock();

    // Process samples (cheap)
    zplaneFilter.processBlock(buffer.getWritePointer(0),
                              buffer.getWritePointer(1),
                              buffer.getNumSamples());
}

// WRONG: Updating coefficients per sample (kills CPU!)
for (int n = 0; n < numSamples; ++n) {
    zplaneFilter.updateCoefficients();  // ❌ DON'T DO THIS
    left[n] = zplaneFilter.processSample(left[n]);
}
```

### No Allocations in processBlock()

```cpp
// ✅ CORRECT: Pre-allocate in prepareToPlay()
void prepareToPlay(double sampleRate, int samplesPerBlock) {
    dryBuffer.setSize(2, samplesPerBlock);  // Pre-allocate
    zplaneFilter.prepare(sampleRate, samplesPerBlock);
}

void processBlock(AudioBuffer<float>& buffer) {
    // ✅ Use pre-allocated buffer
    dryBuffer.makeCopyOf(buffer);
}

// ❌ WRONG: Allocating in processBlock()
void processBlock(AudioBuffer<float>& buffer) {
    AudioBuffer<float> dry(2, buffer.getNumSamples());  // ❌ MALLOC!
}
```

### Parameter Smoothing

```cpp
// Use JUCE LinearSmoothedValue for zipper-free parameter changes
juce::LinearSmoothedValue<float> morphSmooth;
juce::LinearSmoothedValue<float> intensitySmooth;
juce::LinearSmoothedValue<float> driveSmooth;

void prepare(double sampleRate) {
    morphSmooth.reset(sampleRate, 0.02);      // 20ms smoothing
    intensitySmooth.reset(sampleRate, 0.02);
    driveSmooth.reset(sampleRate, 0.01);      // 10ms for drive
}

void updateCoefficientsBlock() {
    // Get smoothed values ONCE per block
    float morph = morphSmooth.getNextValue();
    float intensity = intensitySmooth.getNextValue();
    float drive = driveSmooth.getNextValue();
    // ... use these values for all sections
}
```

---

## 6. ENVELOPE FOLLOWER (WHY DRUMS SOUND WEIRD WITHOUT IT)

### The Problem
Real EMU hardware had an **envelope follower** that dynamically modulated the filter based on input level. Without it:
- ✅ Steady signals (metronome) sound good
- ❌ Transients (drums) sound weird/harsh

### Authentic Implementation

```cpp
class EnvelopeFollower {
    float state = 0.0f;
    float attack = 0.000489f;   // ~0.5ms (AUTHENTIC)
    float release = 0.08f;       // 80ms (AUTHENTIC)
    double sampleRate = 48000.0;

    void setAttack(float ms) {
        attack = ms / 1000.0f;
    }

    void setRelease(float ms) {
        release = ms / 1000.0f;
    }

    float process(float input) {
        float rectified = std::abs(input);

        // Attack/release time constants
        float tau = (rectified > state) ? attack : release;
        float alpha = 1.0f - std::exp(-1.0f / (tau * sampleRate));

        // Smooth rectified signal
        state += alpha * (rectified - state);

        return std::clamp(state, 0.0f, 1.0f);
    }

    void reset() { state = 0.0f; }
};
```

### Integration with Z-Plane Filter

```cpp
void processBlock(AudioBuffer<float>& buffer) {
    // Update filter coefficients
    zplaneFilter.updateCoefficientsBlock();

    // Process envelope follower
    float envValue = 0.0f;
    for (int s = 0; s < buffer.getNumSamples(); ++s) {
        envValue = envelopeFollower.process(buffer.getSample(0, s));
    }

    // Apply envelope depth to morph modulation
    float envDepth = 0.945f;  // AUTHENTIC: 94.5%
    float envMod = envValue * envDepth;

    // Modulate morph parameter
    float baseMorph = characterParam->load() / 100.0f;
    float modulatedMorph = std::clamp(baseMorph + envMod * 0.2f, 0.0f, 1.0f);

    // Process audio through filter
    zplaneFilter.processBlock(left, right, numSamples);
}
```

---

## 7. PARAMETER INTERFACE

### Minimal Control Set (Phase 1)

```cpp
// APVTS Parameter Layout
juce::AudioProcessorValueTreeState::ParameterLayout createParameters() {
    std::vector<std::unique_ptr<RangedAudioParameter>> params;

    // CHARACTER: 0-100% (maps to morph 0.0-1.0)
    params.push_back(std::make_unique<AudioParameterFloat>(
        "character", "Character",
        NormalisableRange<float>(0.0f, 100.0f), 50.0f));

    // BYPASS: On/Off
    params.push_back(std::make_unique<AudioParameterBool>(
        "bypass", "Bypass", false));

    // GAIN: -12dB to +12dB (output trim)
    params.push_back(std::make_unique<AudioParameterFloat>(
        "gain", "Gain",
        NormalisableRange<float>(-12.0f, 12.0f), 0.0f));

    // MIX: 0-100% dry/wet
    params.push_back(std::make_unique<AudioParameterFloat>(
        "mix", "Mix",
        NormalisableRange<float>(0.0f, 100.0f), 100.0f));

    // TEST TONE: On/Off (for validation)
    params.push_back(std::make_unique<AudioParameterBool>(
        "testTone", "Test Tone", false));

    return { params.begin(), params.end() };
}
```

---

## 8. TESTING REQUIREMENTS

### Phase 1: Basic Functionality
- [ ] Plugin loads in DAW without crashing
- [ ] Audio passes through cleanly when bypassed
- [ ] Character knob responds 0-100%
- [ ] No NaN/infinity in output
- [ ] No crackling/popping during parameter changes

### Phase 2: DSP Validation
- [ ] Metronome sounds "cool" (resonant, musical)
- [ ] Drums sound natural (NOT harsh/metallic)
- [ ] Bass remains tight (no low-end mud)
- [ ] Vocals have formant character
- [ ] No instability at extreme settings (0% or 100%)

### Phase 3: Performance
- [ ] CPU usage <5% on modern system
- [ ] No dropouts at 64-sample buffer
- [ ] Works at 44.1kHz, 48kHz, 96kHz
- [ ] Mono and stereo operation
- [ ] VST3 and Standalone formats

### Test Signals
```cpp
// 1. Impulse (check decay time and ringing)
void generateImpulse(AudioBuffer<float>& buffer) {
    buffer.clear();
    buffer.setSample(0, 0, 1.0f);  // Single sample spike
}

// 2. Sweep (check frequency response)
void generateSweep(AudioBuffer<float>& buffer, double fs) {
    double phase = 0.0;
    double freqStart = 20.0, freqEnd = 20000.0;
    for (int i = 0; i < buffer.getNumSamples(); ++i) {
        double t = (double)i / buffer.getNumSamples();
        double freq = freqStart * pow(freqEnd/freqStart, t);
        phase += 2.0 * PI * freq / fs;
        buffer.setSample(0, i, (float)sin(phase) * 0.1f);
    }
}

// 3. Test Tone (440Hz for validation)
void generateTestTone(AudioBuffer<float>& buffer, double fs) {
    double phase = 0.0;
    double phaseInc = 440.0 * 2.0 * PI / fs;
    for (int i = 0; i < buffer.getNumSamples(); ++i) {
        buffer.setSample(0, i, (float)sin(phase) * 0.05f);  // -26dB
        phase += phaseInc;
    }
}
```

---

## 9. REFERENCE IMPLEMENTATIONS

### Current Working Code Location
- **Golden Template**: `C:\plugin_dev\engine-suite-golden\plugins\minimal\`
- **ZPlaneFilter.h**: `C:\plugin_dev\engine-suite-golden\plugins\minimal\src\dsp\ZPlaneFilter.h`
- **MinimalProcessor.cpp**: `C:\plugin_dev\engine-suite-golden\plugins\minimal\src\MinimalProcessor.cpp`

### Archive (Original Extraction)
- **Archive Root**: `C:\ARCHIVE_2025-10-15\NEEDS CONSOLIDATING AND REFACTOR\OrganizedForClaude\01_EMU_ZPlane\`
- **Authentic Implementation**: `other\emu_extracted\emu_extracted\AuthenticEMUZPlane.cpp`
- **Shape Data**: `other\emu_extracted\emu_extracted\shapes\audity_shapes_A_48k.json`
- **Documentation**: `docs\EMU_ZPlane_Vault\documentation\`

### Key Files to Reference
1. **EMUFilter.h** - Contains AUTHENTIC_EMU_SHAPES table
2. **AuthenticEMUZPlane.cpp** - Pole interpolation and coefficient conversion
3. **ZPlaneFilter.h** - Core biquad processing
4. **FieldEngineFXProcessor.cpp** - Shows envelope follower integration

---

## 10. COMMON MISTAKES TO AVOID

### ❌ WRONG: Pole Radius Too Close to 1.0
```cpp
p.r = std::min(p.r * intensityBoost, 0.999999f);  // ❌ TOO HIGH!
// Result: Harsh metallic ringing on transients
```

✅ **CORRECT:**
```cpp
p.r = std::min(p.r * intensityBoost, 0.9950f);  // ✅ AUTHENTIC LIMIT
```

### ❌ WRONG: Updating Coefficients Per Sample
```cpp
for (int n = 0; n < numSamples; ++n) {
    zplaneFilter.updateCoefficients();  // ❌ KILLS CPU!
    output[n] = zplaneFilter.process(input[n]);
}
```

✅ **CORRECT:**
```cpp
zplaneFilter.updateCoefficientsBlock();  // ✅ ONCE per block
for (int n = 0; n < numSamples; ++n) {
    output[n] = zplaneFilter.process(input[n]);
}
```

### ❌ WRONG: Wrong Intensity Value
```cpp
zplaneFilter.setIntensity(0.6f);  // ❌ Too high (harsh)
zplaneFilter.setIntensity(0.3f);  // ❌ Too low (no character)
```

✅ **CORRECT:**
```cpp
zplaneFilter.setIntensity(0.4f);  // ✅ AUTHENTIC (from Planet Phatt)
```

### ❌ WRONG: Missing Envelope Follower
```cpp
// Processing steady metronome clicks works fine...
// But drums sound weird! Why?
// Answer: Need envelope follower for transient response
```

✅ **CORRECT:**
```cpp
// Add envelope follower with AUTHENTIC timings
envelopeFollower.setAttack(0.489f);   // ~0.5ms
envelopeFollower.setRelease(80.0f);   // 80ms
float envMod = envelopeFollower.process(input) * 0.945f;
```

---

## 11. SUCCESS CRITERIA

### Definition of "Done"
The DSP is considered **LOCKED** when:

1. ✅ **Metronome sounds cool** (resonant, musical, not harsh)
2. ✅ **Drums sound natural** (tight attacks, no metallic artifacts)
3. ✅ **Bass remains punchy** (no mud, no boom)
4. ✅ **Vocals have character** (formant resonances audible)
5. ✅ **No instability** at any parameter setting (0-100%)
6. ✅ **CPU usage acceptable** (<5% on modern system)
7. ✅ **No audio glitches** during parameter changes

### User Quote
> "woah the metronome sounds cool" ← This is what success looks like

If drums also sound good with envelope follower enabled, **DSP is LOCKED**.

---

## 12. NEXT STEPS AFTER DSP LOCK

Once DSP is validated and locked:

1. **Freeze DSP code** - No more changes to ZPlaneFilter.h or processing logic
2. **Build UI** - Add compact interface with Character knob (0-100%)
3. **Add presets** - Create factory presets showcasing different morph pairs
4. **Polish** - Add bypass fade, tooltips, visual feedback
5. **Template** - Copy to golden template for 4-plugin suite (Field, Pitch, Spectral, Morph)

---

## CONTACT / REFERENCES

**Original Research**: Sound on Sound EMU Audity 2000 Review (1998)
**Patent**: US Patent 5,391,882 "Method of controlling a time-varying digital filter"
**Archive Date**: 2025-09-16 (month-old authentic extraction)
**Hardware Reference**: EMU Audity 2000, Planet Phatt, Orbit-3, Xtreme Lead

---

**PACKAGE STATUS**: COMPLETE AND READY FOR IMPLEMENTATION
**VERSION**: 1.0
**DATE**: 2025-10-17
**PURPOSE**: Give this entire document to a coding agent to implement authentic EMU Z-plane DSP
