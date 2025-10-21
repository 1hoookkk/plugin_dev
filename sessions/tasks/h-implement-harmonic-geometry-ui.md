---
task: h-implement-harmonic-geometry-ui
branch: feature/implement-harmonic-geometry-ui
status: in-progress
created: 2025-10-18
started: 2025-10-19
modules: [EngineField, shared/ui]
---

# Implement Harmonic Geometry UI for Engine:Field

## Problem/Goal

Replace the current minimal UI with a premium, unexpected visualization that makes the "Character" parameter (EMU filter morphing) instantly clear and visually beautiful.

**Current state:** Basic dark background with "ENGINE:FIELD" text - functional but plain.

**Target:** Hybrid Quantum Constellation UI featuring:
- Central morphing geometric shape (6 states for 6 filter shapes)
- Constellation points indicating active filter position
- Smooth gradient fills and transitions
- Clean typography
- Premium, Apple-esque minimalism

## Success Criteria

### Phase 1: Foundation (MVP)
- [x] Clean minimal UI with dark background
- [ ] Centered hexagon shape rendering correctly
- [ ] "ENGINE:FIELD" typography at top
- [ ] 60fps smooth rendering verified

### Phase 2: Geometric States
- [ ] 6 distinct geometric shapes defined (triangle → octagon)
- [ ] Shapes wire to Character parameter (APVTS)
- [ ] Instant snapping between shapes works
- [ ] Parameter responds correctly (0-100% maps to shapes 0-5)

### Phase 3: Smooth Morphing
- [ ] Interpolation between vertex positions implemented
- [ ] Smooth transitions working (spring physics or linear)
- [ ] Subtle rotation during morphing
- [ ] Timing curves feel premium (no jarring jumps)

### Phase 4: Constellation Points
- [ ] 6 dots arranged in circle around center
- [ ] Active dot highlights based on Character position
- [ ] Connecting lines to active shape
- [ ] Optional: gentle pulsing animation

### Phase 5: Premium Polish
- [ ] Gradient fills (indigo → violet)
- [ ] Subtle glow effects on active elements
- [ ] Character % readout displayed
- [ ] Audio-reactive brightness (optional stretch goal)

## Design Specification

### Color Palette
```cpp
Background:    #1a1a2e  // Dark blue-grey (current)
Shape fill:    #6366F1  // Indigo
Shape stroke:  #A78BFA  // Light purple
Constellation: #FBBF24  // Amber
Active:        #10B981  // Emerald (highlights)
Text:          #F5F5F0  // Soft cream
```

### Geometric Shapes (Character Parameter Mapping)
- 0-16%:   Triangle (3 vertices)
- 17-33%:  Square (4 vertices)
- 34-50%:  Pentagon (5 vertices)
- 51-66%:  Hexagon (6 vertices)
- 67-83%:  Heptagon (7 vertices)
- 84-100%: Octagon (8 vertices)

### Layout
```
┌─────────────────────────────┐
│     ENGINE:FIELD  (14pt)    │  30px header
├─────────────────────────────┤
│                             │
│         ◆  ●  ◆            │  Constellation
│        ●  [▽]  ●           │  + Center shape
│         ◆  ●  ◆            │
│                             │
│      Character: 47%         │  Readout
├─────────────────────────────┤
│  Mix: [────●────]  [BYP]   │  60px footer (future)
└─────────────────────────────┘
```

## Technical Approach

### Rendering
- Pure JUCE 2D graphics (`juce::Graphics`, `juce::Path`)
- No OpenGL complexity
- Pre-calculated vertex positions for each shape
- Timer callback for smooth interpolation (60fps target)
- **No allocations in `paint()` method**

### Data Flow
```
APVTS Character param → normalize to [0,1] → shape index [0,5]
                     ↓
              Target vertices
                     ↓
         Interpolate current → target
                     ↓
              Render in paint()
```

### Files to Modify
- `plugins/EngineField/Source/FieldEditorV2.h` - Add shape data structures, timer
- `plugins/EngineField/Source/FieldEditorV2.cpp` - Implement rendering logic

## Context Files

### Current Implementation
- @plugins/EngineField/Source/FieldEditorV2.h - Current minimal editor header
- @plugins/EngineField/Source/FieldEditorV2.cpp - Current paint() implementation
- @plugins/EngineField/Source/FieldProcessor.h - Processor with APVTS
- @plugins/EngineField/Source/parameters.h - Parameter definitions

