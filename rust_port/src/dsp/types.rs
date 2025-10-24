//! Core Z-plane filter types
//!
//! This module defines the fundamental types for Z-plane filtering,
//! providing 1:1 correspondence with the C++ implementation.

use std::f32::consts::PI;

/// Complex pole pair in polar coordinates (radius, angle)
///
/// # C++ Equivalent
/// ```cpp
/// struct PolePair { float r; float theta; };
/// ```
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PolePair {
    /// Radius (0.0 to 0.995) - controls resonance/bandwidth
    /// Closer to 1.0 → sharper resonance
    pub r: f32,

    /// Angle in radians (± π) - controls frequency
    /// θ = 2π × (freq / sample_rate)
    pub theta: f32,
}

impl PolePair {
    /// Create a new pole pair with validation
    #[inline]
    pub fn new(r: f32, theta: f32) -> Self {
        debug_assert!(r >= 0.0 && r <= 1.0, "Pole radius must be in [0, 1]");
        debug_assert!(theta.is_finite(), "Theta must be finite");

        Self { r, theta }
    }

    /// Check if pole is inside unit circle (stable)
    #[inline]
    pub fn is_stable(&self) -> bool {
        self.r < 1.0
    }

    /// Get frequency in Hz at given sample rate
    #[inline]
    pub fn frequency_hz(&self, sample_rate: f32) -> f32 {
        (self.theta.abs() / (2.0 * PI)) * sample_rate
    }
}

/// Biquad filter coefficients (Direct Form II Transposed)
///
/// Transfer function: H(z) = (b0 + b1*z^-1 + b2*z^-2) / (1 + a1*z^-1 + a2*z^-2)
///
/// # C++ Equivalent
/// ```cpp
/// struct BiquadSection {
///     float b0, b1, b2, a1, a2;
/// };
/// ```
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct BiquadCoeffs {
    pub b0: f32,
    pub b1: f32,
    pub b2: f32,
    pub a1: f32,
    pub a2: f32,
}

impl Default for BiquadCoeffs {
    /// Unity gain passthrough (b0=1, all else=0)
    fn default() -> Self {
        Self {
            b0: 1.0,
            b1: 0.0,
            b2: 0.0,
            a1: 0.0,
            a2: 0.0,
        }
    }
}

/// EMU authentic constants (hardware-calibrated values)
///
/// # C++ Equivalent
/// ```cpp
/// inline constexpr float AUTHENTIC_INTENSITY   = 0.4f;
/// inline constexpr float AUTHENTIC_DRIVE       = 0.2f;
/// inline constexpr float AUTHENTIC_SATURATION  = 0.2f;
/// inline constexpr float MAX_POLE_RADIUS       = 0.9950f;
/// ```
pub mod constants {
    pub const AUTHENTIC_INTENSITY: f32 = 0.4;
    pub const AUTHENTIC_DRIVE: f32 = 0.2;
    pub const AUTHENTIC_SATURATION: f32 = 0.2;
    pub const MAX_POLE_RADIUS: f32 = 0.995;
    pub const MIN_POLE_RADIUS: f32 = 0.10;
    pub const REFERENCE_SR: f64 = 48000.0;

    /// Zero placement factor (empirical EMU hardware match)
    /// Zeros at 90% of pole radius → optimal resonance control
    pub const ZERO_PLACEMENT_FACTOR: f32 = 0.9;

    /// Saturation gain scaling
    /// g = 1 + sat × 4 → soft clipping at ±0.25
    pub const SATURATION_SCALE: f32 = 4.0;

    /// Intensity boost scaling
    /// radius *= (1 + intensity × 0.06) → ~6% boost at max
    pub const INTENSITY_SCALE: f32 = 0.06;

    /// Drive gain range
    /// gain = 1 + drive × 4 → ~12dB boost at max
    pub const DRIVE_SCALE: f32 = 4.0;
}

/// EMU filter shapes (6 pole pairs = 12 floats each)
///
/// # C++ Equivalent
/// ```cpp
/// using Shape = std::array<float, 12>;
/// inline constexpr Shape VOWEL_A = { ... };
/// ```
pub type Shape = [f32; 12];

/// Convert flat shape array to pole pairs
///
/// # Arguments
/// * `shape` - Flat array of [r0, θ0, r1, θ1, ..., r5, θ5]
///
/// # Returns
/// Array of 6 pole pairs
///
/// # C++ Equivalent
/// ```cpp
/// template <size_t N>
/// inline void loadShape(const std::array<float, N>& shape,
///                       std::array<PolePair, N/2>& out) noexcept {
///     for (size_t i = 0; i < N/2; ++i) {
///         out[i] = PolePair{ shape[2*i], shape[2*i + 1] };
///     }
/// }
/// ```
#[inline]
pub fn load_shape(shape: &Shape) -> [PolePair; 6] {
    [
        PolePair::new(shape[0], shape[1]),
        PolePair::new(shape[2], shape[3]),
        PolePair::new(shape[4], shape[5]),
        PolePair::new(shape[6], shape[7]),
        PolePair::new(shape[8], shape[9]),
        PolePair::new(shape[10], shape[11]),
    ]
}

#[cfg(test)]
mod tests {
    use super::*;
    use approx::assert_relative_eq;

    #[test]
    fn test_pole_pair_creation() {
        let pole = PolePair::new(0.95, PI / 4.0);
        assert_eq!(pole.r, 0.95);
        assert_eq!(pole.theta, PI / 4.0);
        assert!(pole.is_stable());
    }

    #[test]
    fn test_pole_frequency() {
        let pole = PolePair::new(0.95, 2.0 * PI * 1000.0 / 48000.0);
        let freq = pole.frequency_hz(48000.0);
        assert_relative_eq!(freq, 1000.0, epsilon = 0.1);
    }

    #[test]
    fn test_biquad_default() {
        let coeffs = BiquadCoeffs::default();
        assert_eq!(coeffs.b0, 1.0);
        assert_eq!(coeffs.b1, 0.0);
    }

    #[test]
    fn test_load_shape() {
        let shape: Shape = [
            0.95, 0.01,
            0.96, 0.02,
            0.97, 0.03,
            0.98, 0.04,
            0.99, 0.05,
            0.995, 0.06,
        ];

        let poles = load_shape(&shape);
        assert_eq!(poles.len(), 6);
        assert_eq!(poles[0].r, 0.95);
        assert_eq!(poles[5].theta, 0.06);
    }
}
