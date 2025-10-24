//! Main plugin implementation
//!
//! Integrates the ZPlaneFilter DSP with NIH-plug framework.

use nih_plug::prelude::*;
use std::sync::Arc;

use crate::dsp::{ZPlaneFilter, EnvelopeFollower, shapes};
use crate::params::FieldParams;

/// Engine:Field plugin - Authentic EMU Z-plane filtering
///
/// # Architecture
///
/// ```text
/// ┌────────────────────────────────────────┐
/// │ DAW (Host)                             │
/// │  - Sends audio buffers                 │
/// │  - Updates parameters                  │
/// │  - Handles automation                  │
/// └──────────────┬─────────────────────────┘
///                ↓
/// ┌────────────────────────────────────────┐
/// │ FieldPlugin (NIH-plug)                 │
/// │  - Parameter management                │
/// │  - Smoothing                           │
/// │  - State persistence                   │
/// └──────────────┬─────────────────────────┘
///                ↓
/// ┌────────────────────────────────────────┐
/// │ ZPlaneFilter (Pure DSP)                │
/// │  - Pole mathematics                    │
/// │  - Biquad cascades                     │
/// │  - No state (stateless transform)      │
/// └────────────────────────────────────────┘
/// ```
pub struct FieldPlugin {
    /// Plugin parameters (shared with host)
    params: Arc<FieldParams>,

    /// Z-plane filter (DSP engine)
    filter: ZPlaneFilter,

    /// Envelope follower (for CHARACTER modulation)
    envelope: EnvelopeFollower,

    /// Test tone phase accumulator
    test_tone_phase: f64,

    /// Sample rate
    sample_rate: f32,
}

impl Default for FieldPlugin {
    fn default() -> Self {
        // Load default shape pair (VOWEL_A / VOWEL_B)
        let (shape_a, shape_b) = shapes::get_pair("vowel");

        Self {
            params: Arc::new(FieldParams::default()),
            filter: ZPlaneFilter::new(shape_a, shape_b),
            envelope: EnvelopeFollower::new(),
            test_tone_phase: 0.0,
            sample_rate: 48000.0,
        }
    }
}

impl Plugin for FieldPlugin {
    const NAME: &'static str = "Engine:Field";
    const VENDOR: &'static str = "Engine Suite";
    const URL: &'static str = "https://github.com/1hoookkk/plugin_dev";
    const EMAIL: &'static str = "";

    const VERSION: &'static str = env!("CARGO_PKG_VERSION");