### Reference Materials
- @shared/ui/Colors.h - Existing color definitions
- @CLAUDE.md - Project overview and DSP lock info

## User Notes

### Design Philosophy
- **Minimal but unexpected** - Not typical audio plugin UI
- **Functional beauty** - Visualization directly represents parameter state
- **Premium through restraint** - Like Apple/Ableton minimalism
- **Build incrementally** - Each phase should be testable and satisfying

### Why This Approach?
After exploring complex ideas (liquid metal, ferrofluid, 3D rendering), we pivoted to something achievable with JUCE 2D graphics that still feels unique. The geometric morphing directly maps to the 6 EMU filter shapes, making it both beautiful and functional.

### Hot-Reload Future
This UI is built with eventual hot-reload in mind (Phase 2+ feature). Colors and geometry parameters can later be externalized to JSON config for live editing during "vibe coding" sessions.

## Context Manifest

### How the Current System Works: Engine:Field UI and DSP Architecture

Engine:Field is a JUCE 8 audio plugin that implements authentic EMU Z-plane filtering. The plugin has undergone a recent simplification from a complex "alien" UI (FieldEditor) to a minimal starting point (FieldEditorV2), which is what we'll build upon.

**The Current UI State (FieldEditorV2):**

When the plugin launches, the editor creates a simple 400x600 pixel window. The `paint()` method (lines 15-41 of FieldEditorV2.cpp) currently renders three things: (1) a dark blue-grey background (`#1a1a2e`), (2) "ENGINE:FIELD" text centered in a 30px header at the top, and (3) a unit circle visualization in the remaining space. The unit circle is drawn with white at 30% alpha, along with cross-hairs representing the real and imaginary axes of the Z-plane. This is a functional but minimal visualization - there are no interactive controls, no parameter attachments, and no animation.

The editor holds a reference to the FieldProcessor (`processor` member variable) but doesn't yet read from it or attach to any parameters. This is the clean slate we're building from.

**How the DSP Engine Works (What Character Parameter Actually Controls):**

The Character parameter is the heart of the plugin's sound. Here's the complete flow from parameter to audio:

When a user adjusts the Character parameter (0-100%), that value flows through several transformations. In `FieldProcessor::processBlock()` (lines 119-141), the raw parameter value is read from APVTS, converted to a 0-1 range (`baseMorph = character * 0.01f`), and then modulated by an envelope follower that tracks the input signal's amplitude. The envelope follower adds dynamic movement - louder transients cause the filter to morph more aggressively (up to 20% additional morph depth). This creates the "transient-aware" behavior that makes EMU filters feel alive.

The modulated morph value is passed to the ZPlaneFilter via `zf_.setMorph()`, which stores it in a LinearSmoothedValue to prevent zipper noise. Once per audio block, `zf_.updateCoeffsBlock()` is called (line 147). This is where the magic happens.

**The Z-Plane Filter Morphing Algorithm:**

The ZPlaneFilter currently operates on a single shape pair: VOWEL_A and VOWEL_B (set in FieldProcessor constructor line 14). Each "shape" is actually 6 complex pole pairs stored in polar form (radius, angle) - 12 floats total. These poles define resonant peaks in the frequency response, creating the characteristic EMU formant filtering sound.

When `updateCoeffsBlock()` is called (ZPlaneFilter.h lines 192-223), it performs these steps for each of the 6 pole pairs:

1. **Interpolate in 48kHz reference domain** (line 202): The `interpolatePole()` function (lines 77-95) blends between the two shape's poles using the morph value (0-1). For the radius component, it can use either geodesic (log-space) or linear interpolation, controlled by the `GEODESIC_RADIUS` constant (default: true, line 19). Geodesic interpolation (`result.r = exp((1-t)*ln(r_A) + t*ln(r_B))`) produces smoother, more "EMU-ish" morphing because it respects the exponential nature of pole magnitudes. For the angle, it always takes the shortest angular path around the unit circle using `wrapAngle()`.

