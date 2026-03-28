//! Builder pattern constructors for synthesizers with complex configuration.
//!
//! Provides ergonomic construction for synthesizers with 3+ parameters
//! where the caller may want to set options incrementally.
//!
//! # Example
//!
//! ```rust
//! use garjan::prelude::*;
//! use garjan::builder::PrecipitationBuilder;
//!
//! let mut precip = PrecipitationBuilder::new(44100.0)
//!     .precip_type(PrecipitationType::Hail)
//!     .stone_size(StoneSize::Large)
//!     .surface(Terrain::Metal)
//!     .build()
//!     .unwrap();
//! ```

use crate::contact::{FrictionType, MovementType, Terrain};
use crate::error::Result;
use crate::friction::Friction;
use crate::material::Material;
use crate::precipitation::{Precipitation, PrecipitationType, StoneSize};

/// Builder for `Precipitation`.
pub struct PrecipitationBuilder {
    sample_rate: f32,
    precip_type: PrecipitationType,
    stone_size: StoneSize,
    surface: Terrain,
}

impl PrecipitationBuilder {
    /// Creates a new builder with defaults (Hail, Medium, Gravel).
    #[must_use]
    pub fn new(sample_rate: f32) -> Self {
        Self {
            sample_rate,
            precip_type: PrecipitationType::Hail,
            stone_size: StoneSize::Medium,
            surface: Terrain::Gravel,
        }
    }

    /// Sets the precipitation type.
    #[must_use]
    pub fn precip_type(mut self, t: PrecipitationType) -> Self {
        self.precip_type = t;
        self
    }

    /// Sets the stone size.
    #[must_use]
    pub fn stone_size(mut self, s: StoneSize) -> Self {
        self.stone_size = s;
        self
    }

    /// Sets the surface terrain.
    #[must_use]
    pub fn surface(mut self, t: Terrain) -> Self {
        self.surface = t;
        self
    }

    /// Builds the `Precipitation` synthesizer.
    pub fn build(self) -> Result<Precipitation> {
        Precipitation::new(
            self.precip_type,
            self.stone_size,
            self.surface,
            self.sample_rate,
        )
    }
}

/// Builder for `Footstep`.
pub struct FootstepBuilder {
    sample_rate: f32,
    terrain: Terrain,
    movement: MovementType,
}

impl FootstepBuilder {
    /// Creates a new builder with defaults (Gravel, Walk).
    #[must_use]
    pub fn new(sample_rate: f32) -> Self {
        Self {
            sample_rate,
            terrain: Terrain::Gravel,
            movement: MovementType::Walk,
        }
    }

    /// Sets the terrain.
    #[must_use]
    pub fn terrain(mut self, t: Terrain) -> Self {
        self.terrain = t;
        self
    }

    /// Sets the movement type.
    #[must_use]
    pub fn movement(mut self, m: MovementType) -> Self {
        self.movement = m;
        self
    }

    /// Builds the `Footstep` synthesizer.
    pub fn build(self) -> Result<crate::footstep::Footstep> {
        crate::footstep::Footstep::new(self.terrain, self.movement, self.sample_rate)
    }
}

/// Builder for `Friction`.
pub struct FrictionBuilder {
    sample_rate: f32,
    friction_type: FrictionType,
    surface: Material,
}

impl FrictionBuilder {
    /// Creates a new builder with defaults (Scrape, Wood).
    #[must_use]
    pub fn new(sample_rate: f32) -> Self {
        Self {
            sample_rate,
            friction_type: FrictionType::Scrape,
            surface: Material::Wood,
        }
    }

    /// Sets the friction type.
    #[must_use]
    pub fn friction_type(mut self, t: FrictionType) -> Self {
        self.friction_type = t;
        self
    }

    /// Sets the surface material.
    #[must_use]
    pub fn surface(mut self, m: Material) -> Self {
        self.surface = m;
        self
    }

    /// Builds the `Friction` synthesizer.
    pub fn build(self) -> Result<Friction> {
        Friction::new(self.friction_type, self.surface, self.sample_rate)
    }
}
