//! Water sound synthesis: streams, drips, splashes, waves.

use alloc::vec::Vec;
use serde::{Deserialize, Serialize};

use crate::dsp::{DcBlocker, validate_sample_rate};
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
    water_type: WaterType,
    intensity: f32,
    sample_rate: f32,
    rng: Rng,
    dc_blocker: DcBlocker,
    sample_position: usize,
    #[cfg(feature = "naad-backend")]
    noise_gen: naad::noise::NoiseGenerator,
    #[cfg(feature = "naad-backend")]
    shape_filter: naad::filter::BiquadFilter,
    #[cfg(feature = "naad-backend")]
    mod_lfo: naad::modulation::Lfo,
}

impl Water {
    /// Creates a new water synthesizer.
    pub fn new(water_type: WaterType, intensity: f32, sample_rate: f32) -> Result<Self> {
        validate_sample_rate(sample_rate)?;
        #[cfg(feature = "naad-backend")]
        let (noise_type, filter_type, filter_freq, filter_q, lfo_rate) = match water_type {
            WaterType::Stream => (
                naad::noise::NoiseType::Pink,
                naad::filter::FilterType::BandPass,
                800.0,
                1.5,
                0.5 + intensity * 2.0,
            ),
            WaterType::Drip => (
                naad::noise::NoiseType::White,
                naad::filter::FilterType::BandPass,
                1200.0,
                2.0,
                1.0,
            ),
            WaterType::Splash => (
                naad::noise::NoiseType::White,
                naad::filter::FilterType::LowPass,
                4000.0,
                0.7,
                1.0,
            ),
            WaterType::Waves => (
                naad::noise::NoiseType::Brown,
                naad::filter::FilterType::LowPass,
                500.0,
                0.5,
                0.1,
            ),
        };
        #[cfg(feature = "naad-backend")]
        let shape_filter =
            naad::filter::BiquadFilter::new(filter_type, sample_rate, filter_freq, filter_q)
                .map_err(|e| crate::error::GarjanError::SynthesisFailed(alloc::format!("{e}")))?;
        #[cfg(feature = "naad-backend")]
        let mod_lfo =
            naad::modulation::Lfo::new(naad::modulation::LfoShape::Sine, lfo_rate, sample_rate)
                .map_err(|e| crate::error::GarjanError::SynthesisFailed(alloc::format!("{e}")))?;
        Ok(Self {
            water_type,
            intensity: intensity.clamp(0.0, 1.0),
            sample_rate,
            rng: Rng::new(2749),
            dc_blocker: DcBlocker::new(sample_rate),
            sample_position: 0,
            #[cfg(feature = "naad-backend")]
            noise_gen: naad::noise::NoiseGenerator::new(noise_type, 2749),
            #[cfg(feature = "naad-backend")]
            shape_filter,
            #[cfg(feature = "naad-backend")]
            mod_lfo,
        })
    }

    /// Synthesizes water audio.
    #[inline]
    pub fn synthesize(&mut self, duration: f32) -> Result<Vec<f32>> {
        let num_samples = (self.sample_rate * duration) as usize;
        let mut output = alloc::vec![0.0f32; num_samples];
        self.process_block(&mut output);
        Ok(output)
    }

    /// Fills output buffer with water audio (streaming).
    #[inline]
    pub fn process_block(&mut self, output: &mut [f32]) {
        match self.water_type {
            WaterType::Stream => self.process_stream(output),
            WaterType::Drip => self.process_drip(output),
            WaterType::Splash => self.process_splash(output),
            WaterType::Waves => self.process_waves(output),
        }
        for sample in output.iter_mut() {
            *sample = self.dc_blocker.process(*sample);
        }
        self.sample_position += output.len();
    }

    #[inline]
    fn process_stream(&mut self, output: &mut [f32]) {
        let amp = self.intensity * 0.3;
        #[cfg(feature = "naad-backend")]
        for sample in output.iter_mut() {
            let noise = self.noise_gen.next_sample();
            let filtered = self.shape_filter.process_sample(noise);
            let modulator = 0.7 + 0.3 * self.mod_lfo.next_value();
            *sample = filtered * amp * modulator;
        }
        #[cfg(not(feature = "naad-backend"))]
        for (i, sample) in output.iter_mut().enumerate() {
            let t = (self.sample_position + i) as f32 / self.sample_rate;
            let mod_freq = 0.5 + self.intensity * 2.0;
            let modulator =
                0.7 + 0.3 * crate::math::f32::sin(core::f32::consts::TAU * mod_freq * t);
            let noise = (self.rng.next_f32() + self.rng.next_f32()) * 0.5;
            *sample = noise * amp * modulator;
        }
    }

    #[inline]
    fn process_drip(&mut self, output: &mut [f32]) {
        // Drip: resonant tone — no naad needed, keep manual sin + exp
        // Use a fixed-seed rng for consistent frequency across process_block calls
        let freq = 1200.0 + Rng::new(2749).next_f32_range(-200.0, 200.0);
        let omega = core::f32::consts::TAU * freq / self.sample_rate;
        let drip_len = (self.sample_rate * 0.05) as usize;

        for (i, sample) in output.iter_mut().enumerate() {
            let abs_pos = self.sample_position + i;
            if abs_pos < drip_len {
                let t = abs_pos as f32 / self.sample_rate;
                let env = crate::math::f32::exp(-30.0 * t);
                *sample = crate::math::f32::sin(omega * abs_pos as f32) * env * self.intensity;
            } else {
                *sample = 0.0;
            }
        }
    }

    #[inline]
    fn process_splash(&mut self, output: &mut [f32]) {
        let amp = self.intensity * 0.6;
        for (i, sample) in output.iter_mut().enumerate() {
            let abs_pos = self.sample_position + i;
            let t = abs_pos as f32 / self.sample_rate;
            let env = crate::math::f32::exp(-8.0 * t);
            #[cfg(feature = "naad-backend")]
            {
                let noise = self.noise_gen.next_sample();
                *sample = self.shape_filter.process_sample(noise) * amp * env;
            }
            #[cfg(not(feature = "naad-backend"))]
            {
                let noise = self.rng.next_f32();
                *sample = noise * amp * env;
            }
        }
    }

    #[inline]
    fn process_waves(&mut self, output: &mut [f32]) {
        let amp = self.intensity * 0.4;
        #[cfg(feature = "naad-backend")]
        for sample in output.iter_mut() {
            let noise = self.noise_gen.next_sample();
            let filtered = self.shape_filter.process_sample(noise);
            let wave_env = 0.5 + 0.5 * self.mod_lfo.next_value();
            *sample = filtered * amp * wave_env;
        }
        #[cfg(not(feature = "naad-backend"))]
        for (i, sample) in output.iter_mut().enumerate() {
            let t = (self.sample_position + i) as f32 / self.sample_rate;
            let wave_env = 0.5 + 0.5 * crate::math::f32::sin(core::f32::consts::TAU * 0.1 * t);
            let noise = (self.rng.next_f32() + self.rng.next_f32() + self.rng.next_f32()) / 3.0;
            *sample = noise * amp * wave_env;
        }
    }
}
