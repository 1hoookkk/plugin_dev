# CLAUDE.md — Engine:Field

**Status:** v1.0.1 Released • **JUCE:** 8.0.10+ • **Formats:** VST3 / Standalone

## Overview
Authentic EMU Z‑plane filtering plugin with waveform visualization UI. 6-stage biquad cascade (12th-order IIR) with bilinear frequency warping, geodesic pole interpolation, and equal-power mixing. Strict RT safety.

## Build
```bash
# Using CMake presets (recommended)
cmake --preset windows-release
cmake --build --preset windows-release

# Or traditional approach
cmake -S . -B build -DCMAKE_BUILD_TYPE=Release \
  -DCMAKE_PREFIX_PATH="C:/JUCE/_install" -DBUILD_STANDALONE=ON -DBUILD_VST3=ON
cmake --build build --config Release
```

## Active UI
**FieldWaveformEditor** (`plugins/EngineField/Source/ui/FieldWaveformUI.h/cpp`)
- 420×560px retro waveform visualization (60 bars, blue/yellow/green palette)
- Interactive controls: MIX slider, CHARACTER slider, EFFECT button (v1.0.1)
- APVTS attachments for two-way parameter sync with DAW automation
- Custom FieldLookAndFeel for retro pixel-perfect rendering
- Standalone mode fully functional

## DSP Architecture
**ZPlaneFilter** (locked for authentic EMU character):
- 6× biquad cascade, per-section tanh saturation (0.2)
- **Locked values:** intensity=0.4, drive=0.2, pole_limit=0.9950
- **Envelope follower:** 0.489ms attack, 80ms release, **depth=0.75** (modulates CHARACTER ±15%, v1.0.1)
- Bilinear frequency warping (48kHz ref → target SR) with fast-path optimization
- Geodesic (log-space) radius interpolation - toggle via `GEODESIC_RADIUS` constant
- Equal-power dry/wet mixing: `wetGain=sqrt(mix)`, `dryGain=sqrt(1-mix)`
- Coefficient update once per block, per-sample filtering loop

**Files never to edit without maintaining authentic sound:**
- `plugins/EngineField/Source/dsp/ZPlaneFilter.h`
- `plugins/EngineField/Source/dsp/EMUAuthenticTables.h`

## Parameters (APVTS)
- **CHARACTER** (0-100%, default 50%): Morph between VOWEL_A/VOWEL_B shapes
- **MIX** (0-100%, default 100%): Dry/wet blend
- **EFFECT** (bool): Wet solo mode (ignores MIX, 100% wet)
- **OUTPUT** (-12 to +12 dB): Makeup gain
- **BYPASS** (bool): True bypass

## Lock-Free Audio↔UI Communication
**Audio thread writes, UI reads:**
- `currentLevel` atomic: Envelope follower for meters
- `uiPoles_[12]` atomics: 6 pole pairs (r, θ) for visualization
- `waveformPeaks_[60]` atomics: Circular buffer for bar display
- `uiWaveformFifo_`: SPSC FIFO for waveform data stream

**RT Safety:**
- `ScopedNoDenormals` in processBlock
- Zero allocations in audio thread
- Pre-allocated dry buffer (2, 2048), conditional resize only if needed
- Parameter pointer caching (no APVTS tree traversal) with `memory_order_relaxed`
- Envelope coefficients precomputed (no per-sample `exp()`)
- `memory_order_relaxed` for UI atomics (no ordering needed)

## Known Issues

### 1. FieldWaveformUI Missing Controls ✅ FIXED in v1.0.1
**Problem (v1.0):** UI displayed parameter values but provided NO controls to change them.

**Solution (v1.0.1):** Added interactive JUCE components with custom FieldLookAndFeel:
- MIX slider (horizontal, pixel-perfect yellow indicator)
- CHARACTER slider (horizontal, matches PNG design)
- EFFECT button (toggle, solos wet signal when ON)
- All controls use APVTS attachments for proper two-way parameter sync
- Standalone mode now fully usable

