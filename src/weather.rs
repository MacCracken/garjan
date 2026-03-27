//! Weather sound synthesis: rain, thunder, wind.
//!
//! Models atmospheric phenomena as physical processes:
//! - Rain: stochastic particle impacts at Poisson-distributed intervals
//! - Thunder: low-frequency impulse with exponential decay and rumble
//! - Wind: filtered noise with amplitude and spectral modulation

use alloc::vec::Vec;
use serde::{Deserialize, Serialize};

use crate::error::Result;
use crate::rng::Rng;

/// Rain intensity levels.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[non_exhaustive]
pub enum RainIntensity {
    /// Light drizzle — sparse, quiet drops.
    Light,
    /// Moderate rainfall — steady, even.
    Moderate,
    /// Heavy rain — dense, loud.
    Heavy,
    /// Downpour — extremely dense, roaring.
    Torrential,
}

impl RainIntensity {
    /// Returns drops per second for this intensity.
    #[must_use]
    fn drops_per_second(self) -> f32 {
        match self {
            Self::Light => 20.0,
            Self::Moderate => 80.0,
            Self::Heavy => 250.0,
            Self::Torrential => 600.0,
        }
    }

    /// Returns amplitude scaling for this intensity.
    #[must_use]
    fn amplitude(self) -> f32 {
        match self {
            Self::Light => 0.15,
            Self::Moderate => 0.3,
            Self::Heavy => 0.5,
            Self::Torrential => 0.8,
        }
    }
}

/// Rain synthesizer — stochastic raindrop impacts.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Rain {
    /// Rain intensity.
    intensity: RainIntensity,
    /// PRNG for stochastic drop timing and character.
    rng: Rng,
}

impl Rain {
    /// Creates a new rain synthesizer.
    #[must_use]
    pub fn new(intensity: RainIntensity) -> Self {
        Self {
            intensity,
            rng: Rng::new(7919),
        }
    }

    /// Synthesizes rain audio.
    pub fn synthesize(&mut self, sample_rate: f32, duration: f32) -> Result<Vec<f32>> {
        let num_samples = (sample_rate * duration) as usize;
        let mut output = alloc::vec![0.0f32; num_samples];
        let amp = self.intensity.amplitude();
        let block_size = (sample_rate * 0.01) as usize; // 10ms blocks
        let drops_per_block = self.intensity.drops_per_second() * 0.01;

        for block_start in (0..num_samples).step_by(block_size.max(1)) {
            let block_end = (block_start + block_size).min(num_samples);
            let n_drops = self.rng.poisson(drops_per_block);

            for _ in 0..n_drops {
                let offset = self
                    .rng
                    .next_f32_range(0.0, (block_end - block_start) as f32)
                    as usize;
                let idx = block_start + offset;
                if idx < num_samples {
                    // Raindrop: sharp impulse with fast exponential decay
                    let drop_amp = amp * self.rng.next_f32_range(0.3, 1.0);
                    let decay_samples = self.rng.next_f32_range(20.0, 80.0) as usize;
                    for j in 0..decay_samples.min(num_samples - idx) {
                        let t = j as f32 / decay_samples as f32;
                        let env = crate::math::f32::exp(-5.0 * t);
                        output[idx + j] += drop_amp * env * self.rng.next_f32();
                    }
                }
            }
        }

        Ok(output)
    }
}

/// Thunder synthesizer — low-frequency impulse with rumble.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Thunder {
    /// Distance to lightning strike in meters.
    distance_m: f32,
    /// PRNG.
    rng: Rng,
}

impl Thunder {
    /// Creates a new thunder synthesizer.
    ///
    /// `distance_m` controls the delay and spectral character:
    /// - Close (< 500m): sharp crack + rumble
    /// - Medium (500-3000m): rolling rumble
    /// - Far (> 3000m): distant low rumble
    #[must_use]
    pub fn new(distance_m: f32) -> Self {
        Self {
            distance_m: distance_m.max(10.0),
            rng: Rng::new(1337),
        }
    }

    /// Synthesizes a thunderclap.
    pub fn synthesize(&mut self, sample_rate: f32, duration: f32) -> Result<Vec<f32>> {
        let num_samples = (sample_rate * duration) as usize;
        let mut output = alloc::vec![0.0f32; num_samples];

        // Delay based on distance (speed of sound ~343 m/s)
        let delay_s = self.distance_m / 343.0;
        let delay_samples = (delay_s * sample_rate) as usize;

        if delay_samples >= num_samples {
            return Ok(output); // Thunder hasn't arrived yet
        }

        // Amplitude decreases with distance (inverse distance law)
        let amp = (100.0 / self.distance_m).min(1.0);

        // Close thunder: sharp crack at the start
        let crack_len = if self.distance_m < 500.0 {
            (sample_rate * 0.05) as usize
        } else {
            0
        };

        for i in 0..crack_len.min(num_samples - delay_samples) {
            let t = i as f32 / crack_len.max(1) as f32;
            let env = crate::math::f32::exp(-10.0 * t);
            output[delay_samples + i] += amp * env * self.rng.next_f32() * 0.8;
        }

        // Rumble: low-frequency noise with slow decay
        let rumble_start = delay_samples + crack_len;
        let rumble_len = num_samples.saturating_sub(rumble_start);
        let decay_time = 0.5 + self.distance_m * 0.001; // Farther = longer rumble

        for i in 0..rumble_len {
            let t = i as f32 / (sample_rate * decay_time);
            let env = crate::math::f32::exp(-2.0 * t);
            // Low-pass effect: average multiple noise samples for low-frequency rumble
            let noise = (self.rng.next_f32() + self.rng.next_f32() + self.rng.next_f32()) / 3.0;
            output[rumble_start + i] += amp * env * noise * 0.6;
        }

        Ok(output)
    }
}

/// Wind synthesizer — filtered noise with modulation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Wind {
    /// Wind speed in m/s (0 = calm, 30+ = storm).
    speed: f32,
    /// Gustiness (0.0 = steady, 1.0 = very gusty).
    gustiness: f32,
    /// PRNG.
    rng: Rng,
}

impl Wind {
    /// Creates a new wind synthesizer.
    #[must_use]
    pub fn new(speed: f32, gustiness: f32) -> Self {
        Self {
            speed: speed.max(0.0),
            gustiness: gustiness.clamp(0.0, 1.0),
            rng: Rng::new(4201),
        }
    }

    /// Synthesizes wind audio.
    pub fn synthesize(&mut self, sample_rate: f32, duration: f32) -> Result<Vec<f32>> {
        let num_samples = (sample_rate * duration) as usize;
        let mut output = Vec::with_capacity(num_samples);

        // Base amplitude from wind speed (logarithmic)
        let base_amp = (self.speed / 30.0).min(1.0) * 0.5;
        // Gust modulation rate
        let gust_rate = 0.3 + self.gustiness * 2.0;

        for i in 0..num_samples {
            let t = i as f32 / sample_rate;

            // Slow amplitude modulation for gusts
            let gust = 1.0
                + self.gustiness
                    * 0.5
                    * crate::math::f32::sin(core::f32::consts::TAU * gust_rate * t);

            // Broadband noise shaped by wind speed
            // Higher speed = more high-frequency content
            let noise = self.rng.next_f32();

            output.push(noise * base_amp * gust);
        }

        Ok(output)
    }
}
