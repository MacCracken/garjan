//! Ambient texture synthesis: continuous environmental backgrounds.
//!
//! Textures are layered noise with characteristic spectral shapes:
//! forest ambience, city hum, ocean surf, cave drip.

use alloc::vec::Vec;
use serde::{Deserialize, Serialize};

use crate::error::Result;
use crate::rng::Rng;

/// An ambient texture — continuous background sound layer.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AmbientTexture {
    /// Texture type.
    texture_type: TextureType,
    /// Overall level (0.0-1.0).
    level: f32,
    /// PRNG.
    rng: Rng,
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
    #[must_use]
    pub fn new(texture_type: TextureType, level: f32) -> Self {
        Self {
            texture_type,
            level: level.clamp(0.0, 1.0),
            rng: Rng::new(9001),
        }
    }

    /// Synthesizes ambient texture audio.
    pub fn synthesize(&mut self, sample_rate: f32, duration: f32) -> Result<Vec<f32>> {
        let num_samples = (sample_rate * duration) as usize;
        let mut output = Vec::with_capacity(num_samples);

        let (low_mix, mid_mix, high_mix, mod_rate) = match self.texture_type {
            TextureType::Forest => (0.2, 0.5, 0.3, 0.2),
            TextureType::City => (0.5, 0.3, 0.2, 0.1),
            TextureType::Ocean => (0.6, 0.3, 0.1, 0.08),
            TextureType::Cave => (0.3, 0.1, 0.05, 0.02),
            TextureType::Desert => (0.1, 0.2, 0.1, 0.15),
            TextureType::Night => (0.1, 0.2, 0.15, 0.05),
        };

        for i in 0..num_samples {
            let t = i as f32 / sample_rate;
            // Slow modulation for natural variation
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

            let sample = (low * low_mix + mid * mid_mix + high * high_mix) * self.level * modulator;
            output.push(sample);
        }

        Ok(output)
    }
}
