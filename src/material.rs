//! Surface material properties affecting impact and contact sounds.
//!
//! Materials define how surfaces resonate when struck, scraped, or contacted.
//! Each material has characteristic spectral properties, decay rates, and
//! resonant frequencies.

use serde::{Deserialize, Serialize};

/// A surface material with acoustic properties.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[non_exhaustive]
pub enum Material {
    /// Dense, bright, long sustain, high-frequency ring.
    Metal,
    /// Warm, mid-frequency resonance, medium decay.
    Wood,
    /// Hard, bright, very short decay, high-frequency crack.
    Stone,
    /// Dull, heavily damped, very short decay.
    Earth,
    /// Brittle, sharp transient, high-frequency shatter.
    Glass,
    /// Soft, damped, low-frequency thud.
    Fabric,
    /// Crisp, mid-frequency snap, moderate decay.
    Leaf,
    /// Variable resonance, depends on fill level.
    Water,
    /// Generic plastic/synthetic — moderate brightness, short decay.
    Plastic,
    /// Hollow, resonant, warm low-mid frequencies.
    Ceramic,
}

/// Acoustic properties of a material for impact/contact synthesis.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MaterialProperties {
    /// Primary resonant frequency (Hz).
    pub resonance: f32,
    /// Resonance bandwidth (Hz) — wider = duller.
    pub bandwidth: f32,
    /// Decay time in seconds (time to -60dB).
    pub decay: f32,
    /// High-frequency content (0.0 = dull, 1.0 = bright).
    pub brightness: f32,
    /// Amount of initial transient noise (0.0-1.0).
    pub transient: f32,
}

impl Material {
    /// Returns the default acoustic properties for this material.
    #[must_use]
    pub fn properties(self) -> MaterialProperties {
        match self {
            Self::Metal => MaterialProperties {
                resonance: 2500.0,
                bandwidth: 200.0,
                decay: 1.5,
                brightness: 0.9,
                transient: 0.7,
            },
            Self::Wood => MaterialProperties {
                resonance: 800.0,
                bandwidth: 400.0,
                decay: 0.3,
                brightness: 0.5,
                transient: 0.6,
            },
            Self::Stone => MaterialProperties {
                resonance: 3000.0,
                bandwidth: 800.0,
                decay: 0.05,
                brightness: 0.8,
                transient: 0.9,
            },
            Self::Earth => MaterialProperties {
                resonance: 200.0,
                bandwidth: 300.0,
                decay: 0.02,
                brightness: 0.1,
                transient: 0.4,
            },
            Self::Glass => MaterialProperties {
                resonance: 4000.0,
                bandwidth: 150.0,
                decay: 0.8,
                brightness: 0.95,
                transient: 1.0,
            },
            Self::Fabric => MaterialProperties {
                resonance: 300.0,
                bandwidth: 500.0,
                decay: 0.01,
                brightness: 0.1,
                transient: 0.2,
            },
            Self::Leaf => MaterialProperties {
                resonance: 1500.0,
                bandwidth: 1000.0,
                decay: 0.02,
                brightness: 0.6,
                transient: 0.5,
            },
            Self::Water => MaterialProperties {
                resonance: 600.0,
                bandwidth: 400.0,
                decay: 0.1,
                brightness: 0.4,
                transient: 0.3,
            },
            Self::Plastic => MaterialProperties {
                resonance: 1800.0,
                bandwidth: 600.0,
                decay: 0.15,
                brightness: 0.5,
                transient: 0.6,
            },
            Self::Ceramic => MaterialProperties {
                resonance: 1200.0,
                bandwidth: 300.0,
                decay: 0.4,
                brightness: 0.7,
                transient: 0.7,
            },
        }
    }
}
