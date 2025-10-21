# Engine:Field v1.0 — Authentic EMU Z-Plane Audio Plugin

A production-ready audio plugin featuring authentic EMU Z-plane filtering with real-time-safe DSP and an intuitive 4x4 sampler pad grid UI. Built with JUCE 8.0.10+ for VST3 and Standalone formats.

## What It Does

Engine:Field provides **authentic EMU character** through a 6-section biquad cascade (12th-order) with per-section tanh saturation. The Z-plane filtering preserves formant frequencies across all sample rates via bilinear frequency remapping, while the sampler pad grid UI gives immediate visual feedback on transient morphing and decay quantization—ideal for drum makers and beatmakers.

**Key Features:**
- 6× biquad cascade (12th-order) with proper bilinear frequency warping (44.1k to 192k)
- Geodesic morphing via log-space radius interpolation (smoother, more "EMU-ish" transitions)
- 4x4 sampler pad grid with audio-reactive visualization and decay trails
- CHARACTER slider (0–100%): controls Z-plane morphing and decay quantization (4–16 steps)
- MIX knob (0–100%): equal-power dry/wet blending (maintains loudness across full range)
- EFFECT button: wet solo mode for inspecting the filtered transient in isolation
- Output level control (−12 to +12 dB)
- Bypass and test tone (440 Hz) for reference work

## Build

### Requirements
- JUCE 8.0.10 or later
- CMake 3.24+
- C++ 20 compiler (MSVC 2022, Clang, GCC)

### Windows (MSVC)
```bash
cmake -S . -B build -G "Visual Studio 17 2022" \
  -DJUCE_SOURCE_DIR="C:/path/to/JUCE" \
  -DBUILD_VST3=ON -DBUILD_STANDALONE=ON \
  -DCMAKE_BUILD_TYPE=Release

cmake --build build --config Release
```

### macOS / Linux
```bash
cmake -S . -B build -DCMAKE_BUILD_TYPE=Release \
  -DJUCE_SOURCE_DIR=/path/to/JUCE \
  -DBUILD_VST3=ON -DBUILD_STANDALONE=ON

cmake --build build --config Release
```

### Build Output

**VST3 Plugin:**
- Windows: `build/plugins/EngineField/EngineField_artefacts/Release/VST3/EngineField.vst3`
- macOS: `build/plugins/EngineField/EngineField_artefacts/Release/VST3/EngineField.vst3`
- Linux: `build/plugins/EngineField/EngineField_artefacts/Release/VST3/EngineField.vst3`

**Standalone Application:**
- Windows: `build/plugins/EngineField/EngineField_artefacts/Release/Standalone/EngineField.exe`
- macOS: `build/plugins/EngineField/EngineField_artefacts/Release/Standalone/EngineField.app`
- Linux: `build/plugins/EngineField/EngineField_artefacts/Release/Standalone/EngineField`

### Optional: Validate with pluginval

```bash
cmake -S . -B build -DCMAKE_BUILD_TYPE=Release \
  -DJUCE_SOURCE_DIR="C:/path/to/JUCE" \
  -DPLUGINVAL_EXE="C:/path/to/pluginval.exe" \
  -DBUILD_VST3=ON -DBUILD_STANDALONE=ON

cmake --build build --config Release
cmake --build build --target pluginval_field
```

## UI & Controls

**Window:** 600×550 px, warm cream/brown aesthetic optimized for transient work

**Layout:**
- **Sampler Pad Grid** (4×4, center): Audio-reactive pads with decay trails; pad brightness follows CHARACTER and MIX
- **CHARACTER Slider** (left, 0–100%): Morphs Z-plane poles and quantizes decay steps
  - 0% = 4 decay steps, minimal morphing
  - 100% = 16 decay steps, maximum pole movement
- **MIX Knob** (right, 0–100%): Equal-power blending of filtered (wet) and dry signal
  - 0% = 100% dry
  - 100% = 100% wet (full effect)
