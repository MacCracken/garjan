//! Weather sound synthesis: rain, thunder, wind.
//!
//! Models atmospheric phenomena as physical processes:
//! - Rain: stochastic particle impacts at Poisson-distributed intervals
//! - Thunder: low-frequency impulse with exponential decay and rumble
//! - Wind: filtered noise with amplitude and spectral modulation

use alloc::vec::Vec;
use serde::{Deserialize, Serialize};

use crate::dsp::{DcBlocker, validate_sample_rate};
use crate::error::Result;
use crate::rng::Rng;

// ---------------------------------------------------------------------------
// Rain
// ---------------------------------------------------------------------------

/// Rain intensity levels.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[non_exhaustive]
pub enum RainIntensity {
    /// Light drizzle — sparse, quiet drops.
    Light,
    /// Moderate rainfall — steady, even.
    Moderate,
    /// Heavy rain — dense, loud.
    Heavy,
    /// Downpour — extremely dense, roaring.
    Torrential,
}

impl RainIntensity {
    /// Returns drops per second for this intensity.
    #[inline]
    #[must_use]
    fn drops_per_second(self) -> f32 {
        match self {
            Self::Light => 20.0,
            Self::Moderate => 80.0,
            Self::Heavy => 250.0,
            Self::Torrential => 600.0,
        }
    }

    /// Returns amplitude scaling for this intensity.
    #[inline]
    #[must_use]
    fn amplitude(self) -> f32 {
        match self {
            Self::Light => 0.15,
            Self::Moderate => 0.3,
            Self::Heavy => 0.5,
            Self::Torrential => 0.8,
        }
    }
}

/// Rain synthesizer — stochastic raindrop impacts.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Rain {
    intensity: RainIntensity,
    sample_rate: f32,
    rng: Rng,
    dc_blocker: DcBlocker,
    sample_position: usize,
    #[cfg(feature = "naad-backend")]
    drop_noise: naad::noise::NoiseGenerator,
    #[cfg(feature = "naad-backend")]
    rain_filter: naad::filter::BiquadFilter,
}

impl Rain {
    /// Creates a new rain synthesizer.
    pub fn new(intensity: RainIntensity, sample_rate: f32) -> Result<Self> {
        validate_sample_rate(sample_rate)?;
        Ok(Self {
            intensity,
            sample_rate,
            rng: Rng::new(7919),
            dc_blocker: DcBlocker::new(sample_rate),
            sample_position: 0,
            #[cfg(feature = "naad-backend")]
            drop_noise: naad::noise::NoiseGenerator::new(naad::noise::NoiseType::White, 7919),
            #[cfg(feature = "naad-backend")]
            rain_filter: naad::filter::BiquadFilter::new(
                naad::filter::FilterType::BandPass,
                sample_rate,
                3000.0,
                2.0,
            )
            .map_err(|e| crate::error::GarjanError::SynthesisFailed(alloc::format!("{e}")))?,
        })
    }

    /// Synthesizes rain audio.
    #[inline]
    pub fn synthesize(&mut self, duration: f32) -> Result<Vec<f32>> {
        crate::dsp::validate_duration(duration)?;
        let num_samples = (self.sample_rate * duration) as usize;
        let mut output = alloc::vec![0.0f32; num_samples];
        self.process_block(&mut output);
        Ok(output)
    }

    /// Fills output buffer with rain audio (streaming).
    #[inline]
    pub fn process_block(&mut self, output: &mut [f32]) {
        // Zero the buffer
        for s in output.iter_mut() {
            *s = 0.0;
        }

        let amp = self.intensity.amplitude();
        let block_size = (self.sample_rate * 0.01) as usize;
        let drops_per_block = self.intensity.drops_per_second() * 0.01;
        let num_samples = output.len();

        for block_start in (0..num_samples).step_by(block_size.max(1)) {
            let block_end = (block_start + block_size).min(num_samples);
            let n_drops = self.rng.poisson(drops_per_block);

            for _ in 0..n_drops {
                let offset = self
                    .rng
                    .next_f32_range(0.0, (block_end - block_start) as f32)
                    as usize;
                let idx = block_start + offset;
                if idx < num_samples {
                    let drop_amp = amp * self.rng.next_f32_range(0.3, 1.0);
                    let decay_samples = self.rng.next_f32_range(20.0, 80.0) as usize;
                    for j in 0..decay_samples.min(num_samples - idx) {
                        let t = j as f32 / decay_samples as f32;
                        let env = crate::math::f32::exp(-5.0 * t);
                        #[cfg(feature = "naad-backend")]
                        {
                            output[idx + j] += drop_amp * env * self.drop_noise.next_sample();
                        }
                        #[cfg(not(feature = "naad-backend"))]
                        {
                            output[idx + j] += drop_amp * env * self.rng.next_f32();
                        }
                    }
                }
            }
        }

        // Apply filter and DC blocking
        #[cfg(feature = "naad-backend")]
        self.rain_filter.process_buffer(output);

        for sample in output.iter_mut() {
            *sample = self.dc_blocker.process(*sample);
        }
        self.sample_position += output.len();
    }
}

