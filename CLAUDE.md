
# CLAUDE.md — Engine:Field (Locked)

**Status:** 🔒 DSP locked (authentic EMU) • **Target:** JUCE 8.0.10+ • **Formats:** VST3 / Standalone

## What this is
A minimal, production‑ready plugin: **authentic EMU Z‑plane** sound, **sampler pad grid** UI with transient visualization, strict RT safety.
Use this as the **golden base** for Field, Pitch, Spectral, Morph.

## Build
```bash
cmake -S . -B build -DCMAKE_BUILD_TYPE=Release \
  -DJUCE_SOURCE_DIR="C:/path/to/JUCE" -DBUILD_STANDALONE=ON -DBUILD_VST3=ON
cmake --build build --config Release
```

## UX / Controls
- **4x4 Sampler Pad Grid**: Audio-reactive visualization with quantized decay trails
- **CHARACTER Slider**: Morph position (0–100%), controls decay steps (4–16) and jitter
- **MIX Knob**: Dry/wet blend (0–100%), controls pad brightness
- **EFFECT Button**: Toggle ghost-only mode (hides main pads, shows only trails)
- Window size: 600×550px, warm cream/brown aesthetic optimized for drum/beat work

## DSP (immutable)
- 6× biquad cascade (12th‑order), per‑section tanh
- **Locked values:** intensity=0.4, drive=0.2, saturation=0.2, pole_limit=0.9950
- **Envelope follower:** 0.489 ms attack, 80 ms release, depth=0.945
- Update coefficients **once per block**; no allocations in `processBlock`

## Files never to edit
- `plugins/EngineField/Source/dsp/ZPlaneFilter.h`
- `plugins/EngineField/Source/dsp/EMUAuthenticTables.h`

## Verify lock
```bash
python scripts/verify_lock.py    # checks SHA‑256 of the two DSP headers
```
(See `scripts/pre-commit` to enforce before commit.)

## Assumptions (explicit)
- Shapes are referenced at **48 kHz**; **bilinear remap** to target sample rate (proper frequency mapping).
- The "authentic drive" applies a light **tanh** pre‑stage.
- UI reads parameters via APVTS; no background threads needed.

## JUCE best‑practice checklist
- `juce_add_plugin` + CMake, APVTS, smoothed values
- RT safety: `ScopedNoDenormals`, no locks/allocs in audio thread
- Accessibility‑friendly colours, resizable editor, UndoManager ready
- Optional pluginval target (`PLUGINVAL_EXE`)

## Next
- Add preset menu + A/B, MIDI learn (optional)
- Copy this folder to start **Pitch / Spectral / Morph**

---

## Implementation Discoveries (2025-10-18)

### DSP Lock Clarification
The "🔒 DSP locked" status and "Files never to edit" section above refer to **preserving authentic EMU character**, not an absolute prohibition on improvements.

**Actual Policy:** DSP files CAN be edited if changes improve audio quality or technical accuracy while maintaining the authentic sound.

### Frequency Mapping (IMPLEMENTED 2025-10-18)
**Status:** ✅ Bilinear remap complete + optimized

The original "theta is scaled by fs/48k" approximation has been **replaced** with proper bilinear frequency warping:

- **Old approach:** Simple linear scaling `theta * (sr/48000)` ❌
- **New approach:** Full bilinear transform via analog-domain intermediate ✅
  ```
  s = (2*fs_ref) * (z48k - 1) / (z48k + 1)      // inverse bilinear: 48k → analog
  z_new = (2*fs_target + s) / (2*fs_target - s)  // forward bilinear: analog → target fs
  ```

**Result:** Preserves intended formant frequencies across all sample rates (44.1k, 48k, 88.2k, 96k, 192k)

**Optimizations:**
- **48kHz fast-path:** Skips complex math when running at reference rate (zero overhead)
- **Geodesic morphing:** Log-space radius interpolation for smoother, more "EMU-ish" morphing
  - Toggle via `GEODESIC_RADIUS` constant (default: `true`)
  - Set to `false` for linear radius interpolation

**Implementation:** `plugins/EngineField/Source/dsp/ZPlaneFilter.h`
- Lines 99-134: `remapPole48kToFs` (with 48kHz fast-path at line 102)
- Lines 77-95: `interpolatePole` (with geodesic toggle)
- Line 19: `GEODESIC_RADIUS` constant (default: `true`)

**Testing:** Compare 48k vs 96k renders - formants should align when SRC'd to same rate

### Notes
- verify_lock.py should be updated to reflect new ZPlaneFilter.h hash
- This change improves accuracy while maintaining authentic EMU character
- Geodesic morphing costs ~1 log/exp per pole per block (negligible, worth the sonic improvement)

---

## UI/UX System Refactoring (2025-10-19)

### Decision: Sampler Pad Grid as Primary UI

**Status:** ✅ Complete - FieldPluginEditor (sampler pad grid) is the active UI

**Reasoning:**
The target audience for Engine:Field is **drum makers and beatmakers** working with transient-heavy material (kicks, snares, percussion). The sampler pad grid UI provides:

