//! Modal synthesis engine: bank of damped complex resonators.
//!
//! Models vibrating objects as a sum of N independent damped harmonic modes.
//! Each mode is a complex resonator producing an exponentially decaying
//! sinusoid at a specific frequency. The sum of all modes produces the
//! characteristic sound of a struck material.
//!
//! ## Architecture
//!
//! ```text
//! Excitation ──> [Mode 1] ──┐
//!            ──> [Mode 2] ──┤
//!            ──> [Mode 3] ──┼──> Sum ──> Output
//!            ──> [  ...  ] ──┤
//!            ──> [Mode N] ──┘
//! ```

use alloc::vec::Vec;
use serde::{Deserialize, Serialize};

use crate::dsp::validate_sample_rate;
use crate::error::Result;
use crate::rng::Rng;

/// Pre-computed free-free beam mode frequency ratios: `(B_k / B_1)^2`.
///
/// Derived from the eigenvalues of the Euler-Bernoulli beam equation.
const BEAM_RATIOS: [f32; 16] = [
    1.0, 2.757, 5.404, 8.933, 13.344, 18.637, 24.812, 31.870, 39.810, 48.632, 58.336, 68.922,
    80.390, 92.741, 105.973, 120.088,
];

// ---------------------------------------------------------------------------
// ModeSpec
// ---------------------------------------------------------------------------

/// Specification for a single resonant mode.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModeSpec {
    /// Mode frequency in Hz.
    pub frequency: f32,
    /// Mode amplitude (linear, typically 0.0–1.0).
    pub amplitude: f32,
    /// Decay time in seconds (T60: time to decay by 60 dB).
    pub decay: f32,
}

// ---------------------------------------------------------------------------
// ModePattern
// ---------------------------------------------------------------------------

/// Pattern for generating mode frequencies from a base frequency.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[non_exhaustive]
pub enum ModePattern {
    /// Harmonic series: `f_k = f0 * k` (strings, tubes).
    Harmonic,
    /// Free-free beam modes (wood, marimba): highly inharmonic.
    Beam,
    /// Plate/shell modes: `f_k = f0 * k^1.7` (metal, glass, ceramic).
    Plate,
    /// Stiff string: `f_k = f0 * k * sqrt(1 + B*k^2)` with stiffness B.
    StiffString,
    /// Heavily damped: all modes near f0 with slight random spread.
    Damped,
}

// ---------------------------------------------------------------------------
// Mode (internal)
// ---------------------------------------------------------------------------

/// A single damped complex resonator.
#[derive(Debug, Clone, Serialize, Deserialize)]
struct Mode {
    state_re: f32,
    state_im: f32,
    coeff_re: f32,
    coeff_im: f32,
    amplitude: f32,
}

impl Mode {
    fn from_spec(spec: &ModeSpec, sample_rate: f32) -> Self {
        let omega = core::f32::consts::TAU * spec.frequency / sample_rate;
        let radius = crate::math::f32::exp(-6.908 / (spec.decay.max(0.001) * sample_rate))
            .clamp(0.0, 0.9999);
        Self {
            state_re: 0.0,
            state_im: 0.0,
            coeff_re: radius * crate::math::f32::cos(omega),
            coeff_im: radius * crate::math::f32::sin(omega),
            amplitude: spec.amplitude,
        }
    }

    /// Process one excitation sample through this mode. Returns the mode's output.
    #[inline]
    fn tick(&mut self, excitation: f32) -> f32 {
        let new_re = excitation + self.coeff_re * self.state_re - self.coeff_im * self.state_im;
        let new_im = self.coeff_im * self.state_re + self.coeff_re * self.state_im;
        self.state_re = new_re;
        self.state_im = new_im;
        self.amplitude * new_re
    }
}

// ---------------------------------------------------------------------------
// ModalBank
// ---------------------------------------------------------------------------

/// A bank of parallel damped resonators for modal sound synthesis.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModalBank {
    modes: Vec<Mode>,
    sample_rate: f32,
}

impl ModalBank {
    /// Creates a new modal bank from mode specifications.
    ///
    /// Modes with frequency at or above Nyquist (`sample_rate / 2`) are excluded.
    /// Modes with frequency below 20 Hz are excluded to prevent DC accumulation.
    pub fn new(specs: &[ModeSpec], sample_rate: f32) -> Result<Self> {
        validate_sample_rate(sample_rate)?;
        let nyquist = sample_rate * 0.5;
        let modes = specs
            .iter()
            .filter(|s| s.frequency >= 20.0 && s.frequency < nyquist && s.decay > 0.0)
            .map(|s| Mode::from_spec(s, sample_rate))
            .collect();
        Ok(Self { modes, sample_rate })
    }

    /// Returns the number of active modes.
    #[inline]
    #[must_use]
    pub fn mode_count(&self) -> usize {
        self.modes.len()
    }

    /// Process a single excitation sample through all modes.
    #[inline]
    pub fn process_sample(&mut self, excitation: f32) -> f32 {
        let mut out = 0.0f32;
        for mode in &mut self.modes {
            out += mode.tick(excitation);
        }
        out
    }

