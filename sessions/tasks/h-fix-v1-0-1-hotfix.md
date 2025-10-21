---
task: h-fix-v1-0-1-hotfix
branch: hotfix/v1.0.1
status: in-progress
created: 2025-10-22
modules: [EngineField/ui, EngineField/FieldProcessor]
---

# v1.0.1 Hotfix: Interactive UI Controls + DSP Calibration

## Problem/Goal

Engine:Field v1.0 was released with two critical issues:

1. **UI Non-Functional (CRITICAL)**: FieldWaveformUI renders SVG graphics showing sliders/buttons, but provides NO interactive controls. Parameters can only be changed via DAW automation, making standalone mode completely unusable.

2. **DSP Too Aggressive (HIGH PRIORITY)**: Default settings (CHARACTER=50%, envelope depth=0.945) produce excessive formant resonance on drum transients. Need to calibrate for "noticeable but balanced" defaults that let users dial in their sweet spot.

## Success Criteria

### UI Fixes
- [ ] Create RetroLookAndFeel with pixel-perfect rectangular controls (no anti-aliasing, blue/yellow palette)
- [ ] Add MIX slider (horizontal, top-left) with APVTS attachment
- [ ] Add CHARACTER slider (horizontal, bottom) with APVTS attachment
- [ ] Add EFFECT button (toggle, top-right) with APVTS attachment
- [ ] Position controls to precisely overlay SVG graphics from PNG design
- [ ] Verify standalone mode: all controls respond to mouse interaction

### DSP Calibration
- [ ] Reduce envelope depth from 0.945 to 0.75 (±18.9% → ±15% modulation)
- [ ] Verify EFFECT toggle correctly solos wet signal (100% wet when ON)
- [ ] Test DSP with drum samples in VST3/DAW mode (standalone cannot test audio)
- [ ] Confirm CHARACTER=50% produces "noticeable but balanced" formant emphasis

### Documentation & Release
- [ ] Create task file with context manifest (context-gathering agent)
- [ ] Update CLAUDE.md: mark UI and DSP issues as FIXED
- [ ] Update hotfix checklist in CLAUDE.md
- [ ] Commit changes with proper message
- [ ] Tag release v1.0.1

## Context Files

**UI Implementation:**
- @plugins/EngineField/Source/ui/FieldWaveformUI.h - Current editor (needs controls added)
- @plugins/EngineField/Source/ui/FieldWaveformUI.cpp - Paint/layout implementation
- @plugins/EngineField/Source/ui/FieldLookAndFeel.h - NEW: Custom retro LookAndFeel
- @Generated Image October 19, 2025 - 11_23PM.png - Target UI design reference

**DSP:**
- @plugins/EngineField/Source/FieldProcessor.cpp:42-44 - Envelope follower configuration
- @plugins/EngineField/Source/FieldProcessor.cpp:130-133 - EFFECT toggle logic (DSP level)
- @plugins/EngineField/Source/dsp/ZPlaneFilter.h - Filter implementation (locked)

**Parameters:**
- @plugins/EngineField/Source/parameters.h - APVTS parameter definitions

**Documentation:**
- @CLAUDE.md - Project documentation (needs update)

## User Notes

### Design Specifications
- **Target UI**: PNG shows blue/yellow retro design with rectangular controls
- **LookAndFeel**: Pixel-snapped rectangles (no circles), no anti-aliasing
- **Control positioning**: Overlay SVG graphics exactly
  - MIX slider: Top-left ~(40, 50, 110x24)
  - EFFECT button: Top-right ~(390, 40, 105x35)
  - CHARACTER slider: Bottom ~(43, 670, 455x24)

### DSP Philosophy
- Defaults should be "noticeable but not extreme"
- User should be able to easily dial in sweet spot
- Still maintain authentic EMU character (pole_radius=0.995, 6-stage cascade)

### Testing Constraints
- **Standalone mode**: Can test UI responsiveness only (no audio testing possible)
- **VST3/DAW mode**: Required for audio/DSP testing with drum samples

## Context Manifest

### How the Current UI System Works: FieldWaveformUI Architecture

