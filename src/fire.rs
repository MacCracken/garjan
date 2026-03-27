//! Fire sound synthesis: crackle, roar, hiss.

use alloc::vec::Vec;
use serde::{Deserialize, Serialize};

use crate::error::Result;
use crate::rng::Rng;

/// Fire sound synthesizer.
///
/// Models fire as a combination of:
/// - Crackle: stochastic high-frequency impulses (gas pockets bursting)
/// - Roar: low-frequency broadband noise (turbulent combustion)
/// - Hiss: high-frequency steady noise (gas flow)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Fire {
    /// Fire intensity (0.0 = embers, 1.0 = inferno).
    intensity: f32,
    /// Crackle rate (events per second).
    crackle_rate: f32,
    /// PRNG.
    rng: Rng,
}

impl Fire {
    /// Creates a new fire synthesizer.
    #[must_use]
    pub fn new(intensity: f32) -> Self {
        let intensity = intensity.clamp(0.0, 1.0);
        Self {
            intensity,
            crackle_rate: 5.0 + intensity * 30.0,
            rng: Rng::new(6661),
        }
    }

    /// Synthesizes fire audio.
    pub fn synthesize(&mut self, sample_rate: f32, duration: f32) -> Result<Vec<f32>> {
        let num_samples = (sample_rate * duration) as usize;
        let mut output = alloc::vec![0.0f32; num_samples];

        // Roar: low-frequency noise, amplitude scales with intensity
        let roar_amp = self.intensity * 0.3;
        for sample in output.iter_mut() {
            let noise = (self.rng.next_f32() + self.rng.next_f32()) * 0.5;
            *sample += noise * roar_amp;
        }

        // Crackle: stochastic impulses
        let block_size = (sample_rate * 0.01) as usize;
        let crackles_per_block = self.crackle_rate * 0.01;

        for block_start in (0..num_samples).step_by(block_size.max(1)) {
            let block_end = (block_start + block_size).min(num_samples);
            let n_crackles = self.rng.poisson(crackles_per_block);

            for _ in 0..n_crackles {
                let offset = self
                    .rng
                    .next_f32_range(0.0, (block_end - block_start) as f32)
                    as usize;
                let idx = block_start + offset;
                if idx < num_samples {
                    let crackle_amp = self.intensity * self.rng.next_f32_range(0.1, 0.6);
                    let decay_len = self.rng.next_f32_range(5.0, 30.0) as usize;
                    for j in 0..decay_len.min(num_samples - idx) {
                        let env = 1.0 - (j as f32 / decay_len as f32);
                        output[idx + j] += crackle_amp * env * self.rng.next_f32();
                    }
                }
            }
        }

        Ok(output)
    }
}
