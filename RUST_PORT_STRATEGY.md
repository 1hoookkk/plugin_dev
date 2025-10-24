# Engine:Field Rust Port Strategy

**Date:** 2025-10-24
**Status:** Planning Phase
**Goal:** Port Engine:Field from JUCE/C++ to NIH-plug/Rust

---

## Executive Summary

**Recommendation: PORT TO RUST using NIH-plug framework**

The existing Rust infrastructure at `github.com/1hoookkk/coder23` provides an **excellent foundation** for a complete Rust port. Key advantages:

| Aspect | C++/JUCE | Rust/NIH-plug | Winner |
|--------|----------|---------------|--------|
| **Memory Safety** | Manual management | Compile-time guarantees | ✅ **Rust** |
| **RT-Safety** | Runtime checks | Compile-time + runtime | ✅ **Rust** |
| **Performance** | Excellent | Excellent (SIMD-ready) | 🟰 **Tie** |
| **Build Speed** | Slow (JUCE templates) | Fast (incremental) | ✅ **Rust** |
| **Plugin Formats** | VST3, AU, Standalone | VST3, CLAP, Standalone | ✅ **Rust** |
| **GUI** | JUCE (mature) | nih_plug_vizia (growing) | ⚠️ **C++** |
| **Learning Curve** | Moderate | Steep (ownership) | ⚠️ **C++** |

**Bottom Line:** Rust port delivers **better safety, faster iteration, and modern plugin formats** with minimal performance trade-off.

---

## Current State Analysis

### What Already Exists in Rust

#### 1. **UNIFIED_DSP_LIBRARY** (`/tmp/coder23/UNIFIED_DSP_LIBRARY/`)

✅ **Implemented:**
- `EmuFilter` struct with RT-safe atomic parameter updates
- Biquad filter state management (Direct Form II Transposed)
- Lock-free parameter snapshots
- FFI layer for C++ interop
- Performance metrics tracking
- Channel-based processing (up to 8 channels)

⚠️ **Missing (vs Engine:Field):**
- ❌ Z-plane pole-pair interpolation (geodesic)
- ❌ Bilinear frequency warping (48kHz → target SR)
- ❌ 6-section biquad cascade
- ❌ Per-section tanh saturation
- ❌ Equal-power dry/wet mixing
- ❌ Envelope follower with depth modulation
- ❌ EMU authentic coefficient tables

**Current:**
```rust
pub struct EmuFilter {
    // Single biquad with basic filtering
    b0, b1, b2, a1, a2: AtomicU32,
    channels: [ChannelState; MAX_CHANNELS],
}
```

**Target:**
```rust
pub struct ZPlaneFilter {
    // 6-section cascade per channel
    cascades: [[BiquadSection; 6]; 2],  // Stereo
    poles_a: [PolePair; 6],
    poles_b: [PolePair; 6],
    // Smoothers, envelope, etc.
}
```

#### 2. **Field Plugin** (`/tmp/coder23/Field/`)

✅ **Implemented:**
- NIH-plug boilerplate (VST3/CLAP export)
- Basic parameter system (gain only)
- Stereo processing skeleton

❌ **Missing:**
- All Engine:Field parameters (CHARACTER, MIX, EFFECT, etc.)
- DSP integration
- UI (currently no GUI)

#### 3. **EMU_ZPlane_Vault** (`/tmp/coder23/EMU_ZPlane_Vault/`)

✅ **Available:**
- Authentic EMU shapes in JSON (`audity_shapes_A_48k.json`, `audity_shapes_B_48k.json`)
- Documentation on Z-plane morphing
- Complete shape library (Vowel, Bell, Low, Sub)

❌ **Not yet:**
- Rust deserialization code
- Shape loader integration

---

## Architecture Comparison

### JUCE (C++) vs NIH-plug (Rust)

