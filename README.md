
# Engine:Field (JUCE 8) — Production Template

A compact, real‑time‑safe audio plugin showcasing **authentic EMU Z‑plane** filtering with
a **modern, minimal UI** (XY pad + spectrum visualiser). Use this as the golden base for the suite.

## Build (Windows / macOS / Linux)

```bash
cmake -S . -B build -DCMAKE_BUILD_TYPE=Release \
  -DJUCE_SOURCE_DIR="C:/path/to/JUCE" \
  -DBUILD_STANDALONE=ON -DBUILD_VST3=ON

cmake --build build --config Release
```

Artifacts:
- VST3: `build/plugins/EngineField/EngineField_artefacts/Release/VST3/EngineField.vst3`
- Standalone: `build/plugins/EngineField/EngineField_artefacts/Release/Standalone/EngineField.exe`

> Optional: define `PLUGINVAL_EXE` and run `cmake --build build --target pluginval_field`

## UI
- **XY Pad**: X → *Character* (morph 0–100%), Y → *Mix* (dry/wet 0–100%)
- **Output**: -12..+12 dB
- **Bypass**: click to A/B instantly (micro crossfade to avoid clicks)
- **Spectrum**: live magnitude overlay for “heard not explained” UX

## DSP (LOCKED)
- 6-section cascade (12th order), per‑section tanh saturation
- **Authentic defaults**: intensity=0.4, drive=0.2, saturation=0.2
- **Pole limit**: 0.9950 (prevents metallic ringing)
- **Envelope follower**: 0.5 ms attack, 80 ms release, 94.5% depth → transient‑aware morphing
- Coefficients recomputed **once per block**

Run `python scripts/verify_lock.py` to validate DSP headers.

## Notes
- Requires JUCE 8.0.10+
- Real‑time policy: no heap allocs in `processBlock`, no locks, no exceptions on the audio thread
