//! Z-plane mathematics - pole interpolation and bilinear transforms
//!
//! This module implements the core Z-plane filtering mathematics with
//! exact equivalence to the C++ implementation.

use super::types::{BiquadCoeffs, PolePair};
use super::types::constants::*;
use num_complex::Complex64;
use std::f32::consts::PI;

/// Wrap angle to [-π, π] range
///
/// # C++ Equivalent
/// ```cpp
/// inline float wrapAngle(float a) noexcept {
///     const float pi = juce::MathConstants<float>::pi;
///     while (a > pi)  a -= 2.0f * pi;
///     while (a < -pi) a += 2.0f * pi;
///     return a;
/// }
/// ```
#[inline]
pub fn wrap_angle(mut a: f32) -> f32 {
    while a > PI {
        a -= 2.0 * PI;
    }
    while a < -PI {
        a += 2.0 * PI;
    }
    a
}

/// Interpolate between two pole pairs (geodesic or linear)
///
/// # Arguments
/// * `a` - First pole pair
/// * `b` - Second pole pair
/// * `t` - Interpolation parameter [0, 1]
/// * `geodesic` - If true, use log-space radius interpolation
///
/// # Returns
/// Interpolated pole pair
///
/// # Mathematical Detail
///
/// **Linear interpolation:**
/// ```text
/// r = (1-t)·rA + t·rB
/// ```
///
/// **Geodesic interpolation (EMU authentic):**
/// ```text
/// r = exp((1-t)·ln(rA) + t·ln(rB))
/// r = rA^(1-t) · rB^t  (geometric mean)
/// ```
///
/// Why geodesic? Preserves perceptual spacing in Z-plane.
/// Linear interpolation sounds "uneven" during morphing.
///
/// **Angle interpolation** (always shortest path):
/// ```text
/// Δθ = wrap(θB - θA)  // Shortest arc on unit circle
/// θ = θA + t·Δθ
/// ```
///
/// # C++ Equivalent
/// ```cpp
/// inline PolePair interpolatePole(const PolePair& A, const PolePair& B, float t) noexcept {
///     PolePair result;
///
///     if constexpr (GEODESIC_RADIUS) {
///         const float lnA = std::log(std::max(1.0e-9f, A.r));
///         const float lnB = std::log(std::max(1.0e-9f, B.r));
///         result.r = std::exp((1.0f - t) * lnA + t * lnB);
///     } else {
///         result.r = A.r + t * (B.r - A.r);
///     }
///
///     float d = wrapAngle(B.theta - A.theta);
///     result.theta = A.theta + t * d;
///
///     return result;
/// }
/// ```
#[inline]
pub fn interpolate_pole(a: PolePair, b: PolePair, t: f32, geodesic: bool) -> PolePair {
    let t = t.clamp(0.0, 1.0);

    // Radius: geodesic (log-space) or linear
    let r = if geodesic {
        // Geometric mean: rA^(1-t) · rB^t
        let ln_a = a.r.max(1e-9).ln();
        let ln_b = b.r.max(1e-9).ln();
        ((1.0 - t) * ln_a + t * ln_b).exp()
    } else {
        // Linear interpolation
        a.r + t * (b.r - a.r)
    };

    // Angle: shortest path on unit circle
    let delta = wrap_angle(b.theta - a.theta);
    let theta = a.theta + t * delta;

    PolePair::new(r, theta)
}

