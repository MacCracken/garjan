//! Foliage synthesis: leaf rustle, grass swish, branch snap.
//!
//! Models vegetation sounds as a combination of continuous noise
//! (wind-driven rustle) and stochastic micro-events (individual
//! leaf/twig contacts).

use alloc::vec::Vec;
use serde::{Deserialize, Serialize};

use crate::contact::FoliageType;
use crate::dsp::{DcBlocker, validate_sample_rate};
use crate::error::Result;
use crate::material::Material;
use crate::modal::{ExcitationType, Exciter, ModalBank, generate_modes};
use crate::rng::Rng;

/// Foliage sound synthesizer — vegetation contact sounds.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Foliage {
    foliage_type: FoliageType,
    sample_rate: f32,
    rng: Rng,
    dc_blocker: DcBlocker,
    sample_position: usize,
    // Real-time parameters
    wind_speed: f32,
    contact_intensity: f32,
    // Branch snap (one-shot)
    snap_modal: Option<ModalBank>,
    snap_exciter: Exciter,
    snap_pending: bool,
    // Event parameters
    event_rate: f32,
    event_duration_samples: usize,
    #[cfg(feature = "naad-backend")]
    rustle_noise: naad::noise::NoiseGenerator,
    #[cfg(feature = "naad-backend")]
    rustle_filter: naad::filter::BiquadFilter,
    #[cfg(feature = "naad-backend")]
    wind_lfo: naad::modulation::Lfo,
}

