---
task: h-refactor-ui-system
branch: feature/refactor-ui-system
status: pending
created: 2025-10-19
modules: [EngineField/ui, EngineField/CMakeLists.txt, build-system]
---

# Refactor UI/UX System in Engine:Field Plugin

## Problem/Goal

The Engine:Field plugin currently has **two competing UI systems** and significant build configuration issues:

1. **Competing UIs:**
   - `FieldPluginEditor` (sampler pad grid) - Currently active and working
   - `FieldEditor` (XY pad + spectrum) - Complete but not in build system

2. **Dead Code:**
   - `FieldEditorV2.h` - Header only, no implementation, never used

3. **Orphaned Files** (complete but not built or used):
   - ZPlaneVisualizer.h/cpp - alien effects visualizer
   - AlienColors.h - alien color palette
   - AlienGlyphs.h - typography helpers
   - MinimalControls.h - custom controls
   - SpringValue.h - animation system
   - EngineLookAndFeel.h - old look and feel

4. **Missing from CMake Build:**
   - FieldEditor.cpp (exists but not compiled)
   - XYPad.cpp (exists but not compiled)
   - SpectrumComponent.cpp (exists but not compiled)
   - EngineLNF.cpp (exists but not compiled)

**Goal:** Analyze, clean up, and consolidate the UI system. Choose the best UI for drum makers/beatmakers, fix the build system, and document the decision.

## Success Criteria

### Step 1: Analysis & Recommendation
- [ ] Analyze both `FieldPluginEditor` and `FieldEditor` implementations
- [ ] Compare completeness, polish, and suitability for drum/beat makers
- [ ] Provide recommendation on which UI to keep as primary
- [ ] Decide whether to delete the alternate UI or archive it

### Step 2: Clean Up Dead Code
- [ ] DELETE `FieldEditorV2.h` (no implementation exists)
- [ ] CREATE `plugins/EngineField/archive/` folder
- [ ] MOVE unused files to archive:
  - [ ] ZPlaneVisualizer.h/cpp
  - [ ] AlienColors.h
  - [ ] AlienGlyphs.h
  - [ ] MinimalControls.h
  - [ ] SpringValue.h
  - [ ] EngineLookAndFeel.h

### Step 3: Fix Build System
- [ ] Update `plugins/EngineField/CMakeLists.txt` to include ALL active UI source files
- [ ] Remove references to archived files from CMakeLists.txt
- [ ] Ensure chosen UI compiles without errors
- [ ] Verify all UI headers are properly included

### Step 4: Verify Integration
- [ ] Confirm `PluginProcessor::createEditor()` works correctly
- [ ] Verify all UI components compile without errors
- [ ] Test parameter connections (APVTS) are working
- [ ] Confirm audio level updates reach the UI properly
- [ ] Build and test VST3/Standalone formats

### Step 5: Document Decision
- [ ] Update `CLAUDE.md` with primary/active UI choice
- [ ] Document reasoning for the decision
- [ ] List what was archived and why
- [ ] Update implementation discoveries section if needed

### Step 6: Successful Build
- [ ] `cmake --build build --config Release` completes without errors
- [ ] VST3 plugin loads in a DAW successfully
- [ ] Standalone app launches successfully
- [ ] UI displays correctly with working controls

## Constraints

**DO NOT:**
- Modify any DSP code (`ZPlaneFilter.h`, `EMUAuthenticTables.h`)
- Change the processor's parameter structure
- Break the currently working `FieldPluginEditor` during cleanup
- Cause build failures

**MUST:**
- Keep authentic EMU DSP locked and untouched
- Maintain RT safety in any changes
- Follow JUCE best practices
- Preserve APVTS parameter management

## User Notes

User specifically requests:
1. **Recommendation with reasoning** - Which UI better serves the target audience (drum/beat makers)?
2. **Complete file list** - Show all files being deleted/archived
3. **Updated CMakeLists.txt** - Show the complete updated build configuration
4. **Build verification** - Confirm everything compiles successfully
5. **Warning/issue report** - Surface any problems found during cleanup

Target audience: **Drum makers and beatmakers** using this for transient shaping, character, and creative effects on drums/samples.

## Context Manifest

### How the Current UI System Works

**Audio Flow and Parameter Binding:**

When FieldProcessor initializes, it creates an AudioProcessorValueTreeState (APVTS) with 6 parameters: CHARACTER (0-100%), MIX (0-100%), OUTPUT gain (-12 to +12 dB), BYPASS (bool), TEST TONE (bool), and EFFECT MODE (bool). The processor's `createEditor()` method at line 204 of FieldProcessor.cpp instantiates `new engine::ui::FieldPluginEditor(*this, apvts_)`, which is the currently active UI defined entirely within FieldPadUI.h (420 lines, lines 11-420).

