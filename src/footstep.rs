//! Footstep synthesis: terrain-aware step sequences.
//!
//! Generates footstep sounds as repeating impact events on surfaces.
//! Each step combines a terrain noise layer (crunch, squish, clack)
//! with optional modal resonance from the surface material.

use alloc::vec::Vec;
use serde::{Deserialize, Serialize};

#[cfg(feature = "naad-backend")]
use crate::contact::NoisePreference;
use crate::contact::{MovementType, Terrain};
use crate::dsp::{DcBlocker, validate_sample_rate};
use crate::error::Result;
use crate::modal::{ExcitationType, Exciter, ModalBank, generate_modes};
use crate::rng::Rng;

/// Footstep synthesizer — terrain-aware step sequences.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Footstep {
    terrain: Terrain,
    movement: MovementType,
    sample_rate: f32,
    rng: Rng,
    dc_blocker: DcBlocker,
    sample_position: usize,
    // Step timing
    base_step_interval: usize,
    step_interval_samples: usize,
    samples_since_last_step: usize,
    step_force: f32,
    step_pending: bool,
    // Modal resonance (Some for Wood, Metal, Tile, Wet)
    modal_bank: Option<ModalBank>,
    exciter: Exciter,
    // Noise layer config (stored for fallback path)
    noise_filter_freq: f32,
    noise_amplitude: f32,
    noise_highpass: bool,
    #[cfg(feature = "naad-backend")]
    noise_gen: naad::noise::NoiseGenerator,
    #[cfg(feature = "naad-backend")]
    shape_filter: naad::filter::BiquadFilter,
}

impl Footstep {
    /// Creates a new footstep synthesizer.
    pub fn new(terrain: Terrain, movement: MovementType, sample_rate: f32) -> Result<Self> {
        validate_sample_rate(sample_rate)?;

        let (interval, force, exc_dur) = movement.config();
        let step_interval_samples = if interval > 0.0 {
            (interval * sample_rate) as usize
        } else {
            0 // one-shot
        };
        let exc_samples = (exc_dur * sample_rate) as usize;
        let exciter = Exciter::new(
            if movement == MovementType::Sneak {
                ExcitationType::HalfSine {
                    duration_samples: exc_samples,
                }
            } else {
                ExcitationType::NoiseBurst {
                    duration_samples: exc_samples,
                }
            },
            force,
        );

        // Modal bank from terrain's resonant material
        let modal_bank = if let Some(mat) = terrain.resonant_material() {
            let props = mat.properties();
            let mode_cfg = mat.mode_config();
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

        let noise_cfg = terrain.noise_config();
        #[cfg(feature = "naad-backend")]
        let noise_gen = {
            let nt = match noise_cfg.noise_type {
                NoisePreference::White => naad::noise::NoiseType::White,
                NoisePreference::Pink => naad::noise::NoiseType::Pink,
                NoisePreference::Brown => naad::noise::NoiseType::Brown,
            };
            naad::noise::NoiseGenerator::new(nt, 8888)
        };
        #[cfg(feature = "naad-backend")]
        let shape_filter = {
            let ft = if noise_cfg.highpass {
                naad::filter::FilterType::HighPass
            } else {
                naad::filter::FilterType::LowPass
            };
            naad::filter::BiquadFilter::new(
                ft,
                sample_rate,
                noise_cfg.filter_freq,
                noise_cfg.filter_q,
            )
            .map_err(|e| crate::error::GarjanError::SynthesisFailed(alloc::format!("{e}")))?
        };

        Ok(Self {
            terrain,
            movement,
            sample_rate,
            rng: Rng::new(8888),
            dc_blocker: DcBlocker::new(sample_rate),
            sample_position: 0,
            base_step_interval: step_interval_samples,
            step_interval_samples,
            samples_since_last_step: step_interval_samples, // trigger immediately
            step_force: force,
            step_pending: movement == MovementType::JumpLand,
            modal_bank,
            exciter,
            noise_filter_freq: noise_cfg.filter_freq,
            noise_amplitude: noise_cfg.amplitude,
            noise_highpass: noise_cfg.highpass,
            #[cfg(feature = "naad-backend")]
            noise_gen,
            #[cfg(feature = "naad-backend")]
            shape_filter,
        })
    }

    /// Manually triggers a single footstep event.
    pub fn trigger_step(&mut self) {
        self.step_pending = true;
    }

    /// Synthesizes footstep audio.
    #[inline]
    pub fn synthesize(&mut self, duration: f32) -> Result<Vec<f32>> {
        crate::dsp::validate_duration(duration)?;
        let num_samples = (self.sample_rate * duration) as usize;
        let mut output = alloc::vec![0.0f32; num_samples];
        self.process_block(&mut output);
        Ok(output)
    }

    /// Fills output buffer with footstep audio (streaming).
    #[inline]
    pub fn process_block(&mut self, output: &mut [f32]) {
        for sample in output.iter_mut() {
            // Check if a step should trigger
            if self.step_pending
                || (self.step_interval_samples > 0
                    && self.samples_since_last_step >= self.step_interval_samples)
            {
                self.fire_step();
            }

            // Generate excitation
            let exc = self.exciter.next_sample();

            // Noise layer (terrain character)
            let noise_env = if self.samples_since_last_step < (self.sample_rate * 0.03) as usize {
                let t = self.samples_since_last_step as f32 / (self.sample_rate * 0.03);
                crate::math::f32::exp(-5.0 * t)
            } else {
                0.0
            };

            let noise = self.generate_noise() * noise_env * self.step_force;

            // Modal resonance
            let resonant = if let Some(ref mut bank) = self.modal_bank {
                bank.process_sample(exc + noise * 0.3)
            } else {
                exc * 0.5
            };

            *sample = resonant + noise * self.noise_amplitude;
            *sample = self.dc_blocker.process(*sample);

            self.samples_since_last_step += 1;
        }
        self.sample_position += output.len();
    }

    fn fire_step(&mut self) {
        self.samples_since_last_step = 0;
        self.step_pending = false;
        self.exciter.trigger();
        if let Some(ref mut bank) = self.modal_bank {
            bank.reset();
        }
        // Apply jitter from base interval (prevents cumulative drift)
        if self.base_step_interval > 0 {
            let jitter = 1.0 + self.rng.next_f32_range(-0.05, 0.05);
            self.step_interval_samples = ((self.base_step_interval as f32) * jitter) as usize;
        }
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
