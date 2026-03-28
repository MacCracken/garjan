//! Insect sound synthesis: wing buzz, chirping, cicada drone.
//!
//! Models insect sounds as physical mechanisms:
//! - Wing buzz: amplitude-modulated tone from rapid wing vibration
//! - Cricket chirp: stridulation (friction pulse train) with silence gaps
//! - Cicada drone: sustained broadband rattle with slow amplitude modulation
//!
//! Swarm mode layers multiple detuned instances for collective sound.

use alloc::vec::Vec;
use serde::{Deserialize, Serialize};

use crate::creature::InsectType;
use crate::dsp::{DcBlocker, validate_sample_rate};
use crate::error::Result;
use crate::rng::Rng;

/// Insect sound synthesizer.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Insect {
    insect_type: InsectType,
    sample_rate: f32,
    rng: Rng,
    dc_blocker: DcBlocker,
    sample_position: usize,
    // Type params
    base_freq: f32,
    mod_rate: f32,
    chirp_rate: f32,
    amplitude: f32,
    // Real-time
    intensity: f32,
    // Swarm
    swarm_count: usize,
    swarm_detunings: [f32; 8],
    #[cfg(feature = "naad-backend")]
    noise_gen: naad::noise::NoiseGenerator,
    #[cfg(feature = "naad-backend")]
    shape_filter: naad::filter::BiquadFilter,
}

impl Insect {
    /// Creates a new single insect synthesizer.
    pub fn new(insect_type: InsectType, sample_rate: f32) -> Result<Self> {
        validate_sample_rate(sample_rate)?;
        let (base_freq, mod_rate, chirp_rate, amplitude) = insect_type.config();

        // Pre-compute swarm detunings (used when swarm_count > 1)
        let mut det_rng = Rng::new(12321);
        let mut swarm_detunings = [0.0f32; 8];
        for d in &mut swarm_detunings {
            *d = det_rng.next_f32_range(-0.08, 0.08); // +/- 8% detuning
        }

        #[cfg(feature = "naad-backend")]
        let noise_gen = naad::noise::NoiseGenerator::new(naad::noise::NoiseType::White, 9999);
        #[cfg(feature = "naad-backend")]
        let shape_filter = naad::filter::BiquadFilter::new(
            naad::filter::FilterType::BandPass,
            sample_rate,
            base_freq,
            (base_freq / 200.0).clamp(1.0, 20.0),
        )
        .map_err(|e| crate::error::GarjanError::SynthesisFailed(alloc::format!("{e}")))?;

        Ok(Self {
            insect_type,
            sample_rate,
            rng: Rng::new(9999),
            dc_blocker: DcBlocker::new(sample_rate),
            sample_position: 0,
            base_freq,
            mod_rate,
            chirp_rate,
            amplitude,
            intensity: 1.0,
            swarm_count: 1,
            swarm_detunings,
            #[cfg(feature = "naad-backend")]
            noise_gen,
            #[cfg(feature = "naad-backend")]
            shape_filter,
        })
    }

    /// Creates a swarm of insects (1–8 overlapping instances with detuning).
    pub fn new_swarm(insect_type: InsectType, count: usize, sample_rate: f32) -> Result<Self> {
        let mut s = Self::new(insect_type, sample_rate)?;
        s.swarm_count = count.clamp(1, 8);
        Ok(s)
    }

    /// Sets the intensity (0.0 = silent, 1.0 = full volume).
    pub fn set_intensity(&mut self, intensity: f32) {
        self.intensity = intensity.clamp(0.0, 1.0);
    }

    /// Synthesizes insect audio.
    #[inline]
    pub fn synthesize(&mut self, duration: f32) -> Result<Vec<f32>> {
        crate::dsp::validate_duration(duration)?;
        let num_samples = (self.sample_rate * duration) as usize;
        let mut output = alloc::vec![0.0f32; num_samples];
        self.process_block(&mut output);
        Ok(output)
    }

    /// Fills output buffer with insect audio (streaming).
    #[inline]
    pub fn process_block(&mut self, output: &mut [f32]) {
        if self.intensity < 0.001 {
            for sample in output.iter_mut() {
                *sample = 0.0;
                self.dc_blocker.process(0.0);
            }
            self.sample_position += output.len();
            return;
        }

        let inv_swarm = 1.0 / self.swarm_count as f32;
        for (i, sample) in output.iter_mut().enumerate() {
            let t = (self.sample_position + i) as f32 / self.sample_rate;
            let mut out = 0.0f32;

            for voice in 0..self.swarm_count {
                let detune = 1.0 + self.swarm_detunings[voice];
                let freq = self.base_freq * detune;
                out += self.synthesize_voice(t, freq, voice);
            }

            *sample = out * self.amplitude * self.intensity * inv_swarm;
            *sample = self.dc_blocker.process(*sample);
        }
        self.sample_position += output.len();
    }

    #[inline]
    fn synthesize_voice(&mut self, t: f32, freq: f32, voice: usize) -> f32 {
        let phase_offset = voice as f32 * 0.37; // arbitrary phase spread

        match self.insect_type {
            InsectType::WingBuzz => {
                // AM tone: carrier * (0.5 + 0.5 * sin(mod_rate * t))
                let carrier =
                    crate::math::f32::sin(core::f32::consts::TAU * freq * t + phase_offset);
                let modulator = 0.5
                    + 0.5
                        * crate::math::f32::sin(
                            core::f32::consts::TAU * self.mod_rate * t + phase_offset * 2.0,
                        );
                carrier * modulator
            }
            InsectType::CricketChirp => {
                // Chirp pattern: bursts of pulses separated by silence
                let chirp_phase = (self.chirp_rate * t + phase_offset * 0.1) % 1.0;
                if chirp_phase < 0.4 {
                    // Active chirp: pulse train
                    let pulse = crate::math::f32::sin(
                        core::f32::consts::TAU * self.mod_rate * t + phase_offset,
                    );
                    let carrier =
                        crate::math::f32::sin(core::f32::consts::TAU * freq * t + phase_offset);
                    // Gate the carrier with the pulse envelope
                    let gate = if pulse > 0.0 { pulse } else { 0.0 };
                    carrier * gate
                } else {
                    0.0 // silence between chirps
                }
            }
            InsectType::CicadaDrone => {
                // Broadband rattle: filtered noise with slow AM
                let modulator = 0.6
                    + 0.4
                        * crate::math::f32::sin(
                            core::f32::consts::TAU * self.mod_rate * t + phase_offset,
                        );
                #[cfg(feature = "naad-backend")]
                {
                    let noise = self.noise_gen.next_sample();
                    self.shape_filter.process_sample(noise) * modulator
                }
                #[cfg(not(feature = "naad-backend"))]
                {
                    self.rng.next_f32() * modulator
                }
            }
        }
    }
}