**Active UI: FieldPluginEditor (Sampler Pad Grid)**

The FieldPluginEditor consists of 3 nested component classes:

1. **PadColors namespace** (lines 14-24): Warm, analog aesthetic palette with cream background (0xFFF5F1E8), brown/tan tones for inactive pads (0xFFE0D5C7), and warm accents (0xFFD55D42 for flash). This palette targets the "sampler" aesthetic.

2. **SamplerPad class** (lines 30-156): A single 4x4 grid pad that maintains a 12-frame history buffer of audio levels. When audio arrives, the pad:
   - Updates its history buffer with the current audio level (from processor's getAudioLevel() atomic)
   - Draws based on CHARACTER parameter: brightness is quantized into 4-16 steps (character * 12 + 4), and holds decay frames (0-4 frames based on character)
   - In EFFECT mode, draws only trail ghosts with ghost offsets and misregistration at high character values (>0.6)
   - In normal mode, draws main pad + trails
   - Uses jittered bounds at high character for subtle roughness

3. **PadGridVisualizer class** (lines 162-247): 4x4 grid (16 total pads) running at 60 Hz via Timer. Each pad gets the current audio level (with per-pad variation for "stereo spread"). When painted, pads fill the available space with padding (12px) and spacing (8px).

4. **FieldPluginEditor class** (lines 252-419): Main editor window (600x550 default). Layout:
   - Header (70px): Title "FIELD" (32pt) + EFFECT toggle button (top-right)
   - Main area: PadGridVisualizer taking most of the space (lines 363-364)
   - Bottom controls (80px):
     - CHARACTER slider (linear, 0-100) occupying left 65% with label and numeric value display
     - MIX knob (rotary, 0-100) occupying right side with label and value display
   - All controls bind via APVTS attachments (lines 271-272, 296-297)
   - Timer running at 60 Hz pulls audio level from processor via `dynamic_cast<FieldProcessor&>(processor).getAudioLevel()` and updates padGrid

**Audio Level Data Flow:**
In FieldProcessor::processBlock (lines 188-201), an envelope follower with 5ms attack and 100ms release calculates `envelopeState` and stores it in `currentLevel` atomic. This is lock-free, so the UI thread can read it without blocking the audio thread. The padGrid updates all 16 pads with this level every 60 Hz, adding per-pad variation (variation = 0.9 + (i%3)*0.05).

**Alternative UI: FieldEditor (XY Pad + Spectrum)**

FieldEditor.h/.cpp (39 lines header, 99 lines implementation) defines a more modern, professional interface:
- XY Pad component: X axis controls CHARACTER, Y axis controls MIX (both 0-1 normalized, sent as 0-100% to APVTS)
- Spectrum visualizer: Real-time FFT display (2048-point FFT, Hann windowing)
- Bypass button and Output gain slider (-12 to +12 dB)
- Modern dark aesthetic (0xFF0B0E13 near-black background, teal-cyan accent 0xFF39E1D0)
- Resizable window (380x260 to 1400x900)
- EngineLNF look-and-feel applied (lines 8)
- Layout (lines 75-90): Spectrum in top-left, XY pad below it, controls on right side

**Components Used by FieldEditor:**
- XYPad.h: Simple crosshair component (60 lines). Renders grid lines at 0.25/0.5/0.75 positions, circular crosshair. Normalized x,y values (0-1). Callback fires on mouse drag/down.
- SpectrumComponent.h: FFT-based spectrum analyzer (84 lines). Maintains circular FIFO of audio samples, computes Hann-windowed FFT, converts to log magnitude (dB scale, -100 to 0 dB), displays as line trace. **Has syntax error at line 56: missing opening brace for `timerCallback()` override**.
- EngineLNF.h/cpp: LookAndFeel_V4 subclass with color definitions for the modern aesthetic.

**Why Two UIs Exist:**

The sampler pad grid (FieldPluginEditor) emphasizes **visual transient feedback and character morphing**, making it ideal for drum/beat makers who work with percussive material. The XY pad + spectrum (FieldEditor) emphasizes **precise parameter control and frequency visualization**, more suited to detailed mixing and analysis workflows.

**Current Build Configuration Issue:**

CMakeLists.txt (lines 31-42) only includes:
- FieldPadUI.h (which contains FieldPluginEditor)
- parameters.h
- DSP headers
- Shared Colors.h

Missing from the build:
- FieldEditor.cpp/.h (complete implementation)
- XYPad.cpp/.h (complete component)
- SpectrumComponent.cpp/.h (complete but has syntax error)
- EngineLNF.cpp/.h (complete look-and-feel)
- FieldEditorV2.h (header-only stub, no implementation)

The cpp files are not compiled because they're not listed in `target_sources()`. The header-only files (FieldPadUI.h, FieldEditorV2.h, XYPad.h inside XYPad.h) compile fine because they're included, but FieldEditor.h tries to use XYPad and SpectrumComponent, which fail to compile if included without their cpp files being linked.

**Parameter Layout Architecture:**

The APVTS in FieldProcessor defines 6 parameters (parameters.h lines 7-47):
- `character` (0-100%, default 50%): Controls EMU morph position between two vowel shapes
- `mix` (0-100%, default 100%): Blend between dry and processed signal
- `gain` (-12 to +12 dB, default 0 dB): Output makeup gain
- `bypass` (bool, default false): Bypasses the DSP
- `testTone` (bool, default false): Generates 440 Hz sine for testing
- `effectMode` (bool, default false): UI-only flag toggled by FieldPluginEditor's EFFECT button

Both UIs read/write to the same APVTS, so they're interchangeable at the parameter level.

**RT Safety and Lock-Free Data:**

FieldProcessor uses lock-free atomics for UI data:
- `currentLevel` atomic<float>: Audio level for visualization (read by UI timer at 60 Hz)
- `uiPoles_` array<atomic<float>, 12>: 6 pole pairs (r, theta) from ZPlaneFilter for potential pole visualization
- These atomics use `memory_order_relaxed` since small data races are acceptable for UI

No locks, no allocations in processBlock.

**Integration Points:**

1. **PluginProcessor** creates the UI via `createEditor()` (line 204): `return new engine::ui::FieldPluginEditor(*this, apvts_);`
2. **UI Timer callbacks** (60 Hz):
   - FieldPluginEditor reads `processor.getAudioLevel()` and updates padGrid
   - FieldEditor reads `processor.popVisualiserBlock()` and feeds SpectrumComponent
3. **Parameter Attachments** (APVTS):
   - CHARACTER slider -> apvts.getParameter("character")
   - MIX slider/knob -> apvts.getParameter("mix")
   - GAIN slider -> apvts.getParameter("gain")
   - BYPASS button -> apvts.getParameter("bypass")
4. **Visualizer FIFO**:
   - Processor writes mono samples to `visBuffer_` via `pushToVisualiser()` after each processBlock
   - FieldEditor's timer reads blocks via `popVisualiserBlock()`

### For This Refactoring Task

**Decision Framework:**

Since the target is **drum makers and beatmakers**, the key criteria are:
1. **Immediate transient feedback** - crucial for timing and character shaping on drums
2. **Character morphing visibility** - the pad grid directly shows how CHARACTER affects decay/quantization
3. **MIX blending feedback** - sampler pads light up with current audio level, showing dry/wet blend
4. **Quick visual loop** - no need for complex frequency analysis when working with punchy transients

The **FieldPluginEditor (sampler pad grid) is better suited** because:
- Transients light up the grid, providing instant visual confirmation
- CHARACTER parameter visibly affects pad decay steps and jitter
- MIX parameter controls pad brightness, showing blend amount
- Warm aesthetic matches "analog sampler" workflow
- Grid layout is intuitive for beat-focused work

The **FieldEditor (XY + spectrum) is better for detail work**:
- XY pad is more precise parameter control
- Spectrum shows frequency content (useful for surgery, less for drums)
- Modern aesthetic suits technical workflow
- Resizable for various screen sizes

**Recommendation: Keep FieldPluginEditor as primary; archive FieldEditor for potential future use or as reference.**

### Technical Reference Details

#### File Locations and Status

**Active Files (Currently Building):**
- `C:\plugin_dev\plugins\EngineField\Source\ui\FieldPadUI.h` - All FieldPluginEditor code (420 lines, single header)
- `C:\plugin_dev\plugins\EngineField\Source\FieldProcessor.h` - Main processor class
- `C:\plugin_dev\plugins\EngineField\Source\FieldProcessor.cpp` - Processor implementation (createEditor on line 204)
- `C:\plugin_dev\plugins\EngineField\Source\parameters.h` - APVTS parameter layout

**Unused but Complete (Not in CMake, must archive):**
- `C:\plugin_dev\plugins\EngineField\Source\FieldEditor.h` - XY+Spectrum editor (39 lines)
- `C:\plugin_dev\plugins\EngineField\Source\FieldEditor.cpp` - Implementation (99 lines, has timer at line 51)
- `C:\plugin_dev\plugins\EngineField\Source\components\XYPad.h` - XY component (60 lines, header-only)
- `C:\plugin_dev\plugins\EngineField\Source\components\XYPad.cpp` - Empty stub (2 lines, just includes header)
- `C:\plugin_dev\plugins\EngineField\Source\components\SpectrumComponent.h` - FFT visualizer (84 lines, **HAS SYNTAX ERROR line 56**)
- `C:\plugin_dev\plugins\EngineField\Source\components\SpectrumComponent.cpp` - Empty stub (2 lines)
- `C:\plugin_dev\plugins\EngineField\Source\lookandfeel\EngineLNF.h` - Look and feel (20 lines, header-only)
- `C:\plugin_dev\plugins\EngineField\Source\lookandfeel\EngineLNF.cpp` - Empty stub (3 lines)

**Dead Code (No Implementation, Delete):**
- `C:\plugin_dev\plugins\EngineField\Source\FieldEditorV2.h` - Stub header (18 lines, no implementation)

**Not Found (Mentioned in task but don't exist):**
- ZPlaneVisualizer.h/cpp - Not in codebase
- AlienColors.h - Not in codebase
- AlienGlyphs.h - Not in codebase
- MinimalControls.h - Not in codebase
- SpringValue.h - Not in codebase

#### Current CMakeLists.txt Structure

```cmake
target_sources(${TARGET_NAME} PRIVATE
    Source/PluginMain.cpp
    Source/FieldProcessor.cpp         # includes FieldProcessor.h
    Source/FieldProcessor.h
    Source/parameters.h
    Source/dsp/ZPlaneFilter.h
    Source/dsp/EMUAuthenticTables.h
    Source/dsp/EnvelopeFollower.h
    Source/ui/FieldPadUI.h            # ONLY UI file included
    ../../shared/ui/Colors.h
)
```

Missing: FieldEditor.cpp, components/*.cpp, lookandfeel/*.cpp

#### Key Function Signatures

**FieldProcessor:**
```cpp
juce::AudioProcessorEditor* createEditor() override;  // Line 204: returns new FieldPluginEditor
void pushToVisualiser(const juce::AudioBuffer<float>& buffer) noexcept;  // Line 63
bool popVisualiserBlock(juce::AudioBuffer<float>& dest) noexcept;  // Line 84
float getAudioLevel() const noexcept { return currentLevel.load(); }  // Line 49
const std::array<std::atomic<float>, 12>& getUIPoles() const noexcept { return uiPoles_; }  // Line 46
```

**FieldPluginEditor (in FieldPadUI.h):**
```cpp
FieldPluginEditor(juce::AudioProcessor& p, juce::AudioProcessorValueTreeState& apvts);  // Line 256
void paint(juce::Graphics& g) override;  // Line 337
void resized() override;  // Line 354
void timerCallback() override;  // Line 391 (60 Hz, updates from processor)
```

**PadGridVisualizer (in FieldPadUI.h):**
```cpp
void setCharacter(float value);  // Line 171
void setMix(float value);  // Line 172
void setEffectMode(bool enabled);  // Line 173
void updateAudioLevel(float level);  // Line 175
```

**FieldEditor:**
```cpp
explicit FieldEditor(FieldProcessor& proc);  // Line 5 (FieldEditor.cpp)
void timerCallback() override;  // Line 92 (pulls spectrum data)
```

**XYPad:**
```cpp
std::function<void(float x, float y)> onChange;  // Line 8
void setXY(float x, float y);  // Line 40 (0-1 normalized)
```

**SpectrumComponent:**
```cpp
explicit SpectrumComponent(int fftOrder = 2048);  // Line 9
void pushAudioSamples(const juce::AudioBuffer<float>& mono);  // Line 21
void timerCallback();  // Line 56 [SYNTAX ERROR: missing opening brace]
```

#### APVTS Parameter IDs and Ranges

```cpp
"character" -> 0.0-100.0%, default 50.0%, label "%"
"mix"       -> 0.0-100.0%, default 100.0%, label "%"
"gain"      -> -12.0 to 12.0 dB, default 0.0 dB, label "dB"
"bypass"    -> bool, default false
"testTone"  -> bool, default false
"effectMode" -> bool, default false
```

#### Build Output Locations (After Fix)
- VST3: `C:\plugin_dev\build\EngineField_artefacts\Release\VST3\EngineField.vst3`
- Standalone: `C:\plugin_dev\build\EngineField_artefacts\Release\Standalone\EngineField.exe`

#### Known Issues to Fix
1. **SpectrumComponent.h line 56**: Missing opening brace for `void timerCallback()` override - needs `{` before `const int size = 1 << order;`
2. **Build system**: XYPad and SpectrumComponent not linked (cpp files not in CMakeLists.txt), but they're header-only components so this may not matter if not used
3. **Dead code**: FieldEditorV2.h has no implementation and is never instantiated
4. **Unused files**: If FieldEditor is not chosen, its dependencies (XYPad, SpectrumComponent, EngineLNF) should be archived

## Work Log
- [2025-10-19] Task created from user request to fix UI/UX system chaos
