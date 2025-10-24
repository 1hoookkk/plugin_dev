# Build Troubleshooting - Engine:Field Rust Port

## Current Issue: Network Access to crates.io Blocked

### Problem
```
error: failed to get successful HTTP response from `https://index.crates.io/config.json` (21.0.0.69), got 403
body: Access denied
```

### Root Cause
The environment has restricted network access to crates.io, preventing Cargo from downloading dependencies.

### Workarounds

#### Option 1: Build in Different Environment (Recommended)
Build the plugin on a machine with unrestricted internet access:

**Windows (FL Studio):**
```powershell
# Install Rust from https://rustup.rs
# Clone repository
git clone https://github.com/1hoookkk/plugin_dev.git
cd plugin_dev/rust_port

# Build VST3 (FL Studio uses VST3)
cargo build --release --lib

# The VST3 will be at: target\release\engine_field.dll
# Manual bundling needed (see below)
```

**Important for FL Studio Users:**
- FL Studio on Windows primarily uses **VST3** format
- VST3 plugins on Windows are actually **folders** with `.vst3` extension
- The folder must contain the DLL in a specific structure

#### Option 2: Manual VST3 Bundle Creation (Without xtask)

Since xtask requires network access, here's how to manually create a VST3 bundle after building:

**Windows:**
```powershell
# 1. Build the plugin DLL
cargo build --release --lib

# 2. Create VST3 bundle structure
mkdir "Engine Field.vst3\Contents\x86_64-win"

# 3. Copy the built DLL
copy target\release\engine_field.dll "Engine Field.vst3\Contents\x86_64-win\Engine Field.vst3"

# 4. Install to FL Studio
# FL Studio typically scans these locations:
# - C:\Program Files\Common Files\VST3\
# - C:\Program Files\VstPlugins\ (legacy)
# - Custom folders set in FL Studio settings

# Copy the entire "Engine Field.vst3" folder to VST3 directory:
xcopy /E /I "Engine Field.vst3" "C:\Program Files\Common Files\VST3\Engine Field.vst3"

# 5. Rescan plugins in FL Studio
# - Open FL Studio
# - Go to: Options → Manage Plugins
# - Click "Find plugins" or F8
```

**Linux (Testing):**
```bash
# 1. Build
cargo build --release --lib

# 2. Create bundle
mkdir -p "Engine Field.vst3/Contents/x86_64-linux"
cp target/release/libengine_field.so "Engine Field.vst3/Contents/x86_64-linux/Engine Field.so"

# 3. Install
cp -r "Engine Field.vst3" ~/.vst3/
```

**macOS (Testing):**
```bash
# 1. Build
cargo build --release --lib

# 2. Create bundle
mkdir -p "Engine Field.vst3/Contents/MacOS"
cp target/release/libengine_field.dylib "Engine Field.vst3/Contents/MacOS/Engine Field"

# 3. Install
cp -r "Engine Field.vst3" ~/Library/Audio/Plug-Ins/VST3/
```

#### Option 3: Use Cargo Vendoring (Advanced)

On a machine with internet access:
```bash
# 1. Download all dependencies
cargo vendor

# 2. Configure cargo to use vendored dependencies
mkdir .cargo
cat > .cargo/config.toml << EOF
[source.crates-io]
replace-with = "vendored-sources"

[source.vendored-sources]
directory = "vendor"
EOF

# 3. Commit vendor directory to repository
git add vendor .cargo/config.toml
git commit -m "Add vendored dependencies for offline builds"

# 4. Now builds work offline
cargo build --release --lib
```

---

## FL Studio-Specific Setup

### Plugin Formats
FL Studio natively supports:
1. **VST3** (Recommended) - Modern standard
2. **VST2** (Legacy) - Requires older SDK
3. **FL Native** - FL Studio proprietary format

**This plugin provides VST3 only.**

### FL Studio Plugin Paths

FL Studio scans these locations by default:

**System Paths:**
- `C:\Program Files\Common Files\VST3\` (VST3, recommended)
- `C:\Program Files\Steinberg\VstPlugins\` (VST2 legacy)
- `C:\Program Files\VstPlugins\` (VST2 legacy)

**User Paths:**
- Can be configured in FL Studio: Options → File Settings → Manage plugins

### Installation Steps for FL Studio

1. **Build the plugin** (requires internet access on build machine):
   ```powershell
   cargo build --release --lib
   ```

2. **Create VST3 bundle** (see Option 2 above)

3. **Copy to FL Studio**:
   ```powershell
   # As Administrator:
   xcopy /E /I "Engine Field.vst3" "C:\Program Files\Common Files\VST3\Engine Field.vst3"
   ```

4. **Rescan in FL Studio**:
   - Options → Manage Plugins (F10)
   - Click "Find plugins" or press F8
   - Wait for scan to complete
   - Look for "Engine:Field" in the plugin list

5. **Add to mixer**:
   - Open Mixer (F9)
   - Right-click on an insert slot
   - Select → More... → Engine:Field

### Expected Behavior

**Parameters visible in FL Studio:**
- CHARACTER (0-100%) - Morph control
- MIX (0-100%) - Dry/wet blend
- EFFECT (toggle) - Wet solo mode
- OUTPUT (-12 to +12 dB) - Makeup gain
- BYPASS (toggle) - True bypass

**CPU Usage:**
- Should be 2-3% on modern Intel/AMD/M1 at 48kHz, 512 buffer
- If higher, check for denormals or increase buffer size in FL Studio

---

## Verification

### Quick Tests (After Build)

**1. Check file size:**
```powershell
# Windows VST3 DLL should be ~500KB-2MB
dir target\release\engine_field.dll
```

**2. Check symbols (if build succeeds):**
```bash
# Linux
nm -D target/release/libengine_field.so | grep "VST3GetPluginFactory"

