//! Surf zone synthesis: breaking waves with approach, crash, and wash phases.
//!
//! Models ocean surf as a cyclic process: wave approaches (rumble builds),
//! breaks (broadband crash), and washes back (receding hiss). More physically
//! accurate than the base Water::Waves synthesizer.

use alloc::vec::Vec;
use serde::{Deserialize, Serialize};

use crate::dsp::{DcBlocker, validate_sample_rate};
use crate::error::Result;
use crate::rng::Rng;

/// Surf intensity affecting wave size and frequency.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[non_exhaustive]
pub enum SurfIntensity {
    /// Gentle lapping — small waves, long period.
    Calm,
    /// Moderate surf — regular breaking waves.
    Moderate,
    /// Heavy surf — large, powerful breakers.
    Heavy,
    /// Storm surf — continuous roar, massive waves.
    Storm,
}

impl SurfIntensity {
    #[inline]
    #[must_use]
    fn config(self) -> (f32, f32, f32) {
        // (wave_period_s, break_amplitude, wash_amplitude)
        match self {
            Self::Calm => (8.0, 0.15, 0.08),
            Self::Moderate => (6.0, 0.35, 0.2),
            Self::Heavy => (4.5, 0.55, 0.35),
            Self::Storm => (3.0, 0.75, 0.5),
        }
    }
}

/// Surf zone synthesizer — breaking wave cycle.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Surf {
    intensity: SurfIntensity,
    sample_rate: f32,
    rng: Rng,
    dc_blocker: DcBlocker,
    sample_position: usize,
    // Wave params
    wave_period_samples: usize,
    break_amp: f32,
    wash_amp: f32,
    // Real-time
    volume: f32,
    // Phase tracking
    wave_phase: f32,
    #[cfg(feature = "naad-backend")]
    crash_noise: naad::noise::NoiseGenerator,
    #[cfg(feature = "naad-backend")]
    crash_filter: naad::filter::BiquadFilter,
    #[cfg(feature = "naad-backend")]
    wash_noise: naad::noise::NoiseGenerator,
    #[cfg(feature = "naad-backend")]
    wash_filter: naad::filter::BiquadFilter,
    #[cfg(feature = "naad-backend")]
    rumble_noise: naad::noise::NoiseGenerator,
    #[cfg(feature = "naad-backend")]
    rumble_filter: naad::filter::BiquadFilter,
}

impl Surf {
    /// Creates a new surf zone synthesizer.
    pub fn new(intensity: SurfIntensity, sample_rate: f32) -> Result<Self> {
        validate_sample_rate(sample_rate)?;
        let (period, break_amp, wash_amp) = intensity.config();
        let wave_period_samples = (period * sample_rate) as usize;

        #[cfg(feature = "naad-backend")]
        let crash_noise = naad::noise::NoiseGenerator::new(naad::noise::NoiseType::White, 8080);
        #[cfg(feature = "naad-backend")]
        let crash_filter = naad::filter::BiquadFilter::new(
            naad::filter::FilterType::BandPass,
            sample_rate,
            2000.0,
            0.8,
        )
        .map_err(|e| crate::error::GarjanError::SynthesisFailed(alloc::format!("{e}")))?;
        #[cfg(feature = "naad-backend")]
        let wash_noise = naad::noise::NoiseGenerator::new(naad::noise::NoiseType::Pink, 8081);
        #[cfg(feature = "naad-backend")]
        let wash_filter = naad::filter::BiquadFilter::new(
            naad::filter::FilterType::LowPass,
            sample_rate,
            3000.0,
            0.5,
        )
        .map_err(|e| crate::error::GarjanError::SynthesisFailed(alloc::format!("{e}")))?;
        #[cfg(feature = "naad-backend")]
        let rumble_noise = naad::noise::NoiseGenerator::new(naad::noise::NoiseType::Brown, 8082);
        #[cfg(feature = "naad-backend")]
        let rumble_filter = naad::filter::BiquadFilter::new(
            naad::filter::FilterType::LowPass,
            sample_rate,
            200.0,
            0.5,
        )
        .map_err(|e| crate::error::GarjanError::SynthesisFailed(alloc::format!("{e}")))?;

        Ok(Self {
            intensity,
            sample_rate,
            rng: Rng::new(8080),
            dc_blocker: DcBlocker::new(sample_rate),
            sample_position: 0,
            wave_period_samples,
            break_amp,
            wash_amp,
            volume: 1.0,
            wave_phase: 0.0,
            #[cfg(feature = "naad-backend")]
            crash_noise,
            #[cfg(feature = "naad-backend")]
            crash_filter,
            #[cfg(feature = "naad-backend")]
            wash_noise,
            #[cfg(feature = "naad-backend")]
            wash_filter,
            #[cfg(feature = "naad-backend")]
            rumble_noise,
            #[cfg(feature = "naad-backend")]
            rumble_filter,
        })
    }

