//! Underwater ambience synthesis.
//!
//! Generates the characteristic muffled, resonant sound of being submerged.
//! Combines low-frequency rumble, filtered noise, and occasional bubble events.
//! Note: underwater *propagation* (Mackenzie speed, absorption) belongs in goonj.

use alloc::vec::Vec;
use serde::{Deserialize, Serialize};

use crate::dsp::{DcBlocker, validate_sample_rate};
use crate::error::Result;
use crate::rng::Rng;

/// Depth affecting underwater character.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[non_exhaustive]
pub enum UnderwaterDepth {
    /// Shallow (just submerged) — more surface noise, brighter.
    Shallow,
    /// Medium depth — balanced, some surface rumble.
    Medium,
    /// Deep — very muffled, pressure rumble, sparse bubbles.
    Deep,
}

impl UnderwaterDepth {
    #[inline]
    #[must_use]
    fn config(self) -> (f32, f32, f32, f32) {
        // (lp_cutoff, rumble_amp, bubble_rate, surface_noise_amp)
        match self {
            Self::Shallow => (3000.0, 0.15, 5.0, 0.2),
            Self::Medium => (1200.0, 0.25, 2.0, 0.08),
            Self::Deep => (400.0, 0.35, 0.5, 0.02),
        }
    }
}

/// Underwater ambience synthesizer.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Underwater {
    depth: UnderwaterDepth,
    sample_rate: f32,
    rng: Rng,
    dc_blocker: DcBlocker,
    sample_position: usize,
    // Params
    rumble_amp: f32,
    bubble_rate: f32,
    surface_noise_amp: f32,
    // Real-time
    intensity: f32,
    #[cfg(feature = "naad-backend")]
    rumble_noise: naad::noise::NoiseGenerator,
    #[cfg(feature = "naad-backend")]
    rumble_filter: naad::filter::BiquadFilter,
    #[cfg(feature = "naad-backend")]
    surface_noise: naad::noise::NoiseGenerator,
    #[cfg(feature = "naad-backend")]
    surface_filter: naad::filter::BiquadFilter,
    #[cfg(feature = "naad-backend")]
    mod_lfo: naad::modulation::Lfo,
}

impl Underwater {
    /// Creates a new underwater ambience synthesizer.
    pub fn new(depth: UnderwaterDepth, sample_rate: f32) -> Result<Self> {
        validate_sample_rate(sample_rate)?;
        let config = depth.config();
        let rumble_amp = config.1;
        let bubble_rate = config.2;
        let surface_noise_amp = config.3;

        #[cfg(feature = "naad-backend")]
        let rumble_noise = naad::noise::NoiseGenerator::new(naad::noise::NoiseType::Brown, 6060);
        #[cfg(feature = "naad-backend")]
        let rumble_filter = naad::filter::BiquadFilter::new(
            naad::filter::FilterType::LowPass,
            sample_rate,
            200.0,
            0.5,
        )
        .map_err(|e| crate::error::GarjanError::SynthesisFailed(alloc::format!("{e}")))?;
        #[cfg(feature = "naad-backend")]
        let surface_noise = naad::noise::NoiseGenerator::new(naad::noise::NoiseType::Pink, 6061);
        #[cfg(feature = "naad-backend")]
        let surface_filter = naad::filter::BiquadFilter::new(
            naad::filter::FilterType::LowPass,
            sample_rate,
            config.0,
            0.7,
        )
        .map_err(|e| crate::error::GarjanError::SynthesisFailed(alloc::format!("{e}")))?;
        #[cfg(feature = "naad-backend")]
        let mod_lfo =
            naad::modulation::Lfo::new(naad::modulation::LfoShape::Sine, 0.08, sample_rate)
                .map_err(|e| crate::error::GarjanError::SynthesisFailed(alloc::format!("{e}")))?;

        Ok(Self {
            depth,
            sample_rate,
            rng: Rng::new(6060),
            dc_blocker: DcBlocker::new(sample_rate),
            sample_position: 0,
            rumble_amp,
            bubble_rate,
            surface_noise_amp,
            intensity: 1.0,
            #[cfg(feature = "naad-backend")]
            rumble_noise,
            #[cfg(feature = "naad-backend")]
            rumble_filter,
            #[cfg(feature = "naad-backend")]
            surface_noise,
            #[cfg(feature = "naad-backend")]
            surface_filter,
            #[cfg(feature = "naad-backend")]
            mod_lfo,
        })
    }

    /// Sets intensity (0.0–1.0).
    pub fn set_intensity(&mut self, intensity: f32) {
        self.intensity = intensity.clamp(0.0, 1.0);
    }

    /// Synthesizes underwater ambience.
    #[inline]
    pub fn synthesize(&mut self, duration: f32) -> Result<Vec<f32>> {
        let num_samples = (self.sample_rate * duration) as usize;
        let mut output = alloc::vec![0.0f32; num_samples];
        self.process_block(&mut output);
        Ok(output)
    }

    /// Fills output buffer with underwater audio (streaming).
    #[inline]
    pub fn process_block(&mut self, output: &mut [f32]) {
        if self.intensity < 0.001 {
            for s in output.iter_mut() {
                *s = 0.0;
                self.dc_blocker.process(0.0);
            }
            self.sample_position += output.len();
            return;
        }

        let num_samples = output.len();

        // Continuous layers
        for sample in output.iter_mut() {
            #[cfg(feature = "naad-backend")]
            {
                let rumble = self
                    .rumble_filter
                    .process_sample(self.rumble_noise.next_sample())
                    * self.rumble_amp;
                let surface = self
                    .surface_filter
                    .process_sample(self.surface_noise.next_sample())
                    * self.surface_noise_amp;
                let modulator = 0.8 + 0.2 * self.mod_lfo.next_value();
                *sample = (rumble + surface) * self.intensity * modulator;
            }
            #[cfg(not(feature = "naad-backend"))]
            {
                let rumble = (self.rng.next_f32()
                    + self.rng.next_f32()
                    + self.rng.next_f32()
                    + self.rng.next_f32())
                    * 0.25
                    * self.rumble_amp;
                let surface = self.rng.next_f32() * self.surface_noise_amp;
                *sample = (rumble + surface) * self.intensity;
            }
        }

        // Stochastic bubble events
        let block_size = (self.sample_rate * 0.01) as usize;
        let bubbles_per_block = self.bubble_rate * self.intensity * 0.01;

        for block_start in (0..num_samples).step_by(block_size.max(1)) {
            let block_end = (block_start + block_size).min(num_samples);
            let n_bubbles = self.rng.poisson(bubbles_per_block);

            for _ in 0..n_bubbles {
                let offset = self
                    .rng
                    .next_f32_range(0.0, (block_end - block_start) as f32)
                    as usize;
                let idx = block_start + offset;
                if idx >= num_samples {
                    continue;
                }

                // Bubble: decaying sinusoid (same model as bubble.rs)
                let freq = self.rng.next_f32_range(200.0, 800.0);
                let omega = core::f32::consts::TAU * freq / self.sample_rate;
                let amp = self.intensity * self.rng.next_f32_range(0.05, 0.15);
                let decay = (self.sample_rate * 0.02) as usize;

                for j in 0..decay.min(num_samples - idx) {
                    let t = j as f32 / self.sample_rate;
                    let env = crate::math::f32::exp(-t / 0.02);
                    output[idx + j] += crate::math::f32::sin(omega * j as f32) * env * amp;
                }
            }
        }

        for sample in output.iter_mut() {
            *sample = self.dc_blocker.process(*sample);
        }
        self.sample_position += num_samples;
    }
}