    /// Process a block of excitation into an output buffer.
    ///
    /// `excitation` and `output` must be the same length.
    /// Output is overwritten (not accumulated).
    #[inline]
    pub fn process_block(&mut self, excitation: &[f32], output: &mut [f32]) {
        let len = excitation.len().min(output.len());
        for i in 0..len {
            output[i] = self.process_sample(excitation[i]);
        }
    }

    /// Reset all mode states to zero.
    pub fn reset(&mut self) {
        for mode in &mut self.modes {
            mode.state_re = 0.0;
            mode.state_im = 0.0;
        }
    }
}

// ---------------------------------------------------------------------------
// Mode generation
// ---------------------------------------------------------------------------

/// Generates mode specifications for a material.
///
/// Uses the material's acoustic properties and a mode frequency pattern to
/// produce `count` modes with physically plausible frequency ratios,
/// amplitudes, and decay times.
///
/// `damping_factor` controls how quickly higher modes decay relative to the
/// fundamental. Higher values produce duller sounds.
#[must_use]
pub fn generate_modes(
    props: &crate::material::MaterialProperties,
    pattern: ModePattern,
    count: usize,
    damping_factor: f32,
) -> Vec<ModeSpec> {
    let f0 = props.resonance;
    // Spectral tilt from brightness: high brightness = slow rolloff
    let rolloff_exp = 2.0 - 1.5 * props.brightness;
    let mut rng = Rng::new((f0 * 1000.0) as u64);

    (1..=count)
        .filter_map(|k| {
            let kf = k as f32;
            let freq = match pattern {
                ModePattern::Harmonic => f0 * kf,
                ModePattern::Beam => {
                    if k <= BEAM_RATIOS.len() {
                        f0 * BEAM_RATIOS[k - 1]
                    } else {
                        // Extrapolate: approximately (k + 0.5)^2 for large k
                        f0 * (kf + 0.5) * (kf + 0.5) / (1.5056 * 1.5056)
                    }
                }
                ModePattern::Plate => {
                    // f_k ~ f0 * k^1.7 — practical plate approximation
                    f0 * kf.powf(1.7)
                }
                ModePattern::StiffString => {
                    let b = 0.001f32;
                    f0 * kf * crate::math::f32::sqrt(1.0 + b * kf * kf)
                }
                ModePattern::Damped => f0 * (1.0 + rng.next_f32_range(-0.1, 0.1)),
            };

            // Skip if frequency is unreasonably high (let ModalBank::new handle Nyquist)
            if freq > 20000.0 {
                return None;
            }

            // Amplitude: spectral tilt from brightness
            let amplitude = 1.0 / kf.powf(rolloff_exp);

            // Frequency-dependent damping: higher modes decay faster
            let freq_ratio = freq / f0;
            let decay =
                props.decay / (1.0 + damping_factor * (freq_ratio - 1.0) * (freq_ratio - 1.0));

            Some(ModeSpec {
                frequency: freq,
                amplitude,
                decay: decay.max(0.001),
            })
        })
        .collect()
}

// ---------------------------------------------------------------------------
// ExcitationType
// ---------------------------------------------------------------------------

/// Shape of the excitation signal that drives a modal bank.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[non_exhaustive]
pub enum ExcitationType {
    /// Single-sample impulse (sharpest, excites all frequencies equally).
    Impulse,
    /// White noise burst of the given duration in samples.
    NoiseBurst {
        /// Duration of the noise burst in samples.
        duration_samples: usize,
    },
    /// Half-sine pulse of the given duration in samples (softest).
    HalfSine {
        /// Duration of the pulse in samples.
        duration_samples: usize,
    },
}

// ---------------------------------------------------------------------------
// Exciter
// ---------------------------------------------------------------------------

/// Generates excitation signals for driving modal banks.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Exciter {
    excitation_type: ExcitationType,
    amplitude: f32,
    position: usize,
    active: bool,
    rng: Rng,
}

impl Exciter {
    /// Creates a new exciter.
    #[must_use]
    pub fn new(excitation_type: ExcitationType, amplitude: f32) -> Self {
        Self {
            excitation_type,
            amplitude,
            position: 0,
            active: false,
            rng: Rng::new(31337),
        }
    }

    /// Trigger the exciter — starts generating excitation from the beginning.
    pub fn trigger(&mut self) {
        self.position = 0;
        self.active = true;
    }

    /// Returns the next excitation sample. Returns 0.0 when spent.
    #[inline]
    pub fn next_sample(&mut self) -> f32 {
        if !self.active {
            return 0.0;
        }

        let sample = match self.excitation_type {
            ExcitationType::Impulse => {
                if self.position == 0 {
                    self.amplitude
                } else {
                    self.active = false;
                    0.0
                }
            }
            ExcitationType::NoiseBurst { duration_samples } => {
                if self.position < duration_samples {
                    self.amplitude * self.rng.next_f32()
                } else {
                    self.active = false;
                    0.0
                }
            }
            ExcitationType::HalfSine { duration_samples } => {
                if self.position < duration_samples {
                    let t = self.position as f32 / duration_samples as f32;
                    self.amplitude * crate::math::f32::sin(core::f32::consts::PI * t)
                } else {
                    self.active = false;
                    0.0
                }
            }
        };

        self.position += 1;
        sample
    }

    /// Returns true if the exciter is still producing output.
    #[inline]
    #[must_use]
    pub fn is_active(&self) -> bool {
        self.active
    }
}
