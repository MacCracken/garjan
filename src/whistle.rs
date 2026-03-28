//! Wind whistle synthesis: air through gaps, pipes, bottles, wires.
//!
//! Models the tonal sound of wind passing through or over openings.
//! Each source type has a characteristic resonant frequency and bandwidth.
//! Wind speed controls amplitude and adds slight pitch modulation.

use alloc::vec::Vec;
use serde::{Deserialize, Serialize};

use crate::aero::WhistleSource;
use crate::dsp::{DcBlocker, validate_sample_rate};
use crate::error::Result;
use crate::rng::Rng;

/// Wind whistle synthesizer — tonal air resonance.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Whistle {
    source: WhistleSource,
    sample_rate: f32,
    rng: Rng,
    dc_blocker: DcBlocker,
    sample_position: usize,
    // Source params
    base_freq: f32,
    bandwidth: f32,
    noise_mix: f32,
    // Real-time
    wind_speed: f32,
    #[cfg(feature = "naad-backend")]
    noise_gen: naad::noise::NoiseGenerator,
    #[cfg(feature = "naad-backend")]
    resonance_filter: naad::filter::StateVariableFilter,
    #[cfg(feature = "naad-backend")]
    pitch_lfo: naad::modulation::Lfo,
}

impl Whistle {
    /// Creates a new wind whistle synthesizer.
    pub fn new(source: WhistleSource, sample_rate: f32) -> Result<Self> {
        validate_sample_rate(sample_rate)?;
        let (base_freq, bandwidth, noise_mix) = source.config();

        #[cfg(feature = "naad-backend")]
        let noise_gen = naad::noise::NoiseGenerator::new(naad::noise::NoiseType::White, 2222);
        #[cfg(feature = "naad-backend")]
        let resonance_filter = {
            let q = (base_freq / bandwidth.max(1.0)).clamp(0.5, 50.0);
            naad::filter::StateVariableFilter::new(base_freq, q, sample_rate)
                .map_err(|e| crate::error::GarjanError::SynthesisFailed(alloc::format!("{e}")))?
        };
        #[cfg(feature = "naad-backend")]
        let pitch_lfo =
            naad::modulation::Lfo::new(naad::modulation::LfoShape::Sine, 3.0, sample_rate)
                .map_err(|e| crate::error::GarjanError::SynthesisFailed(alloc::format!("{e}")))?;

        Ok(Self {
            source,
            sample_rate,
            rng: Rng::new(2222),
            dc_blocker: DcBlocker::new(sample_rate),
            sample_position: 0,
            base_freq,
            bandwidth,
            noise_mix,
            wind_speed: 0.0,
            #[cfg(feature = "naad-backend")]
            noise_gen,
            #[cfg(feature = "naad-backend")]
            resonance_filter,
            #[cfg(feature = "naad-backend")]
            pitch_lfo,
        })
    }

    /// Sets the wind speed (0.0 = calm/silent, 1.0 = strong).
    pub fn set_wind_speed(&mut self, speed: f32) {
        self.wind_speed = speed.clamp(0.0, 1.0);
    }

    /// Synthesizes wind whistle audio.
    #[inline]
    pub fn synthesize(&mut self, duration: f32) -> Result<Vec<f32>> {
        let num_samples = (self.sample_rate * duration) as usize;
        let mut output = alloc::vec![0.0f32; num_samples];
        self.process_block(&mut output);
        Ok(output)
    }

    /// Fills output buffer with whistle audio (streaming).
    #[inline]
    pub fn process_block(&mut self, output: &mut [f32]) {
        #[cfg(feature = "naad-backend")]
        for sample in output.iter_mut() {
            if self.wind_speed < 0.001 {
                *sample = 0.0;
                self.dc_blocker.process(0.0);
                continue;
            }

            let wobble = self.pitch_lfo.next_value() * self.wind_speed * 0.05;
            let modulated_freq = self.base_freq * (1.0 + wobble);
            let _ = self.resonance_filter.set_params(
                modulated_freq.clamp(20.0, self.sample_rate * 0.45),
                (self.base_freq / self.bandwidth.max(1.0)).clamp(0.5, 50.0),
            );

            let noise = self.noise_gen.next_sample();
            let resonant = self.resonance_filter.process_sample(noise).band_pass;

            let broad_noise = noise * self.noise_mix * self.wind_speed * 0.1;
            *sample = resonant * self.wind_speed * 0.4 + broad_noise;
            *sample = self.dc_blocker.process(*sample);
        }
        #[cfg(not(feature = "naad-backend"))]
        for (i, sample) in output.iter_mut().enumerate() {
            if self.wind_speed < 0.001 {
                *sample = 0.0;
                self.dc_blocker.process(0.0);
                continue;
            }

            let t = (self.sample_position + i) as f32 / self.sample_rate;
            let tone = crate::math::f32::sin(core::f32::consts::TAU * self.base_freq * t)
                * self.wind_speed
                * 0.3;
            let noise = self.rng.next_f32() * self.noise_mix * self.wind_speed * 0.1;
            *sample = tone + noise;

            *sample = self.dc_blocker.process(*sample);
        }
        self.sample_position += output.len();
    }
}