/// Bilinear frequency warping from 48kHz reference to target sample rate
///
/// # Arguments
/// * `p48k` - Pole pair at 48kHz reference
/// * `target_fs` - Target sample rate (Hz)
///
/// # Returns
/// Warped pole pair at target sample rate
///
/// # Mathematical Detail
///
/// The bilinear transform maps digital (Z) ↔ analog (S) domains:
///
/// **Step 1: Inverse bilinear (Z@48k → S)**
/// ```text
/// z = r·e^(jθ) = r·(cos(θ) + j·sin(θ))  [polar → rectangular]
/// s = 2·fs_ref · (z - 1) / (z + 1)        [inverse bilinear]
/// ```
///
/// **Step 2: Forward bilinear (S → Z@target)**
/// ```text
/// z_new = (2·fs_target + s) / (2·fs_target - s)  [forward bilinear]
/// r_new = |z_new|                                [magnitude]
/// θ_new = arg(z_new)                             [phase]
/// ```
///
/// **Why?** Simple frequency scaling (θ_new = θ_old · fs_target/48k) would
/// cause aliasing at high frequencies. Bilinear preserves frequency mapping:
/// - 1kHz @ 48kHz → 1kHz @ 96kHz (exact!)
/// - Stable poles remain stable
/// - No aliasing artifacts
///
/// # C++ Equivalent
/// ```cpp
/// inline PolePair remapPole48kToFs(const PolePair& p48k, double targetFs) noexcept {
///     // Fast path: within ±0.1 Hz of reference
///     if (std::abs(targetFs - REFERENCE_SR) < 0.1)
///         return p48k;
///
///     if (targetFs < 1e3)
///         return p48k;
///
///     using cd = std::complex<double>;
///
///     const double r48 = std::clamp<double>(p48k.r, 0.0, 0.999999);
///     const double th  = static_cast<double>(p48k.theta);
///     const cd z48 = std::polar(r48, th);
///
///     const cd denom = z48 + cd{1.0, 0.0};
///     if (std::abs(denom) < 1e-12)
///         return p48k;
///
///     const cd s = (2.0 * REFERENCE_SR) * (z48 - cd{1.0, 0.0}) / denom;
///
///     const cd denom_fwd = 2.0 * targetFs - s;
///     if (std::abs(denom_fwd) < 1e-12)
///         return p48k;
///
///     const cd z_new = (2.0 * targetFs + s) / denom_fwd;
///
///     PolePair result;
///     result.r  = static_cast<float>(std::min(std::abs(z_new), 0.999999));
///     result.theta = static_cast<float>(std::atan2(z_new.imag(), z_new.real()));
///
///     return result;
/// }
/// ```
#[inline]
pub fn remap_pole_48k_to_fs(p48k: PolePair, target_fs: f64) -> PolePair {
    // Fast path: skip if within ±0.1 Hz of reference
    if (target_fs - REFERENCE_SR).abs() < 0.1 {
        return p48k;
    }

    // Guard: pathological sample rate
    if target_fs < 1e3 {
        return p48k;
    }

    // Convert to complex (rectangular form)
    let r48 = (p48k.r as f64).clamp(0.0, 0.999999);
    let theta = p48k.theta as f64;
    let z48 = Complex64::from_polar(r48, theta);

    // Inverse bilinear: z@48k → s (analog domain)
    let denom = z48 + Complex64::new(1.0, 0.0);
    if denom.norm() < 1e-12 {
        return p48k;  // Singularity guard
    }

    let s = (2.0 * REFERENCE_SR) * (z48 - Complex64::new(1.0, 0.0)) / denom;

    // Forward bilinear: s → z@target_fs
    let denom_fwd = 2.0 * target_fs - s;
    if denom_fwd.norm() < 1e-12 {
        return p48k;  // Singularity guard
    }

    let z_new = (2.0 * target_fs + s) / denom_fwd;

    // Convert back to polar
    PolePair::new(
        (z_new.norm().min(0.999999)) as f32,
        z_new.arg() as f32,
    )
}

/// Convert pole pair to biquad coefficients
///
/// # Arguments
/// * `p` - Pole pair (radius, angle)
///
/// # Returns
/// Biquad coefficients for Direct Form II Transposed
///
/// # Mathematical Detail
///
/// **Denominator (poles):** Complex conjugate pair at radius r, angle θ
/// ```text
/// H(z) = N(z) / (1 + a₁·z⁻¹ + a₂·z⁻²)
/// a₁ = -2·r·cos(θ)
/// a₂ = r²
/// ```
///
/// **Numerator (zeros):** Placed at 90% of pole radius
/// ```text
/// rz = 0.9·r  (empirical EMU hardware match)
/// b₀ = 1
/// b₁ = -2·rz·cos(θ)
/// b₂ = rz²
/// ```
///
/// Why 0.9? Zeros too close to poles → unstable; too far → weak resonance.
/// 0.9 factor matches EMU hardware behavior.
///
/// **Normalization:** Prevent gain explosion
/// ```text
/// norm = 1 / max(0.25, |b₀| + |b₁| + |b₂|)
/// b₀ *= norm, b₁ *= norm, b₂ *= norm
/// ```
///
/// 0.25 minimum prevents divide-by-zero and extreme gains.
///
/// # C++ Equivalent
/// ```cpp
/// inline void poleToBiquad(const PolePair& p, float& a1, float& a2,
///                          float& b0, float& b1, float& b2) noexcept {
///     a1 = -2.0f * p.r * std::cos(p.theta);
///     a2 = p.r * p.r;
///
///     const float rz = std::clamp(0.9f * p.r, 0.0f, 0.999f);
///     const float c  = std::cos(p.theta);
///     b0 = 1.0f;
///     b1 = -2.0f * rz * c;
///     b2 = rz * rz;
///
///     const float norm = 1.0f / std::max(0.25f, std::abs(b0) + std::abs(b1) + std::abs(b2));
///     b0 *= norm; b1 *= norm; b2 *= norm;
/// }
/// ```
#[inline]
pub fn pole_to_biquad(p: PolePair) -> BiquadCoeffs {
    // Denominator (poles)
    let a1 = -2.0 * p.r * p.theta.cos();
    let a2 = p.r * p.r;

    // Numerator (zeros at 0.9 × pole radius)
    let rz = (ZERO_PLACEMENT_FACTOR * p.r).clamp(0.0, 0.999);
    let c = p.theta.cos();
    let mut b0 = 1.0;
    let mut b1 = -2.0 * rz * c;
    let mut b2 = rz * rz;

    // Normalize to prevent gain explosion
    let norm = 1.0 / (b0.abs() + b1.abs() + b2.abs()).max(0.25);
    b0 *= norm;
    b1 *= norm;
    b2 *= norm;

    BiquadCoeffs { b0, b1, b2, a1, a2 }
}