The active UI implementation (`plugins/EngineField/Source/ui/FieldWaveformUI.h/.cpp`) is a **visualization-only** system that was never completed with interactive controls. Understanding why it's broken requires tracing through the entire architecture from initialization to rendering.

**Initialization Flow (FieldWaveformEditor Constructor):**

When the plugin creates an editor, `FieldProcessor::createEditor()` instantiates a `FieldWaveformEditor`, passing references to the processor and its APVTS. The constructor performs these steps:

1. **Loads SVG Skins**: Two `juce::Drawable` objects are created from embedded binary data (`BinaryData::_1_svg` and `BinaryData::_2_svg`). These SVG files contain the complete visual design—the blue retro background, yellow slider graphics, the EFFECT button outline, and the black waveform viewport. File `1.svg` shows EFFECT off, `2.svg` shows EFFECT on.

2. **Caches Parameter Pointers**: Calls `state.getRawParameterValue()` for three parameters (mix, character, effectMode), storing the returned `std::atomic<float>*` pointers. This is a lock-free read mechanism—the UI can read current parameter values without blocking the audio thread. However, these pointers provide **READ-ONLY** access from the UI's perspective.

3. **Starts 60Hz Timer**: Kicks off a `juce::Timer` that fires `timerCallback()` 60 times per second to animate the waveform visualization.

**The Critical Missing Piece: No Interactive Components**

The `resized()` method is empty except for a comment: "Fixed layout, no dynamic components." This is the smoking gun. A properly functioning JUCE editor would contain:

- `juce::Slider` components with position/size set in `resized()`
- `juce::TextButton` components for toggles
- `juce::AudioProcessorValueTreeState::SliderAttachment` objects that bind sliders to parameters
- `juce::AudioProcessorValueTreeState::ButtonAttachment` objects that bind buttons to parameters

**None of these exist.** The SVG renders beautiful static graphics showing where sliders and buttons *should* be, but there are no actual UI components to receive mouse input. The parameter pointers let the UI **read** values (for visual feedback like changing waveform colors based on CHARACTER), but they cannot **write** values back to APVTS.

**Rendering Flow (paint method):**

Every 60Hz, `timerCallback()` pulls waveform data from the processor and triggers a repaint. The `paint()` method:

1. Fills background with blue (`#2D6DA9`)
2. Reads current parameter values atomically: `mixParam_->load() * 0.01f`, `characterParam_->load() * 0.01f`, `effectParam_->load() > 0.5f`
3. Draws the appropriate SVG skin (skinOff_ or skinOn_) based on EFFECT state
4. Clips rendering to the viewport rectangle (16, 80, 388, 348)
5. If EFFECT is off, draws a dashed green baseline
6. Calls `drawWaveform()` which renders 60 yellow bars representing audio energy delta (wet - dry)
7. Calls `drawPeakTracer()` which connects the bar peaks with a bright yellow-green path
8. Draws a small "alive pulse" indicator

The visual feedback works (brightness scales with MIX, bar drip scales with CHARACTER) because it can **read** parameters. But users cannot **change** parameters because there are no interactive components.

**What DAW Automation Reveals:**

In a DAW, host automation can write to APVTS parameters directly, bypassing the UI entirely. This is why the plugin appears to "work" from a DSP perspective—automation writes go through APVTS → audio thread parameter cache → DSP processing. But in standalone mode, or when trying to adjust parameters with a mouse, the UI is completely non-functional.

### How the DSP Architecture Processes Audio: Signal Flow and Modulation

Understanding the DSP flow is critical because the hotfix needs to calibrate envelope depth while maintaining the authentic EMU character. Let's trace a drum transient (kick drum hit) through the entire system.

**Stage 1: Audio Arrives (FieldProcessor::processBlock)**

When the host calls `processBlock()`, the buffer contains stereo audio. The processor immediately:

1. **Stores dry signal**: Copies input to `dryBuffer_` before any processing. This preserves the unprocessed signal for mix blending and envelope detection.

2. **Reads parameters atomically**: Uses cached pointers to avoid APVTS lookups on the audio thread:
   - `character` (0-100%) → converted to 0-1 range
   - `mixPct` (0-100%) → converted to 0-1 range, stored as `mixTarget`
   - `bypass` (bool) → controls crossfade via `bypassSmooth_`
   - `effectOn` (bool) → determines if MIX is overridden to 100%

