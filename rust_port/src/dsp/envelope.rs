//! Envelope follower for dynamic CHARACTER modulation
//!
//! RT-safe implementation with precomputed coefficients.

/// Envelope follower with attack/release and depth scaling
///
/// # Algorithm
/// Simple one-pole lowpass envelope detector:
/// ```text
/// rect = |input|  (full-wave rectifier)
/// α = attack_coef  (if rect > state)
///   = release_coef (if rect ≤ state)
/// state += α · (rect - state)
/// output = clamp(state · depth, 0, 1)
/// ```
///
/// # Optimization
/// Coefficients precomputed on parameter change (not per-sample):
/// ```text
/// attack_coef  = 1 - exp(-1 / (attack_sec × sample_rate))
/// release_coef = 1 - exp(-1 / (release_sec × sample_rate))
/// ```
///
/// This eliminates ~150 cycles/sample of exp() computation.
///
/// # C++ Equivalent
/// ```cpp
/// struct EnvelopeFollower {
///     float process(float input) noexcept {
///         const float rect = std::abs(input);
///         const float alpha = (rect > state) ? attackCoef_ : releaseCoef_;
///         state += alpha * (rect - state);
///         return std::clamp(state * depth, 0.0f, 1.0f);
///     }
///
///     void prepare(double sampleRate) noexcept {
///         sr = sampleRate;
///         updateCoefficients();
///     }
///
/// private:
///     void updateCoefficients() noexcept {
///         attackCoef_  = 1.0f - std::exp(-1.0f / (attackMs * 0.001f * sr));
///         releaseCoef_ = 1.0f - std::exp(-1.0f / (releaseMs * 0.001f * sr));
///     }
/// };
/// ```
#[derive(Debug, Clone, Copy)]
pub struct EnvelopeFollower {
    /// Current envelope state
    state: f32,

    /// Precomputed attack coefficient
    attack_coef: f32,

    /// Precomputed release coefficient
    release_coef: f32,

    /// Output scaling factor [0, 1]
    pub depth: f32,

    /// Attack time in milliseconds
    attack_ms: f32,

    /// Release time in milliseconds
    release_ms: f32,

    /// Sample rate
    sample_rate: f32,
}

impl EnvelopeFollower {
    /// Create new envelope follower with default settings
    ///
    /// Defaults:
    /// - Attack: 0.489 ms (EMU authentic)
    /// - Release: 80 ms
    /// - Depth: 0.75 (v1.0.1 calibrated)
    pub fn new() -> Self {
        let mut env = Self {
            state: 0.0,
            attack_coef: 0.0,
            release_coef: 0.0,
            depth: 0.75,  // v1.0.1 default
            attack_ms: 0.489,
            release_ms: 80.0,
            sample_rate: 48000.0,
        };

        env.update_coefficients();
        env
    }

    /// Prepare for processing at given sample rate
    ///
    /// # RT-Safety
    /// ✅ Can be called from audio thread (no allocations)
    /// ⚠️ Typically called in prepareToPlay(), not per-block
    pub fn prepare(&mut self, sample_rate: f32) {
        self.sample_rate = sample_rate;
        self.update_coefficients();
    }

    /// Set attack time in milliseconds
    ///
    /// # RT-Safety
    /// ✅ Can be called from audio thread
    /// ⚠️ Recomputes exp() - prefer to call infrequently
    pub fn set_attack_ms(&mut self, ms: f32) {
        self.attack_ms = ms;
        self.update_coefficients();
    }

    /// Set release time in milliseconds
    ///
    /// # RT-Safety
    /// ✅ Can be called from audio thread
    /// ⚠️ Recomputes exp() - prefer to call infrequently
    pub fn set_release_ms(&mut self, ms: f32) {
        self.release_ms = ms;
        self.update_coefficients();
    }

