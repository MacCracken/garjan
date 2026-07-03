//! Rolling synthesis: ball, wheel, boulder, barrel on surfaces.
//!
//! Models continuous contact between a round body and a surface.
//! Surface noise is shaped by material properties with periodic
//! bumps from rotation imperfections.

use alloc::vec::Vec;
use serde::{Deserialize, Serialize};

use crate::contact::RollingBody;
use crate::dsp::{DcBlocker, validate_sample_rate};
use crate::error::Result;
use crate::material::Material;
use crate::modal::{ModalBank, generate_modes};
use crate::rng::Rng;

/// Rolling sound synthesizer — continuous surface contact.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Rolling {
    body: RollingBody,
    surface: Material,
    sample_rate: f32,
    rng: Rng,
    dc_blocker: DcBlocker,
    sample_position: usize,
    // Body properties
    radius: f32,
    mass_factor: f32,
    hollowness: f32,
    // Rotation state
    rotation_phase: f32,
    // Real-time parameter
    velocity: f32,
    // Modal bank (for hollow bodies and surface resonance)
    modal_bank: Option<ModalBank>,
    #[cfg(feature = "naad-backend")]
    noise_gen: naad::noise::NoiseGenerator,
    #[cfg(feature = "naad-backend")]
    surface_filter: naad::filter::BiquadFilter,
}

impl Rolling {
    /// Creates a new rolling synthesizer.
    pub fn new(body: RollingBody, surface: Material, sample_rate: f32) -> Result<Self> {
        validate_sample_rate(sample_rate)?;

        let (radius, mass_factor, hollowness) = match body {
            RollingBody::Ball => (0.05, 0.2, 0.0),
            RollingBody::Wheel => (0.3, 0.5, 0.0),
            RollingBody::Boulder => (1.0, 1.0, 0.0),
            RollingBody::Barrel => (0.4, 0.6, 0.8),
        };

        // Modal bank for hollow bodies or resonant surfaces
        let modal_bank = if hollowness > 0.0 {
            let mat = Material::Wood; // barrels are wooden
            let props = mat.properties();
            let mode_cfg = mat.mode_config();
            let specs = generate_modes(
                &props,
                mode_cfg.pattern,
                mode_cfg.mode_count.min(6),
                mode_cfg.damping_factor,
            );
            Some(ModalBank::new(&specs, sample_rate)?)
        } else {
            None
        };

        #[cfg(feature = "naad-backend")]
        let noise_gen = naad::noise::NoiseGenerator::new(naad::noise::NoiseType::Pink, 3333);
        #[cfg(feature = "naad-backend")]
        let surface_filter = {
            let base_cutoff = surface.properties().resonance * (1.0 - 0.3 * mass_factor);
            naad::filter::BiquadFilter::new(
                naad::filter::FilterType::BandPass,
                sample_rate,
                base_cutoff.clamp(50.0, 8000.0),
                1.0,
            )
            .map_err(|e| crate::error::GarjanError::SynthesisFailed(alloc::format!("{e}")))?
        };

        Ok(Self {
            body,
            surface,
            sample_rate,
            rng: Rng::new(3333),
            dc_blocker: DcBlocker::new(sample_rate),
            sample_position: 0,
            radius,
            mass_factor,
            hollowness,
            rotation_phase: 0.0,
            velocity: 0.0,
            modal_bank,
            #[cfg(feature = "naad-backend")]
            noise_gen,
            #[cfg(feature = "naad-backend")]
            surface_filter,
        })
    }

    /// Sets the rolling velocity (0.0 = still, 1.0 = fast).
    pub fn set_velocity(&mut self, velocity: f32) {
        self.velocity = velocity.clamp(0.0, 1.0);
    }

    /// Synthesizes rolling audio.
    #[inline]
    pub fn synthesize(&mut self, duration: f32) -> Result<Vec<f32>> {
        crate::dsp::validate_duration(duration)?;
        let num_samples = (self.sample_rate * duration) as usize;
        let mut output = alloc::vec![0.0f32; num_samples];
        self.process_block(&mut output);
        Ok(output)
    }

    /// Fills output buffer with rolling audio (streaming).
    #[inline]
    pub fn process_block(&mut self, output: &mut [f32]) {
        for sample in output.iter_mut() {
            if self.velocity < 0.001 {
                *sample = 0.0;
                self.dc_blocker.process(0.0);
                continue;
            }

            // Rotation rate from velocity and radius
            let rotation_rate = self.velocity / (core::f32::consts::TAU * self.radius);
            self.rotation_phase += rotation_rate / self.sample_rate;
            if self.rotation_phase >= 1.0 {
                self.rotation_phase -= 1.0;
            }

            // Surface noise
            let surface_noise = self.generate_noise() * self.velocity * 0.3;

            // Rotation bumps from imperfections
            let bump_env =
                (1.0 - crate::math::f32::cos(core::f32::consts::TAU * self.rotation_phase)) * 0.5;
            let bump = bump_env * self.velocity * 0.2;

            // Hollow body resonance
            let resonant = if let Some(ref mut bank) = self.modal_bank {
                bank.process_sample((bump + surface_noise * 0.1) * self.hollowness) * 0.5
            } else {
                0.0
            };

            *sample = surface_noise + bump + resonant;
            *sample = self.dc_blocker.process(*sample);
        }
        self.sample_position += output.len();
    }

    #[inline]
    fn generate_noise(&mut self) -> f32 {
        #[cfg(feature = "naad-backend")]
        {
            let raw = self.noise_gen.next_sample();
            self.surface_filter.process_sample(raw)
        }
        #[cfg(not(feature = "naad-backend"))]
        {
            (self.rng.next_f32() + self.rng.next_f32()) * 0.5
        }
    }
}
