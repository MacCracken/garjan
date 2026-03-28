//! Bubble synthesis: underwater, boiling, viscous, pouring.
//!
//! Models bubbles as stochastic resonant events. Each bubble is a
//! short decaying sinusoid at a random frequency near the base,
//! simulating the acoustic resonance of an oscillating gas cavity.
//! Frequency is inversely proportional to bubble radius (Minnaert resonance).

use alloc::vec::Vec;
use serde::{Deserialize, Serialize};

use crate::creature::BubbleType;
use crate::dsp::{DcBlocker, validate_sample_rate};
use crate::error::Result;
use crate::rng::Rng;

/// Bubble sound synthesizer — stochastic resonant events.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Bubble {
    bubble_type: BubbleType,
    sample_rate: f32,
    rng: Rng,
    dc_blocker: DcBlocker,
    sample_position: usize,
    // Type params
    base_freq: f32,
    event_rate: f32,
    freq_spread: f32,
    decay_time: f32,
    // Real-time
    intensity: f32,
    #[cfg(feature = "naad-backend")]
    noise_gen: naad::noise::NoiseGenerator,
}

impl Bubble {
    /// Creates a new bubble synthesizer.
    pub fn new(bubble_type: BubbleType, sample_rate: f32) -> Result<Self> {
        validate_sample_rate(sample_rate)?;
        let (base_freq, event_rate, freq_spread, decay_time) = bubble_type.config();

        #[cfg(feature = "naad-backend")]
        let noise_gen = naad::noise::NoiseGenerator::new(naad::noise::NoiseType::White, 4040);

        Ok(Self {
            bubble_type,
            sample_rate,
            rng: Rng::new(4040),
            dc_blocker: DcBlocker::new(sample_rate),
            sample_position: 0,
            base_freq,
            event_rate,
            freq_spread,
            decay_time,
            intensity: 1.0,
            #[cfg(feature = "naad-backend")]
            noise_gen,
        })
    }

    /// Sets the bubble intensity (0.0 = none, 1.0 = vigorous).
    pub fn set_intensity(&mut self, intensity: f32) {
        self.intensity = intensity.clamp(0.0, 1.0);
    }

    /// Synthesizes bubble audio.
    #[inline]
    pub fn synthesize(&mut self, duration: f32) -> Result<Vec<f32>> {
        let num_samples = (self.sample_rate * duration) as usize;
        let mut output = alloc::vec![0.0f32; num_samples];
        self.process_block(&mut output);
        Ok(output)
    }

    /// Fills output buffer with bubble audio (streaming).
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

        let num_samples = output.len();

        // Zero buffer
        for s in output.iter_mut() {
            *s = 0.0;
        }

        // Schedule bubble events via Poisson process
        let rate = self.event_rate * self.intensity;
        let block_size = (self.sample_rate * 0.01) as usize;
        let events_per_block = rate * 0.01;

        for block_start in (0..num_samples).step_by(block_size.max(1)) {
            let block_end = (block_start + block_size).min(num_samples);
            let n_events = self.rng.poisson(events_per_block);

            for _ in 0..n_events {
                let offset = self
                    .rng
                    .next_f32_range(0.0, (block_end - block_start) as f32)
                    as usize;
                let idx = block_start + offset;
                if idx >= num_samples {
                    continue;
                }

                // Each bubble: decaying sinusoid at randomized frequency
                let freq =
                    self.base_freq + self.rng.next_f32_range(-self.freq_spread, self.freq_spread);
                let freq = freq.max(20.0);
                let omega = core::f32::consts::TAU * freq / self.sample_rate;
                let amp = self.intensity * self.rng.next_f32_range(0.1, 0.4);
                let decay_samples = (self.decay_time * self.sample_rate) as usize;

                // Add a small noise burst at the pop onset
                let pop_len = (self.sample_rate * 0.001) as usize;

                for j in 0..decay_samples.min(num_samples - idx) {
                    let t = j as f32 / self.sample_rate;
                    let env = crate::math::f32::exp(-t / self.decay_time.max(0.001));

                    // Resonant tone
                    let tone = crate::math::f32::sin(omega * j as f32) * env * amp;

                    // Pop noise at onset
                    let pop = if j < pop_len {
                        let pop_env = 1.0 - (j as f32 / pop_len as f32);
                        self.generate_noise() * pop_env * amp * 0.3
                    } else {
                        0.0
                    };

                    output[idx + j] += tone + pop;
                }
            }
        }

        // DC blocking
        for sample in output.iter_mut() {
            *sample = self.dc_blocker.process(*sample);
        }
        self.sample_position += num_samples;
    }

    #[inline]
    fn generate_noise(&mut self) -> f32 {
        #[cfg(feature = "naad-backend")]
        {
            self.noise_gen.next_sample()
        }
        #[cfg(not(feature = "naad-backend"))]
        {
            self.rng.next_f32()
        }
    }
}
