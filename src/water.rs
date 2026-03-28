//! Water sound synthesis: streams, drips, splashes, waves.

use alloc::vec::Vec;
use serde::{Deserialize, Serialize};

use crate::error::Result;
use crate::rng::Rng;

/// Type of water sound.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[non_exhaustive]
pub enum WaterType {
    /// Gentle flowing stream.
    Stream,
    /// Single water drip.
    Drip,
    /// Water splash (object entering water).
    Splash,
    /// Ocean surf / waves.
    Waves,
}

/// Water sound synthesizer.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Water {
    /// Type of water sound.
    water_type: WaterType,
    /// Intensity (0.0-1.0).
    intensity: f32,
    /// PRNG.
    rng: Rng,
}

impl Water {
    /// Creates a new water synthesizer.
    #[must_use]
    pub fn new(water_type: WaterType, intensity: f32) -> Self {
        Self {
            water_type,
            intensity: intensity.clamp(0.0, 1.0),
            rng: Rng::new(2749),
        }
    }

    /// Synthesizes water audio.
    #[inline]
    pub fn synthesize(&mut self, sample_rate: f32, duration: f32) -> Result<Vec<f32>> {
        let num_samples = (sample_rate * duration) as usize;
        match self.water_type {
            WaterType::Stream => self.synthesize_stream(sample_rate, num_samples),
            WaterType::Drip => self.synthesize_drip(sample_rate, num_samples),
            WaterType::Splash => self.synthesize_splash(sample_rate, num_samples),
            WaterType::Waves => self.synthesize_waves(sample_rate, num_samples),
        }
    }

    #[inline]
    fn synthesize_stream(&mut self, sample_rate: f32, num_samples: usize) -> Result<Vec<f32>> {
        let mut output = Vec::with_capacity(num_samples);
        let amp = self.intensity * 0.3;

        for i in 0..num_samples {
            let t = i as f32 / sample_rate;
            // Bandpass-filtered noise with slow modulation
            let mod_freq = 0.5 + self.intensity * 2.0;
            let modulator =
                0.7 + 0.3 * crate::math::f32::sin(core::f32::consts::TAU * mod_freq * t);
            let noise = (self.rng.next_f32() + self.rng.next_f32()) * 0.5;
            output.push(noise * amp * modulator);
        }
        Ok(output)
    }

    #[inline]
    fn synthesize_drip(&mut self, sample_rate: f32, num_samples: usize) -> Result<Vec<f32>> {
        let mut output = alloc::vec![0.0f32; num_samples];
        // Single drip: resonant tone at ~1-2kHz with fast decay
        let freq = 1200.0 + self.rng.next_f32_range(-200.0, 200.0);
        let omega = core::f32::consts::TAU * freq / sample_rate;
        let drip_len = (sample_rate * 0.05) as usize;

        for (i, sample) in output
            .iter_mut()
            .enumerate()
            .take(drip_len.min(num_samples))
        {
            let t = i as f32 / sample_rate;
            let env = crate::math::f32::exp(-30.0 * t);
            *sample = crate::math::f32::sin(omega * i as f32) * env * self.intensity;
        }
        Ok(output)
    }

    #[inline]
    fn synthesize_splash(&mut self, sample_rate: f32, num_samples: usize) -> Result<Vec<f32>> {
        let mut output = Vec::with_capacity(num_samples);
        let amp = self.intensity * 0.6;

        for i in 0..num_samples {
            let t = i as f32 / sample_rate;
            let env = crate::math::f32::exp(-8.0 * t);
            let noise = self.rng.next_f32();
            output.push(noise * amp * env);
        }
        Ok(output)
    }

    #[inline]
    fn synthesize_waves(&mut self, sample_rate: f32, num_samples: usize) -> Result<Vec<f32>> {
        let mut output = Vec::with_capacity(num_samples);
        let amp = self.intensity * 0.4;

        for i in 0..num_samples {
            let t = i as f32 / sample_rate;
            // Slow wave rhythm (~0.1 Hz) with noise fill
            let wave_env = 0.5 + 0.5 * crate::math::f32::sin(core::f32::consts::TAU * 0.1 * t);
            let noise = (self.rng.next_f32() + self.rng.next_f32() + self.rng.next_f32()) / 3.0;
            output.push(noise * amp * wave_env);
        }
        Ok(output)
    }
}