```
┌─────────────────────────────────────────────────────────────┐
│                   JUCE Architecture                         │
├─────────────────────────────────────────────────────────────┤
│ FieldProcessor (AudioProcessor)                             │
│   ├── APVTS (AudioProcessorValueTreeState)                 │
│   ├── ZPlaneFilter (manual allocation)                      │
│   ├── EnvelopeFollower                                      │
│   └── FieldWaveformEditor (Component tree)                  │
│                                                              │
│ Manual memory management, thread-safety via mutexes/atomics │
└─────────────────────────────────────────────────────────────┘

┌─────────────────────────────────────────────────────────────┐
│                  NIH-plug Architecture                       │
├─────────────────────────────────────────────────────────────┤
│ Field (Plugin trait)                                         │
│   ├── FieldParams (Params derive macro)                     │
│   │    ├── FloatParam (built-in smoothing)                 │
│   │    ├── BoolParam                                        │
│   │    └── EnumParam                                        │
│   ├── ZPlaneFilter (owned, lifetime-safe)                   │
│   ├── EnvelopeFollower                                      │
│   └── Editor (nih_plug_vizia or no GUI)                     │
│                                                              │
│ Ownership system ensures safety, Arc for shared state       │
└─────────────────────────────────────────────────────────────┘
```

**Key Differences:**

1. **Parameters:**
   - JUCE: `apvts_.getRawParameterValue()` → `std::atomic<float>*`
   - NIH-plug: `FloatParam` with built-in smoothing, atomic access

2. **Processing:**
   - JUCE: `processBlock(AudioBuffer<float>&)`
   - NIH-plug: `process(&mut Buffer, &mut AuxiliaryBuffers, &mut Context)`

3. **State:**
   - JUCE: `getStateInformation()` / `setStateInformation()` (XML/binary)
   - NIH-plug: Automatic serialization via `Params` derive

4. **UI:**
   - JUCE: Component hierarchy, JUCE graphics
   - NIH-plug: Optional (vizia, egui, or headless)

---

## Porting Plan

### Phase 1: Core DSP (Priority 0 - 2 weeks)

**Goal:** Port ZPlaneFilter to Rust with 100% mathematical equivalence

#### Tasks:

1. **Define Z-Plane Types** (`src/dsp/types.rs`)
   ```rust
   #[derive(Debug, Clone, Copy)]
   pub struct PolePair {
       pub r: f32,      // radius
       pub theta: f32,  // angle (radians)
   }

   #[derive(Debug, Clone, Copy)]
   pub struct BiquadCoeffs {
       pub b0: f32,
       pub b1: f32,
       pub b2: f32,
       pub a1: f32,
       pub a2: f32,
   }
   ```

2. **Implement Core Math** (`src/dsp/zplane.rs`)
   ```rust
   pub fn interpolate_pole(a: PolePair, b: PolePair, t: f32, geodesic: bool) -> PolePair {
       let r = if geodesic {
           (a.r.max(1e-9).ln() * (1.0 - t) + b.r.max(1e-9).ln() * t).exp()
       } else {
           a.r + t * (b.r - a.r)
       };

       let delta = wrap_angle(b.theta - a.theta);
       let theta = a.theta + t * delta;

       PolePair { r, theta }
   }

   pub fn remap_pole_48k_to_fs(p48k: PolePair, target_fs: f64) -> PolePair {
       use num_complex::Complex64;

       // Fast path
       if (target_fs - 48000.0).abs() < 0.1 {
           return p48k;
       }

       // Bilinear transform (inverse + forward)
       let z48 = Complex64::from_polar(p48k.r as f64, p48k.theta as f64);
       let s = (2.0 * 48000.0) * (z48 - 1.0) / (z48 + 1.0);
       let z_new = (2.0 * target_fs + s) / (2.0 * target_fs - s);

       PolePair {
           r: z_new.norm() as f32,
           theta: z_new.arg() as f32,
       }
   }

   pub fn pole_to_biquad(p: PolePair) -> BiquadCoeffs {
       let a1 = -2.0 * p.r * p.theta.cos();
       let a2 = p.r * p.r;

       let rz = (0.9 * p.r).clamp(0.0, 0.999);
       let c = p.theta.cos();
       let mut b0 = 1.0;
       let mut b1 = -2.0 * rz * c;
       let mut b2 = rz * rz;

       let norm = 1.0 / (b0.abs() + b1.abs() + b2.abs()).max(0.25);
       b0 *= norm;
       b1 *= norm;
       b2 *= norm;

       BiquadCoeffs { b0, b1, b2, a1, a2 }
   }
   ```