impl Foliage {
    /// Creates a new foliage synthesizer.
    pub fn new(foliage_type: FoliageType, sample_rate: f32) -> Result<Self> {
        validate_sample_rate(sample_rate)?;

        let (event_rate, event_dur_ms) = match foliage_type {
            FoliageType::LeafRustle => (20.0, 2.0),
            FoliageType::GrassSwish => (12.0, 10.0),
            FoliageType::BranchSnap => (0.0, 0.0),
        };
        let event_duration_samples = (event_dur_ms * sample_rate / 1000.0) as usize;

        // Branch snap modal bank
        let snap_modal = if foliage_type == FoliageType::BranchSnap {
            let props = Material::Wood.properties();
            let mode_cfg = Material::Wood.mode_config();
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

        let snap_exciter = Exciter::new(
            ExcitationType::NoiseBurst {
                duration_samples: (sample_rate * 0.002) as usize,
            },
            0.8,
        );

        #[cfg(feature = "naad-backend")]
        let rustle_noise = naad::noise::NoiseGenerator::new(naad::noise::NoiseType::Pink, 4444);
        #[cfg(feature = "naad-backend")]
        let rustle_filter = {
            let freq = match foliage_type {
                FoliageType::LeafRustle => 4000.0,
                FoliageType::GrassSwish => 2000.0,
                FoliageType::BranchSnap => 1000.0,
            };
            naad::filter::BiquadFilter::new(
                naad::filter::FilterType::BandPass,
                sample_rate,
                freq,
                1.5,
            )
            .map_err(|e| crate::error::GarjanError::SynthesisFailed(alloc::format!("{e}")))?
        };
        #[cfg(feature = "naad-backend")]
        let wind_lfo = {
            let rate = match foliage_type {
                FoliageType::LeafRustle => 0.2,
                FoliageType::GrassSwish => 0.15,
                FoliageType::BranchSnap => 0.1,
            };
            naad::modulation::Lfo::new(naad::modulation::LfoShape::Sine, rate, sample_rate)
                .map_err(|e| crate::error::GarjanError::SynthesisFailed(alloc::format!("{e}")))?
        };

        Ok(Self {
            foliage_type,
            sample_rate,
            rng: Rng::new(4444),
            dc_blocker: DcBlocker::new(sample_rate),
            sample_position: 0,
            wind_speed: 0.0,
            contact_intensity: 0.0,
            snap_modal,
            snap_exciter,
            snap_pending: false,
            event_rate,
            event_duration_samples,
            #[cfg(feature = "naad-backend")]
            rustle_noise,
            #[cfg(feature = "naad-backend")]
            rustle_filter,
            #[cfg(feature = "naad-backend")]
            wind_lfo,
        })
    }

    /// Sets the wind speed (0.0 = calm, 1.0 = strong wind).
    pub fn set_wind_speed(&mut self, speed: f32) {
        self.wind_speed = speed.clamp(0.0, 1.0);
    }

    /// Sets the contact intensity (0.0 = no contact, 1.0 = heavy movement through foliage).
    pub fn set_contact_intensity(&mut self, intensity: f32) {
        self.contact_intensity = intensity.clamp(0.0, 1.0);
    }

    /// Triggers a branch snap (one-shot). Only has effect for `BranchSnap` type.
    pub fn trigger_snap(&mut self) {
        if self.foliage_type == FoliageType::BranchSnap {
            self.snap_pending = true;
        }
    }

    /// Synthesizes foliage audio.
    #[inline]
    pub fn synthesize(&mut self, duration: f32) -> Result<Vec<f32>> {
        let num_samples = (self.sample_rate * duration) as usize;
        let mut output = alloc::vec![0.0f32; num_samples];
        self.process_block(&mut output);
        Ok(output)
    }

    /// Fills output buffer with foliage audio (streaming).
    #[inline]
    pub fn process_block(&mut self, output: &mut [f32]) {
        if self.foliage_type == FoliageType::BranchSnap {
            self.process_branch_snap(output);
        } else {
            self.process_rustle(output);
        }
        for sample in output.iter_mut() {
            *sample = self.dc_blocker.process(*sample);
        }
        self.sample_position += output.len();
    }

    #[inline]
    fn process_rustle(&mut self, output: &mut [f32]) {
        let total_rate =
            self.wind_speed * self.event_rate + self.contact_intensity * self.event_rate * 2.0;
        let block_size = (self.sample_rate * 0.01) as usize;
        let events_per_block = total_rate * 0.01;
        let num_samples = output.len();

        // Continuous rustle bed
        for sample in output.iter_mut() {
            let wind_amp = self.wind_speed * 0.15;
            #[cfg(feature = "naad-backend")]
            {
                let noise = self.rustle_noise.next_sample();
                let filtered = self.rustle_filter.process_sample(noise);
                let lfo = 0.7 + 0.3 * self.wind_lfo.next_value();
                *sample = filtered * wind_amp * lfo;
            }
            #[cfg(not(feature = "naad-backend"))]
            {
                *sample = self.rng.next_f32() * wind_amp;
            }
        }

        // Stochastic micro-events
        if total_rate > 0.0 && self.event_duration_samples > 0 {
            for block_start in (0..num_samples).step_by(block_size.max(1)) {
                let block_end = (block_start + block_size).min(num_samples);
                let n_events = self.rng.poisson(events_per_block);
                for _ in 0..n_events {
                    let offset = self
                        .rng
                        .next_f32_range(0.0, (block_end - block_start) as f32)
                        as usize;
                    let idx = block_start + offset;
                    if idx < num_samples {
                        let amp = self.rng.next_f32_range(0.05, 0.15);
                        for j in 0..self.event_duration_samples.min(num_samples - idx) {
                            let env = 1.0 - (j as f32 / self.event_duration_samples as f32);
                            output[idx + j] += amp * env * self.rng.next_f32();
                        }
                    }
                }
            }
        }
    }

    #[inline]
    fn process_branch_snap(&mut self, output: &mut [f32]) {
        if self.snap_pending {
            self.snap_pending = false;
            self.snap_exciter.trigger();
            if let Some(ref mut bank) = self.snap_modal {
                bank.reset();
            }
        }

        for sample in output.iter_mut() {
            let exc = self.snap_exciter.next_sample();
            *sample = if let Some(ref mut bank) = self.snap_modal {
                bank.process_sample(exc)
            } else {
                exc * 0.5
            };
        }
    }
}