2. **Bilinear frequency warping** (line 205): The interpolated pole exists in a 48kHz reference domain (the EMU's native sample rate). To make the filter work correctly at other sample rates (44.1k, 96k, 192k, etc.), `remapPole48kToFs()` (lines 99-134) performs proper bilinear transform frequency mapping. This is NOT just `theta *= fs/48000` - that would distort formant frequencies. Instead, it converts the 48kHz z-plane pole to the analog s-domain (`s = 2*fs_ref * (z-1)/(z+1)`) and then forward-transforms to the target sample rate's z-domain (`z_new = (2*fs_target + s)/(2*fs_target - s)`). This preserves the intended resonant frequencies across all sample rates. There's a fast-path optimization at line 102: if the target sample rate is within 0.1 Hz of 48kHz, it skips the complex math entirely.

3. **Intensity boost and hardware clamping** (lines 197-208): The intensity parameter (locked to 0.4 in this plugin) scales the pole radius by 1.0 to 1.06x (`intensityBoost = 1.0 + 0.4*0.06 = 1.024`). This makes resonances sharper. The result is clamped to 0.9950 (`MAX_POLE_RADIUS`) to match the EMU hardware's stability limit - poles too close to the unit circle cause metallic ringing.

4. **Convert to biquad coefficients** (lines 217-220): The poles are converted from polar form to standard biquad IIR coefficients (a1, a2, b0, b1, b2) using `poleToBiquad()` (lines 136-149). The zeros are placed slightly inside the pole radius (90% of pole radius) to flatten the low-frequency response.

These coefficients are then loaded into a cascade of 6 biquad sections (one per pole pair), giving a 12th-order filter. Each section applies per-sample tanh saturation with a fixed amount (0.2, locked per CLAUDE.md).

**Important: UI Pole Data for Visualization:**

After computing the interpolated poles, the processor copies them into lock-free atomics (FieldProcessor.cpp lines 150-155):
```cpp
for (int i = 0; i < 6; ++i) {
    uiPoles_[i * 2].store(poles[i].r, std::memory_order_relaxed);
    uiPoles_[i * 2 + 1].store(poles[i].theta, std::memory_order_relaxed);
}
```

This means the UI can read the current pole positions at any time via `processor.getUIPoles()` (declared at line 46 of FieldProcessor.h). These are the ACTUAL filter poles being used, post-interpolation, post-bilinear-warping, post-intensity-boost. If we wanted to visualize the true Z-plane state, we'd read these and draw dots at `(r*cos(theta), r*sin(theta))` on a unit circle.

**However, the task is NOT asking us to visualize the actual poles.** The task wants a metaphorical geometric visualization where different polygon shapes (triangle, square, pentagon, etc.) represent different Character parameter values. This is a design decision - showing geometric shapes morphing is more visually clear than showing 6 dots moving around a circle.

**Parameter System (APVTS):**

The Character parameter is defined in `parameters.h` (lines 18-22). It's an `AudioParameterFloat` with:
- ID: `"character"` (version 1)
- Name: `"Character"`
- Range: 0.0 to 100.0 (step: 0.01)
- Default: 50.0%
- Label: `"%"` (displayed in DAW automation)

The parameter lives in the processor's `apvts_` member (AudioProcessorValueTreeState, line 54-56 of FieldProcessor.h). To read it from the UI, you call:
```cpp
auto* param = processor.getAPVTS().getRawParameterValue(enginefield::params::characterId);
float characterPercent = param->load(); // atomic read, range 0-100
```

This is a lock-free atomic read - safe to call from the UI thread at 60fps. The processor reads the same atomic in its audio thread (FieldProcessor.cpp line 120), so both threads see consistent values without locks.

**Existing UI Patterns in the Codebase:**

The project has established patterns for smooth UI animations:

1. **SpringValue** (shared/ui/SpringValue.h): An overdamped spring physics simulator. You set a target value, call `update(deltaTime)` in a timer callback (typically 60fps), and read the smoothly-interpolated current value. It uses a critically-damped spring-damper ODE with configurable settle time (default 200ms). Perfect for smooth shape morphing without jarring jumps.

2. **Timer-based rendering** (seen in FieldEditor.cpp line 51): Components inherit from `juce::Timer` and call `startTimer(17)` to get ~60fps callbacks. In `timerCallback()`, you update animation state and call `repaint()` to trigger a fresh `paint()` call.

3. **JUCE Graphics primitives**: All rendering uses `juce::Graphics`, `juce::Path`, `juce::Colour`, etc. No OpenGL for Phase 1 (though ZPlaneVisualizer.h shows the project CAN use OpenGL for advanced visualizations - that's out of scope here). Typical pattern:
   - `paint()` method receives `juce::Graphics& g`
   - Use `g.setColour()` to set color
   - Use `g.fillAll()` for backgrounds
   - Use `g.fillPath()` / `g.strokePath()` for shapes
   - Use `juce::Path` to build polygons: `path.startNewSubPath(x, y)`, `path.lineTo(x, y)`, `path.closeSubPath()`
   - Call `g.drawText()` for labels

4. **Color scheme**: The project has two color systems:
   - Shared colors (shared/ui/Colors.h): High-contrast black/white/red scheme
   - Task-specified colors (in task description): Indigo/purple/amber scheme for the geometric UI

   The task colors are:
   ```cpp
   Background:    juce::Colour(0xFF1a1a2e)  // Already in FieldEditorV2!
   Shape fill:    juce::Colour(0xFF6366F1)  // Indigo
   Shape stroke:  juce::Colour(0xFFA78BFA)  // Light purple
   Constellation: juce::Colour(0xFFFBBF24)  // Amber
   Active:        juce::Colour(0xFF10B981)  // Emerald
   Text:          juce::Colour(0xFFF5F5F0)  // Soft cream
   ```

**What Needs to Be Implemented:**

The task asks us to create a "Hybrid Quantum Constellation UI" with morphing geometric shapes. Here's what connects to what:

1. **Central geometric shape** (triangle → octagon):
   - Maps to Character parameter via APVTS read
   - Shape determined by parameter ranges (task lines 68-74):
     - 0-16%: Triangle (3 vertices)
     - 17-33%: Square (4 vertices)
     - 34-50%: Pentagon (5 vertices)
     - 51-66%: Hexagon (6 vertices)
     - 67-83%: Heptagon (7 vertices)
     - 84-100%: Octagon (8 vertices)
   - Smooth interpolation between shapes requires pre-calculating vertex positions for each polygon and lerping between them
   - Shapes should be regular polygons centered in the visualization area

2. **Animation approach**:
   - Use `juce::Timer` at 60fps (17ms intervals)
   - Store current vertex positions and target vertex positions
   - Option A: Use SpringValue to smooth each vertex independently
   - Option B: Smooth the Character parameter reading itself, then calculate vertices from smoothed value
   - The task mentions "spring physics or linear" - SpringValue gives better results

3. **Rendering flow** (task lines 102-109):
   ```
   Timer callback (60fps):
     → Read Character from APVTS
     → Normalize to [0,1]
     → Determine shape index [0,5]
     → Compute target vertices for that shape
     → Update SpringValue(s) toward target
     → repaint()

   paint(Graphics& g):
     → Draw background
     → Draw header text
     → Build juce::Path from current vertex positions
     → Fill path with indigo gradient
     → Stroke path with purple
     → Draw constellation points (Phase 4)
     → Draw Character % readout (Phase 5)
   ```

4. **Data structures to add to FieldEditorV2.h**:
   ```cpp
   // Timer callback for animation
   private:
       void timerCallback() override;

       // Shape rendering
       static constexpr int kNumShapes = 6;
       static constexpr int kMaxVertices = 8; // octagon

       // Pre-calculated vertex angles for each shape (constant)
       std::array<std::array<float, kMaxVertices>, kNumShapes> shapeAngles_;

       // Current animated vertices (smoothed)
       std::array<juce::Point<float>, kMaxVertices> currentVertices_;

       // Animation state
       engine::ui::SpringValue<float> characterSmooth_{0.2f}; // 200ms settle
       float currentShapeBlend_{0.0f}; // For inter-shape morphing

       // Rendering helpers
       juce::Path createShapePath(float centerX, float centerY, float radius) const;
       void updateShapeGeometry(); // Called from timerCallback
   ```

5. **Vertex calculation math**:
   For a regular polygon with N sides centered at (cx, cy) with radius R:
   ```cpp
   for (int i = 0; i < N; ++i) {
       float angle = -juce::MathConstants<float>::pi/2 + (2*pi*i)/N; // Start at top
       vertices[i].x = cx + R * std::cos(angle);
       vertices[i].y = cy + R * std::sin(angle);
   }
   ```

   When morphing from N1 to N2 vertices, interpolate like this:
   - If N1 < N2: Duplicate some vertices (or interpolate around perimeter)
   - If N1 == N2: Direct vertex-to-vertex lerp
   - If N1 > N2: Collapse multiple vertices to single target points

   Cleaner approach: Always work with 8-point representations, with some points at the same position for lower-order shapes. Lerp all 8 points every frame.

6. **Integration with existing processor**:
   - The editor already has `processor` member variable
   - Call `processor.getAPVTS()` to access parameters
   - No need for APVTS attachments (those are for sliders/buttons) - we're doing custom visualization
   - The processor's `uiPoles_` data is available but NOT NEEDED for this visualization (that's for Z-plane plots)

**Phase Breakdown and Implementation Strategy:**

The task is structured in 5 phases (lines 26-54). Here's how they map to actual code changes:

**Phase 1: Foundation (DONE)** - The current FieldEditorV2 already has this. Dark background and text rendering work.

**Phase 2: Geometric States (Next)** - This requires:
- Adding Timer inheritance and `startTimer(17)` in constructor
- Pre-calculating the 6 shape vertex angle arrays in constructor
- Reading Character parameter in `timerCallback()`
- Mapping Character to shape index (0-5)
- Rendering a static shape (no morphing yet) based on current parameter

**Phase 3: Smooth Morphing** - This requires:
- Adding SpringValue for smooth parameter tracking
- Implementing vertex interpolation between shapes
- Updating vertices in `timerCallback()`, reading from SpringValue
- Optional subtle rotation during morph (track rotation angle separately)

**Phase 4: Constellation Points** - This requires:
- Drawing 6 dots in a circle around center
- Determining which dot is "active" based on Character (similar to shape index)
- Optional pulsing animation using sine wave based on time

**Phase 5: Premium Polish** - This requires:
- Using `juce::ColourGradient` for indigo→violet fill
- Adding glow effects (draw shape multiple times with increasing alpha?)
- Drawing text overlay showing Character % (read from APVTS)
- Audio-reactive brightness (read envelope from processor if we expose it)

**Critical RT-Safety Note:**

The task says "No allocations in paint() method" (line 99). This is crucial. Pre-allocate all data structures in the constructor or `resized()`. In `paint()`, only read from pre-allocated buffers and use stack variables. JUCE's Path objects are fine to use locally in `paint()` - they're designed for this. Just don't allocate vectors or arrays on the heap during rendering.

**Files to Modify:**

Only two files need changes (task lines 112-114):
1. `plugins/EngineField/Source/FieldEditorV2.h` - Add member variables, Timer inheritance, helper methods
2. `plugins/EngineField/Source/FieldEditorV2.cpp` - Implement timer callback, shape rendering, interpolation logic

The processor, parameters, and DSP files remain untouched (DSP is locked per CLAUDE.md).

**Why This Approach Works:**

This visualization is a metaphor, not a direct DSP representation. The user sees geometric shapes morphing as they adjust Character, which provides instant visual feedback about where they are in the 0-100% range. It's more intuitive than showing 6 dots moving on a unit circle (which would be accurate to the DSP but harder to parse at a glance).

The 6 shape states roughly divide the 0-100% range into sixths, mapping to the emotional progression of the filter's timbre. Triangle feels sharp and focused (low Character), octagon feels full and lush (high Character). The smooth morphing between shapes (via spring physics) mirrors the smooth coefficient interpolation happening in the DSP - both are critically damped systems that settle elegantly.

The constellation points (Phase 4) can either be purely decorative or could later represent the 6 actual filter poles (if we read `processor.getUIPoles()` and map them to constellation positions). For now, the task suggests they're just indicators of which "shape zone" is active.

### Technical Reference Details

#### Component Interfaces & Signatures

**FieldProcessor (existing):**
```cpp
class FieldProcessor : public juce::AudioProcessor {
    juce::AudioProcessorValueTreeState& getAPVTS() noexcept;
    const std::array<std::atomic<float>, 12>& getUIPoles() const noexcept; // 6 poles × 2 (r, theta)
};
```

**FieldEditorV2 (current):**
```cpp
class FieldEditorV2 : public juce::AudioProcessorEditor {
    explicit FieldEditorV2(FieldProcessor& processor);
    void paint(juce::Graphics& g) override;
    void resized() override;
private:
    FieldProcessor& processor;
};
```

**FieldEditorV2 (to be extended):**
```cpp
class FieldEditorV2 : public juce::AudioProcessorEditor, private juce::Timer {
    // ... existing ...
private:
    void timerCallback() override; // NEW - for animation

    // Shape geometry (pre-calculated in constructor)
    static constexpr int kNumShapes = 6;
    static constexpr int kMaxVertices = 8;
    std::array<std::array<float, kMaxVertices>, kNumShapes> shapeAngles_;

    // Animation state
    std::array<juce::Point<float>, kMaxVertices> currentVertices_;
    engine::ui::SpringValue<float> characterSmooth_{0.2f};

    // Rendering helpers
    void updateShapeGeometry();
    juce::Path createShapePath(const juce::Rectangle<int>& bounds) const;
};
```

#### Data Structures

**Shape Vertex Angles (pre-calculated constant):**
```cpp
// In constructor initialization:
// Triangle (3 vertices): angles at 0°, 120°, 240°
// Square (4 vertices): angles at 0°, 90°, 180°, 270°
// Pentagon (5 vertices): angles at 0°, 72°, 144°, 216°, 288°
// Hexagon (6 vertices): angles at 0°, 60°, 120°, 180°, 240°, 300°
// Heptagon (7 vertices): angles at 0°, 51.43°, 102.86°, 154.29°, 205.71°, 257.14°, 308.57°
// Octagon (8 vertices): angles at 0°, 45°, 90°, 135°, 180°, 225°, 270°, 315°
```

**Character Parameter Mapping:**
```cpp
// Character % → Shape Index
float characterPercent = ...; // 0-100
int shapeIndex = std::clamp((int)(characterPercent / 16.666f), 0, 5);
// 0-16.666% → 0 (triangle)
// 16.666-33.333% → 1 (square)
// 33.333-50% → 2 (pentagon)
// 50-66.666% → 3 (hexagon)
// 66.666-83.333% → 4 (heptagon)
// 83.333-100% → 5 (octagon)
```

**SpringValue Usage:**
```cpp
// In constructor:
characterSmooth_.setCurrentAndTarget(50.0f); // Start at default

// In timerCallback():
auto* param = processor.getAPVTS().getRawParameterValue(enginefield::params::characterId);
float targetCharacter = param->load(); // 0-100
characterSmooth_.setTarget(targetCharacter);
characterSmooth_.update(1.0f / 60.0f); // ~16.67ms
float smoothedCharacter = characterSmooth_.getValue();
```

#### Configuration Requirements

**CMakeLists.txt additions (if SpringValue not already included):**
The SpringValue.h header is already in the shared/ui directory. To use it, add to FieldEditorV2.cpp:
```cpp
#include "../../shared/ui/SpringValue.h"
```

No CMakeLists changes needed - it's a header-only template.

**Build configuration:**
- Requires JUCE 8.0.10+
- C++20 standard (already set in root CMakeLists.txt line 7)
- No additional dependencies

#### File Locations

**Implementation files:**
- Primary: `C:\plugin_dev\plugins\EngineField\Source\FieldEditorV2.h`
- Primary: `C:\plugin_dev\plugins\EngineField\Source\FieldEditorV2.cpp`

**Referenced headers (read-only):**
- Parameter access: `C:\plugin_dev\plugins\EngineField\Source\parameters.h`
- Processor: `C:\plugin_dev\plugins\EngineField\Source\FieldProcessor.h`
- Spring physics: `C:\plugin_dev\shared\ui\SpringValue.h`
- Colors (optional): `C:\plugin_dev\shared\ui\Colors.h`

**Testing:**
- Build standalone: `cmake --build build --config Release`
- Run: `build\plugins\EngineField\EngineField_artefacts\Release\Standalone\EngineField.exe`
- Load VST3 in DAW for hosted testing

**Key design constraints:**
- Fixed 400×600 size (FieldEditorV2.cpp line 8) - no resizing in Phase 1
- 60fps target (17ms timer interval)
- No heap allocations in `paint()` or `timerCallback()`
- Dark background `#1a1a2e` already established
- Header text "ENGINE:FIELD" already positioned

## Work Log

- [2025-10-18] Task created after design exploration with GPT-5 Pro concepts
- [2025-10-18] Removed alien UI components, established clean slate
- [2025-10-18] Verified basic rendering works (dark bg + text)
- [2025-10-19] Context manifest created - full DSP flow documented, animation patterns identified