3. **BiquadSection** (`src/dsp/biquad.rs`)
   ```rust
   #[derive(Debug, Clone, Copy)]
   pub struct BiquadSection {
       // State (Direct Form II Transposed)
       z1: f32,
       z2: f32,

       // Coefficients
       coeffs: BiquadCoeffs,

       // Saturation
       sat: f32,
   }

   impl BiquadSection {
       #[inline]
       pub fn process(&mut self, x: f32) -> f32 {
           let y = self.coeffs.b0 * x + self.z1;
           self.z1 = self.coeffs.b1 * x - self.coeffs.a1 * y + self.z2;
           self.z2 = self.coeffs.b2 * x - self.coeffs.a2 * y;

           // Per-section saturation
           let y = if self.sat > 0.0 {
               let g = 1.0 + self.sat * 4.0;
               (y * g).tanh()
           } else {
               y
           };

           if !y.is_finite() { 0.0 } else { y }
       }

       pub fn reset(&mut self) {
           self.z1 = 0.0;
           self.z2 = 0.0;
       }
   }
   ```

4. **ZPlaneFilter** (`src/dsp/filter.rs`)
   ```rust
   pub struct ZPlaneFilter {
       // Per-channel cascades
       cascade_l: [BiquadSection; 6],
       cascade_r: [BiquadSection; 6],

       // Shape definitions
       poles_a: [PolePair; 6],
       poles_b: [PolePair; 6],

       // Cached interpolated poles
       last_interp_poles: [PolePair; 6],

       // Sample rate
       sample_rate: f32,
   }

   impl ZPlaneFilter {
       pub fn update_coeffs(&mut self, morph: f32, intensity: f32) {
           let intensity_boost = 1.0 + intensity * 0.06;

           for i in 0..6 {
               // 1. Interpolate in 48k domain
               let p48k = interpolate_pole(
                   self.poles_a[i],
                   self.poles_b[i],
                   morph,
                   true  // geodesic
               );

               // 2. Bilinear remap
               let pm = remap_pole_48k_to_fs(p48k, self.sample_rate as f64);

               // 3. Apply intensity
               let r_boosted = (pm.r * intensity_boost).min(0.995);
               let pm = PolePair { r: r_boosted, theta: pm.theta };

               self.last_interp_poles[i] = pm;

               // 4. Convert to biquad
               let coeffs = pole_to_biquad(pm);
               self.cascade_l[i].coeffs = coeffs;
               self.cascade_r[i].coeffs = coeffs;
           }
       }

       #[inline]
       pub fn process_stereo(&mut self, l: &mut [f32], r: &mut [f32], drive: f32, mix: f32) {
           let drive_gain = 1.0 + drive * 4.0;

           for (l_samp, r_samp) in l.iter_mut().zip(r.iter_mut()) {
               let dry_l = *l_samp;
               let dry_r = *r_samp;

               // Pre-drive
               let mut wet_l = (dry_l * drive_gain).tanh();
               let mut wet_r = (dry_r * drive_gain).tanh();

               // Cascade
               for section in &mut self.cascade_l {
                   wet_l = section.process(wet_l);
               }
               for section in &mut self.cascade_r {
                   wet_r = section.process(wet_r);
               }

               // Equal-power mix
               let wet_g = mix.sqrt();
               let dry_g = (1.0 - mix).sqrt();

               *l_samp = wet_l * wet_g + dry_l * dry_g;
               *r_samp = wet_r * wet_g + dry_r * dry_g;
           }
       }
   }
   ```

**Testing:**
- Unit tests for each math function
- Compare output with C++ version (bit-exact for same inputs)
- Validate stability (all poles inside unit circle)

---

### Phase 2: NIH-plug Integration (Priority 1 - 1 week)

**Goal:** Wire ZPlaneFilter into Field plugin

#### Tasks:

1. **Define Parameters** (`src/params.rs`)
   ```rust
   #[derive(Params)]
   pub struct FieldParams {
       #[id = "character"]
       pub character: FloatParam,

       #[id = "mix"]
       pub mix: FloatParam,

       #[id = "effect"]
       pub effect: BoolParam,

       #[id = "output"]
       pub output: FloatParam,

       #[id = "bypass"]
       pub bypass: BoolParam,

       // Hidden test tone for validation
       #[id = "test_tone"]
       pub test_tone: BoolParam,
   }

   impl Default for FieldParams {
       fn default() -> Self {
           Self {
               character: FloatParam::new(
                   "Character",
                   50.0,
                   FloatRange::Linear { min: 0.0, max: 100.0 }
               )
               .with_smoother(SmoothingStyle::Linear(20.0))  // 20ms
               .with_unit("%"),

               mix: FloatParam::new(
                   "Mix",
                   100.0,
                   FloatRange::Linear { min: 0.0, max: 100.0 }
               )
               .with_smoother(SmoothingStyle::Linear(20.0))
               .with_unit("%"),

               // ... etc
           }
       }
   }
   ```

2. **Plugin Implementation** (`src/lib.rs`)
   ```rust
   use nih_plug::prelude::*;
   mod dsp;
   mod params;

   struct Field {
       params: Arc<params::FieldParams>,
       filter: dsp::ZPlaneFilter,
       envelope: dsp::EnvelopeFollower,
   }

   impl Plugin for Field {
       fn process(&mut self, buffer: &mut Buffer, _aux: &mut AuxiliaryBuffers, _ctx: &mut impl ProcessContext<Self>) -> ProcessStatus {
           let character = self.params.character.smoothed.next() * 0.01;
           let mix = self.params.mix.smoothed.next() * 0.01;
           let drive = 0.2;  // Locked authentic value

           // Update filter coefficients (once per block)
           self.filter.update_coeffs(character, 0.4);

           // Process stereo
           for mut channel_samples in buffer.iter_samples() {
               let mut samples = [0.0f32; 2];
               for (i, sample) in channel_samples.iter_mut().enumerate() {
                   samples[i] = *sample;
               }

               self.filter.process_stereo(&mut samples[0..1], &mut samples[1..2], drive, mix);

               for (i, sample) in channel_samples.iter_mut().enumerate() {
                   *sample = samples[i];
               }
           }

           ProcessStatus::Normal
       }
   }
   ```

3. **Build Configuration**
   - Update `Cargo.toml` with dependencies:
     ```toml
     [dependencies]
     nih_plug = { git = "https://github.com/robbert-vdh/nih-plug.git" }
     num-complex = "0.4"
     ```

**Testing:**
- Build VST3/CLAP
- Load in DAW (Reaper, Bitwig)
- Compare output with JUCE version

---

### Phase 3: Envelope Follower (Priority 2 - 2 days)

**Goal:** Add envelope-based CHARACTER modulation

```rust
pub struct EnvelopeFollower {
    state: f32,
    attack_coef: f32,
    release_coef: f32,
    depth: f32,
}

impl EnvelopeFollower {
    pub fn prepare(&mut self, sample_rate: f32, attack_ms: f32, release_ms: f32) {
        let attack_sec = attack_ms * 0.001;
        let release_sec = release_ms * 0.001;

        self.attack_coef = 1.0 - (-1.0 / (attack_sec * sample_rate).max(1e-6)).exp();
        self.release_coef = 1.0 - (-1.0 / (release_sec * sample_rate).max(1e-6)).exp();
    }

    #[inline]
    pub fn process(&mut self, input: f32) -> f32 {
        let rect = input.abs();
        let alpha = if rect > self.state {
            self.attack_coef
        } else {
            self.release_coef
        };

        self.state += alpha * (rect - self.state);
        (self.state * self.depth).clamp(0.0, 1.0)
    }
}
```

---

### Phase 4: UI (Priority 3 - Optional, 1 week)

**Options:**

1. **No GUI** (headless, parameters via DAW automation)
   - Fastest to ship
   - Suitable for power users

2. **nih_plug_vizia** (Rust native)
   - Modern, GPU-accelerated
   - Learning curve
   - Example: https://github.com/robbert-vdh/nih-plug/tree/master/plugins/examples/sine

3. **JUCE GUI wrapper** (hybrid approach)
   - Reuse existing FieldWaveformEditor
   - FFI bridge to Rust DSP
   - Best of both worlds