// ---------------------------------------------------------------------------
// Thunder
// ---------------------------------------------------------------------------

/// Thunder synthesizer — low-frequency impulse with rumble.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Thunder {
    /// Distance to lightning strike in meters.
    distance_m: f32,
    sample_rate: f32,
    rng: Rng,
    dc_blocker: DcBlocker,
    sample_position: usize,
    #[cfg(feature = "naad-backend")]
    crack_noise: naad::noise::NoiseGenerator,
    #[cfg(feature = "naad-backend")]
    rumble_noise: naad::noise::NoiseGenerator,
    #[cfg(feature = "naad-backend")]
    rumble_filter: naad::filter::BiquadFilter,
}

impl Thunder {
    /// Creates a new thunder synthesizer.
    ///
    /// `distance_m` controls the delay and spectral character:
    /// - Close (< 500m): sharp crack + rumble
    /// - Medium (500-3000m): rolling rumble
    /// - Far (> 3000m): distant low rumble
    pub fn new(distance_m: f32, sample_rate: f32) -> Result<Self> {
        validate_sample_rate(sample_rate)?;
        let distance_m = distance_m.max(10.0);
        #[cfg(feature = "naad-backend")]
        let rumble_filter = {
            let cutoff = (5000.0 / (1.0 + distance_m * 0.002)).clamp(60.0, 5000.0);
            naad::filter::BiquadFilter::new(
                naad::filter::FilterType::LowPass,
                sample_rate,
                cutoff,
                0.7,
            )
            .map_err(|e| crate::error::GarjanError::SynthesisFailed(alloc::format!("{e}")))?
        };
        Ok(Self {
            distance_m,
            sample_rate,
            rng: Rng::new(1337),
            dc_blocker: DcBlocker::new(sample_rate),
            sample_position: 0,
            #[cfg(feature = "naad-backend")]
            crack_noise: naad::noise::NoiseGenerator::new(naad::noise::NoiseType::White, 1337),
            #[cfg(feature = "naad-backend")]
            rumble_noise: naad::noise::NoiseGenerator::new(naad::noise::NoiseType::Brown, 1338),
            #[cfg(feature = "naad-backend")]
            rumble_filter,
        })
    }

    /// Synthesizes a thunderclap.
    #[inline]
    pub fn synthesize(&mut self, duration: f32) -> Result<Vec<f32>> {
        crate::dsp::validate_duration(duration)?;
        let num_samples = (self.sample_rate * duration) as usize;
        let mut output = alloc::vec![0.0f32; num_samples];
        self.process_block(&mut output);
        Ok(output)
    }

    /// Fills output buffer with thunder audio (streaming).
    #[inline]
    pub fn process_block(&mut self, output: &mut [f32]) {
        #[cfg(feature = "naad-backend")]
        self.process_block_naad(output);
        #[cfg(not(feature = "naad-backend"))]
        self.process_block_fallback(output);

        for sample in output.iter_mut() {
            *sample = self.dc_blocker.process(*sample);
        }
        self.sample_position += output.len();
    }

    #[cfg(feature = "naad-backend")]
    fn process_block_naad(&mut self, output: &mut [f32]) {
        let delay_s = self.distance_m / 343.0;
        let delay_samples = (delay_s * self.sample_rate) as usize;
        for (i, sample) in output.iter_mut().enumerate() {
            let abs_pos = self.sample_position + i;
            *sample = 0.0;

            if abs_pos < delay_samples {
                continue;
            }

            let pos_after_delay = abs_pos - delay_samples;
            let amp = (100.0 / self.distance_m).min(1.0);

            // Crack (close thunder only)
            let crack_len = if self.distance_m < 500.0 {
                (self.sample_rate * 0.05) as usize
            } else {
                0
            };

            if pos_after_delay < crack_len {
                let t = pos_after_delay as f32 / crack_len.max(1) as f32;
                let env = crate::math::f32::exp(-10.0 * t);
                *sample += amp * env * self.crack_noise.next_sample() * 0.8;
            } else {
                // Rumble
                let rumble_pos = pos_after_delay - crack_len;
                let decay_time = 0.5 + self.distance_m * 0.001;
                let t = rumble_pos as f32 / (self.sample_rate * decay_time);
                let env = crate::math::f32::exp(-2.0 * t);
                let noise = self.rumble_noise.next_sample();
                let filtered = self.rumble_filter.process_sample(noise);
                *sample += amp * env * filtered * 0.6;
            }
        }
    }

