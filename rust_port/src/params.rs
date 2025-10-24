//! Plugin parameters for Engine:Field
//!
//! Defines all user-facing parameters with proper smoothing and formatting.

use nih_plug::prelude::*;
use std::sync::Arc;

/// All plugin parameters
///
/// This struct is automatically serialized/deserialized by NIH-plug for
/// state persistence and DAW automation.
#[derive(Params)]
pub struct FieldParams {
    /// CHARACTER: Morph between shape A and shape B [0-100%]
    ///
    /// Controls Z-plane pole interpolation (geodesic).
    /// - 0% = Shape A (e.g., VOWEL_A: open vowel)
    /// - 50% = Midpoint (geometric mean in log-space)
    /// - 100% = Shape B (e.g., VOWEL_B: closed vowel)
    ///
    /// **Modulated by envelope follower** (±15% at depth=0.75)
    #[id = "character"]
    pub character: FloatParam,

    /// MIX: Dry/wet blend [0-100%]
    ///
    /// Uses equal-power mixing to maintain perceived loudness:
    /// - wet_gain = √(mix)
    /// - dry_gain = √(1 - mix)
    ///
    /// Ignored when EFFECT mode is ON.
    #[id = "mix"]
    pub mix: FloatParam,

    /// EFFECT: Wet solo mode (ON/OFF)
    ///
    /// When ON: 100% wet signal (ignores MIX knob)
    /// When OFF: Normal dry/wet blending via MIX knob
    ///
    /// Useful for hearing exactly what the filter is doing.
    #[id = "effect"]
    pub effect: BoolParam,

    /// OUTPUT: Makeup gain [-12 to +12 dB]
    ///
    /// Post-processing gain to compensate for filter loudness changes.
    /// Smoothed internally by NIH-plug (20ms ramp).
    #[id = "output"]
    pub output: FloatParam,

    /// BYPASS: True bypass (ON/OFF)
    ///
    /// When ON: Direct passthrough (no DSP)
    /// Smoothed crossfade to avoid clicks.
    #[id = "bypass"]
    pub bypass: BoolParam,

    /// TEST TONE: 440 Hz sine wave generator (hidden, for validation)
    ///
    /// Useful for:
    /// - Frequency response testing
    /// - Phase coherence validation
    /// - Latency measurement
    #[id = "test_tone"]
    pub test_tone: BoolParam,

    /// INTENSITY: Pole radius boost [0-100%] (locked to 40% for authentic EMU sound)
    ///
    /// Hidden parameter - locked to AUTHENTIC_INTENSITY (0.4).
    /// Exposed here for future flexibility, but not user-facing.
    #[id = "intensity"]
    pub intensity: FloatParam,
}

impl Default for FieldParams {
    fn default() -> Self {
        Self {
            // CHARACTER: 50% default (midpoint between shapes)
            character: FloatParam::new(
                "Character",
                50.0,
                FloatRange::Linear {
                    min: 0.0,
                    max: 100.0,
                },
            )
            .with_smoother(SmoothingStyle::Linear(20.0))  // 20ms smoothing
            .with_unit("%")
            .with_value_to_string(formatters::v2s_f32_percentage(0))
            .with_string_to_value(formatters::s2v_f32_percentage()),

            // MIX: 100% default (full wet)
            mix: FloatParam::new(
                "Mix",
                100.0,
                FloatRange::Linear {
                    min: 0.0,
                    max: 100.0,
                },
            )
            .with_smoother(SmoothingStyle::Linear(20.0))  // 20ms smoothing
            .with_unit("%")
            .with_value_to_string(formatters::v2s_f32_percentage(0))
            .with_string_to_value(formatters::s2v_f32_percentage()),

            // EFFECT: OFF by default
            effect: BoolParam::new("Effect", false),

            // OUTPUT: 0 dB default (unity gain)
            output: FloatParam::new(
                "Output",
                0.0,
                FloatRange::Linear {
                    min: -12.0,
                    max: 12.0,
                },
            )
            .with_smoother(SmoothingStyle::Linear(20.0))
            .with_unit(" dB")
            .with_value_to_string(formatters::v2s_f32_rounded(2)),

            // BYPASS: OFF by default
            bypass: BoolParam::new("Bypass", false),

            // TEST TONE: OFF by default (hidden parameter)
            test_tone: BoolParam::new("Test Tone", false)
                .hide(),

            // INTENSITY: 40% locked (authentic EMU)
            intensity: FloatParam::new(
                "Intensity",
                40.0,  // Locked to AUTHENTIC_INTENSITY
                FloatRange::Linear {
                    min: 0.0,
                    max: 100.0,
                },
            )
            .hide()  // Hidden - not exposed to user
            .with_unit("%"),
        }
    }
}

impl FieldParams {
    /// Get CHARACTER as normalized float [0.0, 1.0]
    #[inline]
    pub fn character_normalized(&self) -> f32 {
        self.character.value() * 0.01
    }

    /// Get MIX as normalized float [0.0, 1.0]
    #[inline]
    pub fn mix_normalized(&self) -> f32 {
        self.mix.value() * 0.01
    }

    /// Get INTENSITY as normalized float [0.0, 1.0]
    #[inline]
    pub fn intensity_normalized(&self) -> f32 {
        self.intensity.value() * 0.01
    }

    /// Get OUTPUT as linear gain (from dB)
    #[inline]
    pub fn output_gain(&self) -> f32 {
        util::db_to_gain(self.output.value())
    }

    /// Check if EFFECT mode is active (overrides MIX)
    #[inline]
    pub fn is_effect_mode(&self) -> bool {
        self.effect.value()
    }

    /// Check if bypassed
    #[inline]
    pub fn is_bypassed(&self) -> bool {
        self.bypass.value()
    }

    /// Check if test tone is enabled
    #[inline]
    pub fn is_test_tone_enabled(&self) -> bool {
        self.test_tone.value()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_values() {
        let params = FieldParams::default();

        assert_eq!(params.character.value(), 50.0);
        assert_eq!(params.mix.value(), 100.0);
        assert!(!params.effect.value());
        assert_eq!(params.output.value(), 0.0);
        assert!(!params.bypass.value());
    }

    #[test]
    fn test_normalized_getters() {
        let params = FieldParams::default();

        assert_eq!(params.character_normalized(), 0.5);  // 50% → 0.5
        assert_eq!(params.mix_normalized(), 1.0);        // 100% → 1.0
        assert_eq!(params.intensity_normalized(), 0.4);  // 40% → 0.4
    }

    #[test]
    fn test_output_gain() {
        let params = FieldParams::default();

        // 0 dB → gain of 1.0
        assert!((params.output_gain() - 1.0).abs() < 1e-6);
    }
}
