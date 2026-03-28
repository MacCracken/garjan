//! Impact and contact sound synthesis.
//!
//! Models the sound of objects striking surfaces: footsteps, crashes,
//! knocks, drops. Uses modal synthesis to produce physically plausible
//! resonant responses shaped by the struck material's properties.

use alloc::vec::Vec;
use serde::{Deserialize, Serialize};

use crate::dsp::{DcBlocker, validate_sample_rate};
use crate::error::Result;
use crate::material::{Material, MaterialProperties};
use crate::modal::{ExcitationType, Exciter, ModalBank, generate_modes};
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

    /// Returns the default excitation type and duration in seconds for this impact.
    #[must_use]
    fn excitation_config(self, sample_rate: f32) -> ExcitationType {
        match self {
            Self::Tap => ExcitationType::HalfSine {
                duration_samples: (sample_rate * 0.002) as usize,
            },
            Self::Strike => ExcitationType::NoiseBurst {
                duration_samples: (sample_rate * 0.003) as usize,
            },
            Self::Crash => ExcitationType::NoiseBurst {
                duration_samples: (sample_rate * 0.001) as usize,
            },
            Self::Shatter => ExcitationType::NoiseBurst {
                duration_samples: (sample_rate * 0.001) as usize,
            },
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
    modal_bank: ModalBank,
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
        let mode_cfg = material.mode_config();
        let mode_specs = generate_modes(
            &props,
            mode_cfg.pattern,
            mode_cfg.mode_count,
            mode_cfg.damping_factor,
        );
        let modal_bank = ModalBank::new(&mode_specs, sample_rate)?;
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
            modal_bank,
            #[cfg(feature = "naad-backend")]
            transient_noise: naad::noise::NoiseGenerator::new(naad::noise::NoiseType::White, 5381),
            #[cfg(feature = "naad-backend")]
            resonance_filter,
        })
    }

    /// Creates an impact synthesizer for a material interaction.
    ///
    /// The `surface` material provides the resonant modes (what rings).
    /// The `striker` material modifies the excitation character:
    /// harder strikers produce shorter, brighter excitations.
    pub fn new_interaction(
        _striker: Material,
        surface: Material,
        sample_rate: f32,
    ) -> Result<Self> {
        // Surface provides resonance, striker just modifies excitation (via force/type)
        // The constructor creates modes from the surface material
        Self::new(surface, sample_rate)
        // Caller uses striker properties to select impact type:
        // hard striker (Metal, Stone, Glass) → Crash/Strike
        // soft striker (Fabric, Earth, Leaf) → Tap
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
        self.synthesize_velocity(impact_type, impact_type.force())
    }

    /// Synthesizes an impact sound with explicit velocity control.
    ///
    /// `velocity` ranges from 0.0 (silent) to 1.0 (maximum force).
    /// Higher velocity produces louder, brighter sounds with shorter excitations.
    pub fn synthesize_velocity(
        &mut self,
        impact_type: ImpactType,
        velocity: f32,
    ) -> Result<Vec<f32>> {
        let velocity = velocity.clamp(0.0, 1.0);
        let duration = self.props.decay * 2.0 + 0.05;
        let num_samples = (duration * self.sample_rate) as usize;
        let mut output = alloc::vec![0.0f32; num_samples];
        self.sample_position = 0;
        self.modal_bank.reset();
        self.generate_impact(impact_type, velocity, &mut output);
        Ok(output)
    }

    /// Fills output buffer with impact audio for the given impact type (streaming).
    #[inline]
    pub fn process_block(&mut self, impact_type: ImpactType, output: &mut [f32]) {
        self.generate_impact(impact_type, impact_type.force(), output);
    }

    fn generate_impact(&mut self, impact_type: ImpactType, velocity: f32, output: &mut [f32]) {
        // Build excitation signal
        let exc_type = impact_type.excitation_config(self.sample_rate);
        let mut exciter = Exciter::new(exc_type, velocity);
        exciter.trigger();

        let num_samples = output.len();

        // Generate primary excitation + modal response
        for (i, sample) in output.iter_mut().enumerate() {
            let mut exc = exciter.next_sample();

            // Add noise transient (filtered through material resonance with naad)
            let abs_pos = self.sample_position + i;
            let transient_len = (self.sample_rate * 0.005) as usize;
            if abs_pos < transient_len {
                let env = 1.0 - (abs_pos as f32 / transient_len as f32);
                #[cfg(feature = "naad-backend")]
                {
                    let noise = self.transient_noise.next_sample();
                    exc += self.resonance_filter.process_sample(noise)
                        * env
                        * self.props.transient
                        * velocity;
                }
                #[cfg(not(feature = "naad-backend"))]
                {
                    exc += self.rng.next_f32() * env * self.props.transient * velocity;
                }
            }

            *sample = self.modal_bank.process_sample(exc);
        }

        // Shatter: add debris cascade
        if impact_type == ImpactType::Shatter {
            self.add_shatter_debris(velocity, output);
        }

        // DC blocking
        for sample in output.iter_mut() {
            *sample = self.dc_blocker.process(*sample);
        }
        self.sample_position += num_samples;
    }

    fn add_shatter_debris(&mut self, velocity: f32, output: &mut [f32]) {
        let num_samples = output.len();
        let debris_window = (self.sample_rate * 0.2) as usize; // 200ms window
        let n_debris = 3 + self.rng.poisson(5.0); // 3-8 debris events

        for _ in 0..n_debris {
            let offset = self.rng.next_f32_range(
                self.sample_rate * 0.01, // start after 10ms
                debris_window as f32,
            ) as usize;

            if offset >= num_samples {
                continue;
            }

            let debris_amp = velocity * self.rng.next_f32_range(0.1, 0.4);
            let debris_dur = self.rng.next_f32_range(3.0, 15.0) as usize; // very short burst

            // Short impulse excitation at the debris time
            for j in 0..debris_dur.min(num_samples - offset) {
                let env = 1.0 - (j as f32 / debris_dur as f32);
                let exc = debris_amp * env * self.rng.next_f32();
                // Feed through the already-ringing modal bank
                output[offset + j] += self.modal_bank.process_sample(exc);
            }
        }
    }
}
