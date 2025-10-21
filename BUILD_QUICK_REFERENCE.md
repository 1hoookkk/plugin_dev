# Build System Quick Reference Card

## One-Line Build Commands

### Windows (Release, VST3 + Standalone)
```powershell
$env:JUCE_SOURCE_DIR = "C:\path\to\JUCE"; cmake --preset windows-release; cmake --build --preset windows-release
```

### macOS (Release, VST3 + Standalone)
```bash
export JUCE_SOURCE_DIR=/path/to/JUCE; cmake --preset macos-release; cmake --build --preset macos-release
```

### Linux (Release, VST3 + Standalone)
```bash
export JUCE_SOURCE_DIR=/path/to/JUCE; cmake --preset linux-release; cmake --build --preset linux-release
```

---

## Available Presets

### Configure Presets (List all with `cmake --list-presets`)
```
windows-debug       Windows Debug (MSVC)
windows-release     Windows Release (MSVC, LTO)
macos-debug         macOS Debug (Clang)
macos-release       macOS Release (Clang, LTO)
linux-debug         Linux Debug (GCC/Clang)
linux-release       Linux Release (GCC/Clang, LTO)
```

---

## Essential Commands

| Task | Command |
|------|---------|
| **Configure** | `cmake --preset <preset>` |
| **Build** | `cmake --build --preset <preset>` |
| **Build (4 parallel jobs)** | `cmake --build --preset <preset> --parallel 4` |
| **Build (verbose)** | `cmake --build --preset <preset> --verbose` |
| **Clean rebuild** | `rm -rf build && cmake --preset <preset> && cmake --build --preset <preset>` |
| **List presets** | `cmake --list-presets` |
| **Validate (VST3)** | `cmake --build --preset <preset> --target pluginval_vst3` |

---

## Locate Binaries After Build

| Format | Path |
|--------|------|
| **VST3** | `build/<preset>/plugins/EngineField/EngineField_artefacts/VST3/EngineField.vst3` |
| **Standalone** | `build/<preset>/plugins/EngineField/EngineField_artefacts/Standalone/EngineField[.exe/.app]` |

---

## Environment Setup (First Time Only)

### Windows
```powershell
# Set environment variable permanently
[Environment]::SetEnvironmentVariable("JUCE_SOURCE_DIR", "C:\path\to\JUCE", "User")

# Or temporarily for current shell
$env:JUCE_SOURCE_DIR = "C:\path\to\JUCE"
```

### macOS / Linux
```bash
# Set in shell profile (~/.bashrc, ~/.zshrc)
export JUCE_SOURCE_DIR=/path/to/JUCE

# Or temporarily
export JUCE_SOURCE_DIR=/path/to/JUCE
```

---

## CI/Local Testing

### Test Single Platform Locally
```bash
# Test Windows Release build
cmake --preset windows-release -DTREAT_WARNINGS_AS_ERRORS=ON
cmake --build --preset windows-release

# Expected output:
# ✓ Compile successful
# ✓ VST3 binary created at build/windows-release/plugins/EngineField/EngineField_artefacts/VST3/
```

### Test All Platforms (Local CI Simulation)
```bash
for preset in windows-debug windows-release macos-debug macos-release linux-debug linux-release; do
  echo "Testing $preset..."
  cmake --preset $preset || exit 1
  cmake --build --preset $preset || exit 1
done
echo "All builds passed!"
```

---

## Compiler Flags Summary

| Platform | Release Flags | Debug Flags |
|----------|---------------|------------|
| **MSVC** | `/Ox /GL /LTCG` | `/Zi /Od /RTC1` |
| **Clang** | `-O3 -flto` | `-g -O0` |
| **GCC** | `-O3 -flto` | `-g -O0` |

**All platforms:** `-Wall -Wextra -Wpedantic` (warnings enabled, errors optional)

---

## Common Issues & Fixes

| Issue | Fix |
|-------|-----|
| "JUCE not found" | `cmake --preset <preset> -DJUCE_SOURCE_DIR=/path/to/juce` |
| "juceaide not found" | Verify JUCE installation path; check `JUCE/bin/juceaide[.exe]` exists |
| "pluginval failed" | Run `pluginval --validate path/to/plugin.vst3 --strictness-level 5` locally for details |
| Slow Windows build | Use Ninja: `cmake --preset windows-release -G Ninja` |
| Build takes >30s | Use parallel: `cmake --build --preset <preset> --parallel 8` |

---

## Enable Strict Mode (CI)

```bash
# Enable warnings-as-errors for stricter validation
cmake --preset <preset> -DTREAT_WARNINGS_AS_ERRORS=ON
cmake --build --preset <preset>
```

---

## Memory/Performance Options

| Option | Command | Purpose |
|--------|---------|---------|
| **Address Sanitizer** | `-DENABLE_ASAN=ON` | Detect memory issues (Linux/macOS Debug only) |
| **LTO (Link-Time Optimization)** | Automatic in Release | Faster startup, smaller binary |
| **Fast Math** | Built-in Release | Audio DSP optimization |

---

## GitHub Actions CI

**Automatic triggers:**
- Push to `main`, `master`, or `develop` branch
- Pull requests to these branches
- Manual trigger via "Actions" tab

**CI validates:**
- ✓ Windows (MSVC, Debug + Release)
- ✓ macOS (Clang, Debug + Release)
- ✓ Linux (GCC, Debug + Release)
- ✓ pluginval (strictness=5) on all platforms
- ✓ Artifacts uploaded for inspection

**View results:** `.github/workflows/build.yml` in GitHub Actions tab

---

## Preset Configuration Details

### Windows
- Generator: Visual Studio 17 2022
- Architecture: x64
- CRT: MultiThreaded (static) `/MD` (Release), `/MDd` (Debug)

### macOS
- Generator: Ninja
- Deployment Target: 10.13 (configurable)
- Flags: `-fvectorize -fslp-vectorize` (Clang optimizations)

### Linux
- Generator: Ninja
- ISA: `-march=x86-64` (portable, baseline)
- Optional: `-fsanitize=address` (ASAN, if enabled)

---

## Building for Distribution

1. **Build Release variant** for your platform
2. **Run pluginval** to validate: `pluginval --strictness-level 5 --validate <plugin.vst3>`
3. **Test in DAW** (Reaper, Studio One, etc.)
4. **Sign/notarize** (macOS only, required for distribution):
   ```bash
   codesign -s - EngineField.vst3
   ```
5. **Package** binaries + LICENSE + README for distribution

---

## Documentation

- **Full documentation:** `C:\plugin_dev\BUILD.md`
- **CMake reference:** `C:\plugin_dev\CMakeLists.txt`
- **Presets definition:** `C:\plugin_dev\CMakePresets.json`
- **CI workflow:** `C:\plugin_dev\.github\workflows\build.yml`

---

**Engine:Field Build System**
*Last Updated: 2025-10-22*