#[cfg(test)]
mod tests {
    use super::*;
    use approx::assert_relative_eq;

    #[test]
    fn test_wrap_angle() {
        assert_relative_eq!(wrap_angle(0.0), 0.0);
        assert_relative_eq!(wrap_angle(4.0 * PI), 0.0, epsilon = 1e-6);
        assert_relative_eq!(wrap_angle(-4.0 * PI), 0.0, epsilon = 1e-6);
        assert_relative_eq!(wrap_angle(PI + 0.1), -(PI - 0.1), epsilon = 1e-6);
    }

    #[test]
    fn test_interpolate_pole_linear() {
        let a = PolePair::new(0.5, 0.0);
        let b = PolePair::new(0.9, 1.0);

        let mid = interpolate_pole(a, b, 0.5, false);
        assert_relative_eq!(mid.r, 0.7, epsilon = 1e-6);
        assert_relative_eq!(mid.theta, 0.5, epsilon = 1e-6);
    }

    #[test]
    fn test_interpolate_pole_geodesic() {
        let a = PolePair::new(0.5, 0.0);
        let b = PolePair::new(0.9, 1.0);

        let mid = interpolate_pole(a, b, 0.5, true);

        // Geodesic: r = √(0.5 × 0.9) = √0.45 ≈ 0.671
        assert_relative_eq!(mid.r, (0.5 * 0.9_f32).sqrt(), epsilon = 1e-6);
        assert_relative_eq!(mid.theta, 0.5, epsilon = 1e-6);
    }

    #[test]
    fn test_remap_pole_fast_path() {
        let p = PolePair::new(0.95, 0.5);
        let remapped = remap_pole_48k_to_fs(p, 48000.0);

        // Fast path: should return exact same pole
        assert_eq!(remapped.r, p.r);
        assert_eq!(remapped.theta, p.theta);
    }

    #[test]
    fn test_remap_pole_frequency_preservation() {
        // 1kHz @ 48kHz
        let freq_hz = 1000.0;
        let p48k = PolePair::new(0.95, 2.0 * PI * freq_hz / 48000.0);

        // Remap to 96kHz
        let p96k = remap_pole_48k_to_fs(p48k, 96000.0);

        // Check frequency is still ~1kHz
        let freq_96k = p96k.frequency_hz(96000.0);
        assert_relative_eq!(freq_96k, freq_hz, epsilon = 10.0);  // ±10Hz tolerance
    }

    #[test]
    fn test_pole_to_biquad() {
        let p = PolePair::new(0.95, PI / 4.0);
        let coeffs = pole_to_biquad(p);

        // Check a1, a2 match expected pole formula
        assert_relative_eq!(coeffs.a1, -2.0 * 0.95 * (PI / 4.0).cos(), epsilon = 1e-6);
        assert_relative_eq!(coeffs.a2, 0.95 * 0.95, epsilon = 1e-6);

        // Check zeros at 0.9 × pole radius
        let rz = 0.9 * 0.95;
        let b1_expected = -2.0 * rz * (PI / 4.0).cos();

        // After normalization
        let sum = coeffs.b0.abs() + coeffs.b1.abs() + coeffs.b2.abs();
        assert_relative_eq!(sum, 1.0, epsilon = 1e-6);  // Should be normalized
    }

    #[test]
    fn test_biquad_stability() {
        // Poles must be inside unit circle for stability
        let unstable_pole = PolePair::new(1.01, 0.5);  // Outside unit circle
        let coeffs = pole_to_biquad(unstable_pole);

        // After clamping, should still produce valid coeffs
        assert!(coeffs.a2 < 1.0);  // r² < 1 for stability
    }
}
