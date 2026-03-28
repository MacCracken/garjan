//! Extended precipitation synthesis: hail, snow, surface-dependent rain.
//!
//! Enhances the base rain module with physically-varied precipitation types
//! and surface interaction. Hail uses the modal bank for impact resonance,
//! snow uses filtered noise with very short decay.

use alloc::vec::Vec;
use serde::{Deserialize, Serialize};

use crate::contact::Terrain;
use crate::dsp::{DcBlocker, validate_sample_rate};
use crate::error::Result;
use crate::modal::{ModalBank, generate_modes};
use crate::rng::Rng;

/// Type of precipitation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[non_exhaustive]
pub enum PrecipitationType {
    /// Hailstones — impacts with modal resonance from surface.
    Hail,
    /// Snow — quiet, muffled crunch, very short decay.
    Snow,
    /// Rain on a specific surface — splatter varies by terrain.
    SurfaceRain,
}

/// Hail/snow stone size affecting impact character.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[non_exhaustive]
pub enum StoneSize {
    /// Small (pea-sized hail, light snow).
    Small,
    /// Medium (marble-sized hail, moderate snow).
    Medium,
    /// Large (golf-ball hail, heavy wet snow).
    Large,
}

impl StoneSize {
    #[inline]
    #[must_use]
    fn config(self) -> (f32, f32, f32) {
        // (rate_per_sec, amplitude, decay_ms)
        match self {
            Self::Small => (30.0, 0.2, 5.0),
            Self::Medium => (15.0, 0.4, 10.0),
            Self::Large => (5.0, 0.7, 20.0),
        }
    }
}

/// Extended precipitation synthesizer.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Precipitation {
    precip_type: PrecipitationType,
    stone_size: StoneSize,
    surface: Terrain,
    sample_rate: f32,
    rng: Rng,
    dc_blocker: DcBlocker,
    sample_position: usize,
    // Params
    rate: f32,
    amplitude: f32,
    decay_samples: usize,
    // Real-time
    intensity: f32,
    // Modal bank for hail surface resonance
    modal_bank: Option<ModalBank>,
    #[cfg(feature = "naad-backend")]
    noise_gen: naad::noise::NoiseGenerator,
    #[cfg(feature = "naad-backend")]
    shape_filter: naad::filter::BiquadFilter,
}

impl Precipitation {
    /// Creates a new extended precipitation synthesizer.
    pub fn new(
        precip_type: PrecipitationType,
        stone_size: StoneSize,
        surface: Terrain,
        sample_rate: f32,
    ) -> Result<Self> {
        validate_sample_rate(sample_rate)?;
        let (rate, amplitude, decay_ms) = stone_size.config();
        let decay_samples = (decay_ms * sample_rate / 1000.0) as usize;

        // Modal bank for hail impacts on resonant surfaces
        let modal_bank = if precip_type == PrecipitationType::Hail {
            if let Some(mat) = surface.resonant_material() {
                let props = mat.properties();
                let mode_cfg = mat.mode_config();
                let specs = generate_modes(
                    &props,
                    mode_cfg.pattern,
                    mode_cfg.mode_count.min(4),
                    mode_cfg.damping_factor,
                );
                Some(ModalBank::new(&specs, sample_rate)?)
            } else {
                None
            }
        } else {
            None
        };

        #[cfg(feature = "naad-backend")]
        let (noise_gen, shape_filter) = {
            let (nt, filter_freq) = match precip_type {
                PrecipitationType::Hail => (naad::noise::NoiseType::White, 3000.0),
                PrecipitationType::Snow => (naad::noise::NoiseType::Brown, 1500.0),
                PrecipitationType::SurfaceRain => {
                    let cfg = surface.noise_config();
                    let nt = match cfg.noise_type {
                        crate::contact::NoisePreference::White => naad::noise::NoiseType::White,
                        crate::contact::NoisePreference::Pink => naad::noise::NoiseType::Pink,
                        crate::contact::NoisePreference::Brown => naad::noise::NoiseType::Brown,
                    };
                    (nt, cfg.filter_freq)
                }
            };
            let ng = naad::noise::NoiseGenerator::new(nt, 5050);
            let sf = naad::filter::BiquadFilter::new(
                naad::filter::FilterType::BandPass,
                sample_rate,
                filter_freq,
                1.5,
            )
            .map_err(|e| crate::error::GarjanError::SynthesisFailed(alloc::format!("{e}")))?;
            (ng, sf)
        };

        Ok(Self {
            precip_type,
            stone_size,
            surface,
            sample_rate,
            rng: Rng::new(5050),
            dc_blocker: DcBlocker::new(sample_rate),
            sample_position: 0,
            rate,
            amplitude,
            decay_samples,
            intensity: 1.0,
            modal_bank,
            #[cfg(feature = "naad-backend")]
            noise_gen,
            #[cfg(feature = "naad-backend")]
            shape_filter,
        })
    }

    /// Sets intensity (0.0–1.0).
    pub fn set_intensity(&mut self, intensity: f32) {
        self.intensity = intensity.clamp(0.0, 1.0);
    }

    /// Synthesizes precipitation audio.
    #[inline]
    pub fn synthesize(&mut self, duration: f32) -> Result<Vec<f32>> {
        crate::dsp::validate_duration(duration)?;
        let num_samples = (self.sample_rate * duration) as usize;
        let mut output = alloc::vec![0.0f32; num_samples];
        self.process_block(&mut output);
        Ok(output)
    }

    /// Fills output buffer with precipitation audio (streaming).
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
        for s in output.iter_mut() {
            *s = 0.0;
        }

        let effective_rate = self.rate * self.intensity;
        let block_size = (self.sample_rate * 0.01) as usize;
        let events_per_block = effective_rate * 0.01;

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

                let amp = self.amplitude * self.intensity * self.rng.next_f32_range(0.3, 1.0);
                let this_decay =
                    (self.rng.next_f32_range(0.5, 1.5) * self.decay_samples as f32) as usize;

                for j in 0..this_decay.min(num_samples - idx) {
                    let t = j as f32 / this_decay as f32;
                    let env = crate::math::f32::exp(-5.0 * t);
                    let noise = self.generate_noise();

                    let sample = if let Some(ref mut bank) = self.modal_bank {
                        let exc = noise * env * amp;
                        bank.process_sample(exc)
                    } else {
                        noise * env * amp
                    };

                    output[idx + j] += sample;
                }
            }
        }

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
