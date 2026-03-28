//! Bird wing flap synthesis: aerodynamic displacement sounds.
//!
//! Models the sound of bird wings as periodic air displacement events.
//! Each flap is a short filtered noise burst, with rate and character
//! determined by bird size.

use alloc::vec::Vec;
use serde::{Deserialize, Serialize};

use crate::dsp::{DcBlocker, validate_sample_rate};
use crate::error::Result;
use crate::rng::Rng;

/// Bird size affecting wing flap character.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[non_exhaustive]
pub enum BirdSize {
    /// Small bird (sparrow, finch) — fast, high-frequency flaps.
    Small,
    /// Medium bird (pigeon, crow) — moderate flaps.
    Medium,
    /// Large bird (eagle, heron) — slow, powerful whooshing flaps.
    Large,
}

impl BirdSize {
    /// Returns (flap_rate_hz, flap_duration_s, filter_freq, amplitude).
    #[inline]
    #[must_use]
    fn config(self) -> (f32, f32, f32, f32) {
        match self {
            Self::Small => (12.0, 0.015, 4000.0, 0.15),
            Self::Medium => (6.0, 0.03, 2000.0, 0.25),
            Self::Large => (2.5, 0.06, 800.0, 0.4),
        }
    }
}

/// Bird wing flap synthesizer — periodic air displacement.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WingFlap {
    bird_size: BirdSize,
    sample_rate: f32,
    rng: Rng,
    dc_blocker: DcBlocker,
    sample_position: usize,
    // Flap params
    flap_rate: f32,
    flap_duration_samples: usize,
    amplitude: f32,
    // Phase tracking
    flap_phase: f32,
    // Real-time
    intensity: f32,
    #[cfg(feature = "naad-backend")]
    noise_gen: naad::noise::NoiseGenerator,
    #[cfg(feature = "naad-backend")]
    shape_filter: naad::filter::BiquadFilter,
}

impl WingFlap {
    /// Creates a new wing flap synthesizer.
    pub fn new(bird_size: BirdSize, sample_rate: f32) -> Result<Self> {
        validate_sample_rate(sample_rate)?;
        let config = bird_size.config();
        let flap_rate = config.0;
        let flap_duration_samples = (config.1 * sample_rate) as usize;
        let amplitude = config.3;

        #[cfg(feature = "naad-backend")]
        let noise_gen = naad::noise::NoiseGenerator::new(naad::noise::NoiseType::Pink, 7070);
        #[cfg(feature = "naad-backend")]
        let shape_filter = naad::filter::BiquadFilter::new(
            naad::filter::FilterType::BandPass,
            sample_rate,
            config.2,
            1.0,
        )
        .map_err(|e| crate::error::GarjanError::SynthesisFailed(alloc::format!("{e}")))?;

        Ok(Self {
            bird_size,
            sample_rate,
            rng: Rng::new(7070),
            dc_blocker: DcBlocker::new(sample_rate),
            sample_position: 0,
            flap_rate,
            flap_duration_samples,
            amplitude,
            flap_phase: 0.0,
            intensity: 1.0,
            #[cfg(feature = "naad-backend")]
            noise_gen,
            #[cfg(feature = "naad-backend")]
            shape_filter,
        })
    }

    /// Sets the flapping intensity (0.0 = still, 1.0 = active flight).
    pub fn set_intensity(&mut self, intensity: f32) {
        self.intensity = intensity.clamp(0.0, 1.0);
    }

    /// Synthesizes wing flap audio.
    #[inline]
    pub fn synthesize(&mut self, duration: f32) -> Result<Vec<f32>> {
        let num_samples = (self.sample_rate * duration) as usize;
        let mut output = alloc::vec![0.0f32; num_samples];
        self.process_block(&mut output);
        Ok(output)
    }

    /// Fills output buffer with wing flap audio (streaming).
    #[inline]
    pub fn process_block(&mut self, output: &mut [f32]) {
        if self.intensity < 0.001 {
            for sample in output.iter_mut() {
                *sample = 0.0;
                self.dc_blocker.process(0.0);
            }
            self.sample_position += output.len();
            return;
        }

        for sample in output.iter_mut() {
            // Advance flap phase
            self.flap_phase += self.flap_rate * self.intensity / self.sample_rate;
            if self.flap_phase >= 1.0 {
                self.flap_phase -= 1.0;
            }

            // Each flap: short burst at the start of the cycle
            let flap_progress = self.flap_phase * self.sample_rate / self.flap_rate;
            let in_flap = flap_progress < self.flap_duration_samples as f32;

            if in_flap {
                let t = flap_progress / self.flap_duration_samples as f32;
                // Envelope: fast attack, smooth decay
                let env = if t < 0.15 {
                    t / 0.15
                } else {
                    crate::math::f32::exp(-4.0 * (t - 0.15))
                };

                let noise = self.generate_noise();
                *sample = noise * env * self.amplitude * self.intensity;
            } else {
                *sample = 0.0;
            }

            *sample = self.dc_blocker.process(*sample);
        }
        self.sample_position += output.len();
    }

    #[inline]
    fn generate_noise(&mut self) -> f32 {
        #[cfg(feature = "naad-backend")]
        {
            let raw = self.noise_gen.next_sample();
            self.shape_filter.process_sample(raw)
        }
        #[cfg(not(feature = "naad-backend"))]
        {
            (self.rng.next_f32() + self.rng.next_f32()) * 0.5
        }
    }
}