    #[cfg(not(feature = "naad-backend"))]
    fn process_block_fallback(&mut self, output: &mut [f32]) {
        let delay_s = self.distance_m / 343.0;
        let delay_samples = (delay_s * self.sample_rate) as usize;

        for (i, sample) in output.iter_mut().enumerate() {
            let abs_pos = self.sample_position + i;
            *sample = 0.0;

            if abs_pos < delay_samples {
                continue;
            }

            let pos_after_delay = abs_pos - delay_samples;
            let amp = (100.0 / self.distance_m).min(1.0);

            let crack_len = if self.distance_m < 500.0 {
                (self.sample_rate * 0.05) as usize
            } else {
                0
            };

            if pos_after_delay < crack_len {
                let t = pos_after_delay as f32 / crack_len.max(1) as f32;
                let env = crate::math::f32::exp(-10.0 * t);
                *sample += amp * env * self.rng.next_f32() * 0.8;
            } else {
                let rumble_pos = pos_after_delay - crack_len;
                let decay_time = 0.5 + self.distance_m * 0.001;
                let t = rumble_pos as f32 / (self.sample_rate * decay_time);
                let env = crate::math::f32::exp(-2.0 * t);
                let noise = (self.rng.next_f32() + self.rng.next_f32() + self.rng.next_f32()) / 3.0;
                *sample += amp * env * noise * 0.6;
            }
        }
    }
}

// ---------------------------------------------------------------------------
// Wind
// ---------------------------------------------------------------------------

/// Wind synthesizer — filtered noise with modulation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Wind {
    /// Wind speed in m/s (0 = calm, 30+ = storm).
    speed: f32,
    /// Gustiness (0.0 = steady, 1.0 = very gusty).
    gustiness: f32,
    sample_rate: f32,
    rng: Rng,
    dc_blocker: DcBlocker,
    sample_position: usize,
    #[cfg(feature = "naad-backend")]
    noise_gen: naad::noise::NoiseGenerator,
    #[cfg(feature = "naad-backend")]
    shape_filter: naad::filter::StateVariableFilter,
    #[cfg(feature = "naad-backend")]
    gust_lfo: naad::modulation::Lfo,
}

impl Wind {
    /// Creates a new wind synthesizer.
    pub fn new(speed: f32, gustiness: f32, sample_rate: f32) -> Result<Self> {
        validate_sample_rate(sample_rate)?;
        let speed = speed.max(0.0);
        let gustiness = gustiness.clamp(0.0, 1.0);
        #[cfg(feature = "naad-backend")]
        let shape_filter = {
            let cutoff = ((speed / 30.0).clamp(0.05, 1.0) * 8000.0).clamp(50.0, 8000.0);
            naad::filter::StateVariableFilter::new(cutoff, 0.5, sample_rate)
                .map_err(|e| crate::error::GarjanError::SynthesisFailed(alloc::format!("{e}")))?
        };
        #[cfg(feature = "naad-backend")]
        let gust_lfo = {
            let gust_rate = 0.3 + gustiness * 2.0;
            naad::modulation::Lfo::new(naad::modulation::LfoShape::Sine, gust_rate, sample_rate)
                .map_err(|e| crate::error::GarjanError::SynthesisFailed(alloc::format!("{e}")))?
        };
        Ok(Self {
            speed,
            gustiness,
            sample_rate,
            rng: Rng::new(4201),
            dc_blocker: DcBlocker::new(sample_rate),
            sample_position: 0,
            #[cfg(feature = "naad-backend")]
            noise_gen: naad::noise::NoiseGenerator::new(naad::noise::NoiseType::Pink, 4201),
            #[cfg(feature = "naad-backend")]
            shape_filter,
            #[cfg(feature = "naad-backend")]
            gust_lfo,
        })
    }

    /// Synthesizes wind audio.
    #[inline]
    pub fn synthesize(&mut self, duration: f32) -> Result<Vec<f32>> {
        crate::dsp::validate_duration(duration)?;
        let num_samples = (self.sample_rate * duration) as usize;
        let mut output = alloc::vec![0.0f32; num_samples];
        self.process_block(&mut output);
        Ok(output)
    }

    /// Fills output buffer with wind audio (streaming).
    #[inline]
    pub fn process_block(&mut self, output: &mut [f32]) {
        #[cfg(feature = "naad-backend")]
        self.process_block_naad(output);
        #[cfg(not(feature = "naad-backend"))]
        self.process_block_fallback(output);

        for sample in output.iter_mut() {
            *sample = self.dc_blocker.process(*sample);
        }
        self.sample_position += output.len();
    }

    #[cfg(feature = "naad-backend")]
    fn process_block_naad(&mut self, output: &mut [f32]) {
        let base_amp = (self.speed / 30.0).min(1.0) * 0.5;
        for sample in output.iter_mut() {
            let noise = self.noise_gen.next_sample();
            let filtered = self.shape_filter.process_sample_lowpass(noise);
            let gust = 1.0 + self.gustiness * 0.5 * self.gust_lfo.next_value();
            *sample = filtered * base_amp * gust;
        }
    }

    #[cfg(not(feature = "naad-backend"))]
    fn process_block_fallback(&mut self, output: &mut [f32]) {
        let base_amp = (self.speed / 30.0).min(1.0) * 0.5;
        let gust_rate = 0.3 + self.gustiness * 2.0;
        for (i, sample) in output.iter_mut().enumerate() {
            let t = (self.sample_position + i) as f32 / self.sample_rate;
            let gust = 1.0
                + self.gustiness
                    * 0.5
                    * crate::math::f32::sin(core::f32::consts::TAU * gust_rate * t);
            let noise = self.rng.next_f32();
            *sample = noise * base_amp * gust;
        }
    }
}