**Stage 2: Envelope Following (The Critical Calibration Point)**

The envelope follower (`env_`) processes the left channel sample-by-sample using a peak detector with asymmetric time constants:

- **Attack**: 0.489ms (very fast to catch transients)
- **Release**: 80ms (holds peaks briefly for smooth modulation)
- **Current depth**: 0.945 (scales output to ±0.945 range)

For our kick drum example: The initial transient causes `env_.process(sample)` to return a value ramping toward 0.945. The depth parameter directly controls modulation range—at 0.945, the envelope can add up to ±18.9% to the CHARACTER morph position (because it's scaled by 0.2 in the next stage).

**The problem**: 0.945 depth is too aggressive. On sharp transients, this causes the Z-plane poles to move dramatically, creating excessive formant resonance. Reducing to 0.75 will limit modulation to ±15%, providing noticeable-but-balanced dynamic character.

**Stage 3: Morph Modulation**

The base morph position comes from the CHARACTER parameter: `baseMorph = character * 0.01f` (0-1 range). The envelope value is then added with 20% scaling:

```cpp
modulatedMorph = juce::jlimit(0.0f, 1.0f, baseMorph + envValue * 0.2f);
```

At CHARACTER=50% and depth=0.945:
- baseMorph = 0.5
- Max envelope swing: ±0.945 * 0.2 = ±0.189
- Modulated range: 0.311 to 0.689 (38% swing)

After hotfix (depth=0.75):
- Max envelope swing: ±0.75 * 0.2 = ±0.15
- Modulated range: 0.35 to 0.65 (30% swing)

**Stage 4: EFFECT Mode Override**

This is where the EFFECT toggle comes into play (lines 130-133 in FieldProcessor.cpp):

```cpp
const float effectiveMix = effectOn ? 1.0f : mixTarget;
```

When EFFECT is ON, the MIX parameter is completely ignored and the filter receives 100% wet. This "solos" the Z-plane filtering so users can hear exactly what the Engine is doing without dry signal masking the effect. Off = normal operation (respects MIX knob).

**Stage 5: Z-Plane Filter Coefficient Computation**

The `ZPlaneFilter` updates coefficients **once per block** (expensive operation with transcendental math). Here's the critical three-step pole calculation:

1. **Interpolate in 48kHz reference domain**: The EMU authentic shapes in `EMUAuthenticTables.h` store poles as [radius, theta] pairs referenced at 48kHz. For VOWEL_A and VOWEL_B shapes, `interpolatePole()` blends between corresponding poles using the modulated morph value. Radius interpolation uses **geodesic (log-space) blending** by default (`GEODESIC_RADIUS = true` at line 18 of ZPlaneFilter.h):

```cpp
const float lnA = std::log(A.r);
const float lnB = std::log(B.r);
result.r = std::exp((1-t) * lnA + t * lnB);
```

This is more "EMU-ish" than linear interpolation—it preserves the resonant character during morphing. Angle interpolation uses shortest-path wrapping.

2. **Bilinear frequency warping**: The interpolated 48kHz pole is remapped to the actual sample rate using proper bilinear transform (lines 96-137). This is NOT a simple frequency scaling—it goes:
   - 48kHz z-domain → analog s-domain (inverse bilinear)
   - analog s-domain → target-fs z-domain (forward bilinear)

   This preserves the intended formant frequencies across all sample rates (44.1k, 48k, 96k, 192k). There's a fast-path at line 101: if the target rate is within 0.1 Hz of 48kHz, skip the expensive complex math.

3. **Apply intensity boost and clamp**: The remapped radius is scaled by `intensityBoost = 1.0 + lastIntensity * 0.06` where intensity is locked at 0.4 (authentic value). Final radius is clamped to `MAX_POLE_RADIUS = 0.9950` (hardware limit that prevents instability).

4. **Convert poles to biquad coefficients**: `poleToBiquad()` generates Direct Form II Transposed coefficients. The denominator coefficients come from the pole directly: `a1 = -2*r*cos(theta)`, `a2 = r*r`. The numerator places a zero at 0.9 times the pole radius to create the resonant peak shape.

**Stage 6: Per-Sample Processing**

Now the actual audio flows through. For each sample in the block:

1. **Pre-drive**: Input passes through `tanh(input * driveGain)` where `driveGain = 1.0 + drive * 4.0` with drive locked at 0.2 (authentic). This adds subtle harmonic coloration before filtering.

2. **6-stage biquad cascade**: The driven signal flows through 6 cascaded biquad sections. After each section, if saturation is enabled (locked at 0.2), the output passes through `tanh(y * (1 + sat*4))`. This per-section saturation prevents inter-stage resonance buildup from becoming harsh.

3. **Equal-power mix**: The final wet signal is blended with the **original dry input** (not the driven signal—important for bypass tone preservation):
   ```cpp
   const float wetG = std::sqrt(mix);
   const float dryG = std::sqrt(1.0f - mix);
   output = wetL * wetG + inL * dryG;
   ```

   This equal-power crossfade (implemented at lines 258-263 of ZPlaneFilter.h) maintains constant perceived loudness across the entire MIX range. Linear mixing causes a loudness dip around 50% and poor tone preservation when nonlinearities are involved.

**Stage 7: Bypass Crossfade and Output Gain**

If bypass is engaged, the processor performs a smooth per-sample crossfade using `bypassSmooth_` (10ms ramp time). This prevents clicks when toggling bypass. Finally, the output gain (±12 dB range) is applied with 20ms smoothing via `juce::dsp::Gain`.

**Stage 8: Visualization Data Updates**

The processor computes visualization data for the UI using lock-free atomics:

1. **Waveform delta**: Computes block peaks for wet and dry signals separately, then calculates `delta = max(0, wetBlockMax - dryBlockMax)`. This delta represents how much energy the Z-plane filter is adding. The delta is smoothed with its own envelope follower (attack=10ms, release=150ms) and written to a circular buffer of 60 `std::atomic<float>` values. The UI reads these to draw the yellow bar visualization.

2. **Output level meter**: A separate envelope follower tracks the final output level (after all processing) and writes to `uiCurrentLevel_` atomic. The UI uses this for meter displays.

3. **Pole data**: The 6 interpolated poles (12 floats: r0, theta0, r1, theta1...) are copied to `uiPoles_` array of atomics. This could be used for visualizing pole positions on a z-plane plot (not currently rendered in FieldWaveformUI, but the data is available).

### Parameter Flow (APVTS): The Two-Way Binding System

JUCE's `AudioProcessorValueTreeState` (APVTS) is the central parameter management system. Understanding how parameters flow in **both directions** is essential for adding interactive controls.

**Parameter Definition (parameters.h):**

The `createLayout()` function defines six parameters with explicit ranges, defaults, and metadata:

- **character**: 0-100% (default 50%), labeled "%"
- **mix**: 0-100% (default 100%), labeled "%"
- **gain**: -12 to +12 dB (default 0), labeled "dB"
- **bypass**: bool (default false)
- **testTone**: bool (default false), generates 440Hz sine for testing
- **effectMode**: bool (default false), labeled "EFFECT (Wet Solo)"

Each parameter gets a `ParameterID` with version number (all version 1). This versioning allows future parameter additions without breaking saved presets.

**Host → Audio Thread Flow:**

When a DAW automates a parameter or a user moves a control, the flow is:

1. **Host or UI writes to APVTS**: Either via DAW automation (`AudioProcessor::setParameter()`) or via attachment (`SliderAttachment` calls `setValueNotifyingHost()`).

2. **APVTS updates atomic storage**: The parameter's value is stored in an internal `std::atomic<float>` managed by APVTS. This storage is the "ground truth" for parameter state.

3. **Audio thread reads atomically**: `FieldProcessor` caches pointers to these atomics in its constructor:
   ```cpp
   characterParam_ = apvts_.getRawParameterValue(enginefield::params::characterId);
   ```

   In `processBlock()`, reading is lock-free: `const auto character = characterParam_->load();`

The atomic read is the **only** thread synchronization. No mutexes, no message queues, no allocations. This is RT-safe by design.

**UI → APVTS Flow (The Missing Piece in FieldWaveformUI):**

For interactive controls to work, the UI must attach JUCE components to APVTS parameters. Here's how the archived FieldEditor (in `archive/ui-alternate/`) did it correctly:

```cpp
// Create the component
juce::Slider gainSlider;
gainSlider.setSliderStyle(juce::Slider::RotaryHorizontalVerticalDrag);
addAndMakeVisible(gainSlider);

// Create attachment (binds component to parameter two-way)
gainAttach = std::make_unique<juce::AudioProcessorValueTreeState::SliderAttachment>(
    processor.getAPVTS(), enginefield::params::gainId, gainSlider);
```

The `SliderAttachment` object:
- Reads the current parameter value and updates the slider position on construction
- Listens to slider changes and writes to APVTS via `setValueNotifyingHost()`
- Listens to parameter changes (from DAW automation) and updates the slider
- Handles gesture begin/end for automation recording

**FieldWaveformUI currently has NONE of this.** It reads parameter values for visual feedback but cannot write them back.

**APVTS State Persistence:**

The `getStateInformation()` and `setStateInformation()` methods handle preset saving/loading. APVTS exports its entire state as XML:

```cpp
auto state = apvts_.copyState();  // ValueTree snapshot
std::unique_ptr<juce::XmlElement> xml(state.createXml());
copyXmlToBinary(*xml, destData);  // Serialize to binary for host
```

This captures all parameter values, undo history, and metadata. When a preset loads, `replaceState()` updates all parameters atomically—attachments automatically update their UI components.

### Lock-Free Audio↔UI Communication: Atomic Patterns

The visualization system uses three independent lock-free communication channels. This section explains the atomic memory ordering and threading guarantees.

**Pattern 1: Circular Buffer (Waveform Peaks)**

The 60-bar waveform visualization uses a circular buffer declared as:
```cpp
std::array<std::atomic<float>, NUM_WAVEFORM_BARS> waveformPeaks_{};  // 60 elements
std::atomic<int> waveformIndex_ { 0 };
```

**Audio thread writes (producer):**
```cpp
int idx = waveformIndex_.load(std::memory_order_relaxed);
waveformPeaks_[idx].store(deltaEnvelopeState, std::memory_order_relaxed);
waveformIndex_.store((idx + 1) % NUM_WAVEFORM_BARS, std::memory_order_relaxed);
```

**UI thread reads (consumer):**
```cpp
void getWaveformPeaks(std::array<float, NUM_WAVEFORM_BARS>& dest) const noexcept
{
    int currentIndex = waveformIndex_.load(std::memory_order_relaxed);
    for (int i = 0; i < NUM_WAVEFORM_BARS; ++i)
    {
        int readIndex = (currentIndex + i + 1) % NUM_WAVEFORM_BARS;
        dest[i] = waveformPeaks_[readIndex].load(std::memory_order_relaxed);
    }
}
```

**Why this is safe with relaxed ordering**: The UI reads stale data occasionally (when it reads mid-write), but this is acceptable for visualization. The worst case is a single frame shows a slightly inconsistent waveform—this is imperceptible at 60 FPS. Using `memory_order_relaxed` avoids memory barriers and cache synchronization overhead.

**Pattern 2: Single Atomic Value (Current Level)**

Simple read/write with no dependencies:
```cpp
std::atomic<float> uiCurrentLevel_{ 0.0f };

// Audio thread (producer)
uiCurrentLevel_.store(uiEnvelopeState_, std::memory_order_relaxed);

// UI thread (consumer)
float getCurrentLevel() const noexcept {
    return uiCurrentLevel_.load(std::memory_order_relaxed);
}
```

**Pattern 3: APVTS Raw Parameter Pointers**

APVTS provides pre-allocated atomics:
```cpp
std::atomic<float>* mixParam_ = state.getRawParameterValue(enginefield::params::mixId);

// UI reads in paint()
const float mixAlpha = mixParam_ ? clamp01(mixParam_->load() * 0.01f) : 1.0f;
```

The null-check (`mixParam_ ?`) is defensive programming—the pointer should never be null after initialization, but checking prevents crashes if APVTS setup fails.

**Why No Sequential Consistency?**

All these use `memory_order_relaxed` rather than the default `memory_order_seq_cst`. This is an optimization: visualization data doesn't require strict ordering guarantees. If the UI reads a level value from 2ms ago instead of 0.5ms ago, it's visually identical. The performance gain (avoiding memory fences) is significant on weak memory architectures (ARM, older x86).

**Important**: If we were synchronizing control flow (e.g., a "processing complete" flag), we'd need acquire/release semantics. But for continuous data streams (waveforms, meters), relaxed is correct.

### SVG Skin System and Control Positioning

The visual design is split between static SVG graphics and dynamic UI components. Understanding the pixel-precise layout is critical for overlaying interactive controls.

**SVG Loading (BinaryData Embedding):**

The CMakeLists.txt includes:
```cmake
juce_add_binary_data(EngineFieldAssets
    SOURCES
        ${CMAKE_SOURCE_DIR}/1.svg
        ${CMAKE_SOURCE_DIR}/2.svg
)
target_link_libraries(${TARGET_NAME} PRIVATE EngineFieldAssets)
```

At compile time, these SVG files are converted to C++ byte arrays in `BinaryData.h`. The constructor loads them:
```cpp
skinOff_ = juce::Drawable::createFromImageData(BinaryData::_1_svg, BinaryData::_1_svgSize);
skinOn_  = juce::Drawable::createFromImageData(BinaryData::_2_svg, BinaryData::_2_svgSize);
```

`juce::Drawable` parses the SVG DOM and renders it using `Graphics::draw()`.

**Design Analysis (from Generated Image and 1.svg):**

The reference image shows a 420×560 pixel retro oscilloscope aesthetic:

1. **Top Section (0-112px):**
   - MIX label + slider graphic: Blue background, yellow fill rectangle
   - EFFECT button graphic: White border, blue fill when off
   - Both have pixel-perfect rectangular shapes (no anti-aliasing)

2. **Viewport (112-472px):**
   - Black rectangle (396×356 px) bordered by white/blue frame
   - Waveform rendering area defined in code as `viewportPx_ { 16, 80, 388, 348 }`
   - Green dashed baseline when EFFECT off
   - Yellow bars (#F9F034) with "drip" effect below baseline
   - Yellow-green peak tracer path (#C3FF00)

3. **Bottom Section (472-560px):**
   - CHARACTER label + slider graphic: Blue background with blue/yellow rectangle
   - Large yellow thumb area on the right

**Control Overlay Positions (from task requirements):**

The task specifies approximate coordinates for interactive controls that must overlay the SVG graphics:

- **MIX slider**: ~(40, 50, 110×24) - Horizontal, top-left
- **EFFECT button**: ~(390, 40, 105×35) - Top-right toggle
- **CHARACTER slider**: ~(43, 670, 455×24) - Horizontal, bottom

**Note**: The CHARACTER y-coordinate (670) seems incorrect given the window height is 560px. This needs verification against actual SVG element positions. The SVG at line 34-36 shows the CHARACTER slider background at y=486-536, so the control should overlay around y=500-510.

**Color Palette (RetroPalette namespace):**

```cpp
kBackground         = 0xFF2D6DA9  // Blue background
kViewportBackground = 0xFF000000  // Black viewport
kBaseline           = 0xFF59B850  // Green baseline
kBarFill            = 0xFFE8D348  // Yellow bars
kPeakTracer         = 0xFFC3FF00  // Yellow-green peak trace
```

These colors must inform the RetroLookAndFeel design—sliders and buttons should use #F9F034 (bright yellow) for active elements and #2D6DA9 (blue) for backgrounds.

### Technical Reference Details

#### Component Interfaces & Signatures

**FieldWaveformEditor Constructor:**
```cpp
FieldWaveformEditor(FieldProcessor& processor,
                    juce::AudioProcessorValueTreeState& state);
```

**Lock-Free Data Access:**
```cpp
// Waveform peaks (60-element circular buffer)
void getWaveformPeaks(std::array<float, NUM_WAVEFORM_BARS>& dest) const noexcept;

// Current output level (0-1 range, post-envelope-follower)
float getCurrentLevel() const noexcept;
```

**Required APVTS Attachments:**
```cpp
std::unique_ptr<juce::AudioProcessorValueTreeState::SliderAttachment> mixAttachment;
std::unique_ptr<juce::AudioProcessorValueTreeState::SliderAttachment> characterAttachment;
std::unique_ptr<juce::AudioProcessorValueTreeState::ButtonAttachment> effectAttachment;
```

**LookAndFeel Methods to Override:**
```cpp
// Custom slider rendering (rectangular, no anti-aliasing)
void drawLinearSlider(juce::Graphics&, int x, int y, int width, int height,
                      float sliderPos, float minSliderPos, float maxSliderPos,
                      const juce::Slider::SliderStyle, juce::Slider&) override;

// Custom button rendering (rectangular toggle)
void drawButtonBackground(juce::Graphics&, juce::Button&, const juce::Colour& backgroundColour,
                          bool isMouseOverButton, bool isButtonDown) override;
```

#### Data Structures

**ZPlaneFilter Pole Pair:**
```cpp
struct PolePair {
    float r;      // Radius (0-0.995)
    float theta;  // Angle in radians (-π to +π)
};
```

**Envelope Follower Configuration:**
```cpp
struct EnvelopeFollower {
    double sr;           // Sample rate
    float attackMs;      // Attack time (0.489ms default)
    float releaseMs;     // Release time (80ms default)
    float depth;         // Modulation depth (0.945 → 0.75 in hotfix)
};
```

**Waveform Visualization:**
```cpp
static constexpr int NUM_WAVEFORM_BARS = 60;  // engine::viz::kWaveformBarCount
std::array<std::atomic<float>, 60> waveformPeaks_;  // Circular buffer
std::atomic<int> waveformIndex_;                     // Write pointer
```

#### Configuration Requirements

**DSP Calibration (FieldProcessor.cpp):**
```cpp
// Line 44: CHANGE FROM
env_.setDepth(0.945f);
// TO
env_.setDepth(0.75f);
```

**Parameter IDs (parameters.h):**
```cpp
namespace enginefield::params {
    constexpr auto characterId = "character";   // 0-100%, default 50%
    constexpr auto mixId       = "mix";         // 0-100%, default 100%
    constexpr auto effectModeId = "effectMode"; // bool, default false
}
```

#### File Locations

**Primary Implementation Files:**
- UI Header: `C:\plugin_dev\plugins\EngineField\Source\ui\FieldWaveformUI.h`
- UI Implementation: `C:\plugin_dev\plugins\EngineField\Source\ui\FieldWaveformUI.cpp`
- Processor: `C:\plugin_dev\plugins\EngineField\Source\FieldProcessor.cpp` (line 44 for DSP calibration)
- Parameters: `C:\plugin_dev\plugins\EngineField\Source\parameters.h`

**New File to Create:**
- LookAndFeel: `C:\plugin_dev\plugins\EngineField\Source\ui\FieldLookAndFeel.h` (header-only recommended)

**Reference Files:**
- Working APVTS example: `C:\plugin_dev\plugins\EngineField\archive\ui-alternate\FieldEditor.cpp` (lines 32-47 show proper attachments)
- SVG skins: `C:\plugin_dev\1.svg` and `C:\plugin_dev\2.svg`
- Design reference: `C:\plugin_dev\Generated Image October 19, 2025 - 11_23PM.png`

**Build Configuration:**
- CMakeLists.txt: `C:\plugin_dev\plugins\EngineField\CMakeLists.txt` (may need to add FieldLookAndFeel.h to source list if not header-only)

**Documentation Updates Required:**
- `C:\plugin_dev\CLAUDE.md` - Update "Implementation Issues" section to mark UI and DSP issues as FIXED in v1.0.1

## Work Log
- [2025-10-22] Task created following proper sessions protocol
- [2025-10-22] Plan approved - ready for implementation
- [2025-10-22] Context-gathering agent completed comprehensive context manifest