    // Stereo in/out
    const AUDIO_IO_LAYOUTS: &'static [AudioIOLayout] = &[AudioIOLayout {
        main_input_channels: NonZeroU32::new(2),
        main_output_channels: NonZeroU32::new(2),
        aux_input_ports: &[],
        aux_output_ports: &[],
        names: PortNames::const_default(),
    }];

    const MIDI_INPUT: MidiConfig = MidiConfig::None;
    const MIDI_OUTPUT: MidiConfig = MidiConfig::None;

    const SAMPLE_ACCURATE_AUTOMATION: bool = true;

    type SysExMessage = ();
    type BackgroundTask = ();

    fn params(&self) -> Arc<dyn Params> {
        self.params.clone()
    }

    fn initialize(
        &mut self,
        _audio_io_layout: &AudioIOLayout,
        buffer_config: &BufferConfig,
        _context: &mut impl InitContext<Self>,
    ) -> bool {
        // Store sample rate
        self.sample_rate = buffer_config.sample_rate;

        // Prepare DSP
        self.filter.prepare(self.sample_rate);
        self.filter.set_saturation(crate::dsp::constants::AUTHENTIC_SATURATION);

        self.envelope.prepare(self.sample_rate);
        self.envelope.set_attack_ms(0.489);  // EMU authentic
        self.envelope.set_release_ms(80.0);
        self.envelope.set_depth(0.75);  // v1.0.1 calibrated

        // Reset test tone
        self.test_tone_phase = 0.0;

        true
    }

    fn reset(&mut self) {
        // Reset all DSP state
        self.filter.reset();
        self.envelope.reset();
        self.test_tone_phase = 0.0;
    }

    fn process(
        &mut self,
        buffer: &mut Buffer,
        _aux: &mut AuxiliaryBuffers,
        _context: &mut impl ProcessContext<Self>,
    ) -> ProcessStatus {
        // Quick bypass check (no processing if bypassed)
        if self.params.is_bypassed() {
            return ProcessStatus::Normal;
        }

        let num_samples = buffer.samples();

        // Get channel slices
        let mut channel_samples = buffer.as_slice();
        if channel_samples.len() < 2 {
            return ProcessStatus::Normal;  // Need stereo
        }

        let (left_channel, right_channel) = channel_samples.split_at_mut(1);
        let left = &mut left_channel[0];
        let right = &mut right_channel[0];

        // Test tone (if enabled)
        if self.params.is_test_tone_enabled() {
            self.generate_test_tone(left, right);
            return ProcessStatus::Normal;  // Skip DSP when test tone is on
        }

        // Get parameter values
        let character_base = self.params.character.smoothed.next();
        let mix = self.params.mix.smoothed.next();
        let output_gain = util::db_to_gain(self.params.output.smoothed.next());
        let effect_mode = self.params.is_effect_mode();
        let intensity = self.params.intensity_normalized();

        // Envelope follower on left channel
        let mut env_value = 0.0;
        for &sample in left.iter() {
            env_value = self.envelope.process(sample);
        }

        // Modulate CHARACTER by envelope (±20% range)
        // Convert percentage to normalized [0, 1]
        let character_normalized = character_base * 0.01;
        let modulated_character = (character_normalized + env_value * 0.2).clamp(0.0, 1.0);

        // EFFECT mode: solo wet signal (100% wet, ignores MIX)
        let effective_mix = if effect_mode {
            1.0
        } else {
            mix * 0.01
        };

        // Update filter coefficients (once per block)
        self.filter.update_coeffs(modulated_character, intensity);

        // Process stereo audio
        let drive = crate::dsp::constants::AUTHENTIC_DRIVE;
        self.filter.process_stereo(left, right, drive, effective_mix);

        // Apply output gain
        for sample in left.iter_mut() {
            *sample *= output_gain;
        }
        for sample in right.iter_mut() {
            *sample *= output_gain;
        }

        ProcessStatus::Normal
    }
}

impl ClapPlugin for FieldPlugin {
    const CLAP_ID: &'static str = "com.engine-suite.engine-field";
    const CLAP_DESCRIPTION: Option<&'static str> =
        Some("Authentic EMU Z-plane filtering with geodesic morphing");
    const CLAP_MANUAL_URL: Option<&'static str> = Some(Self::URL);
    const CLAP_SUPPORT_URL: Option<&'static str> = None;
    const CLAP_FEATURES: &'static [ClapFeature] = &[
        ClapFeature::AudioEffect,
        ClapFeature::Stereo,
        ClapFeature::Filter,
    ];
}

impl Vst3Plugin for FieldPlugin {
    const VST3_CLASS_ID: [u8; 16] = *b"EngineFieldRust1";
    const VST3_SUBCATEGORIES: &'static [Vst3SubCategory] = &[
        Vst3SubCategory::Fx,
        Vst3SubCategory::Filter,
    ];
}

impl FieldPlugin {
    /// Generate 440 Hz test tone (stereo)
    ///
    /// Used for frequency response testing and validation.
    fn generate_test_tone(&mut self, left: &mut [f32], right: &mut [f32]) {
        const FREQ_HZ: f64 = 440.0;
        const AMPLITUDE: f32 = 0.05;  // -26 dB

        let increment = 2.0 * std::f64::consts::PI * FREQ_HZ / self.sample_rate as f64;

        for (l, r) in left.iter_mut().zip(right.iter_mut()) {
            let sample = (self.test_tone_phase.sin() as f32) * AMPLITUDE;
            *l = sample;
            *r = sample;

            self.test_tone_phase += increment;
            if self.test_tone_phase >= 2.0 * std::f64::consts::PI {
                self.test_tone_phase -= 2.0 * std::f64::consts::PI;
            }
        }
    }
}

// Export plugin
nih_export_clap!(FieldPlugin);
nih_export_vst3!(FieldPlugin);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_plugin_creation() {
        let plugin = FieldPlugin::default();
        assert_eq!(plugin.sample_rate, 48000.0);
    }

    #[test]
    fn test_plugin_metadata() {
        assert_eq!(FieldPlugin::NAME, "Engine:Field");
        assert_eq!(FieldPlugin::VENDOR, "Engine Suite");
        assert!(!FieldPlugin::VERSION.is_empty());
    }
}