**Recommendation:** Ship **Phase 1-3 without GUI first**, add vizia later.

---

## Risk Assessment

### High Risk

| Risk | Impact | Mitigation |
|------|--------|------------|
| **Floating-point precision differences** | Audio artifacts | Unit tests comparing C++ vs Rust bit-exact |
| **Performance regression** | CPU spikes | Benchmark with criterion.rs, profile with perf |
| **Complex number library bugs** | Wrong frequency mapping | Validate bilinear transform against known cases |

### Medium Risk

| Risk | Impact | Mitigation |
|------|--------|------------|
| **NIH-plug learning curve** | Delayed timeline | Study examples, ask in Discord |
| **SIMD optimization complexity** | Suboptimal perf | Start scalar, add SIMD later |
| **Missing JUCE features** | Feature parity | Document gaps, prioritize essentials |

### Low Risk

| Risk | Impact | Mitigation |
|------|--------|------------|
| **DAW compatibility** | Plugin loading issues | Test in Reaper, Bitwig, Ableton |
| **State persistence** | Lost presets | Test save/load extensively |

---

## Benefits of Rust Port

### Safety

```rust
// C++ (manual lifetime management)
std::atomic<float>* mixParam_ = apvts_.getRawParameterValue("mix");
// ⚠️ Dangling pointer if APVTS reallocates

// Rust (compiler-enforced lifetime)
pub mix: FloatParam  // Owned, can't dangle
```

### Performance

```rust
// Rust: SIMD-ready with wide crate
use wide::f32x4;

let samples_simd = f32x4::new(s[0], s[1], s[2], s[3]);
let wet = cascade_process_simd(samples_simd);
```

### Build Speed

```
JUCE/C++:     2-3 minutes (full rebuild)
Rust:         10-30 seconds (incremental)
```

### Modern Plugin Formats

- ✅ **CLAP** (next-gen, better automation)
- ✅ **VST3** (industry standard)
- ⚠️ **AU** (macOS only, requires coreaudio-rs)

---

## Timeline Estimate

| Phase | Duration | Deliverable |
|-------|----------|-------------|
| Phase 1 (DSP) | 2 weeks | ZPlaneFilter parity with C++ |
| Phase 2 (Plugin) | 1 week | Working VST3/CLAP |
| Phase 3 (Envelope) | 2 days | CHARACTER modulation |
| Phase 4 (UI - optional) | 1 week | Basic GUI |
| **Total (MVP)** | **~3 weeks** | Headless plugin with full DSP |
| **Total (GUI)** | **~4 weeks** | Complete replacement |

---

## Next Steps

### Immediate (This Week)

1. ✅ **Set up Field plugin structure**
   ```bash
   cd /tmp/coder23/Field
   cargo build --release
   cargo xtask bundle Field --release
   ```

2. ✅ **Port PolePair types and interpolation**
   - Create `src/dsp/types.rs`
   - Implement `interpolate_pole()` with tests

3. ✅ **Port bilinear transform**
   - Implement `remap_pole_48k_to_fs()`
   - Validate against C++ with known test cases

### Week 2

4. ⏳ **BiquadSection + cascade**
   - Port Direct Form II Transposed
   - Add saturation
   - Benchmark vs C++

5. ⏳ **Integration with NIH-plug**
   - Wire parameters
   - Connect to process()

### Week 3

6. ⏳ **Envelope follower**
7. ⏳ **Testing in DAWs**
8. ⏳ **Documentation + README**

---

## Conclusion

**The Rust port is FEASIBLE and RECOMMENDED.**

You already have:
- ✅ NIH-plug framework set up
- ✅ Basic DSP library structure
- ✅ EMU shape data in JSON

You need to port:
- ⚠️ Z-plane math (2 days)
- ⚠️ 6-section cascade (1 day)
- ⚠️ NIH-plug integration (3 days)
- ⚠️ Envelope follower (1 day)

**Total: ~3 weeks to a production-ready Rust plugin** with better safety, faster builds, and modern plugin formats.

**Recommendation: START WITH PHASE 1 (DSP math) THIS WEEK.**

---

**End of Strategy Document**
