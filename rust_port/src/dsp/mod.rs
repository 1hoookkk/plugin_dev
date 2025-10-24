//! DSP module - Z-plane filtering implementation
//!
//! This module provides a complete, RT-safe implementation of the
//! Engine:Field Z-plane filter in Rust.

pub mod types;
pub mod zplane_math;
pub mod biquad;
pub mod filter;
pub mod envelope;
pub mod shapes;

// Re-export main types
pub use types::{PolePair, BiquadCoeffs, Shape, constants};
pub use biquad::{BiquadSection, BiquadCascade, Cascade6};
pub use filter::ZPlaneFilter;
pub use envelope::EnvelopeFollower;

// Re-export math functions
pub use zplane_math::{
    interpolate_pole,
    remap_pole_48k_to_fs,
    pole_to_biquad,
    wrap_angle,
};