    /// Set depth (output scaling) [0, 1]
    ///
    /// # RT-Safety
    /// ✅ Fully RT-safe (no exp() computation)
    #[inline]
    pub fn set_depth(&mut self, depth: f32) {
        self.depth = depth.clamp(0.0, 1.0);
    }

    /// Reset state to zero
    #[inline]
    pub fn reset(&mut self) {
        self.state = 0.0;
    }

    /// Process one sample
    ///
    /// # Arguments
    /// * `input` - Input sample
    ///
    /// # Returns
    /// Envelope value scaled by depth [0, 1]
    ///
    /// # Performance
    /// - 1 abs() → ~1 cycle
    /// - 1 compare → ~1 cycle
    /// - 2 muls, 2 adds → ~4 cycles
    /// - Total: ~7 cycles/sample
    ///
    /// Compare to per-sample exp() version: ~157 cycles/sample
    /// Speedup: **95% reduction**
    ///
    /// # RT-Safety
    /// ✅ No allocations
    /// ✅ No system calls
    /// ✅ Deterministic (no exp())
    #[inline]
    pub fn process(&mut self, input: f32) -> f32 {
        let rect = input.abs();

        // Choose coefficient based on attack/release
        let alpha = if rect > self.state {
            self.attack_coef
        } else {
            self.release_coef
        };

        // One-pole lowpass
        self.state += alpha * (rect - self.state);

        // Scale and clamp
        (self.state * self.depth).clamp(0.0, 1.0)
    }

    /// Precompute exponential coefficients
    ///
    /// Called when sample rate or time constants change.
    ///
    /// # Performance
    /// - 2× exp() → ~300 cycles total
    /// - Amortized over block: negligible
    fn update_coefficients(&mut self) {
        let attack_sec = (self.attack_ms * 0.001).max(1e-6);
        let release_sec = (self.release_ms * 0.001).max(1e-6);

        self.attack_coef = 1.0 - (-1.0 / (attack_sec * self.sample_rate)).exp();
        self.release_coef = 1.0 - (-1.0 / (release_sec * self.sample_rate)).exp();
    }
}

impl Default for EnvelopeFollower {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use approx::assert_relative_eq;

    #[test]
    fn test_envelope_creation() {
        let env = EnvelopeFollower::new();
        assert_eq!(env.state, 0.0);
        assert_eq!(env.depth, 0.75);
    }

    #[test]
    fn test_envelope_attack() {
        let mut env = EnvelopeFollower::new();
        env.prepare(48000.0);

        // Send impulse, state should rise quickly (0.489ms attack)
        let mut state_values = Vec::new();
        for _ in 0..50 {
            let val = env.process(1.0);
            state_values.push(val);
        }

        // State should be rising
        assert!(state_values[10] > state_values[0]);
        assert!(state_values[20] > state_values[10]);
    }

    #[test]
    fn test_envelope_release() {
        let mut env = EnvelopeFollower::new();
        env.prepare(48000.0);

        // Prime the envelope
        for _ in 0..100 {
            env.process(1.0);
        }

        let peak = env.state;

        // Release (80ms)
        let mut state_values = Vec::new();
        for _ in 0..1000 {
            let val = env.process(0.0);
            state_values.push(val);
        }

        // State should be falling
        assert!(state_values[100] < peak);
        assert!(state_values[500] < state_values[100]);
    }

    #[test]
    fn test_envelope_depth_scaling() {
        let mut env = EnvelopeFollower::new();
        env.set_depth(0.5);
        env.prepare(48000.0);

        // Process large signal
        for _ in 0..100 {
            env.process(1.0);
        }

        let output = env.process(1.0);

        // With depth=0.5, output should be ≤ 0.5
        assert!(output <= 0.5);
    }

    #[test]
    fn test_envelope_reset() {
        let mut env = EnvelopeFollower::new();
        env.prepare(48000.0);

        // Prime envelope
        for _ in 0..100 {
            env.process(1.0);
        }

        assert!(env.state > 0.0);

        env.reset();
        assert_eq!(env.state, 0.0);
    }
}
