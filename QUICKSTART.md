# Engine:Field v1.0 — Quick Start Guide

Get Engine:Field built and tested in under 5 minutes.

## Prerequisites

- **JUCE 8.0.10+** — [Download from juce.com](https://juce.com/discover/downloads)
- **CMake 3.24+** — [cmake.org](https://cmake.org/download/)
- **C++ 20 Compiler:**
  - Windows: Visual Studio 2022 Community (free)
  - macOS: Xcode 14+ (via `xcode-select --install`)
  - Linux: GCC 11+ or Clang 14+

## Step 1: Obtain JUCE

### Windows
```bash
# Download JUCE 8.0.10 or later (e.g., JUCE-8.0.10.zip)
# Extract to a known location, e.g.:
mkdir C:\JUCE
# Extract the JUCE archive to C:\JUCE
```

### macOS
```bash
# Use Homebrew (optional)
brew install juce
# OR download manually from juce.com and extract to ~/JUCE
```

### Linux
```bash
# Fedora/RHEL
sudo dnf install juce-devel

# Ubuntu/Debian
sudo apt-get install libjuce-dev

# OR download JUCE source and extract to ~/JUCE
```

## Step 2: Clone & Enter Directory

```bash
cd /path/to/plugin_dev
```

Verify the directory structure:
```bash
ls -la
# Should show: README.md, QUICKSTART.md, CMakeLists.txt, plugins/, scripts/
```

## Step 3: Configure & Build

### Windows (MSVC)
```bash
cmake -S . -B build -G "Visual Studio 17 2022" ^
  -DJUCE_SOURCE_DIR="C:\JUCE" ^
  -DBUILD_VST3=ON -DBUILD_STANDALONE=ON ^
  -DCMAKE_BUILD_TYPE=Release

cmake --build build --config Release
```

### macOS
```bash
cmake -S . -B build -DCMAKE_BUILD_TYPE=Release \
  -DJUCE_SOURCE_DIR=~/JUCE \
  -DBUILD_VST3=ON -DBUILD_STANDALONE=ON

cmake --build build --config Release
```

### Linux
```bash
cmake -S . -B build -DCMAKE_BUILD_TYPE=Release \
  -DJUCE_SOURCE_DIR=~/JUCE \
  -DBUILD_VST3=ON -DBUILD_STANDALONE=ON

cmake --build build --config Release
```

**Expected output:**
```
[  0%] Building CXX object ...
...
[100%] Built target EngineField_VST3
[100%] Built target EngineField_Standalone
```

Build time: ~30–60 seconds (first-time Ninja/Makefile builds may take 1–2 minutes).

## Step 4: Locate Built Artifacts

### VST3 Plugin
- **Windows:** `build\plugins\EngineField\EngineField_artefacts\Release\VST3\EngineField.vst3`
- **macOS:** `build/plugins/EngineField/EngineField_artefacts/Release/VST3/EngineField.vst3`
- **Linux:** `build/plugins/EngineField/EngineField_artefacts/Release/VST3/EngineField.vst3`

### Standalone App
- **Windows:** `build\plugins\EngineField\EngineField_artefacts\Release\Standalone\EngineField.exe`
- **macOS:** `build/plugins/EngineField/EngineField_artefacts/Release/Standalone/EngineField.app`
- **Linux:** `build/plugins/EngineField/EngineField_artefacts/Release/Standalone/EngineField`

## Step 5: Test the Standalone

### Windows
```bash
.\build\plugins\EngineField\EngineField_artefacts\Release\Standalone\EngineField.exe
```

### macOS
```bash
open build/plugins/EngineField/EngineField_artefacts/Release/Standalone/EngineField.app
```

### Linux
```bash
./build/plugins/EngineField/EngineField_artefacts/Release/Standalone/EngineField
```

**You should see:**
- A window with warm cream/brown background (600×550 px)
- 4×4 sampler pad grid in the center
- CHARACTER slider on the left
- MIX knob on the right
- EFFECT button at the bottom

## Step 6: Install VST3 Plugin (Optional)

See **INSTALL.md** for VST3 installation in your DAW.

## Troubleshooting

### CMake not found
```bash
# Windows: Install Visual Studio Build Tools with CMake component
# macOS: brew install cmake
# Linux: sudo apt-get install cmake
```

### JUCE not found
```bash
# Verify JUCE_SOURCE_DIR path is correct and contains juce_appconfig.h
ls <JUCE_SOURCE_DIR>/modules/juce_core/juce_core.h
```

### Compiler error (C++ 20 not available)
```bash
# Update compiler:
# Windows: Install Visual Studio 2022
# macOS: xcode-select --install && sudo xcode-select --switch /Applications/Xcode.app/Contents/Developer
# Linux: sudo apt-get install g++-11 && export CXX=g++-11
```

### Build fails with "permission denied"
```bash
# On Linux/macOS, ensure write permission:
chmod -R u+w .
```

## Next Steps

1. **Load in DAW:** Copy VST3 plugin to your DAW's VST3 folder and rescan
2. **Tweak Parameters:** Use CHARACTER (0–100%) and MIX (0–100%) to morph transients
3. **Test with Drums:** Load a drum loop and hear the Z-plane filtering in action
4. **Read README.md:** Full feature set and technical details

## Quick Parameter Tuning

- **CHARACTER 0–50%:** Subtle morphing with fewer decay steps (good for preserving attack)
- **CHARACTER 50–100%:** Aggressive morphing with many decay steps (good for shaping decay tails)
- **MIX 0–50%:** Blended dry/wet (preserves original tone + effect)
- **MIX 50–100%:** More filtered signal (stronger effect character)
- **EFFECT button ON:** Hear only the wet signal; useful for dialing in CHARACTER without dry interference

## Build Variants

### Development Build (faster iteration, warnings allowed)
```bash
cmake -S . -B build -DCMAKE_BUILD_TYPE=Debug \
  -DJUCE_SOURCE_DIR=~/JUCE
cmake --build build --config Debug
```

### Strict CI Build (warnings → errors)
```bash
cmake -S . -B build -DCMAKE_BUILD_TYPE=Release \
  -DENABLE_WARNINGS_AS_ERRORS=ON \
  -DJUCE_SOURCE_DIR=~/JUCE
cmake --build build --config Release
```

## Support

- See **README.md** for full documentation
- See **INSTALL.md** for platform-specific VST3 setup
- See **CLAUDE.md** for developer reference and DSP architecture
- Questions? Contact support@engineaudio.com
