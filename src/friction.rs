//! Friction synthesis: scraping, sliding, grinding.
//!
//! Models stick-slip friction where the contact alternates between
//! static friction (stuck) and kinetic friction (slipping), creating
//! quasi-periodic excitation through a material's resonant modes.

use alloc::vec::Vec;
use serde::{Deserialize, Serialize};

use crate::contact::FrictionType;
use crate::dsp::{DcBlocker, validate_sample_rate};
use crate::error::Result;
use crate::material::Material;
use crate::modal::{ModalBank, generate_modes};
use crate::rng::Rng;

/// Friction sound synthesizer — stick-slip model.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Friction {
    friction_type: FrictionType,
    surface: Material,
    sample_rate: f32,
    rng: Rng,
    dc_blocker: DcBlocker,
    sample_position: usize,
    // Stick-slip state
    slip_phase: f32,
    stick_threshold: f32,
    base_rate: f32,
    noise_mix: f32,
    // Real-time parameters
    velocity: f32,
    pressure: f32,
    // Modal resonance
    modal_bank: ModalBank,
    #[cfg(feature = "naad-backend")]
    noise_gen: naad::noise::NoiseGenerator,
    #[cfg(feature = "naad-backend")]
    shape_filter: naad::filter::BiquadFilter,
}

impl Friction {
    /// Creates a new friction synthesizer.
    pub fn new(friction_type: FrictionType, surface: Material, sample_rate: f32) -> Result<Self> {
        validate_sample_rate(sample_rate)?;

        let props = surface.properties();
        let mode_cfg = surface.mode_config();
        let specs = generate_modes(
            &props,
            mode_cfg.pattern,
            mode_cfg.mode_count.min(6),
            mode_cfg.damping_factor,
        );
        let modal_bank = ModalBank::new(&specs, sample_rate)?;

        let (base_rate, stick_threshold, noise_mix) = match friction_type {
            FrictionType::Scrape => (200.0, 0.8, 0.3),
            FrictionType::Slide => (400.0, 0.5, 0.15),
            FrictionType::Grind => (60.0, 1.2, 0.5),
        };

        #[cfg(feature = "naad-backend")]
        let noise_gen = naad::noise::NoiseGenerator::new(naad::noise::NoiseType::White, 7777);
        #[cfg(feature = "naad-backend")]
        let shape_filter = {
            let filter_freq = match friction_type {
                FrictionType::Scrape => 2000.0,
                FrictionType::Slide => 3000.0,
                FrictionType::Grind => 800.0,
            };
            naad::filter::BiquadFilter::new(
                naad::filter::FilterType::BandPass,
                sample_rate,
                filter_freq,
                1.5,
            )
            .map_err(|e| crate::error::GarjanError::SynthesisFailed(alloc::format!("{e}")))?
        };

        Ok(Self {
            friction_type,
            surface,
            sample_rate,
            rng: Rng::new(7777),
            dc_blocker: DcBlocker::new(sample_rate),
            sample_position: 0,
            slip_phase: 0.0,
            stick_threshold,
            base_rate,
            noise_mix,
            velocity: 0.0,
            pressure: 0.5,
            modal_bank,
            #[cfg(feature = "naad-backend")]
            noise_gen,
            #[cfg(feature = "naad-backend")]
            shape_filter,
        })
    }

    /// Sets the friction velocity (0.0 = still, 1.0 = fast).
    pub fn set_velocity(&mut self, velocity: f32) {
        self.velocity = velocity.clamp(0.0, 1.0);
    }

    /// Sets the contact pressure (0.0 = light, 1.0 = heavy).
    pub fn set_pressure(&mut self, pressure: f32) {
        self.pressure = pressure.clamp(0.0, 1.0);
    }

    /// Synthesizes friction audio.
    #[inline]
    pub fn synthesize(&mut self, duration: f32) -> Result<Vec<f32>> {
        let num_samples = (self.sample_rate * duration) as usize;
        let mut output = alloc::vec![0.0f32; num_samples];
        self.process_block(&mut output);
        Ok(output)
    }

    /// Fills output buffer with friction audio (streaming).
    #[inline]
    pub fn process_block(&mut self, output: &mut [f32]) {
        for sample in output.iter_mut() {
            if self.velocity < 0.001 {
                *sample = 0.0;
                self.dc_blocker.process(0.0);
                continue;
            }

            // Advance stick-slip phase
            let jitter = 1.0 + self.rng.next_f32_range(-0.1, 0.1);
            self.slip_phase += self.velocity * self.base_rate * jitter / self.sample_rate;

            // Slip event
            let excitation = if self.slip_phase >= self.stick_threshold {
                self.slip_phase -= self.stick_threshold;
                self.pressure * 0.5
            } else {
                // Micro-noise during stick phase
                self.pressure * self.velocity * self.rng.next_f32() * 0.02
            };

            // Continuous noise component
            let noise = self.generate_noise() * self.velocity * self.pressure * self.noise_mix;

            // Modal resonance
            let resonant = self.modal_bank.process_sample(excitation);

            *sample = resonant + noise;
            *sample = self.dc_blocker.process(*sample);
        }
        self.sample_position += output.len();
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
