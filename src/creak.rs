//! Creak synthesis: doors, hinges, rope, wood stress.
//!
//! Models creaking as low-frequency stick-slip oscillation through
//! material-specific resonant modes. Tension controls pitch,
//! speed controls amplitude.

use alloc::vec::Vec;
use serde::{Deserialize, Serialize};

use crate::contact::CreakSource;
use crate::dsp::{DcBlocker, validate_sample_rate};
use crate::error::Result;
use crate::material::Material;
use crate::modal::{ModalBank, ModePattern, generate_modes};
use crate::rng::Rng;

/// Creak sound synthesizer — low-frequency stick-slip.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Creak {
    source: CreakSource,
    sample_rate: f32,
    rng: Rng,
    dc_blocker: DcBlocker,
    sample_position: usize,
    // Stick-slip state
    slip_phase: f32,
    // Frequency range for this source
    freq_lo: f32,
    freq_hi: f32,
    noise_mix: f32,
    // Real-time parameters
    tension: f32,
    speed: f32,
    // Modal resonance
    modal_bank: ModalBank,
    #[cfg(feature = "naad-backend")]
    noise_gen: naad::noise::NoiseGenerator,
    #[cfg(feature = "naad-backend")]
    shape_filter: naad::filter::BiquadFilter,
}

impl Creak {
    /// Creates a new creak synthesizer.
    pub fn new(source: CreakSource, sample_rate: f32) -> Result<Self> {
        validate_sample_rate(sample_rate)?;

        let (mat, pattern, mode_count, freq_lo, freq_hi, noise_mix) = match source {
            CreakSource::Door => (Material::Wood, ModePattern::Beam, 5, 40.0, 200.0, 0.1),
            CreakSource::Hinge => (Material::Metal, ModePattern::Plate, 6, 100.0, 500.0, 0.05),
            CreakSource::Rope => (Material::Fabric, ModePattern::Harmonic, 4, 30.0, 150.0, 0.3),
            CreakSource::WoodStress => (Material::Wood, ModePattern::Damped, 4, 25.0, 105.0, 0.15),
        };

        let props = mat.properties();
        let mode_cfg = mat.mode_config();
        let specs = generate_modes(&props, pattern, mode_count, mode_cfg.damping_factor);
        let modal_bank = ModalBank::new(&specs, sample_rate)?;

        #[cfg(feature = "naad-backend")]
        let noise_gen = naad::noise::NoiseGenerator::new(naad::noise::NoiseType::Pink, 5555);
        #[cfg(feature = "naad-backend")]
        let shape_filter = {
            let filter_freq = match source {
                CreakSource::Door => 400.0,
                CreakSource::Hinge => 800.0,
                CreakSource::Rope => 300.0,
                CreakSource::WoodStress => 250.0,
            };
            naad::filter::BiquadFilter::new(
                naad::filter::FilterType::LowPass,
                sample_rate,
                filter_freq,
                0.7,
            )
        }
        .map_err(|e| crate::error::GarjanError::SynthesisFailed(alloc::format!("{e}")))?;

        Ok(Self {
            source,
            sample_rate,
            rng: Rng::new(5555),
            dc_blocker: DcBlocker::new(sample_rate),
            sample_position: 0,
            slip_phase: 0.0,
            freq_lo,
            freq_hi,
            noise_mix,
            tension: 0.5,
            speed: 0.0,
            modal_bank,
            #[cfg(feature = "naad-backend")]
            noise_gen,
            #[cfg(feature = "naad-backend")]
            shape_filter,
        })
    }

    /// Sets the tension (0.0 = low pitch, 1.0 = high pitch).
    pub fn set_tension(&mut self, tension: f32) {
        self.tension = tension.clamp(0.0, 1.0);
    }

    /// Sets the speed (0.0 = silent, 1.0 = fast creak).
    pub fn set_speed(&mut self, speed: f32) {
        self.speed = speed.clamp(0.0, 1.0);
    }

    /// Synthesizes creak audio.
    #[inline]
    pub fn synthesize(&mut self, duration: f32) -> Result<Vec<f32>> {
        let num_samples = (self.sample_rate * duration) as usize;
        let mut output = alloc::vec![0.0f32; num_samples];
        self.process_block(&mut output);
        Ok(output)
    }

    /// Fills output buffer with creak audio (streaming).
    #[inline]
    pub fn process_block(&mut self, output: &mut [f32]) {
        for sample in output.iter_mut() {
            if self.speed < 0.001 {
                *sample = 0.0;
                self.dc_blocker.process(0.0);
                continue;
            }

            // Base frequency from tension
            let freq = self.freq_lo + self.tension * (self.freq_hi - self.freq_lo);

            // Advance slip phase with jitter for organic quality
            let jitter = 1.0 + self.rng.next_f32_range(-0.05, 0.05);
            self.slip_phase += freq * jitter / self.sample_rate;

            if self.slip_phase >= 1.0 {
                self.slip_phase -= 1.0;
            }

            // Asymmetric sawtooth-like excitation (fast slip, slow stick)
            let excitation = (self.slip_phase * 2.0 - 1.0) * self.speed;

            // Add noise for texture
            let noise = self.generate_noise() * self.noise_mix * self.speed;

            // Through modal bank
            let resonant = self.modal_bank.process_sample(excitation + noise);

            *sample = resonant;
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
