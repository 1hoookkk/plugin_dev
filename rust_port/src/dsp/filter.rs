//! Complete Z-plane filter implementation - the "generative model"
//!
//! This module implements the heart of Engine:Field: converting a simple
//! morph parameter into complex filter behavior via pole mathematics.
//!
//! **Generative Model Approach:**
//! Instead of storing pre-baked coefficient tables, we:
//! 1. Store compact pole pairs (r, θ) for shapes A and B
//! 2. Interpolate poles mathematically (geodesic or linear)
//! 3. Remap poles via bilinear transform for sample rate independence
//! 4. Convert poles → biquad coefficients on-the-fly
//!
//! **Memory Efficiency:**
//! - Shape storage: 6 pole pairs × 2 floats = 48 bytes per shape
//! - Coefficient table alternative: 6 biquads × 5 coeffs × 256 morph steps = 7.5 KB
//! - Generative model: **16× less memory** + smooth interpolation at any resolution

use super::types::{PolePair, BiquadCoeffs, Shape, constants};
use super::biquad::{BiquadSection, Cascade6};
use super::zplane_math::{interpolate_pole, remap_pole_48k_to_fs, pole_to_biquad};
use super::envelope::EnvelopeFollower;

/// Z-plane filter - the "generative model"
///
/// This struct encapsulates the mathematical model that generates
/// filter coefficients from a morph parameter.
///
/// # Architecture Decision
///
/// ```text
/// ┌──────────────────────────────────────────────┐
/// │ User turns MORPH knob (0-100%)               │
/// └──────────────────┬───────────────────────────┘
///                    ↓
/// ┌──────────────────────────────────────────────┐
/// │ ZPlaneFilter (Generative Model)              │
/// │   1. Interpolate poles (geodesic)            │
/// │   2. Remap for sample rate (bilinear)        │
/// │   3. Boost by intensity                      │
/// │   4. Convert poles → biquad coeffs           │
/// └──────────────────┬───────────────────────────┘
///                    ↓
/// ┌──────────────────────────────────────────────┐
/// │ 6× BiquadSection (signal processing)         │
/// │   - Direct Form II Transposed                │
/// │   - Per-section saturation                   │
/// └──────────────────────────────────────────────┘
/// ```
///
/// # Memory Layout
/// ```text
/// ZPlaneFilter (512 bytes):
/// ┌────────────────────────────────────────┐
/// │ cascade_l: Cascade6   (192 bytes)     │  Left channel filter chain
/// ├────────────────────────────────────────┤
/// │ cascade_r: Cascade6   (192 bytes)     │  Right channel filter chain
/// ├────────────────────────────────────────┤
/// │ poles_a: [PolePair; 6] (48 bytes)     │  Shape A (compact!)
/// ├────────────────────────────────────────┤
/// │ poles_b: [PolePair; 6] (48 bytes)     │  Shape B (compact!)
/// ├────────────────────────────────────────┤
/// │ last_interp_poles: [PolePair; 6]      │  Cached for UI
/// ├────────────────────────────────────────┤
/// │ sample_rate: f32                       │
/// └────────────────────────────────────────┘
/// ```
///
/// # C++ Equivalent
/// ```cpp
/// struct ZPlaneFilter {
///     BiquadCascade<6> cascadeL, cascadeR;
///     std::array<PolePair, 6> polesA, polesB;
///     std::array<PolePair, 6> lastInterpPoles;
///     double sr;
///
///     void updateCoeffsBlock(int samplesPerBlock);
///     void process(float* left, float* right, int num);
/// };
/// ```
#[derive(Debug)]
pub struct ZPlaneFilter {
    // Per-channel biquad cascades (12th-order IIR)
    cascade_l: Cascade6,
    cascade_r: Cascade6,

    // Shape definitions (compact pole-pair storage)
    poles_a: [PolePair; 6],
    poles_b: [PolePair; 6],

    // Cached interpolated poles (for UI visualization)
    last_interp_poles: [PolePair; 6],

    // Sample rate
    sample_rate: f32,

    // Last computed morph/intensity (for static parameter fast-path)
    last_morph: f32,
    last_intensity: f32,
}

