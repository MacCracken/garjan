//! Shared types for creature and fluid sound synthesis.

use serde::{Deserialize, Serialize};

/// Type of insect sound.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[non_exhaustive]
pub enum InsectType {
    /// Wing buzz — frequency-modulated vibration (bee, fly, mosquito).
    WingBuzz,
    /// Cricket chirp — stridulation (leg-on-wing friction).
    CricketChirp,
    /// Cicada drone — sustained high-frequency rattle.
    CicadaDrone,
}

/// Type of bubble sound.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[non_exhaustive]
pub enum BubbleType {
    /// Underwater air bubbles — rising, popping.
    Underwater,
    /// Boiling water — rapid stochastic bubbles.
    Boiling,
    /// Viscous fluid — slow, thick bubbles (lava, mud).
    Viscous,
    /// Pouring liquid — stream of small bubbles.
    Pouring,
}

impl InsectType {
    /// Returns (base_freq_hz, modulation_rate_hz, chirp_rate_hz, amplitude).
    #[inline]
    #[must_use]
    pub(crate) fn config(self) -> (f32, f32, f32, f32) {
        match self {
            // Wing buzz: high freq carrier, slow AM modulation
            Self::WingBuzz => (200.0, 8.0, 0.0, 0.3),
            // Cricket: silence-chirp pattern, ~4kHz carrier, ~15Hz pulse rate
            Self::CricketChirp => (4000.0, 15.0, 3.0, 0.2),
            // Cicada: broadband ~6kHz, slow AM
            Self::CicadaDrone => (6000.0, 20.0, 0.5, 0.25),
        }
    }
}

impl BubbleType {
    /// Returns (base_freq_hz, event_rate, freq_spread, decay_time_s).
    #[inline]
    #[must_use]
    pub(crate) fn config(self) -> (f32, f32, f32, f32) {
        match self {
            Self::Underwater => (400.0, 5.0, 300.0, 0.03),
            Self::Boiling => (800.0, 30.0, 600.0, 0.01),
            Self::Viscous => (150.0, 2.0, 100.0, 0.08),
            Self::Pouring => (600.0, 40.0, 400.0, 0.005),
        }
    }
}
