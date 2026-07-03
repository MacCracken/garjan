//! Whoosh synthesis: object pass-by and swing sounds.
//!
//! Models the aerodynamic noise of objects moving quickly through air.
//! Speed controls brightness (faster = more high-frequency content)
//! and envelope duration. Size controls low-frequency rumble.

use alloc::vec::Vec;
use serde::{Deserialize, Serialize};

use crate::aero::WhooshType;
use crate::dsp::{DcBlocker, validate_sample_rate};
use crate::error::Result;
use crate::rng::Rng;

/// Whoosh sound synthesizer — aerodynamic pass-by / swing.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Whoosh {
    whoosh_type: WhooshType,
    sample_rate: f32,
    rng: Rng,
    dc_blocker: DcBlocker,
    sample_position: usize,
    // Envelope params
    envelope_samples: usize,
    brightness: f32,
    low_content: f32,
    // Trigger state
    active: bool,
    trigger_position: usize,
    // Real-time
    speed: f32,
    #[cfg(feature = "naad-backend")]
    noise_gen: naad::noise::NoiseGenerator,
    #[cfg(feature = "naad-backend")]
    hi_filter: naad::filter::BiquadFilter,
    #[cfg(feature = "naad-backend")]
    lo_filter: naad::filter::BiquadFilter,
}

impl Whoosh {
    /// Creates a new whoosh synthesizer.
    pub fn new(whoosh_type: WhooshType, sample_rate: f32) -> Result<Self> {
        validate_sample_rate(sample_rate)?;
        let (env_dur, brightness, low_content) = whoosh_type.config();
        let envelope_samples = (env_dur * sample_rate) as usize;

        #[cfg(feature = "naad-backend")]
        let noise_gen = naad::noise::NoiseGenerator::new(naad::noise::NoiseType::Pink, 1111);
        #[cfg(feature = "naad-backend")]
        let hi_filter = naad::filter::BiquadFilter::new(
            naad::filter::FilterType::HighPass,
            sample_rate,
            (brightness * 4000.0).clamp(200.0, 8000.0),
            0.7,
        )
        .map_err(|e| crate::error::GarjanError::SynthesisFailed(alloc::format!("{e}")))?;
        #[cfg(feature = "naad-backend")]
        let lo_filter = naad::filter::BiquadFilter::new(
            naad::filter::FilterType::LowPass,
            sample_rate,
            (300.0 + low_content * 500.0).clamp(100.0, 1000.0),
            0.5,
        )
        .map_err(|e| crate::error::GarjanError::SynthesisFailed(alloc::format!("{e}")))?;

        Ok(Self {
            whoosh_type,
            sample_rate,
            rng: Rng::new(1111),
            dc_blocker: DcBlocker::new(sample_rate),
            sample_position: 0,
            envelope_samples,
            brightness,
            low_content,
            active: false,
            trigger_position: 0,
            speed: 0.5,
            #[cfg(feature = "naad-backend")]
            noise_gen,
            #[cfg(feature = "naad-backend")]
            hi_filter,
            #[cfg(feature = "naad-backend")]
            lo_filter,
        })
    }

    /// Sets the speed (0.0–1.0). Higher speed = brighter and shorter.
    pub fn set_speed(&mut self, speed: f32) {
        self.speed = speed.clamp(0.0, 1.0);
    }

    /// Triggers a whoosh event.
    pub fn trigger(&mut self) {
        self.active = true;
        self.trigger_position = 0;
    }

    /// Synthesizes a single whoosh event.
    #[inline]
    pub fn synthesize(&mut self, duration: f32) -> Result<Vec<f32>> {
        crate::dsp::validate_duration(duration)?;
        let num_samples = (self.sample_rate * duration) as usize;
        let mut output = alloc::vec![0.0f32; num_samples];
        self.trigger();
        self.process_block(&mut output);
        Ok(output)
    }

    /// Fills output buffer with whoosh audio (streaming).
    #[inline]
    pub fn process_block(&mut self, output: &mut [f32]) {
        let speed_scale = 0.5 + self.speed * 0.5;
        let effective_len = (self.envelope_samples as f32 / speed_scale) as usize;

        for sample in output.iter_mut() {
            if !self.active {
                *sample = 0.0;
                self.dc_blocker.process(0.0);
                continue;
            }

            // Hump-shaped envelope: sin^2 over the duration
            let t = self.trigger_position as f32 / effective_len.max(1) as f32;
            let env = if t < 1.0 {
                let s = crate::math::f32::sin(core::f32::consts::PI * t);
                s * s
            } else {
                self.active = false;
                0.0
            };

            // High-frequency whoosh layer
            let hi = self.generate_noise_hi() * env * self.brightness;

            // Low-frequency rumble layer
            let lo = self.generate_noise_lo() * env * self.low_content;

            *sample = (hi + lo) * self.speed;
            *sample = self.dc_blocker.process(*sample);

            self.trigger_position += 1;
        }
        self.sample_position += output.len();
    }

    #[inline]
    fn generate_noise_hi(&mut self) -> f32 {
        #[cfg(feature = "naad-backend")]
        {
            let raw = self.noise_gen.next_sample();
            self.hi_filter.process_sample(raw)
        }
        #[cfg(not(feature = "naad-backend"))]
        {
            self.rng.next_f32()
        }
    }

    #[inline]
    fn generate_noise_lo(&mut self) -> f32 {
        #[cfg(feature = "naad-backend")]
        {
            let raw = self.noise_gen.next_sample();
            self.lo_filter.process_sample(raw)
        }
        #[cfg(not(feature = "naad-backend"))]
        {
            (self.rng.next_f32() + self.rng.next_f32()) * 0.5
        }
    }
}
