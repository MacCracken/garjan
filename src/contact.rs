//! Shared types for contact and surface sound synthesis.
//!
//! Enums and configuration used by footstep, friction, rolling, foliage,
//! and creak synthesizers.

use serde::{Deserialize, Serialize};

use crate::material::Material;

/// Surface terrain type for footsteps and contact sounds.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[non_exhaustive]
pub enum Terrain {
    /// Loose gravel / pebbles — crunchy, broadband.
    Gravel,
    /// Sand — soft, muffled.
    Sand,
    /// Wet mud / dirt — squelchy, low.
    Mud,
    /// Snow — quiet, compressed crunch.
    Snow,
    /// Wooden floor / planks — hollow thump with resonance.
    Wood,
    /// Metal grate / plate — bright ring.
    Metal,
    /// Stone tile / ceramic — sharp clack.
    Tile,
    /// Wet hard surface — clack with splash component.
    Wet,
}

impl Terrain {
    /// Returns the material for modal resonance, if this terrain resonates.
    #[must_use]
    pub fn resonant_material(self) -> Option<Material> {
        match self {
            Self::Wood => Some(Material::Wood),
            Self::Metal => Some(Material::Metal),
            Self::Tile => Some(Material::Ceramic),
            Self::Wet => Some(Material::Ceramic),
            _ => None,
        }
    }

    /// Returns the noise configuration for this terrain's contact layer.
    #[must_use]
    pub(crate) fn noise_config(self) -> TerrainNoiseConfig {
        match self {
            Self::Gravel => TerrainNoiseConfig {
                noise_type: NoisePreference::Pink,
                filter_freq: 1500.0,
                filter_q: 1.0,
                highpass: true,
                amplitude: 0.4,
            },
            Self::Sand => TerrainNoiseConfig {
                noise_type: NoisePreference::Brown,
                filter_freq: 2000.0,
                filter_q: 0.7,
                highpass: false,
                amplitude: 0.2,
            },
            Self::Mud => TerrainNoiseConfig {
                noise_type: NoisePreference::Brown,
                filter_freq: 800.0,
                filter_q: 1.0,
                highpass: false,
                amplitude: 0.3,
            },
            Self::Snow => TerrainNoiseConfig {
                noise_type: NoisePreference::White,
                filter_freq: 4000.0,
                filter_q: 0.5,
                highpass: false,
                amplitude: 0.1,
            },
            Self::Wood => TerrainNoiseConfig {
                noise_type: NoisePreference::Pink,
                filter_freq: 1000.0,
                filter_q: 1.5,
                highpass: false,
                amplitude: 0.15,
            },
            Self::Metal => TerrainNoiseConfig {
                noise_type: NoisePreference::White,
                filter_freq: 3000.0,
                filter_q: 2.0,
                highpass: true,
                amplitude: 0.2,
            },
            Self::Tile => TerrainNoiseConfig {
                noise_type: NoisePreference::White,
                filter_freq: 2500.0,
                filter_q: 1.5,
                highpass: false,
                amplitude: 0.25,
            },
            Self::Wet => TerrainNoiseConfig {
                noise_type: NoisePreference::White,
                filter_freq: 3500.0,
                filter_q: 1.0,
                highpass: false,
                amplitude: 0.3,
            },
        }
    }
}

/// Internal noise preference (maps to naad NoiseType when available).
#[derive(Debug, Clone, Copy)]
pub(crate) enum NoisePreference {
    White,
    Pink,
    Brown,
}

/// Internal terrain noise configuration.
#[derive(Debug, Clone, Copy)]
#[allow(dead_code)]
pub(crate) struct TerrainNoiseConfig {
    pub noise_type: NoisePreference,
    pub filter_freq: f32,
    pub filter_q: f32,
    /// If true, use highpass instead of lowpass.
    pub highpass: bool,
    pub amplitude: f32,
}

/// Movement type affecting footstep timing and force.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[non_exhaustive]
pub enum MovementType {
    /// Normal walking pace.
    Walk,
    /// Running / jogging.
    Run,
    /// Slow, quiet sneaking.
    Sneak,
    /// Single jump landing (one-shot, no repeat).
    JumpLand,
}

impl MovementType {
    /// Returns (step_interval_seconds, force, excitation_duration_seconds).
    #[must_use]
    pub(crate) fn config(self) -> (f32, f32, f32) {
        match self {
            Self::Walk => (0.5, 0.5, 0.004),
            Self::Run => (0.3, 0.7, 0.0025),
            Self::Sneak => (0.7, 0.15, 0.010),
            Self::JumpLand => (0.0, 1.0, 0.0015), // 0.0 = one-shot
        }
    }
}

/// Type of friction contact.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[non_exhaustive]
pub enum FrictionType {
    /// Object dragged across rough surface — chattery.
    Scrape,
    /// Smooth sliding — more tonal.
    Slide,
    /// Heavy, sustained, low-frequency grinding.
    Grind,
}

/// Type of rolling body.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[non_exhaustive]
pub enum RollingBody {
    /// Small ball — high rotation rate, bright.
    Ball,
    /// Medium wheel — moderate rate.
    Wheel,
    /// Large boulder — slow, heavy rumble.
    Boulder,
    /// Hollow barrel — resonant.
    Barrel,
}

/// Type of foliage sound.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[non_exhaustive]
pub enum FoliageType {
    /// Dry leaves rustling (wind or contact).
    LeafRustle,
    /// Tall grass brushing / swishing.
    GrassSwish,
    /// Triggered branch snap (one-shot).
    BranchSnap,
}

/// Source of creaking sound.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[non_exhaustive]
pub enum CreakSource {
    /// Wooden door on hinges.
    Door,
    /// Metal hinge — higher pitch.
    Hinge,
    /// Rope under tension — fibrous.
    Rope,
    /// Structural wood stress — deep.
    WoodStress,
}
