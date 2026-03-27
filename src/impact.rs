//! Impact and contact sound synthesis.
//!
//! Models the sound of objects striking surfaces: footsteps, crashes,
//! knocks, drops. Uses the material's resonant properties to shape
//! a noise transient into a physically plausible impact sound.

use alloc::vec::Vec;
use serde::{Deserialize, Serialize};

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
    /// Surface material.
    material: Material,
    /// Material properties (cached).
    props: MaterialProperties,
    /// PRNG.
    rng: Rng,
}

impl Impact {
    /// Creates a new impact synthesizer for the given material.
    #[must_use]
    pub fn new(material: Material) -> Self {
        let props = material.properties();
        Self {
            material,
            props,
            rng: Rng::new(5381),
        }
    }

    /// Returns the material.
    #[must_use]
    pub fn material(&self) -> Material {
        self.material
    }

    /// Synthesizes an impact sound.
    pub fn synthesize(&mut self, impact_type: ImpactType, sample_rate: f32) -> Result<Vec<f32>> {
        let force = impact_type.force();
        let duration = self.props.decay * 2.0 + 0.02; // Transient + decay
        let num_samples = (duration * sample_rate) as usize;
        let mut output = Vec::with_capacity(num_samples);

        let transient_len = (sample_rate * 0.005) as usize; // 5ms transient
        let omega = core::f32::consts::TAU * self.props.resonance / sample_rate;

        for i in 0..num_samples {
            let t = i as f32 / sample_rate;

            // Transient: noise burst shaped by material brightness
            let transient = if i < transient_len {
                let env = 1.0 - (i as f32 / transient_len as f32);
                self.rng.next_f32() * env * self.props.transient * force
            } else {
                0.0
            };

            // Resonance: decaying sinusoid at material's resonant frequency
            let decay_env = crate::math::f32::exp(-t / self.props.decay.max(0.001));
            let resonance = crate::math::f32::sin(omega * i as f32) * decay_env * force * 0.5;

            output.push(transient + resonance);
        }

        // Shatter: add secondary high-frequency debris
        if impact_type == ImpactType::Shatter {
            let debris_start = transient_len;
            let debris_freq = self.props.resonance * 2.5;
            let debris_omega = core::f32::consts::TAU * debris_freq / sample_rate;
            for (i, sample) in output.iter_mut().enumerate().skip(debris_start) {
                let t = (i - debris_start) as f32 / sample_rate;
                let env = crate::math::f32::exp(-t / 0.1);
                *sample += crate::math::f32::sin(debris_omega * i as f32)
                    * env
                    * force
                    * 0.3
                    * self.props.brightness;
            }
        }

        Ok(output)
    }
}
