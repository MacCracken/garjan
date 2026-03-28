//! Ambient texture synthesis: continuous environmental backgrounds.
//!
//! Textures are layered noise with characteristic spectral shapes:
//! forest ambience, city hum, ocean surf, cave drip.

use alloc::vec::Vec;
use serde::{Deserialize, Serialize};

use crate::dsp::{DcBlocker, validate_sample_rate};
use crate::error::Result;
use crate::rng::Rng;

/// An ambient texture — continuous background sound layer.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AmbientTexture {
    texture_type: TextureType,
    level: f32,
    sample_rate: f32,
    rng: Rng,
    dc_blocker: DcBlocker,
    sample_position: usize,
    #[cfg(feature = "naad-backend")]
    low_noise: naad::noise::NoiseGenerator,
    #[cfg(feature = "naad-backend")]
    mid_noise: naad::noise::NoiseGenerator,
    #[cfg(feature = "naad-backend")]
    high_noise: naad::noise::NoiseGenerator,
    #[cfg(feature = "naad-backend")]
    low_filter: naad::filter::BiquadFilter,
    #[cfg(feature = "naad-backend")]
    mid_filter: naad::filter::BiquadFilter,
    #[cfg(feature = "naad-backend")]
    high_filter: naad::filter::BiquadFilter,
    #[cfg(feature = "naad-backend")]
    mod_lfo: naad::modulation::Lfo,
}

/// Type of ambient texture.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[non_exhaustive]
pub enum TextureType {
    /// Forest: birds, insects, rustling leaves, distant wind.
    Forest,
    /// City: traffic hum, distant voices, machinery.
    City,
    /// Ocean: surf, wind, seabirds.
    Ocean,
    /// Cave: drips, low resonance, silence.
    Cave,
    /// Desert: wind, sand, sparse insects.
    Desert,
    /// Night: crickets, owls, quiet wind.
    Night,
}

impl AmbientTexture {
    /// Creates a new ambient texture.
    pub fn new(texture_type: TextureType, level: f32, sample_rate: f32) -> Result<Self> {
        validate_sample_rate(sample_rate)?;
        #[cfg(feature = "naad-backend")]
        let low_filter = naad::filter::BiquadFilter::new(
            naad::filter::FilterType::LowPass,
            sample_rate,
            300.0,
            0.7,
        )
        .map_err(|e| crate::error::GarjanError::SynthesisFailed(alloc::format!("{e}")))?;
        #[cfg(feature = "naad-backend")]
        let mid_filter = naad::filter::BiquadFilter::new(
            naad::filter::FilterType::BandPass,
            sample_rate,
            1500.0,
            1.0,
        )
        .map_err(|e| crate::error::GarjanError::SynthesisFailed(alloc::format!("{e}")))?;
        #[cfg(feature = "naad-backend")]
        let high_filter = naad::filter::BiquadFilter::new(
            naad::filter::FilterType::HighPass,
            sample_rate,
            4000.0,
            0.7,
        )
        .map_err(|e| crate::error::GarjanError::SynthesisFailed(alloc::format!("{e}")))?;
        #[cfg(feature = "naad-backend")]
        let mod_lfo = {
            let mod_rate = match texture_type {
                TextureType::Forest => 0.2,
                TextureType::City => 0.1,
                TextureType::Ocean => 0.08,
                TextureType::Cave => 0.02,
                TextureType::Desert => 0.15,
                TextureType::Night => 0.05,
            };
            naad::modulation::Lfo::new(naad::modulation::LfoShape::Sine, mod_rate, sample_rate)
                .map_err(|e| crate::error::GarjanError::SynthesisFailed(alloc::format!("{e}")))?
        };
        Ok(Self {
            texture_type,
            level: level.clamp(0.0, 1.0),
            sample_rate,
            rng: Rng::new(9001),
            dc_blocker: DcBlocker::new(sample_rate),
            sample_position: 0,
            #[cfg(feature = "naad-backend")]
            low_noise: naad::noise::NoiseGenerator::new(naad::noise::NoiseType::Brown, 9001),
            #[cfg(feature = "naad-backend")]
            mid_noise: naad::noise::NoiseGenerator::new(naad::noise::NoiseType::Pink, 9002),
            #[cfg(feature = "naad-backend")]
            high_noise: naad::noise::NoiseGenerator::new(naad::noise::NoiseType::White, 9003),
            #[cfg(feature = "naad-backend")]
            low_filter,
            #[cfg(feature = "naad-backend")]
            mid_filter,
            #[cfg(feature = "naad-backend")]
            high_filter,
            #[cfg(feature = "naad-backend")]
            mod_lfo,
        })
    }