impl ZPlaneFilter {
    /// Create new Z-plane filter with given shape pair
    ///
    /// # Arguments
    /// * `shape_a` - First shape (12 floats: r0,θ0, r1,θ1, ..., r5,θ5)
    /// * `shape_b` - Second shape (12 floats)
    ///
    /// # Example
    /// ```no_run
    /// use engine_field::dsp::ZPlaneFilter;
    ///
    /// // VOWEL_A shape from EMU hardware
    /// let shape_a = [
    ///     0.95, 0.01047, 0.96, 0.01963, 0.985, 0.03926,
    ///     0.992, 0.1178, 0.993, 0.3272, 0.985, 0.4581
    /// ];
    ///
    /// // VOWEL_B shape
    /// let shape_b = [
    ///     0.88, 0.00524, 0.90, 0.01047, 0.92, 0.02094,
    ///     0.94, 0.04189, 0.96, 0.08378, 0.97, 0.16755
    /// ];
    ///
    /// let mut filter = ZPlaneFilter::new(&shape_a, &shape_b);
    /// filter.prepare(48000.0);
    /// ```
    pub fn new(shape_a: &Shape, shape_b: &Shape) -> Self {
        use super::types::load_shape;

        Self {
            cascade_l: Cascade6::new(),
            cascade_r: Cascade6::new(),
            poles_a: load_shape(shape_a),
            poles_b: load_shape(shape_b),
            last_interp_poles: [PolePair::new(0.5, 0.0); 6],
            sample_rate: constants::REFERENCE_SR as f32,
            last_morph: 0.5,
            last_intensity: constants::AUTHENTIC_INTENSITY,
        }
    }

    /// Prepare for processing at given sample rate
    ///
    /// # RT-Safety
    /// ✅ Can be called from audio thread (no allocations)
    /// ⚠️ Typically called once in prepareToPlay()
    pub fn prepare(&mut self, sample_rate: f32) {
        self.sample_rate = sample_rate;
        self.cascade_l.reset();
        self.cascade_r.reset();

        // Initial coefficient calculation
        self.update_coeffs(0.5, constants::AUTHENTIC_INTENSITY);
    }

    /// Reset all filter state
    #[inline]
    pub fn reset(&mut self) {
        self.cascade_l.reset();
        self.cascade_r.reset();
    }

    /// Update filter coefficients from morph and intensity parameters
    ///
    /// **This is the generative model in action!**
    ///
    /// # Arguments
    /// * `morph` - Interpolation parameter [0, 1] (0=shape A, 1=shape B)
    /// * `intensity` - Pole radius boost [0, 1] (higher → sharper resonance)
    ///
    /// # Algorithm
    /// ```text
    /// For each of 6 pole pairs:
    ///   1. Interpolate poles A[i] and B[i] at morph parameter
    ///      - Use geodesic (log-space) interpolation
    ///      - Shortest path for angle
    ///
    ///   2. Remap pole from 48kHz reference to target sample rate
    ///      - Bilinear transform preserves frequency mapping
    ///      - Fast-path if sample rate = 48kHz (skip complex math)
    ///
    ///   3. Apply intensity boost
    ///      - radius *= (1 + intensity × 0.06)
    ///      - Clamp to MAX_POLE_RADIUS (0.995) for stability
    ///
    ///   4. Convert pole → biquad coefficients
    ///      - Denominator (poles): a1 = -2r·cos(θ), a2 = r²
    ///      - Numerator (zeros): rz = 0.9r
    ///      - Normalize to prevent gain explosion
    ///
    ///   5. Update both L/R cascades with same coefficients
    /// ```
    ///
    /// # Performance
    /// - 6 poles × ~100 cycles/pole = ~600 cycles/block
    /// - Amortized over 512 samples = ~1.2 cycles/sample
    /// - Static parameter fast-path: skip if no change (see C++ optimization)
    ///
    /// # RT-Safety
    /// ✅ No allocations
    /// ✅ No system calls
    /// ✅ Deterministic latency
    ///
    /// # C++ Equivalent
    /// ```cpp
    /// void updateCoeffsBlock(int samplesPerBlock) {
    ///     morphSmooth.skip(samplesPerBlock);
    ///     intensitySmooth.skip(samplesPerBlock);
    ///
    ///     // Fast-path: skip if parameters stable (Rust TODO)
    ///     lastMorph = morphSmooth.getCurrentValue();
    ///     lastIntensity = intensitySmooth.getCurrentValue();
    ///
    ///     const float intensityBoost = 1.0f + lastIntensity * 0.06f;
    ///
    ///     for (int i = 0; i < 6; ++i) {
    ///         // 1) Interpolate
    ///         PolePair p48k = interpolatePole(polesA[i], polesB[i], lastMorph);
    ///         // 2) Bilinear remap
    ///         PolePair pm = remapPole48kToFs(p48k, sr);
    ///         // 3) Intensity boost
    ///         pm.r = std::min(pm.r * intensityBoost, MAX_POLE_RADIUS);
    ///         lastInterpPoles[i] = pm;
    ///     }
    ///
    ///     // 4-5) Convert to biquad and update cascades
    ///     for (int ch = 0; ch < 2; ++ch) {
    ///         auto& cas = (ch == 0 ? cascadeL : cascadeR);
    ///         for (int i = 0; i < 6; ++i) {
    ///             float a1, a2, b0, b1, b2;
    ///             poleToBiquad(lastInterpPoles[i], a1, a2, b0, b1, b2);
    ///             cas.sections[i].setCoeffs(b0, b1, b2, a1, a2);
    ///         }
    ///     }
    /// }
    /// ```
    pub fn update_coeffs(&mut self, morph: f32, intensity: f32) {
        let morph = morph.clamp(0.0, 1.0);
        let intensity = intensity.clamp(0.0, 1.0);

        // TODO: Fast-path optimization (skip if morph and intensity unchanged)
        // This would save ~60-80% of CPU when parameters are static
        // if morph == self.last_morph && intensity == self.last_intensity {
        //     return;
        // }

        self.last_morph = morph;
        self.last_intensity = intensity;

        // Intensity boost: scales pole radius (higher → sharper resonance)
        // 0.06 factor empirically calibrated to EMU hardware response curve
        let intensity_boost = 1.0 + intensity * constants::INTENSITY_SCALE;

        // Generate coefficients for each pole pair
        for i in 0..6 {
            // 1. Interpolate in 48k reference domain (geodesic for authentic EMU sound)
            let p48k = interpolate_pole(
                self.poles_a[i],
                self.poles_b[i],
                morph,
                true  // Use geodesic interpolation
            );

            // 2. Bilinear remap from 48k to actual sample rate
            let mut pm = remap_pole_48k_to_fs(p48k, self.sample_rate as f64);

            // 3. Apply intensity boost and EMU hardware clamp
            pm.r = (pm.r * intensity_boost).min(constants::MAX_POLE_RADIUS);

            // Cache for UI visualization
            self.last_interp_poles[i] = pm;

            // 4. Convert pole → biquad coefficients
            let coeffs = pole_to_biquad(pm);

            // 5. Update both L/R cascades (stereo uses same coefficients)
            self.cascade_l.sections[i].coeffs = coeffs;
            self.cascade_r.sections[i].coeffs = coeffs;
        }
    }