    /// Sets the volume (0.0 = silent, 1.0 = full).
    pub fn set_volume(&mut self, volume: f32) {
        self.volume = volume.clamp(0.0, 1.0);
    }

    /// Synthesizes surf audio.
    #[inline]
    pub fn synthesize(&mut self, duration: f32) -> Result<Vec<f32>> {
        let num_samples = (self.sample_rate * duration) as usize;
        let mut output = alloc::vec![0.0f32; num_samples];
        self.process_block(&mut output);
        Ok(output)
    }

    /// Fills output buffer with surf audio (streaming).
    #[inline]
    pub fn process_block(&mut self, output: &mut [f32]) {
        if self.volume < 0.001 {
            for s in output.iter_mut() {
                *s = 0.0;
                self.dc_blocker.process(0.0);
            }
            self.sample_position += output.len();
            return;
        }

        let mut period = self.wave_period_samples as f32;

        for sample in output.iter_mut() {
            // Advance wave phase
            self.wave_phase += 1.0 / period;
            if self.wave_phase >= 1.0 {
                self.wave_phase -= 1.0;
                // New jitter per wave cycle
                let jitter = 1.0 + self.rng.next_f32_range(-0.1, 0.1);
                period = self.wave_period_samples as f32 * jitter;
            }

            let p = self.wave_phase;

            // Phase-dependent envelopes
            // Approach rumble: builds from 0.0 to 0.3
            let rumble_env = if p < 0.3 {
                p / 0.3
            } else if p < 0.5 {
                1.0 - (p - 0.3) / 0.2
            } else {
                0.0
            };

            // Break crash: peaks at 0.35, fast decay
            let crash_env = if p > 0.25 && p < 0.55 {
                let t = (p - 0.25) / 0.3;
                let s = crate::math::f32::sin(core::f32::consts::PI * t);
                s * s
            } else {
                0.0
            };

            // Wash: builds after crash, slow decay 0.5 to 1.0
            let wash_env = if p > 0.45 {
                let t = (p - 0.45) / 0.55;
                crate::math::f32::exp(-3.0 * t)
            } else {
                0.0
            };

            // Generate noise layers
            #[cfg(feature = "naad-backend")]
            {
                let rumble = self
                    .rumble_filter
                    .process_sample(self.rumble_noise.next_sample())
                    * rumble_env
                    * self.break_amp
                    * 0.5;
                let crash = self
                    .crash_filter
                    .process_sample(self.crash_noise.next_sample())
                    * crash_env
                    * self.break_amp;
                let wash = self
                    .wash_filter
                    .process_sample(self.wash_noise.next_sample())
                    * wash_env
                    * self.wash_amp;
                *sample = rumble + crash + wash;
            }
            #[cfg(not(feature = "naad-backend"))]
            {
                let rumble = (self.rng.next_f32()
                    + self.rng.next_f32()
                    + self.rng.next_f32()
                    + self.rng.next_f32())
                    * 0.25
                    * rumble_env
                    * self.break_amp
                    * 0.5;
                let crash = self.rng.next_f32() * crash_env * self.break_amp;
                let wash =
                    (self.rng.next_f32() + self.rng.next_f32()) * 0.5 * wash_env * self.wash_amp;
                *sample = rumble + crash + wash;
            }

            *sample *= self.volume;
            *sample = self.dc_blocker.process(*sample);
        }
        self.sample_position += output.len();
    }
}