**Files Modified:**
- `plugins/EngineField/Source/ui/FieldWaveformUI.h` - Added component declarations
- `plugins/EngineField/Source/ui/FieldWaveformUI.cpp` - Initialized controls + layout in resized()
- `plugins/EngineField/Source/ui/FieldLookAndFeel.h` - Custom retro LookAndFeel (already existed)

### 2. Overly Aggressive DSP at Defaults ✅ FIXED in v1.0.1
**Problem (v1.0):** CHARACTER=50%, MIX=100% produced excessive resonance on drum transients.

**Solution (v1.0.1):** Reduced envelope depth to 0.75 (was 0.945):
- CHARACTER=50% now modulates ±15% (was ±18.9%)
- Balanced transient processing at default settings
- User can easily dial in desired intensity
- Maintains authentic EMU character (pole_radius=0.995, 6-stage cascade)

**File Modified:**
- `plugins/EngineField/Source/FieldProcessor.cpp:44` - `env_.setDepth(0.75f)`

### 3. RT-Safety & Performance Optimizations ✅ APPLIED (post-v1.0.1)
**Problem:** hot-path-guardian audit identified CPU waste and potential RT violations

**Solutions Applied:**
- **Patch 1**: Cache testTone parameter pointer (FieldProcessor.cpp:96) - eliminates APVTS tree traversal
- **Patch 2**: Precompute envelope coefficients (EnvelopeFollower.h) - ~95% CPU reduction (per-sample `exp()` → parameter-change-rate)
- **Patch 3**: Preallocate dry buffer (FieldProcessor.cpp:38) - defense against prepareToPlay allocation
- **UI Hotfix B**: Viewport-only repaint + cached path/baseline - 97% reduction in draw calls per frame

**Impact:**
- processBlock: ~2-5% CPU reduction
- Envelope follower: ~95% CPU reduction (7.2M → 0.3M cycles/sec @ 48kHz)
- UI rendering: ~50% CPU reduction (viewport repaint + cached resources)

**Files Modified:**
- `plugins/EngineField/Source/FieldProcessor.{h,cpp}` - Cached pointers, preallocated buffer
- `plugins/EngineField/Source/dsp/EnvelopeFollower.h` - Precomputed coefficients
- `plugins/EngineField/Source/ui/FieldWaveformUI.{h,cpp}` - Cached rendering resources

### 4. Build System ✅ WORKING
- JUCE 8.0.10 + CMake presets functional
- VST3/Standalone targets build without errors
- Asset embedding (1.svg, 2.svg) working via `juce_add_binary_data`

### 5. Documentation Drift
- CLAUDE.md references "FieldPadUI" as active editor (incorrect - that's old UI)
- Actual active: FieldWaveformUI (FieldProcessor.cpp:227)
- Update references to current implementation

## Technical References
- Bilinear transform: `ZPlaneFilter.h:99-134` (`remapPole48kToFs`)
- Geodesic interpolation: `ZPlaneFilter.h:77-95` (`interpolatePole`)
- Equal-power mix: `ZPlaneFilter.h:260-262`
- Envelope follower: `FieldProcessor.cpp:42-44`
- UI data flow: `FieldProcessor.h:50-76` (lock-free atomics)

## v1.0.1 Release Checklist
- [x] Implement FieldWaveformUI controls (sliders, knobs, APVTS attachments)
- [x] Calibrate envelope depth to 0.75 for balanced defaults
- [x] Verify EFFECT toggle solos wet signal correctly (FieldProcessor.cpp:133)
- [x] Update CLAUDE.md to reflect v1.0.1 shipped
- [x] Build successfully (VST3: 5.8MB, Standalone: 6.9MB)
- [x] Verify standalone mode has functional controls
- [x] Clean repository hygiene (remove duplicates, update gitignore)
- [ ] Tag release v1.0.1 and push

## Sessions System
@CLAUDE.sessions.md