    /// Get last interpolated poles (for UI visualization)
    ///
    /// Returns array of 6 pole pairs showing current filter shape in Z-plane.
    /// Useful for drawing pole-zero plots or frequency response curves.
    #[inline]
    pub fn last_poles(&self) -> &[PolePair; 6] {
        &self.last_interp_poles
    }

    /// Process stereo audio with dry/wet mix
    ///
    /// # Arguments
    /// * `left` - Left channel buffer (input/output)
    /// * `right` - Right channel buffer (input/output)
    /// * `drive` - Pre-drive amount [0, 1] (tanh saturation before filtering)
    /// * `mix` - Dry/wet blend [0, 1] (0=dry, 1=wet, uses equal-power)
    ///
    /// # Algorithm
    /// ```text
    /// For each sample:
    ///   1. Capture dry input (before any processing)
    ///   2. Apply pre-drive: tanh(input × (1 + drive × 4))
    ///   3. Process through 6-section cascade (12th-order filtering)
    ///   4. Equal-power mix: wet×√mix + dry×√(1-mix)
    /// ```
    ///
    /// # Performance
    /// Per sample (stereo):
    /// - 2× pre-drive tanh: ~80 cycles
    /// - 2× 6-section cascade: ~840 cycles
    /// - 2× equal-power mix: ~12 cycles
    /// **Total: ~932 cycles/frame @ 48kHz**
    ///
    /// # RT-Safety
    /// ✅ No allocations
    /// ✅ No branches (except tanh fast-path in biquad)
    /// ✅ Deterministic latency
    ///
    /// # C++ Equivalent
    /// ```cpp
    /// void process(float* left, float* right, int num) {
    ///     for (int n = 0; n < num; ++n) {
    ///         const float drive = driveSmooth.getNextValue();
    ///         const float mix = mixSmooth.getNextValue();
    ///         const float driveGain = 1.0f + drive * 4.0f;
    ///
    ///         const float inL = left[n];
    ///         const float inR = right[n];
    ///
    ///         float l = std::tanh(inL * driveGain);
    ///         float r = std::tanh(inR * driveGain);
    ///
    ///         float wetL = cascadeL.process(l);
    ///         float wetR = cascadeR.process(r);
    ///
    ///         const float wetG = std::sqrt(mix);
    ///         const float dryG = std::sqrt(1.0f - mix);
    ///         left[n] = wetL * wetG + inL * dryG;
    ///         right[n] = wetR * wetG + inR * dryG;
    ///     }
    /// }
    /// ```
    #[inline]
    pub fn process_stereo(&mut self, left: &mut [f32], right: &mut [f32], drive: f32, mix: f32) {
        let drive = drive.clamp(0.0, 1.0);
        let mix = mix.clamp(0.0, 1.0);

        // Pre-drive gain (1.0 to 5.0 range) → tanh soft clipping
        // 4.0 scaling gives ~12dB boost at max drive setting
        let drive_gain = 1.0 + drive * constants::DRIVE_SCALE;

        // Equal-power mixing coefficients
        let wet_g = mix.sqrt();
        let dry_g = (1.0 - mix).sqrt();

        for (l_samp, r_samp) in left.iter_mut().zip(right.iter_mut()) {
            // Capture true dry input BEFORE any processing
            let dry_l = *l_samp;
            let dry_r = *r_samp;

            // Pre-drive (authentic: tanh on input)
            let mut wet_l = (dry_l * drive_gain).tanh();
            let mut wet_r = (dry_r * drive_gain).tanh();

            // Process through 6-section cascade
            wet_l = self.cascade_l.process(wet_l);
            wet_r = self.cascade_r.process(wet_r);

            // Equal-power mix (preserves perceived loudness)
            // Use TRUE dry signal (not driven) for authentic bypass tone
            *l_samp = wet_l * wet_g + dry_l * dry_g;
            *r_samp = wet_r * wet_g + dry_r * dry_g;
        }
    }

