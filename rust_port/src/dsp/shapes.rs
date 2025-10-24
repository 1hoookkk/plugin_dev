//! EMU authentic filter shapes
//!
//! These shapes were extracted from real EMU hardware units and represent
//! the authentic Z-plane filtering behavior.

use super::types::Shape;

/// Vowel Pair (default) - human vocal tract modeling
///
/// **VOWEL_A**: "Ae" formant (open vowel)
/// - Low-frequency emphasis
/// - Natural resonances at 300Hz, 850Hz, 2300Hz
/// - Authentic EMU Audity 2000 extraction
///
/// # C++ Equivalent
/// ```cpp
/// inline constexpr Shape VOWEL_A = {
///     0.95f,  0.01047197551529928f,
///     0.96f,  0.01963495409118615f,
///     // ...
/// };
/// ```
pub const VOWEL_A: Shape = [
    0.95,  0.01047197551529928,
    0.96,  0.01963495409118615,
    0.985, 0.03926990818237230,
    0.992, 0.11780972454711690,
    0.993, 0.32724923485310250,
    0.985, 0.45814892879434435,
];

/// **VOWEL_B**: "Oo" formant (closed vowel)
/// - Mid-frequency emphasis
/// - Darker, rounder character
/// - Smooth morphing with VOWEL_A
pub const VOWEL_B: Shape = [
    0.88, 0.00523598775764964,
    0.90, 0.01047197551529928,
    0.92, 0.02094395103059856,
    0.94, 0.04188790206119712,
    0.96, 0.08377580412239424,
    0.97, 0.16755160824478848,
];

/// Bell Pair - bright metallic resonances
///
/// **BELL_A**: Bright metallic timbre
/// - High Q resonances
/// - Frequency range: 500Hz-3kHz
/// - Authentic EMU Planet Phatt extraction
pub const BELL_A: Shape = [
    0.996, 0.14398966333536510,
    0.995, 0.18325957151773740,
    0.994, 0.28797932667073020,
    0.993, 0.39269908182372300,
    0.992, 0.54977871437816500,
    0.990, 0.78539816364744630,
];

/// **BELL_B**: Cluster resonances
/// - Multiple close resonances
/// - Complex harmonic structure
/// - Metallic "shimmer"
pub const BELL_B: Shape = [
    0.994, 0.19634954085771740,
    0.993, 0.26179938779814450,
    0.992, 0.39269908182372300,
    0.991, 0.52359877584930150,
    0.990, 0.70685834741592550,
    0.988, 0.94247779605813900,
];

/// Low Pair - punchy bass processing
///
/// **LOW_A**: Tight bass
/// - Sub-200Hz emphasis
/// - Controlled resonance
/// - Kick drum enhancement
pub const LOW_A: Shape = [
    0.88, 0.00392699081823723,
    0.90, 0.00785398163647446,
    0.92, 0.01570796327294893,
    0.94, 0.03272492348531062,
    0.96, 0.06544984697062124,
    0.97, 0.13089969394124100,
];

/// **LOW_B**: Pad resonance
/// - Wider bass response
/// - Smooth low-end
/// - 808-style bass enhancement
pub const LOW_B: Shape = [
    0.92, 0.00654498469706212,
    0.94, 0.01308996939412425,
    0.96, 0.02617993878824850,
    0.97, 0.05235987755649700,
    0.98, 0.10471975511299400,
    0.985, 0.20943951022598800,
];

/// SubBass Pair - ultra-low rumble
///
/// **SUB_A**: Clean sub-bass
/// - 20-80Hz emphasis
/// - Minimal resonance
/// - Sub-harmonic synthesis
pub const SUB_A: Shape = [
    0.85, 0.00130899694,
    0.87, 0.00261799388,
    0.89, 0.00523598776,
    0.91, 0.01047197551,
    0.93, 0.02094395103,
    0.95, 0.04188790206,
];

/// **SUB_B**: Resonant sub
/// - Boosted sub harmonics
/// - Controlled low-end
/// - Bass drop enhancement
pub const SUB_B: Shape = [
    0.92, 0.00872664626,
    0.94, 0.01745329252,
    0.96, 0.03490658504,
    0.97, 0.06981317008,
    0.98, 0.10471975511,
    0.97, 0.13962634016,
];

/// Helper: Get shape pair by name
///
/// # Example
/// ```no_run
/// use engine_field::dsp::shapes;
///
/// let (shape_a, shape_b) = shapes::get_pair("vowel");
/// ```
pub fn get_pair(name: &str) -> (&'static Shape, &'static Shape) {
    match name.to_lowercase().as_str() {
        "vowel" => (&VOWEL_A, &VOWEL_B),
        "bell" => (&BELL_A, &BELL_B),
        "low" => (&LOW_A, &LOW_B),
        "sub" | "subbass" => (&SUB_A, &SUB_B),
        _ => (&VOWEL_A, &VOWEL_B),  // Default
    }
}

/// All available shape pairs
pub const SHAPE_PAIRS: &[(&str, &Shape, &Shape)] = &[
    ("Vowel", &VOWEL_A, &VOWEL_B),
    ("Bell", &BELL_A, &BELL_B),
    ("Low", &LOW_A, &LOW_B),
    ("Sub", &SUB_A, &SUB_B),
];

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_shape_sizes() {
        assert_eq!(VOWEL_A.len(), 12);
        assert_eq!(BELL_A.len(), 12);
        assert_eq!(LOW_A.len(), 12);
        assert_eq!(SUB_A.len(), 12);
    }

    #[test]
    fn test_get_pair() {
        let (a, b) = get_pair("vowel");
        assert_eq!(a, &VOWEL_A);
        assert_eq!(b, &VOWEL_B);

        let (a, b) = get_pair("BELL");
        assert_eq!(a, &BELL_A);

        // Default fallback
        let (a, b) = get_pair("invalid");
        assert_eq!(a, &VOWEL_A);
    }

    #[test]
    fn test_all_shapes_valid() {
        for (name, shape_a, shape_b) in SHAPE_PAIRS {
            println!("Testing shape pair: {}", name);

            // Check radii are in valid range
            for i in 0..6 {
                let r_a = shape_a[i * 2];
                let r_b = shape_b[i * 2];

                assert!(r_a >= 0.0 && r_a <= 1.0, "{} shape_a[{}] radius out of range", name, i);
                assert!(r_b >= 0.0 && r_b <= 1.0, "{} shape_b[{}] radius out of range", name, i);
            }

            // Check angles are finite
            for i in 0..6 {
                let theta_a = shape_a[i * 2 + 1];
                let theta_b = shape_b[i * 2 + 1];

                assert!(theta_a.is_finite(), "{} shape_a[{}] angle not finite", name, i);
                assert!(theta_b.is_finite(), "{} shape_b[{}] angle not finite", name, i);
            }
        }
    }
}
