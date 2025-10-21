# Engine:Field Build System Documentation

## Overview

This is a production-ready, multi-platform build system for **Engine:Field**, an authentic EMU Z-plane audio plugin. The build system runs identically on macOS, Windows (MSVC), and Linux (GCC/Clang) using CMake 3.24+.

### Key Features

- **Cross-Platform**: Builds on Windows (MSVC 2022), macOS (10.13+), and Linux (Ubuntu 20.04+)
- **Optimized**: LTO in Release builds, fast-math flags, platform-specific vectorization
- **Validated**: Automatic pluginval (strictness=5) validation in CI
- **Developer-Friendly**: Clear presets with `-Werror` toggle, sanitizer options
- **Plugin Formats**: VST3 and Standalone (both enabled by default)

---

## Prerequisites

### All Platforms

- **CMake 3.24+** — Download from [cmake.org](https://cmake.org/download/)
- **JUCE 8.0.10+** — Get from [juce.com](https://juce.com/download)
- **C++ Compiler**:
  - Windows: Visual Studio 2022 Community (or clang-cl)
  - macOS: Apple Clang (via Xcode Command Line Tools)
  - Linux: GCC 10+ or Clang 12+

### Windows

```powershell
# Install Visual Studio Build Tools or Community Edition
# Ensure "Desktop development with C++" workload is selected

# Alternative: Install LLVM/Clang for clang-cl builds
# choco install llvm
```

### macOS

```bash
# Install Xcode Command Line Tools
xcode-select --install

# Install Homebrew (if not present)
/bin/bash -c "$(curl -fsSL https://raw.githubusercontent.com/Homebrew/install/HEAD/install.sh)"

# Install build tools
brew install cmake ninja
```

### Linux (Ubuntu 20.04+)

```bash
# Install build essentials
sudo apt-get update
sudo apt-get install -y \
  build-essential cmake ninja-build \
  libx11-dev libxrandr-dev libxinerama-dev \
  libxcursor-dev libxi-dev libgl1-mesa-dev \
  libfreetype6-dev libfontconfig1-dev \
  libasound2-dev libdbus-1-dev libglib2.0-dev
```

---

## Quick Start

### 1. Set JUCE Location

Choose one method:

**Option A: Environment Variable**
```bash
# Windows (PowerShell)
$env:JUCE_SOURCE_DIR = "C:\path\to\JUCE"

# macOS/Linux (Bash)
export JUCE_SOURCE_DIR=/path/to/JUCE
```

**Option B: CMake Cache Variable (via preset)**
Edit `CMakePresets.json`, line 19:
```json
"JUCE_SOURCE_DIR": "/your/juce/path"
```

### 2. Configure & Build

**Windows (MSVC, Release):**
```powershell
cmake --preset windows-release
cmake --build --preset windows-release
```

**macOS (Clang, Release):**
```bash
cmake --preset macos-release
cmake --build --preset macos-release
```

**Linux (GCC, Release):**
```bash
cmake --preset linux-release
cmake --build --preset linux-release
```

### 3. Locate Binaries

- **VST3 Plugin**: `build/plugins/EngineField/EngineField_artefacts/VST3/EngineField.vst3`
- **Standalone**: `build/plugins/EngineField/EngineField_artefacts/Standalone/EngineField`

---

## Build Presets

### Configure Presets (CMake configuration)

| Name | Platform | Build Type | Compiler | Notes |
|------|----------|-----------|----------|-------|
| `windows-debug` | Windows | Debug | MSVC 2022 | Symbols, no optimization |
| `windows-release` | Windows | Release | MSVC 2022 | Aggressive optimization + LTO |
| `macos-debug` | macOS | Debug | Apple Clang | Debug symbols, no optimization |
| `macos-release` | macOS | Release | Apple Clang | LTO + vectorization |
| `linux-debug` | Linux | Debug | GCC/Clang | Debug symbols, optional ASAN |
| `linux-release` | Linux | Release | GCC/Clang | LTO + march=x86-64 |

### Build Presets (CMake build step)

Use matching preset names (e.g., `windows-release` configure → `windows-release` build).

```bash
cmake --build --preset <build-preset> --parallel <N>
```

---

## Command Reference

### Configure (One-Time Setup)

```bash
# Windows
cmake --preset windows-release -DJUCE_SOURCE_DIR="C:/path/to/JUCE"

# macOS
cmake --preset macos-release -DJUCE_SOURCE_DIR="/path/to/JUCE"

# Linux
cmake --preset linux-release -DJUCE_SOURCE_DIR="/path/to/JUCE"
```

### Build (Incremental)

```bash
# Build with automatic parallelism (CMake detects CPU count)
cmake --build --preset <build-preset>

# Build with explicit parallel jobs
cmake --build --preset <build-preset> --parallel 4

# Verbose output (for debugging)
cmake --build --preset <build-preset> --verbose
```

### Clean Rebuild

```bash
# Full rebuild (delete build directory)
rm -rf build
cmake --preset <preset-name>
cmake --build --preset <build-preset>
```

### Validate with pluginval

```bash
# Requires: PLUGINVAL_EXE environment variable set

# Windows
set PLUGINVAL_EXE=C:\path\to\pluginval.exe
cmake --preset windows-release
cmake --build --preset windows-release
cmake --build --preset windows-release --target pluginval_vst3

# macOS/Linux
export PLUGINVAL_EXE=/path/to/pluginval
cmake --preset macos-release
cmake --build --preset macos-release
cmake --build --preset macos-release --target pluginval_vst3
```

### Enable Warnings as Errors (CI Mode)

```bash
# Add to any preset configuration
cmake --preset <preset-name> -DTREAT_WARNINGS_AS_ERRORS=ON
cmake --build --preset <build-preset>
```

### Enable Address Sanitizer (Debug only, Linux/macOS)

```bash
cmake --preset linux-debug -DENABLE_ASAN=ON
cmake --build --preset linux-debug
# Run binary with ASAN output
./build/plugins/EngineField/EngineField_artefacts/Standalone/EngineField
```

---

## Build Configuration Details

### Compiler Flags by Platform

#### Windows (MSVC 2022)

**All Builds:**
- `/W4` — Warning level 4
- `/permissive-` — Standards conformance
- `/Zc:__cplusplus` — Correct `__cplusplus` macro
- `/fp:fast` — Fast floating-point mode

**Release:**
- `/Ox /Ob2 /Oi /Ot` — Maximum optimization
- `/GL /LTCG` — Link-time code generation
- `/MD` — Dynamic CRT

**Debug:**
- `/Zi /Od` — Debug info, no optimization
- `/RTC1` — Runtime checks
- `/MDd` — Debug CRT

#### macOS (Apple Clang)

**All Builds:**
- `-fvectorize -fslp-vectorize` — Vectorization
- `-ffast-math -fno-finite-math-only` — Fast math (safe for audio)
- Deployment target: 10.13 (binary compatibility)

**Release:**
- `-O3 -flto` — Max optimization + Link-Time Optimization
- `-DNDEBUG` — Release mode

**Debug:**
- `-g -O0` — Debug symbols, no optimization

#### Linux (GCC/Clang)

**All Builds:**
- `-ffast-math -fno-finite-math-only` — Fast math
- `-march=x86-64` — Baseline ISA (portable across systems)

**Release:**
- `-O3 -flto` — Max optimization + LTO
- `-DNDEBUG`

**Debug:**
- `-g -O0`
- Optional: `-fsanitize=address` (if `ENABLE_ASAN=ON`)

### Warning Configuration

**Enabled by default** (all platforms):
- `-Wall -Wextra -Wpedantic` (GCC/Clang)
- `/W4` (MSVC)
- Platform-specific checks for audio code

**Opt-in `-Werror`** (warnings → errors):
```bash
cmake --preset <preset> -DTREAT_WARNINGS_AS_ERRORS=ON
```
This is enabled in CI workflows automatically.

**Suppressed false positives** (JUCE compatibility):
- `-Wno-c++98-compat`
- `-Wno-unknown-pragmas`
- `-Wno-deprecated-declarations`
- MSVC: `/wd4458` (declaration hides member), `/wd4275` (non-DLL-interface base)

### LTO (Link-Time Optimization)

**Enabled in Release builds** for:
- Faster startup (better DSP loading in DAWs)
- Smaller binary size (~5-10% reduction)
- No runtime overhead once linked

**Why LTO is safe for plugins:**
- Performed at link time, not requiring extra runtime complexity
- Tested extensively in production audio plugins
- No denormal handling or RT-safety implications

**Performance impact:**
- Link time: +10-20% slower than non-LTO Release builds
- Runtime: Negligible overhead (often faster due to better inlining)

---

## Directory Structure

```
C:\plugin_dev\
├── CMakeLists.txt                 # Root CMake configuration
├── CMakePresets.json              # Build presets (platforms × build types)
├── BUILD.md                       # This file
├── JUCE/                          # JUCE framework (git-ignored, set via JUCE_SOURCE_DIR)
├── plugins/
│   └── EngineField/
│       ├── CMakeLists.txt         # Plugin build configuration
│       ├── Source/
│       │   ├── PluginMain.cpp
│       │   ├── FieldProcessor.h/cpp
│       │   ├── parameters.h
│       │   └── dsp/
│       │       ├── ZPlaneFilter.h        # Authentic EMU filter (locked)
│       │       ├── EMUAuthenticTables.h  # Locked tables
│       │       └── EnvelopeFollower.h
│       └── EngineField_artefacts/       # Built binaries (after build)
│           └── VST3/EngineField.vst3
├── .github/
│   └── workflows/
│       └── build.yml              # GitHub Actions multi-platform CI
└── build/                         # CMake build directory (git-ignored)
    ├── windows-debug/
    ├── windows-release/
    ├── macos-debug/
    ├── macos-release/
    ├── linux-debug/
    └── linux-release/
```

---

## GitHub Actions CI/CD

The `.github/workflows/build.yml` workflow automatically:

1. **Builds** on Windows (MSVC), macOS (Clang), and Linux (GCC)
2. **Runs debug and release variants** for each platform
3. **Downloads & installs pluginval**
4. **Validates VST3 plugin** with strictness level 5 (strict checks)
5. **Uploads artifacts** for download and inspection
6. **Fails the workflow** if pluginval validation fails (non-negotiable)

### Triggering CI

- **Push to main/master/develop** — Automatically run
- **Pull request** — Automatically run
- **Manual dispatch** — GitHub Actions tab → "Run workflow"

### CI Matrix

| OS | Compiler | Build Types | Format |
|---|---|---|---|
| Windows | MSVC 2022 | Debug, Release | VST3 + Standalone |
| macOS | Apple Clang | Debug, Release | VST3 + Standalone |
| Linux | GCC | Debug, Release | VST3 + Standalone |

---

## Troubleshooting

### Issue: "JUCE not found"

**Solution:**
```bash
# Set JUCE_SOURCE_DIR before configure step
cmake --preset windows-release -DJUCE_SOURCE_DIR="C:/path/to/JUCE"

# Or add to environment
export JUCE_SOURCE_DIR=/path/to/JUCE
```

### Issue: "juceaide not found"

**Solution:**
juceaide is auto-resolved from JUCE installation. If not found:
1. Verify JUCE_SOURCE_DIR points to valid JUCE installation
2. Check that `JUCE/bin/juceaide[.exe]` exists in JUCE folder

### Issue: Slow builds on Windows

**Solution:**
- Use Ninja instead of Visual Studio generator (faster parallel builds):
  ```powershell
  # Edit CMakePresets.json: change "Visual Studio 17 2022" to "Ninja"
  cmake --preset windows-release -G Ninja
  ```
- Enable parallel builds: `cmake --build --preset windows-release --parallel 8`

### Issue: "pluginval failed" in CI

**Common causes:**
- Missing plugin identifier metadata
- Incorrect JUCE configuration
- DSP processing errors

**Debug locally:**
```bash
pluginval --strictness-level 5 --validate path/to/EngineField.vst3 --timeout-ms 120000
```

### Issue: "Warnings as errors" failing build

**Solution:**
Fix warnings or suppress if false positive:
```bash
# Disable warnings-as-errors for investigation
cmake --preset <preset> -DTREAT_WARNINGS_AS_ERRORS=OFF
cmake --build --preset <build-preset>
```

---

## Performance Profiling

### Debug vs. Release

- **Debug builds** are ~3-5x slower (no optimization, debug info)
- **Release builds** are production-ready (LTO, aggressive optimization)
- For plugin testing in DAW, always use Release builds

### Measuring Build Time

```bash
# Measure configure time
time cmake --preset linux-release

# Measure build time
time cmake --build --preset linux-release --parallel $(nproc)
```

Expected times (Release, incremental build on modern CPU):
- **Configure:** 2-5 seconds
- **Build:** 5-15 seconds (initial: 20-30 seconds)
- **Incremental rebuild:** <2 seconds (typical)

### Profile binary startup

In DAW, monitor plugin loading time:
- Release build (LTO): 50-100ms
- Debug build: 200-500ms

---

## Advanced Configuration

### Custom Compiler

To use a different compiler:

```bash
# Use Clang on Linux
cmake --preset linux-release -DCMAKE_CXX_COMPILER=clang++

# Use LLVM/clang-cl on Windows
cmake -G Ninja -DCMAKE_CXX_COMPILER=clang-cl --preset windows-release
```

### Disable LTO (for faster linking)

```bash
# LTO is built into CMAKE_CXX_FLAGS_RELEASE; remove manually if needed
cmake --preset linux-release \
  -DCMAKE_CXX_FLAGS_RELEASE="-O3 -DNDEBUG"
```

### Override Deployment Target (macOS)

```bash
cmake --preset macos-release -DCMAKE_OSX_DEPLOYMENT_TARGET=11.0
```

### Static Linking (Advanced)

Plugins typically use dynamic linking (plugin host provides JUCE). Do not change `CMAKE_MSVC_RUNTIME_LIBRARY` without understanding implications.

---

## Validation & Testing

### pluginval Strictness Levels

Engine:Field CI uses **strictness level 5** (strict):

| Level | Checks | Use Case |
|-------|--------|----------|
| 1 | Basic structure | Initial plugin development |
| 3 | Standard checks | Most production plugins |
| 5 | Strict (Engine:Field) | Production + distribution |
| 10 | Maximum strictness | Intensive verification |

### Manual pluginval Testing

```bash
# Download pluginval from https://github.com/Tracktion/pluginval/releases

# Run with report
pluginval --strictness-level 5 \
  --validate path/to/EngineField.vst3 \
  --report-filename report.xml \
  --timeout-ms 120000

# Check report
cat report.xml
```

---

## Next Steps

1. **Test locally** on all three platforms:
   ```bash
   cmake --preset <platform>-release
   cmake --build --preset <platform>-release
   ```

2. **Load in DAW** (Reaper, Studio One, etc.):
   - Windows: VST3 folder at `%APPDATA%\vst3\` or DAW VST3 path
   - macOS: `~/Library/Audio/Plug-Ins/VST3/`
   - Linux: `~/.vst3/`

3. **Push to GitHub** to trigger CI validation across all platforms

4. **Monitor builds** in GitHub Actions tab for cross-platform validation

---

## Support & References

- **JUCE Documentation**: https://docs.juce.com/
- **CMake Documentation**: https://cmake.org/cmake/help/latest/
- **pluginval Repository**: https://github.com/Tracktion/pluginval
- **VST3 Standard**: https://github.com/steinbergmedia/vst3sdk

---

**Engine:Field Build System — Production Ready**
*Last Updated: 2025-10-22*