    /// Set saturation amount for all sections
    ///
    /// # Arguments
    /// * `sat` - Saturation amount [0, 1] (0=clean, 1=heavy saturation)
    pub fn set_saturation(&mut self, sat: f32) {
        let sat = sat.clamp(0.0, 1.0);
        for section in &mut self.cascade_l.sections {
            section.set_saturation(sat);
        }
        for section in &mut self.cascade_r.sections {
            section.set_saturation(sat);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use approx::assert_relative_eq;

    // Load example shapes (VOWEL_A and VOWEL_B from EMU hardware)
    const VOWEL_A: Shape = [
        0.95, 0.01047, 0.96, 0.01963, 0.985, 0.03926,
        0.992, 0.1178, 0.993, 0.3272, 0.985, 0.4581
    ];

    const VOWEL_B: Shape = [
        0.88, 0.00524, 0.90, 0.01047, 0.92, 0.02094,
        0.94, 0.04189, 0.96, 0.08378, 0.97, 0.16755
    ];

    #[test]
    fn test_filter_creation() {
        let filter = ZPlaneFilter::new(&VOWEL_A, &VOWEL_B);
        assert_eq!(filter.sample_rate, 48000.0);
    }

    #[test]
    fn test_coefficient_update() {
        let mut filter = ZPlaneFilter::new(&VOWEL_A, &VOWEL_B);
        filter.prepare(48000.0);

        // Update with morph=0.5 (midpoint)
        filter.update_coeffs(0.5, 0.4);

        // Check that poles were interpolated
        let poles = filter.last_poles();
        assert!(poles[0].r > 0.0 && poles[0].r < 1.0);
    }

    #[test]
    fn test_process_stability() {
        let mut filter = ZPlaneFilter::new(&VOWEL_A, &VOWEL_B);
        filter.prepare(48000.0);
        filter.update_coeffs(0.5, 0.4);

        // Process impulse
        let mut left = vec![1.0; 512];
        let mut right = vec![1.0; 512];

        filter.process_stereo(&mut left, &mut right, 0.0, 1.0);

        // All outputs should be finite
        for &sample in &left {
            assert!(sample.is_finite());
        }
    }

    #[test]
    fn test_dry_wet_mix() {
        let mut filter = ZPlaneFilter::new(&VOWEL_A, &VOWEL_B);
        filter.prepare(48000.0);

        // Test 100% dry (mix=0.0)
        let mut left = vec![1.0; 64];
        let mut right = vec![1.0; 64];
        let left_copy = left.clone();

        filter.process_stereo(&mut left, &mut right, 0.0, 0.0);

        // With 100% dry, output should be very close to input
        for (out, inp) in left.iter().zip(left_copy.iter()) {
            assert_relative_eq!(out, inp, epsilon = 0.1);
        }
    }

    #[test]
    fn test_morph_interpolation() {
        let mut filter = ZPlaneFilter::new(&VOWEL_A, &VOWEL_B);
        filter.prepare(48000.0);

        // Get poles at 0%, 50%, 100% morph
        filter.update_coeffs(0.0, 0.4);
        let poles_0 = *filter.last_poles();

        filter.update_coeffs(0.5, 0.4);
        let poles_50 = *filter.last_poles();

        filter.update_coeffs(1.0, 0.4);
        let poles_100 = *filter.last_poles();

        // Midpoint poles should be between extremes (geodesic interpolation)
        assert!(poles_50[0].r > poles_0[0].r.min(poles_100[0].r));
        assert!(poles_50[0].r < poles_0[0].r.max(poles_100[0].r));
    }
}
