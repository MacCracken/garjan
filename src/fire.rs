//! Fire sound synthesis: crackle, roar, hiss.

use alloc::vec::Vec;
use serde::{Deserialize, Serialize};

use crate::dsp::{DcBlocker, validate_sample_rate};
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
    sample_rate: f32,
    rng: Rng,
    dc_blocker: DcBlocker,
    sample_position: usize,
    #[cfg(feature = "naad-backend")]
    roar_noise: naad::noise::NoiseGenerator,
    #[cfg(feature = "naad-backend")]
    roar_filter: naad::filter::BiquadFilter,
    #[cfg(feature = "naad-backend")]
    crackle_noise: naad::noise::NoiseGenerator,
    #[cfg(feature = "naad-backend")]
    crackle_filter: naad::filter::BiquadFilter,
}

impl Fire {
    /// Creates a new fire synthesizer.
    pub fn new(intensity: f32, sample_rate: f32) -> Result<Self> {
        validate_sample_rate(sample_rate)?;
        let intensity = intensity.clamp(0.0, 1.0);
        #[cfg(feature = "naad-backend")]
        let roar_filter = naad::filter::BiquadFilter::new(
            naad::filter::FilterType::LowPass,
            sample_rate,
            600.0,
            0.7,
        )
        .map_err(|e| crate::error::GarjanError::SynthesisFailed(alloc::format!("{e}")))?;
        #[cfg(feature = "naad-backend")]
        let crackle_filter = naad::filter::BiquadFilter::new(
            naad::filter::FilterType::HighPass,
            sample_rate,
            2000.0,
            1.0,
        )
        .map_err(|e| crate::error::GarjanError::SynthesisFailed(alloc::format!("{e}")))?;
        Ok(Self {
            intensity,
            crackle_rate: 5.0 + intensity * 30.0,
            sample_rate,
            rng: Rng::new(6661),
            dc_blocker: DcBlocker::new(sample_rate),
            sample_position: 0,
            #[cfg(feature = "naad-backend")]
            roar_noise: naad::noise::NoiseGenerator::new(naad::noise::NoiseType::Brown, 6661),
            #[cfg(feature = "naad-backend")]
            roar_filter,
            #[cfg(feature = "naad-backend")]
            crackle_noise: naad::noise::NoiseGenerator::new(naad::noise::NoiseType::White, 6662),
            #[cfg(feature = "naad-backend")]
            crackle_filter,
        })
    }

    /// Synthesizes fire audio.
    #[inline]
    pub fn synthesize(&mut self, duration: f32) -> Result<Vec<f32>> {
        crate::dsp::validate_duration(duration)?;
        let num_samples = (self.sample_rate * duration) as usize;
        let mut output = alloc::vec![0.0f32; num_samples];
        self.process_block(&mut output);
        Ok(output)
    }

    /// Fills output buffer with fire audio (streaming).
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

    #[cfg(feature = "naad-backend")]
    fn process_block_naad(&mut self, output: &mut [f32]) {
        let num_samples = output.len();
        let roar_amp = self.intensity * 0.3;

        // Roar: brown noise through lowpass
        for sample in output.iter_mut() {
            let noise = self.roar_noise.next_sample();
            *sample = self.roar_filter.process_sample(noise) * roar_amp;
        }

        // Crackle: stochastic impulses through highpass
        let block_size = (self.sample_rate * 0.01) as usize;
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
                        let noise = self.crackle_noise.next_sample();
                        output[idx + j] +=
                            crackle_amp * env * self.crackle_filter.process_sample(noise);
                    }
                }
            }
        }
    }

    #[cfg(not(feature = "naad-backend"))]
    fn process_block_fallback(&mut self, output: &mut [f32]) {
        let num_samples = output.len();
        let roar_amp = self.intensity * 0.3;

        // Roar: averaged noise for low-frequency character
        for sample in output.iter_mut() {
            let noise = (self.rng.next_f32() + self.rng.next_f32()) * 0.5;
            *sample = noise * roar_amp;
        }

        // Crackle: stochastic impulses
        let block_size = (self.sample_rate * 0.01) as usize;
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
    }
}