- **EFFECT Button** (bottom-left): Toggles wet-solo mode to hear the filtered signal alone
- **Output Level** (bottom-right): −12 to +12 dB makeup gain
- **Bypass** (top menu): A/B instant comparison
- **Test Tone** (top menu): 440 Hz sine for reference work

## DSP Architecture

**Real-Time Safe:** No heap allocations in `processBlock`, lock-free atomic communication, ScopedNoDenormals denormal handling.

**Core Chain:**
1. **Pre-Filter Envelope Follower** — 0.489 ms attack, 80 ms release, 94.5% depth
2. **Z-Plane Biquad Cascade** — 6 sections (12th-order) with per-section tanh saturation
3. **Bilinear Frequency Remapping** — Preserves formant frequencies across sample rates
4. **Equal-Power Mix** — Dry/wet blending with `sqrt(mix)` / `sqrt(1-mix)` gains
5. **Output Gain** — User-controllable level makeup

**Locked DSP Values (Preserve Authentic Character):**
- Intensity: 0.4
- Drive: 0.2
- Saturation: 0.2
- Pole limit: 0.9950 (prevents metallic ringing)

**Frequency Mapping:**
- Reference sample rate: 48 kHz
- Supported rates: 44.1k, 48k, 88.2k, 96k, 192k
- Z-plane poles mapped via full bilinear transform (not linear scaling)
- 48 kHz fast-path: zero overhead when running at reference rate

**Geodesic Morphing:**
- Log-space radius interpolation enabled by default (`GEODESIC_RADIUS = true`)
- Smoother, more "EMU-ish" transient shaping compared to linear interpolation
- Configurable via `GEODESIC_RADIUS` constant in `ZPlaneFilter.h:19`

## Installation

See **INSTALL.md** for detailed VST3 and Standalone installation instructions per OS.

## Compatibility

- **VST3:** Windows 10+, macOS 10.13+, Linux (ALSA/PulseAudio)
- **Standalone:** All above platforms
- **DAWs:** Any DAW supporting VST3 (Reaper, Studio One, Cubase, etc.)
- **Host Formats:** Both buffered and real-time processing supported

## Technical Details

**Parameters (all DAW-automatable):**
- CHARACTER: 0.0–100.0% (default 50%)
- MIX: 0.0–100.0% (default 100%)
- Output: −12 to +12 dB (default 0 dB)
- Bypass: boolean
- Test Tone: boolean
- EFFECT (Wet Solo): boolean

**Performance:**
- CPU usage: ~2–4% per instance @ 48 kHz (release build on modern CPU)
- Memory footprint: ~4.8 MB plugin size
- Latency: 0 samples (zero latency filter)

**Thread Safety:**
- Audio thread: no locks, no allocations, ScopedNoDenormals enabled
- UI thread: lock-free atomic reads from audio thread for level tracking
- Parameter updates: smoothed via LinearSmoothedValue (50 ms ramp time)

## Development & Advanced Options

### Build Flags

- `BUILD_VST3` (ON/OFF): Build VST3 plugin format
- `BUILD_STANDALONE` (ON/OFF): Build standalone application
- `ENABLE_WARNINGS_AS_ERRORS` (ON/OFF, default OFF): Treat compiler warnings as errors (useful for CI)

### Verify DSP Lock

The core DSP files are protected from accidental modification:
```bash
python scripts/verify_lock.py
```

This validates SHA-256 checksums of:
- `plugins/EngineField/Source/dsp/ZPlaneFilter.h`
- `plugins/EngineField/Source/dsp/EMUAuthenticTables.h`

### Next Steps

- **Presets:** A/B preset switching and recall via host preset manager
- **MIDI Learn:** Assign any hardware knob to CHARACTER or MIX
- **Suite:** Copy this folder to create Pitch, Spectral, or Morph variants

## License

See LICENSE.txt for full terms.

## Support

For issues, feature requests, or technical questions, consult CLAUDE.md (developer reference) or contact support@engineaudio.com.