# Should see: VST3GetPluginFactory (exported symbol)
```

**3. Test in pluginval (if available):**
```bash
# Download from: https://github.com/Tracktion/pluginval
pluginval --validate "Engine Field.vst3" --strictness-level 5
```

---

## Common Issues

### Issue 1: "Failed to load plugin" in FL Studio
**Causes:**
- DLL architecture mismatch (32-bit vs 64-bit)
- Missing Visual C++ Redistributable
- Incorrect VST3 bundle structure

**Solutions:**
```powershell
# 1. Verify FL Studio architecture (64-bit is standard now)
# FL Studio 20.8+ is 64-bit only

# 2. Install VC++ Redistributable (2015-2022)
# Download from: https://aka.ms/vs/17/release/vc_redist.x64.exe

# 3. Check bundle structure
tree /F "Engine Field.vst3"
# Should show:
# Engine Field.vst3
# └── Contents
#     └── x86_64-win
#         └── Engine Field.vst3 (the actual DLL)
```

### Issue 2: Plugin loads but no audio
**Causes:**
- BYPASS parameter enabled
- MIX parameter at 0%
- EFFECT mode on with no input

**Solutions:**
- Check parameter values in FL Studio
- Ensure audio is routed correctly
- Enable TEST_TONE parameter (hidden) for debugging

### Issue 3: Audio glitches/dropouts
**Causes:**
- Buffer size too small
- RT-safety violation
- CPU overload

**Solutions:**
```
1. Increase buffer size in FL Studio:
   - Options → Audio Settings
   - Set Buffer Length to 512 or 1024

2. Check CPU usage:
   - Should be <5% per instance
   - If higher, disable other plugins to isolate

3. Check for denormals:
   - Plugin includes ScopedNoDenormals protection
   - If issues persist, check input signal level
```

---

## Development Build (Debug Mode)

For testing during development:

```powershell
# Build debug version (faster compile, slower execution)
cargo build --lib

# Creates: target\debug\engine_field.dll
# Same bundle process as release, but use debug DLL
```

---

## Next Steps When Network Available

1. **Complete build** with internet access
2. **Test in FL Studio** on Windows
3. **Run pluginval** validation
4. **Profile CPU** usage under realistic load
5. **Compare output** with C++ version (if available)

---

## Technical Notes

### NIH-plug Framework
- Modern Rust plugin framework by Robbert van der Helm
- Supports VST3 and CLAP formats
- RT-safety built into the API design
- No GUI framework included (headless plugin for now)

### Build Requirements
- Rust 1.70+ (2021 edition)
- Cargo (comes with Rust)
- Internet access for initial build
- Windows: Visual Studio 2019+ or MSVC Build Tools
- macOS: Xcode Command Line Tools
- Linux: gcc/clang

### Performance Expectations
Based on C++ profiling:
- **update_coeffs()**: ~1.2 cycles/sample (0.1% CPU)
- **process_stereo()**: ~932 cycles/sample (99.9% CPU)
- **Total**: ~934 cycles/sample ≈ 2-3% CPU @ 48kHz

Rust should match or slightly exceed C++ performance due to:
- LLVM optimizations
- Zero-cost abstractions
- No vtable overhead
- Inline assembly potential

---

## Support

If build issues persist:
1. Check Rust version: `rustc --version` (need 1.70+)
2. Update Rust: `rustup update stable`
3. Clean build: `cargo clean && cargo build --release`
4. Check NIH-plug docs: https://github.com/robbert-vdh/nih-plug
5. Open issue: https://github.com/1hoookkk/plugin_dev/issues
