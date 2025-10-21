# Engine:Field v1.0 — Installation Guide

After building Engine:Field (see **QUICKSTART.md**), use this guide to install the VST3 plugin and Standalone application.

## Table of Contents

1. [VST3 Plugin Installation](#vst3-plugin-installation)
   - [Windows](#windows-vst3)
   - [macOS](#macos-vst3)
   - [Linux](#linux-vst3)
2. [Standalone Installation](#standalone-installation)
3. [Verification & Testing](#verification--testing)
4. [Troubleshooting](#troubleshooting)

---

## VST3 Plugin Installation

### Windows VST3

**Automatic Installation (via CMake option)**

During build, enable auto-install:
```bash
cmake -S . -B build -DJUCE_SOURCE_DIR="C:\JUCE" \
  -DFIELD_INSTALL_AFTER_BUILD=ON \
  -DBUILD_VST3=ON -DBUILD_STANDALONE=ON

cmake --build build --config Release
```

The VST3 plugin is automatically copied to the standard location.

**Manual Installation**

1. Locate the built plugin:
   ```
   build\plugins\EngineField\EngineField_artefacts\Release\VST3\EngineField.vst3
   ```

2. Copy to the VST3 folder:
   ```
   C:\Program Files\Common Files\VST3\EngineField.vst3
   ```
   (Create the folder if it doesn't exist)

3. Restart your DAW

4. Rescan VST3 plugins in your DAW:
   - **Reaper:** Options → Preferences → Plug-ins → Re-scan VST3
   - **Studio One:** Studio One → Options (macOS: Preferences) → Plug-in Manager → Rescan
   - **Cubase:** Devices → Plug-in Manager → Rescan (or press Alt+R)
   - **FL Studio:** Image-Line Browser → Plug-in Database → Refresh (or press F5)

5. The plugin should appear in your DAW's plugin browser under **Engine Audio** or **EngineField**

### macOS VST3

**Automatic Installation (via CMake option)**

During build, enable auto-install:
```bash
cmake -S . -B build -DCMAKE_BUILD_TYPE=Release \
  -DJUCE_SOURCE_DIR=~/JUCE \
  -DFIELD_INSTALL_AFTER_BUILD=ON \
  -DBUILD_VST3=ON -DBUILD_STANDALONE=ON

cmake --build build --config Release
```

The VST3 plugin is automatically copied to `~/Library/Audio/Plug-Ins/VST3/`.

**Manual Installation**

1. Locate the built plugin:
   ```
   build/plugins/EngineField/EngineField_artefacts/Release/VST3/EngineField.vst3
   ```

2. Copy to the VST3 folder:
   ```bash
   cp -r build/plugins/EngineField/EngineField_artefacts/Release/VST3/EngineField.vst3 \
     ~/Library/Audio/Plug-Ins/VST3/
   ```
   (Create the folder if it doesn't exist)

3. Restart your DAW

4. Rescan VST3 plugins in your DAW (see Windows steps above; process identical on macOS)

5. The plugin should appear in your DAW's plugin browser

**Note on Notarization:** If your DAW or macOS displays a security warning, you may need to approve the plugin in **System Preferences → Security & Privacy** (or **System Settings → Privacy & Security** on Sonoma+).

### Linux VST3

**Automatic Installation (via CMake option)**

During build, enable auto-install:
```bash
cmake -S . -B build -DCMAKE_BUILD_TYPE=Release \
  -DJUCE_SOURCE_DIR=~/JUCE \
  -DFIELD_INSTALL_AFTER_BUILD=ON \
  -DBUILD_VST3=ON -DBUILD_STANDALONE=ON

cmake --build build --config Release
```

The VST3 plugin is automatically copied to `~/.vst3/` or `/usr/local/lib/vst3/` (depending on JUCE configuration).

**Manual Installation**

1. Locate the built plugin:
   ```
   build/plugins/EngineField/EngineField_artefacts/Release/VST3/EngineField.vst3
   ```

2. Copy to the VST3 folder:
   ```bash
   # User-local installation (recommended)
   mkdir -p ~/.vst3
   cp -r build/plugins/EngineField/EngineField_artefacts/Release/VST3/EngineField.vst3 ~/.vst3/

   # OR system-wide installation (requires sudo)
   sudo mkdir -p /usr/local/lib/vst3
   sudo cp -r build/plugins/EngineField/EngineField_artefacts/Release/VST3/EngineField.vst3 /usr/local/lib/vst3/
   ```

3. Restart your DAW

4. Rescan VST3 plugins in your DAW

5. The plugin should appear in your DAW's plugin browser

---

## Standalone Installation

### Windows Standalone

1. Locate the standalone application:
   ```
   build\plugins\EngineField\EngineField_artefacts\Release\Standalone\EngineField.exe
   ```

2. **(Optional) Create Shortcut**
   - Right-click on `EngineField.exe` → Send to → Desktop (create shortcut)
   - OR move `EngineField.exe` to `C:\Program Files\Engine Audio\EngineField\` (create folders if needed)

3. **Run:**
   - Double-click `EngineField.exe` or the shortcut
   - OR from command line: `.\build\plugins\EngineField\EngineField_artefacts\Release\Standalone\EngineField.exe`

4. The standalone application opens with audio I/O device selection on first run

### macOS Standalone

1. Locate the standalone application:
   ```
   build/plugins/EngineField/EngineField_artefacts/Release/Standalone/EngineField.app
   ```

2. **(Optional) Install to Applications**
   ```bash
   cp -r build/plugins/EngineField/EngineField_artefacts/Release/Standalone/EngineField.app \
     /Applications/
   ```

3. **Run:**
   - Double-click `EngineField.app` OR drag to Applications folder
   - OR from command line: `open build/plugins/EngineField/EngineField_artefacts/Release/Standalone/EngineField.app`
   - First run may prompt for microphone/audio permission (grant access)

4. The standalone application opens with audio I/O device selection on first run

### Linux Standalone

1. Locate the standalone application:
   ```
   build/plugins/EngineField/EngineField_artefacts/Release/Standalone/EngineField
   ```

2. **(Optional) Install to PATH**
   ```bash
   sudo mkdir -p /opt/engineaudio
   sudo cp build/plugins/EngineField/EngineField_artefacts/Release/Standalone/EngineField \
     /opt/engineaudio/
   sudo ln -sf /opt/engineaudio/EngineField /usr/local/bin/enginefield
   ```

3. **Run:**
   - `./build/plugins/EngineField/EngineField_artefacts/Release/Standalone/EngineField`
   - OR (if installed to PATH): `enginefield`
   - First run may prompt for audio permission (e.g., JACK/PulseAudio access)

4. The standalone application opens with audio I/O device selection on first run

---

## Verification & Testing

### Verify VST3 in DAW

**Test in Reaper:**
1. Open Reaper
2. Create a new track (Cmd/Ctrl+T)
3. Click the "+" in the FX section
4. Search for "EngineField" or find under **Engine Audio**
5. Click to insert
6. The plugin editor should open with the 4×4 sampler pad grid UI

**Test in Studio One:**
1. Open Studio One
2. Create an Audio track
3. Click **Insert** on the track's FX section
4. Search for "EngineField"
5. Double-click to add
6. The plugin editor should open

### Verify Standalone

1. Launch the standalone application
2. You should see:
   - A 600×550 px window with warm cream/brown background
   - 4×4 sampler pad grid in the center
   - CHARACTER slider (left), MIX knob (right)
   - EFFECT button (bottom-left)
   - Output level (bottom-right)
   - Bypass & Test Tone buttons (top)

3. **Quick Audio Test:**
   - Enable "Test Tone" button (bottom menu) to generate a 440 Hz sine wave
   - You should hear a steady tone through your speakers/headphones
   - Adjust CHARACTER (0–100%) and MIX (0–100%) to hear the Z-plane filtering effect
   - Disable "Test Tone" when done

### Check Plugin Size

- **VST3 plugin:** ~4.8 MB
- **Standalone app:** ~4.8 MB

If significantly larger or smaller, rebuild with `CMAKE_BUILD_TYPE=Release` to ensure optimized binary.

---

## Troubleshooting

### VST3 Not Appearing in DAW

**Symptom:** Plugin browser doesn't show EngineField

**Solutions:**
1. Verify correct installation path (see above for your OS)
2. Rescan plugins (see each DAW in "Manual Installation" section)
3. Check DAW's plugin folder settings:
   - Reaper: Options → Preferences → Plug-ins → VST3 paths
   - Studio One: Preferences → External Plugins → VST3 paths
   - Cubase: Devices → Plug-in Manager → VST-Plugin Paths
4. Restart the DAW (don't just rescan)
5. Check build artifacts exist at the expected path

### VST3 Plugin Crashes on Load

**Symptom:** DAW freezes or crashes when inserting EngineField

**Solutions:**
1. Ensure VST3 plugin is built (check `build/.../Release/VST3/EngineField.vst3` exists)
2. Rebuild with exact JUCE version used (JUCE mismatch can cause crashes)
3. Test with standalone first to verify core DSP works
4. Enable pluginval validation (see README.md):
   ```bash
   cmake --build build --target pluginval_field
   ```
5. Check console output or DAW logs for specific error

### Standalone Won't Start

**Symptom:** Executable launches but immediately closes or hangs

**Solutions (Windows):**
1. Run from command prompt to see error messages:
   ```bash
   EngineField.exe
   ```
2. If missing JUCE dependencies, install Visual C++ Redistributable:
   - Download from [microsoft.com](https://support.microsoft.com/en-us/help/2977003)
   - Install and restart

**Solutions (macOS/Linux):**
1. Check file permissions:
   ```bash
   chmod +x ./build/plugins/EngineField/EngineField_artefacts/Release/Standalone/EngineField
   ```
2. Run from terminal to see errors:
   ```bash
   ./build/plugins/EngineField/EngineField_artefacts/Release/Standalone/EngineField
   ```

### Audio Not Playing Through Standalone

**Symptom:** Standalone launches but no audio output despite "Test Tone" enabled

**Solutions:**
1. Check system audio device is not muted/disconnected
2. In standalone window, verify:
   - Output device dropdown shows correct speaker/headphone
   - Output level is visible on the OUTPUT control (bottom-right)
3. Increase OUTPUT level (bottom-right control) to +6 dB
4. Try a different audio device (click device dropdown if available)
5. On macOS, grant microphone permission if prompted:
   - System Preferences → Security & Privacy → Microphone → Allow EngineField

### File Not Found / Permission Denied

**Symptom:** Installation command fails with "permission denied" or "file not found"

**Solutions:**
1. Ensure build directory exists: `ls -la build/` (should show plugin artifacts)
2. If permission denied, use `sudo` (macOS/Linux):
   ```bash
   sudo cp -r build/plugins/EngineField/.../EngineField.vst3 /usr/local/lib/vst3/
   ```
3. Verify source path is correct (no typos)
4. Check available disk space: `df -h`

---

## Quick Links

- **Build instructions:** See QUICKSTART.md
- **Full documentation:** See README.md
- **Developer reference:** See CLAUDE.md
- **Change log:** See CHANGELOG.md

---

## Support

For additional help:
1. Re-read the relevant section above for your OS
2. Check the DAW's plugin documentation
3. Contact support@engineaudio.com with:
   - OS and version (e.g., Windows 11, macOS 13.1)
   - DAW and version
   - Build log output (run with `-DCMAKE_VERBOSE_MAKEFILE=ON` for details)
   - Error message or symptoms