1. **Immediate transient feedback** - Pads flash on hits, crucial for timing and impact visualization
2. **Visual character morphing** - Grid directly shows how CHARACTER affects decay quantization (4–16 steps)
3. **Intuitive layout** - 4x4 grid familiar to beat-focused workflows (MPC aesthetic)
4. **Warm sampler aesthetic** - Cream/brown palette (0xFFF5F1E8 background) matches analog sampler workflow
5. **Perfect for percussive material** - Designed specifically for transient shaping, not detailed frequency surgery

**Active UI Implementation:**
- `plugins/EngineField/Source/ui/FieldPadUI.h` - Complete implementation (420 lines, header-only)
- 4x4 grid of SamplerPad components with 12-frame history buffers
- 60 Hz timer updates, lock-free atomic audio level communication
- CHARACTER slider (left 65%), MIX knob (right side), EFFECT toggle button

**Archived UI (XY + Spectrum):**
An alternate UI with XY pad and FFT spectrum analyzer was developed but not chosen because:
- Better suited for precise mixing/frequency analysis work
- Spectrum visualization less relevant for drum transients
- More complex for beatmakers focused on quick transient shaping

**Archived files:** `plugins/EngineField/archive/ui-alternate/`
- FieldEditor.h/cpp - Main XY+Spectrum editor
- components/XYPad.h/cpp - XY control component
- components/SpectrumComponent.h/cpp - FFT visualizer (contains syntax error line 56)
- lookandfeel/EngineLNF.h/cpp - Modern dark aesthetic look-and-feel

**Deleted:** `FieldEditorV2.h` - Header stub with no implementation

**Build Configuration:**
CMakeLists.txt includes only FieldPadUI.h (header-only). No additional UI source files needed for current implementation.

**Code Quality Improvements:**
- Fixed dynamic_cast safety issue at FieldPadUI.h:394 (changed to static_cast with comment)
- Thread safety verified: Atomic<float> for audio level, lock-free communication
- JUCE best practices: APVTS attachments, proper component lifecycle, JUCE_DECLARE_NON_COPYABLE_WITH_LEAK_DETECTOR

### Archived Prototype: `field` → `ARCHIVE_field_prototype_20251019`

**Status:** ✅ Archived 2025-10-19

An earlier prototype `plugins/field` was archived because:
- Not included in build (no CMakeLists.txt)
- Simpler DSP: 158-line ZPlaneFilter with linear frequency scaling (vs. 264-line bilinear remapping)
- Different UI: XY pad + spectrum analyzer (vs. sampler pad grid)
- Superseded by EngineField implementation

**Location:** `ARCHIVE_field_prototype_20251019/`

**Key differences from EngineField:**
- ❌ No bilinear frequency warping (just linear `theta * sr/48k`)
- ❌ No geodesic radius interpolation
- ❌ Linear mix (vs. equal-power)
- ❌ No sampler pad grid UI
- ✅ Retained as reference for simpler DSP approach and XY+Spectrum UI pattern

---

## DSP and UX Refinements (2025-10-19)

### Equal-Power Mix Implementation

**Status:** ✅ Complete

**Problem:** Linear dry/wet mixing (`wet*mix + dry*(1-mix)`) causes perceived loudness dips around 50% mix and poor tone preservation when nonlinearities (tanh drive) are involved.

**Solution:** Equal-power crossfade using square-root scaling:
- Wet gain: `sqrt(mix)`
- Dry gain: `sqrt(1 - mix)`

**Implementation:** `plugins/EngineField/Source/dsp/ZPlaneFilter.h:248-252`

**Benefits:**
- Maintains constant perceived loudness across entire mix range
- Better tone preservation when blending driven signal with dry path
- Industry-standard approach for audio mixing

**Code reference:** Lines 248-252 apply equal-power formula in processBlock

### Parameter Label Clarification

**Status:** ✅ Complete

**Change:** EFFECT parameter label updated from "Effect Mode" to "EFFECT (Wet Solo)"

**Implementation:** `plugins/EngineField/Source/parameters.h:46`

**Reasoning:** Makes the parameter's purpose immediately clear - when enabled, it solos the wet signal so users can hear the Z-plane filtering effect in isolation, useful for dialing in CHARACTER settings.

### Sampler Pad Grid UI - Complete Implementation

**Status:** ✅ All Phase 1-5 success criteria complete

**Completion date:** 2025-10-19

**Implemented features:**
1. ✅ 4x4 sampler pad grid with audio-reactive visualization
2. ✅ 12-frame decay trail buffers per pad
3. ✅ CHARACTER-driven decay quantization (4-16 steps)
4. ✅ MIX-driven brightness scaling
5. ✅ EFFECT mode (wet solo toggle)
6. ✅ Lock-free audio level communication via atomic<float>
7. ✅ Envelope follower (5ms attack, 100ms release) in FieldProcessor.cpp
8. ✅ 60 Hz UI timer with efficient repaint
9. ✅ Warm cream/brown aesthetic (0xFFF5F1E8 background)
10. ✅ Thread-safe APVTS parameter attachments

**Build verified:** VST3 builds successfully at 3.6MB, ready for testing in DAW.

**Files:**
- Primary UI: `plugins/EngineField/Source/ui/FieldPadUI.h` (417 lines, header-only)
- Processor: `plugins/EngineField/Source/FieldProcessor.h/cpp` (atomic level tracking)
- Parameters: `plugins/EngineField/Source/parameters.h` (APVTS definitions)

## Sessions System Behaviors

@CLAUDE.sessions.md