    /// Synthesizes ambient texture audio.
    #[inline]
    pub fn synthesize(&mut self, duration: f32) -> Result<Vec<f32>> {
        let num_samples = (self.sample_rate * duration) as usize;
        let mut output = alloc::vec![0.0f32; num_samples];
        self.process_block(&mut output);
        Ok(output)
    }

    /// Fills output buffer with ambient texture audio (streaming).
    #[inline]
    pub fn process_block(&mut self, output: &mut [f32]) {
        #[cfg(feature = "naad-backend")]
        self.process_block_naad(output);
        #[cfg(not(feature = "naad-backend"))]
        self.process_block_fallback(output);

        for sample in output.iter_mut() {
            *sample = self.dc_blocker.process(*sample);
        }
        self.sample_position += output.len();
    }

    /// Returns the band mix ratios `(low, mid, high)` for the texture type.
    #[must_use]
    fn band_mix(&self) -> (f32, f32, f32) {
        match self.texture_type {
            TextureType::Forest => (0.2, 0.5, 0.3),
            TextureType::City => (0.5, 0.3, 0.2),
            TextureType::Ocean => (0.6, 0.3, 0.1),
            TextureType::Cave => (0.3, 0.1, 0.05),
            TextureType::Desert => (0.1, 0.2, 0.1),
            TextureType::Night => (0.1, 0.2, 0.15),
        }
    }

    #[cfg(feature = "naad-backend")]
    fn process_block_naad(&mut self, output: &mut [f32]) {
        let (low_mix, mid_mix, high_mix) = self.band_mix();
        for sample in output.iter_mut() {
            let modulator = 0.7 + 0.3 * self.mod_lfo.next_value();

            let low = self.low_filter.process_sample(self.low_noise.next_sample());
            let mid = self.mid_filter.process_sample(self.mid_noise.next_sample());
            let high = self
                .high_filter
                .process_sample(self.high_noise.next_sample());

            *sample = (low * low_mix + mid * mid_mix + high * high_mix) * self.level * modulator;
        }
    }

    #[cfg(not(feature = "naad-backend"))]
    fn process_block_fallback(&mut self, output: &mut [f32]) {
        let (low_mix, mid_mix, high_mix) = self.band_mix();
        let mod_rate = match self.texture_type {
            TextureType::Forest => 0.2,
            TextureType::City => 0.1,
            TextureType::Ocean => 0.08,
            TextureType::Cave => 0.02,
            TextureType::Desert => 0.15,
            TextureType::Night => 0.05,
        };

        for (i, sample) in output.iter_mut().enumerate() {
            let t = (self.sample_position + i) as f32 / self.sample_rate;
            let modulator =
                0.7 + 0.3 * crate::math::f32::sin(core::f32::consts::TAU * mod_rate * t);

            // Multi-band noise (simple averaging for low-pass effect)
            let high = self.rng.next_f32();
            let mid = (self.rng.next_f32() + self.rng.next_f32()) * 0.5;
            let low = (self.rng.next_f32()
                + self.rng.next_f32()
                + self.rng.next_f32()
                + self.rng.next_f32())
                * 0.25;

            *sample = (low * low_mix + mid * mid_mix + high * high_mix) * self.level * modulator;
        }
    }
}
