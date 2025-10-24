//! Engine:Field - Rust Port
//!
//! Authentic EMU Z-plane filtering plugin with waveform visualization.
//! 6-stage biquad cascade (12th-order IIR) with bilinear frequency warping,
//! geodesic pole interpolation, and equal-power mixing.
//!
//! # Architecture
//!
//! This plugin uses a **generative model** approach:
//! - Compact pole-pair storage (48 bytes per shape)
//! - On-the-fly coefficient generation via pole mathematics
//! - Smooth morphing at any resolution (not limited to LUT quantization)
//! - Sample-rate independent via bilinear transform
//!
//! # Example
//!
//! ```no_run
//! use engine_field::dsp::{ZPlaneFilter, shapes};
//!
//! // Create filter with VOWEL shapes
//! let (shape_a, shape_b) = shapes::get_pair("vowel");
//! let mut filter = ZPlaneFilter::new(shape_a, shape_b);
//!
//! // Prepare for 48kHz processing
//! filter.prepare(48000.0);
//!
//! // Update coefficients (morph=50%, intensity=40%)
//! filter.update_coeffs(0.5, 0.4);
//!
//! // Process audio
//! let mut left = vec![0.0f32; 512];
//! let mut right = vec![0.0f32; 512];
//! filter.process_stereo(&mut left, &mut right, 0.2, 1.0);
//! ```

#![deny(unsafe_op_in_unsafe_fn)]
#![warn(missing_docs)]
#![warn(clippy::all)]

pub mod dsp;

// Re-export main types for convenience
pub use dsp::{ZPlaneFilter, EnvelopeFollower, PolePair, BiquadCoeffs};

/// Library version
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

/// Library name
pub const NAME: &str = "Engine:Field (Rust)";

/// Library description
pub const DESCRIPTION: &str = "Authentic EMU Z-plane filtering with geodesic morphing";

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_version_info() {
        assert!(!VERSION.is_empty());
        println!("{} v{}", NAME, VERSION);
    }

    #[test]
    fn test_complete_workflow() {
        use dsp::shapes;

        // Load shapes
        let (shape_a, shape_b) = shapes::get_pair("vowel");

        // Create filter
        let mut filter = ZPlaneFilter::new(shape_a, shape_b);
        filter.prepare(48000.0);

        // Update coefficients
        filter.update_coeffs(0.5, 0.4);

        // Process audio
        let mut left = vec![0.5f32; 512];
        let mut right = vec![0.5f32; 512];

        filter.process_stereo(&mut left, &mut right, 0.0, 1.0);

        // Verify output
        for &sample in &left {
            assert!(sample.is_finite());
        }
    }
}
