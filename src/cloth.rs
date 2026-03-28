//! Cloth flapping synthesis: flags, capes, sails, tarps.
//!
//! Models fabric flapping as a stochastic impulse train at a
//! wind-dependent rate. Each flap is a short noise burst shaped
//! by the cloth material's spectral character.

use alloc::vec::Vec;
use serde::{Deserialize, Serialize};

use crate::aero::ClothType;
use crate::dsp::{DcBlocker, validate_sample_rate};
use crate::error::Result;
use crate::material::Material;
use crate::modal::{ModalBank, generate_modes};
use crate::rng::Rng;

/// Cloth flapping synthesizer — stochastic flap events.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Cloth {
    cloth_type: ClothType,
    sample_rate: f32,
    rng: Rng,
    dc_blocker: DcBlocker,
    sample_position: usize,
    // Cloth params
    base_flap_rate: f32,
    amplitude: f32,
    // Real-time
    wind_speed: f32,
    // Modal resonance for heavy cloth (Sail)
    modal_bank: Option<ModalBank>,
    #[cfg(feature = "naad-backend")]
    noise_gen: naad::noise::NoiseGenerator,
    #[cfg(feature = "naad-backend")]
    shape_filter: naad::filter::BiquadFilter,
}

impl Cloth {
    /// Creates a new cloth flapping synthesizer.
    pub fn new(cloth_type: ClothType, sample_rate: f32) -> Result<Self> {
        validate_sample_rate(sample_rate)?;
        let config = cloth_type.config();
        let base_flap_rate = config.0;
        let amplitude = config.1;
        let use_modal = config.3;

        let modal_bank = if use_modal {
            let props = Material::Fabric.properties();
            let mode_cfg = Material::Fabric.mode_config();
            let specs = generate_modes(
                &props,
                mode_cfg.pattern,
                mode_cfg.mode_count,
                mode_cfg.damping_factor,
            );
            Some(ModalBank::new(&specs, sample_rate)?)
        } else {
            None
        };

        #[cfg(feature = "naad-backend")]
        let noise_gen = naad::noise::NoiseGenerator::new(naad::noise::NoiseType::White, 6666);
        #[cfg(feature = "naad-backend")]
        let shape_filter = naad::filter::BiquadFilter::new(
            naad::filter::FilterType::BandPass,
            sample_rate,
            config.2,
            1.5,
        )
        .map_err(|e| crate::error::GarjanError::SynthesisFailed(alloc::format!("{e}")))?;

        Ok(Self {
            cloth_type,
            sample_rate,
            rng: Rng::new(6666),
            dc_blocker: DcBlocker::new(sample_rate),
            sample_position: 0,
            base_flap_rate,
            amplitude,
            wind_speed: 0.0,
            modal_bank,
            #[cfg(feature = "naad-backend")]
            noise_gen,
            #[cfg(feature = "naad-backend")]
            shape_filter,
        })
    }

    /// Sets the wind speed (0.0 = still, 1.0 = strong wind).
    pub fn set_wind_speed(&mut self, speed: f32) {
        self.wind_speed = speed.clamp(0.0, 1.0);
    }

    /// Synthesizes cloth flapping audio.
    #[inline]
    pub fn synthesize(&mut self, duration: f32) -> Result<Vec<f32>> {
        let num_samples = (self.sample_rate * duration) as usize;
        let mut output = alloc::vec![0.0f32; num_samples];
        self.process_block(&mut output);
        Ok(output)
    }

    /// Fills output buffer with cloth flapping audio (streaming).
    #[inline]
    pub fn process_block(&mut self, output: &mut [f32]) {
        if self.wind_speed < 0.001 {
            for sample in output.iter_mut() {
                *sample = 0.0;
                self.dc_blocker.process(0.0);
            }
            self.sample_position += output.len();
            return;
        }

        let num_samples = output.len();
        let flap_rate = self.base_flap_rate * self.wind_speed;
        let block_size = (self.sample_rate * 0.01) as usize;
        let flaps_per_block = flap_rate * 0.01;

        // Initialize output to zero
        for s in output.iter_mut() {
            *s = 0.0;
        }

        // Schedule flap events via Poisson process
        for block_start in (0..num_samples).step_by(block_size.max(1)) {
            let block_end = (block_start + block_size).min(num_samples);
            let n_flaps = self.rng.poisson(flaps_per_block);

            for _ in 0..n_flaps {
                let offset = self
                    .rng
                    .next_f32_range(0.0, (block_end - block_start) as f32)
                    as usize;
                let idx = block_start + offset;
                if idx >= num_samples {
                    continue;
                }

                // Each flap: noise burst with fast attack, medium decay
                let flap_amp = self.amplitude * self.wind_speed * self.rng.next_f32_range(0.5, 1.0);
                let flap_dur = (self.sample_rate * self.rng.next_f32_range(0.005, 0.02)) as usize;

                for j in 0..flap_dur.min(num_samples - idx) {
                    let t = j as f32 / flap_dur as f32;
                    // Fast attack, longer decay
                    let env = if t < 0.1 {
                        t / 0.1
                    } else {
                        crate::math::f32::exp(-5.0 * (t - 0.1))
                    };

                    let noise = self.generate_noise();
                    let exc = noise * env * flap_amp;

                    // Feed through modal bank for heavy cloth
                    let out = if let Some(ref mut bank) = self.modal_bank {
                        bank.process_sample(exc) + exc * 0.3
                    } else {
                        exc
                    };

                    output[idx + j] += out;
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
            let raw = self.noise_gen.next_sample();
            self.shape_filter.process_sample(raw)
        }
        #[cfg(not(feature = "naad-backend"))]
        {
            self.rng.next_f32()
        }
    }
}
