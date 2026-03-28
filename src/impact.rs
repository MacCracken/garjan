//! Impact and contact sound synthesis.
//!
//! Models the sound of objects striking surfaces: footsteps, crashes,
//! knocks, drops. Uses the material's resonant properties to shape
//! a noise transient into a physically plausible impact sound.

use alloc::vec::Vec;
use serde::{Deserialize, Serialize};

use crate::dsp::{DcBlocker, validate_sample_rate};
use crate::error::Result;
use crate::material::{Material, MaterialProperties};
use crate::rng::Rng;

/// Type of impact event.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[non_exhaustive]
pub enum ImpactType {
    /// Light tap or touch.
    Tap,
    /// Moderate strike (footstep, knock).
    Strike,
    /// Heavy blow (hammer, collision).
    Crash,
    /// Object breaking apart.
    Shatter,
}

impl ImpactType {
    /// Returns the force multiplier for this impact type.
    #[must_use]
    fn force(self) -> f32 {
        match self {
            Self::Tap => 0.2,
            Self::Strike => 0.5,
            Self::Crash => 1.0,
            Self::Shatter => 0.8,
        }
    }
}

/// Impact sound synthesizer.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Impact {
    material: Material,
    props: MaterialProperties,
    sample_rate: f32,
    rng: Rng,
    dc_blocker: DcBlocker,
    sample_position: usize,
    #[cfg(feature = "naad-backend")]
    transient_noise: naad::noise::NoiseGenerator,
    #[cfg(feature = "naad-backend")]
    resonance_filter: naad::filter::BiquadFilter,
}

impl Impact {
    /// Creates a new impact synthesizer for the given material.
    pub fn new(material: Material, sample_rate: f32) -> Result<Self> {
        validate_sample_rate(sample_rate)?;
        let props = material.properties();
        #[cfg(feature = "naad-backend")]
        let resonance_filter = {
            let q = (props.resonance / props.bandwidth.max(1.0)).clamp(0.1, 20.0);
            naad::filter::BiquadFilter::new(
                naad::filter::FilterType::BandPass,
                sample_rate,
                props.resonance,
                q,
            )
            .map_err(|e| crate::error::GarjanError::SynthesisFailed(alloc::format!("{e}")))?
        };
        Ok(Self {
            material,
            props,
            sample_rate,
            rng: Rng::new(5381),
            dc_blocker: DcBlocker::new(sample_rate),
            sample_position: 0,
            #[cfg(feature = "naad-backend")]
            transient_noise: naad::noise::NoiseGenerator::new(naad::noise::NoiseType::White, 5381),
            #[cfg(feature = "naad-backend")]
            resonance_filter,
        })
    }

    /// Returns the material.
    #[inline]
    #[must_use]
    pub fn material(&self) -> Material {
        self.material
    }

    /// Synthesizes an impact sound.
    #[inline]
    pub fn synthesize(&mut self, impact_type: ImpactType) -> Result<Vec<f32>> {
        let duration = self.props.decay * 2.0 + 0.02;
        let num_samples = (duration * self.sample_rate) as usize;
        let mut output = Vec::with_capacity(num_samples);
        output.resize(num_samples, 0.0);
        self.sample_position = 0;
        self.process_block_for_type(impact_type, &mut output);
        Ok(output)
    }

    /// Fills output buffer with impact audio for the given impact type (streaming).
    #[inline]
    pub fn process_block(&mut self, impact_type: ImpactType, output: &mut [f32]) {
        self.process_block_for_type(impact_type, output);
    }

    fn process_block_for_type(&mut self, impact_type: ImpactType, output: &mut [f32]) {
        #[cfg(feature = "naad-backend")]
        self.process_block_naad(impact_type, output);
        #[cfg(not(feature = "naad-backend"))]
        self.process_block_fallback(impact_type, output);

        for sample in output.iter_mut() {
            *sample = self.dc_blocker.process(*sample);
        }
        self.sample_position += output.len();
    }

    #[cfg(feature = "naad-backend")]
    fn process_block_naad(&mut self, impact_type: ImpactType, output: &mut [f32]) {
        let force = impact_type.force();
        let transient_len = (self.sample_rate * 0.005) as usize;
        let omega = core::f32::consts::TAU * self.props.resonance / self.sample_rate;

        for (i, sample) in output.iter_mut().enumerate() {
            let abs_pos = self.sample_position + i;
            let t = abs_pos as f32 / self.sample_rate;

            // Transient: filtered noise burst
            let transient = if abs_pos < transient_len {
                let env = 1.0 - (abs_pos as f32 / transient_len as f32);
                let noise = self.transient_noise.next_sample();
                self.resonance_filter.process_sample(noise) * env * self.props.transient * force
            } else {
                0.0
            };

            // Resonance: decaying sinusoid
            let decay_env = crate::math::f32::exp(-t / self.props.decay.max(0.001));
            let resonance = crate::math::f32::sin(omega * abs_pos as f32) * decay_env * force * 0.5;

            *sample = transient + resonance;
        }

        // Shatter: secondary high-frequency debris
        if impact_type == ImpactType::Shatter {
            let transient_len = (self.sample_rate * 0.005) as usize;
            let debris_freq = self.props.resonance * 2.5;
            let debris_omega = core::f32::consts::TAU * debris_freq / self.sample_rate;
            for (i, sample) in output.iter_mut().enumerate() {
                let abs_pos = self.sample_position + i;
                if abs_pos >= transient_len {
                    let dt = (abs_pos - transient_len) as f32 / self.sample_rate;
                    let env = crate::math::f32::exp(-dt / 0.1);
                    *sample += crate::math::f32::sin(debris_omega * abs_pos as f32)
                        * env
                        * force
                        * 0.3
                        * self.props.brightness;
                }
            }
        }
    }

    #[cfg(not(feature = "naad-backend"))]
    fn process_block_fallback(&mut self, impact_type: ImpactType, output: &mut [f32]) {
        let force = impact_type.force();
        let transient_len = (self.sample_rate * 0.005) as usize;
        let omega = core::f32::consts::TAU * self.props.resonance / self.sample_rate;

        for (i, sample) in output.iter_mut().enumerate() {
            let abs_pos = self.sample_position + i;
            let t = abs_pos as f32 / self.sample_rate;

            let transient = if abs_pos < transient_len {
                let env = 1.0 - (abs_pos as f32 / transient_len as f32);
                self.rng.next_f32() * env * self.props.transient * force
            } else {
                0.0
            };

            let decay_env = crate::math::f32::exp(-t / self.props.decay.max(0.001));
            let resonance = crate::math::f32::sin(omega * abs_pos as f32) * decay_env * force * 0.5;

            *sample = transient + resonance;
        }

        if impact_type == ImpactType::Shatter {
            let debris_freq = self.props.resonance * 2.5;
            let debris_omega = core::f32::consts::TAU * debris_freq / self.sample_rate;
            for (i, sample) in output.iter_mut().enumerate() {
                let abs_pos = self.sample_position + i;
                if abs_pos >= transient_len {
                    let dt = (abs_pos - transient_len) as f32 / self.sample_rate;
                    let env = crate::math::f32::exp(-dt / 0.1);
                    *sample += crate::math::f32::sin(debris_omega * abs_pos as f32)
                        * env
                        * force
                        * 0.3
                        * self.props.brightness;
                }
            }
        }
    }
}
