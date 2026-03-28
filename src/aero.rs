//! Shared types for aerodynamic sound synthesis.

use serde::{Deserialize, Serialize};

/// Type of whoosh sound.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[non_exhaustive]
pub enum WhooshType {
    /// Short arc (sword swing, bat, whip).
    Swing,
    /// Fast, narrow pass-by (arrow, bullet).
    Projectile,
    /// Long, broad pass-by (vehicle, large object).
    Vehicle,
    /// Medium arc (thrown ball, stone).
    Throw,
}

impl WhooshType {
    /// Returns (envelope_duration_s, brightness, low_freq_content).
    #[inline]
    #[must_use]
    pub(crate) fn config(self) -> (f32, f32, f32) {
        match self {
            Self::Swing => (0.3, 0.7, 0.2),
            Self::Projectile => (0.15, 0.9, 0.1),
            Self::Vehicle => (1.5, 0.4, 0.6),
            Self::Throw => (0.5, 0.5, 0.3),
        }
    }
}

/// Source of wind whistling.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[non_exhaustive]
pub enum WhistleSource {
    /// Thin crack or gap — high, reedy whistle.
    Gap,
    /// Tube or pipe resonance — clear, tonal.
    Pipe,
    /// Helmholtz resonator (bottle) — low, hollow.
    Bottle,
    /// Wire or cable (aeolian tone) — thin, singing.
    Wire,
}

impl WhistleSource {
    /// Returns (base_frequency_hz, bandwidth, noise_mix).
    #[inline]
    #[must_use]
    pub(crate) fn config(self) -> (f32, f32, f32) {
        match self {
            Self::Gap => (2000.0, 200.0, 0.3),
            Self::Pipe => (800.0, 50.0, 0.1),
            Self::Bottle => (300.0, 30.0, 0.15),
            Self::Wire => (1500.0, 80.0, 0.2),
        }
    }
}

/// Type of cloth / fabric.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[non_exhaustive]
pub enum ClothType {
    /// Flag — rhythmic, speed-dependent period.
    Flag,
    /// Cape — irregular, character-driven.
    Cape,
    /// Sail — heavy, slow, powerful.
    Sail,
    /// Tarp — stiff, loud, sharp.
    Tarp,
}

impl ClothType {
    /// Returns (base_flap_rate_hz, amplitude, filter_freq, use_modal).
    #[inline]
    #[must_use]
    pub(crate) fn config(self) -> (f32, f32, f32, bool) {
        match self {
            Self::Flag => (8.0, 0.3, 3000.0, false),
            Self::Cape => (5.0, 0.25, 2500.0, false),
            Self::Sail => (2.0, 0.5, 1000.0, true),
            Self::Tarp => (6.0, 0.4, 4000.0, false),
        }
    }
}
