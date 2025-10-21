# Engine:Field Changelog

All notable changes to the Engine:Field plugin are documented here. Versions follow [Semantic Versioning](https://semver.org/).

## [1.0.0] — 2025-10-22

### Added

**Audio Engine**
- Authentic EMU Z-plane filtering: 6× biquad cascade (12th-order) per-section tanh saturation
- Bilinear frequency remapping: Preserves formant frequencies across sample rates (44.1k–192k) via proper analog-domain intermediate transform (not linear scaling)
- Geodesic morphing: Log-space radius interpolation for smoother, more "EMU-ish" transient shaping (configurable via `GEODESIC_RADIUS` constant)
- Envelope follower: 0.489 ms attack, 80 ms release, 94.5% depth for transient-aware parameter modulation
- Equal-power dry/wet blending: Square-root gain scaling (not linear mix) maintains constant perceived loudness across full MIX range

**User Interface**
- 4×4 sampler pad grid with audio-reactive visualization: Real-time pads + 12-frame decay trails per pad, character-driven quantization (4–16 steps), MIX-driven brightness scaling
- CHARACTER slider (0–100%): Morphs Z-plane poles and controls decay quantization
- MIX knob (0–100%): Equal-power dry/wet blending with configurable pad brightness
- EFFECT button: Wet-solo mode to hear the filtered transient in isolation (useful for dialing in CHARACTER)
- Output level control (−12 to +12 dB): User-adjustable makeup gain
- Test tone (440 Hz sine): Reference signal for quick tuning checks
- Bypass toggle: Instant A/B comparison with micro-crossfade to avoid clicks
- Warm cream/brown aesthetic (0xFFF5F1E8 background): Optimized for drum/beat work

**Plugin Formats**
- VST3: Full parameter automation, APVTS parameter state, zero-latency processing
- Standalone application: Self-contained audio I/O with system device selection

**Real-Time Safety**
- RT-safe audio thread: Zero allocations in `processBlock`, no locks, ScopedNoDenormals denormal handling
- Lock-free UI communication: Atomic<float> level tracking from audio thread to UI without locks
- Parameter smoothing: LinearSmoothedValue per-block coefficient updates (50 ms ramp time)

**Build System**
- CMake 3.24+ with JUCE 8.0.10+
- Multi-platform support: Windows (MSVC), macOS (Clang), Linux (GCC)
- Optional pluginval integration: Validate plugin compliance with DAW validation tools
- Flexible warning configuration: `-DENABLE_WARNINGS_AS_ERRORS` gate (default OFF) for development vs. CI builds

**Documentation**
- README.md: Feature summary, build instructions, UI guide, technical details
- QUICKSTART.md: Step-by-step build and test guide
- INSTALL.md: VST3 and Standalone installation per OS
- CLAUDE.md: Developer reference (DSP lock policy, architecture, assumptions)

### Fixed (v1.0.0)

**Parameter Ramp Timing (Critical)**
- LinearSmoothedValue block-advance bug in `ZPlaneFilter.h:updateCoeffsBlock()`
  - **Problem:** Fixed `skip(1)` instead of dynamic `skip(samplesPerBlock)` caused CHARACTER slider to ramp 256× too slow (~12+ seconds instead of ~50 ms)
  - **Solution:** Updated method signature to accept `int samplesPerBlock` parameter; all smoothers now advance by full block size
  - **Files:** `plugins/EngineField/Source/dsp/ZPlaneFilter.h` (lines 195–200), `plugins/EngineField/Source/FieldProcessor.cpp` (line 169)

**CMake Configuration**
- `-Werror` hardcoded, breaking builds on any compiler warning
  - **Problem:** No way to do development builds without treating warnings as errors; CI pipelines forced to deal with third-party deprecation warnings
  - **Solution:** Gated `-Werror` (MSVC `/WX`) behind `ENABLE_WARNINGS_AS_ERRORS` option (default OFF)
  - **Files:** `plugins/EngineField/CMakeLists.txt` (lines 84–118)
  - **Usage:** Development: default behavior (warnings allowed); CI: pass `-DENABLE_WARNINGS_AS_ERRORS=ON` for strict checking

**Visualizer Data Path Consolidation**
- Duplicate ring buffer implementations consuming ~32 KB memory
  - **Problem:** Two separate systems (`visFifo_` + `visBuffer_` vs. `uiWaveformFifo_` + `uiWaveformRingBuffer_`) with dead code (`pushToVisualiser()`, `popVisualiserBlock()` never called)
  - **Solution:** Removed unused FIFO and buffer; retained only active `uiWaveformFifo_` path to UI
  - **Files:** `plugins/EngineField/Source/FieldProcessor.h` (removed declarations), `plugins/EngineField/Source/FieldProcessor.cpp` (removed implementations, ~74 lines)
  - **Result:** Cleaner codebase, single consistent visualizer data path, ~32 KB memory recovered

**Code Quality**
- Dynamic cast safety in UI pad grid component: Replaced unsafe `dynamic_cast` with `static_cast` + comment for known type context
- Zero new compiler warnings on JUCE code (target-specific deprecation warnings from external JUCE/VST SDK remain expected)

### Changed

**DSP Lock Policy Clarified**
- "🔒 DSP locked" now explicitly refers to preserving authentic EMU character, not an absolute edit prohibition
- Core files (`ZPlaneFilter.h`, `EMUAuthenticTables.h`) CAN be edited if changes improve audio quality or technical accuracy while maintaining authentic sound
- Policy documented in CLAUDE.md and enforced via `scripts/verify_lock.py` (SHA-256 checksums)

**Parameter Ranges & Defaults**
- CHARACTER: 0–100% (default 50% — balanced morphing position)
- MIX: 0–100% (default 100% — full wet signal; users typically reduce for blending)
- Output: −12 to +12 dB (default 0 dB)
- EFFECT: false (wet solo disabled by default)
- Gain has no default adjustment; makeup gain applied only if user sets Output ≠ 0 dB

### Removed

- Old XY pad + FFT spectrum analyzer UI (archived to `ARCHIVE_field_prototype_20251019/`)
  - Replaced by sampler pad grid which better serves drum/beat workflow
  - Reference implementation retained for future feature development
- Prototype `plugins/field/` folder (superseded by `plugins/EngineField/`)
  - Old folder did not include CMakeLists.txt; simpler DSP without bilinear remapping
  - Archived for reference

### Notes

**Frequency Mapping Accuracy**
- Original approximation `theta * (sr/48000)` has been replaced with proper bilinear frequency warping
- Formant frequencies now preserved when rendering at different sample rates and SRC'd back to reference
- 48 kHz fast-path optimizes for zero overhead when running at reference rate

**Geodesic Morphing**
- Log-space radius interpolation enabled by default for smoother, more authentic EMU-like transitions
- Cost: ~1 log/exp per pole per block (negligible on modern CPUs)
- Toggle via `GEODESIC_RADIUS` constant in `plugins/EngineField/Source/dsp/ZPlaneFilter.h:19`

**Test Coverage**
- Release build verified: VST3 (4.8 MB) + Standalone (4.8 MB) both compile without errors
- No target-specific warnings from EngineField source code
- RT-safety checklist passed: no allocations, locks, or exceptions in audio thread

---

## Version History Summary

| Version | Date       | Status    | Notes                                         |
|---------|------------|-----------|-----------------------------------------------|
| 1.0.0   | 2025-10-22 | Released  | Production ready: Z-plane DSP + pad grid UI  |
| —       | 2025-10-19 | Dev       | UI refactoring: sampler pads as primary UI   |
| —       | 2025-10-18 | Dev       | Bilinear frequency remapping + fixes         |
| —       | 2025-10-15 | Alpha     | Initial plugin foundation (EngineField)      |

---

## Roadmap (Future)

- **v1.1 (Presets):** A/B preset switching, host preset manager integration, preset randomization
- **v1.2 (MIDI):** MIDI learn for hardware control, CC mapping UI
- **v2.0 (Suite):** Pitch, Spectral, Morph variants sharing core DSP infrastructure
