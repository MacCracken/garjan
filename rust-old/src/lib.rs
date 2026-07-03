//! # garjan — Environmental and Nature Sound Synthesis
//!
//! **garjan** (Sanskrit: roar / thunder) provides procedural synthesis of
//! environmental and nature sounds: weather, impacts, surfaces, fluids, fire,
//! and ambient textures. All sounds are generated from physical models — no
//! samples, no assets, pure math.
//!
//! ## Architecture
//!
//! ```text
//! Environment (weather, terrain, materials)
//!       |
//!       v
//! Source Generators ─────────────────── Output
//!   Weather:  rain, thunder, wind       (samples)
//!   Impact:   footsteps, crashes, cracks
//!   Surface:  rustling, scraping, rolling
//!   Fluid:    water flow, drips, splashes
//!   Fire:     crackle, roar, hiss
//!   Ambient:  room tone, forest, city
//! ```
//!
//! ## Key Concepts
//!
//! - **Source**: A physical sound generator (rain drop, thunder bolt, wind gust)
//! - **Material**: Surface properties affecting impact/contact sounds (wood, metal, stone, earth)
//! - **Weather**: Atmospheric conditions driving weather sounds (rain intensity, wind speed, storm distance)
//! - **Texture**: Continuous ambient sound layer (forest background, city hum, ocean surf)
//!
//! ## Quick Start
//!
//! ```rust
//! use garjan::prelude::*;
//!
//! // Synthesize a thunderclap 2km away
//! let mut thunder = Thunder::new(2000.0, 44100.0).unwrap();
//! let samples = thunder.synthesize(3.0).unwrap();
//!
//! // Generate rain at medium intensity
//! let mut rain = Rain::new(RainIntensity::Moderate, 44100.0).unwrap();
//! let samples = rain.synthesize(5.0).unwrap();
//! ```
//!
//! ## Feature Flags
//!
//! | Feature | Default | Description |
//! |---------|---------|-------------|
//! | `std` | Yes | Standard library support. Disable for `no_std` + `alloc` |
//! | `naad-backend` | Yes | Use naad crate for oscillators and filters |
//! | `logging` | No | Structured tracing via the `tracing` crate |
#![cfg_attr(not(feature = "std"), no_std)]

extern crate alloc;

pub mod aero;
pub mod bridge;
pub mod bubble;
pub mod builder;
pub mod cloth;
pub mod contact;
pub mod creak;
pub mod creature;
mod dsp;
pub mod error;
pub mod fire;
pub mod foliage;
pub mod footstep;
pub mod friction;
pub mod impact;
pub mod insect;
/// Integration APIs for downstream consumers (soorat rendering).
pub mod integration;
pub mod lod;
pub mod material;
mod math;
pub mod modal;
pub mod precipitation;
mod rng;
pub mod rolling;
pub mod surf;
pub mod texture;
pub mod underwater;
pub mod voice;
pub mod water;
pub mod weather;
pub mod whistle;
pub mod whoosh;
pub mod wingflap;

/// Convenience re-exports for common usage.
pub mod prelude {
    pub use crate::aero::{ClothType, WhistleSource, WhooshType};
    pub use crate::bubble::Bubble;
    pub use crate::cloth::Cloth;
    pub use crate::creature::{BubbleType, InsectType};

    pub use crate::contact::{
        CreakSource, FoliageType, FrictionType, MovementType, RollingBody, Terrain,
    };
    pub use crate::creak::Creak;
    pub use crate::error::{GarjanError, Result};
    pub use crate::fire::Fire;
    pub use crate::foliage::Foliage;
    pub use crate::footstep::Footstep;
    pub use crate::friction::Friction;
    pub use crate::impact::{Impact, ImpactType};
    pub use crate::insect::Insect;
    pub use crate::lod::Quality;
    pub use crate::material::Material;
    pub use crate::modal::{ExcitationType, Exciter, ModalBank, ModePattern, ModeSpec};
    pub use crate::precipitation::{Precipitation, PrecipitationType, StoneSize};
    pub use crate::rolling::Rolling;
    pub use crate::surf::{Surf, SurfIntensity};
    pub use crate::texture::{AmbientTexture, TextureType};
    pub use crate::underwater::{Underwater, UnderwaterDepth};
    pub use crate::voice::{StealPolicy, VoicePool, VoiceSlot};
    pub use crate::water::{Water, WaterType};
    pub use crate::weather::{Rain, RainIntensity, Thunder, Wind};
    pub use crate::whistle::Whistle;
    pub use crate::whoosh::Whoosh;
    pub use crate::wingflap::{BirdSize, WingFlap};
}

// Compile-time trait assertions: all public types must be Send + Sync.
#[cfg(test)]
mod assert_traits {
    fn _assert_send_sync<T: Send + Sync>() {}

    #[test]
    fn public_types_are_send_sync() {
        _assert_send_sync::<crate::error::GarjanError>();
        _assert_send_sync::<crate::material::Material>();
        _assert_send_sync::<crate::weather::Thunder>();
        _assert_send_sync::<crate::weather::Rain>();
        _assert_send_sync::<crate::weather::Wind>();
        _assert_send_sync::<crate::weather::RainIntensity>();
        _assert_send_sync::<crate::fire::Fire>();
        _assert_send_sync::<crate::impact::Impact>();
        _assert_send_sync::<crate::impact::ImpactType>();
        _assert_send_sync::<crate::water::Water>();
        _assert_send_sync::<crate::water::WaterType>();
        _assert_send_sync::<crate::texture::AmbientTexture>();
        _assert_send_sync::<crate::modal::ModalBank>();
        _assert_send_sync::<crate::modal::ModeSpec>();
        _assert_send_sync::<crate::modal::ModePattern>();
        _assert_send_sync::<crate::modal::ExcitationType>();
        _assert_send_sync::<crate::modal::Exciter>();
        _assert_send_sync::<crate::contact::Terrain>();
        _assert_send_sync::<crate::contact::MovementType>();
        _assert_send_sync::<crate::contact::FrictionType>();
        _assert_send_sync::<crate::contact::RollingBody>();
        _assert_send_sync::<crate::contact::FoliageType>();
        _assert_send_sync::<crate::contact::CreakSource>();
        _assert_send_sync::<crate::footstep::Footstep>();
        _assert_send_sync::<crate::friction::Friction>();
        _assert_send_sync::<crate::creak::Creak>();
        _assert_send_sync::<crate::rolling::Rolling>();
        _assert_send_sync::<crate::foliage::Foliage>();
        _assert_send_sync::<crate::aero::WhooshType>();
        _assert_send_sync::<crate::aero::WhistleSource>();
        _assert_send_sync::<crate::aero::ClothType>();
        _assert_send_sync::<crate::whoosh::Whoosh>();
        _assert_send_sync::<crate::whistle::Whistle>();
        _assert_send_sync::<crate::cloth::Cloth>();
        _assert_send_sync::<crate::creature::InsectType>();
        _assert_send_sync::<crate::creature::BubbleType>();
        _assert_send_sync::<crate::insect::Insect>();
        _assert_send_sync::<crate::wingflap::BirdSize>();
        _assert_send_sync::<crate::wingflap::WingFlap>();
        _assert_send_sync::<crate::bubble::Bubble>();
        _assert_send_sync::<crate::voice::VoicePool>();
        _assert_send_sync::<crate::voice::VoiceSlot>();
        _assert_send_sync::<crate::voice::StealPolicy>();
        _assert_send_sync::<crate::lod::Quality>();
        _assert_send_sync::<crate::builder::PrecipitationBuilder>();
        _assert_send_sync::<crate::builder::FootstepBuilder>();
        _assert_send_sync::<crate::builder::FrictionBuilder>();
        _assert_send_sync::<crate::precipitation::Precipitation>();
        _assert_send_sync::<crate::precipitation::PrecipitationType>();
        _assert_send_sync::<crate::precipitation::StoneSize>();
        _assert_send_sync::<crate::underwater::Underwater>();
        _assert_send_sync::<crate::underwater::UnderwaterDepth>();
        _assert_send_sync::<crate::surf::Surf>();
        _assert_send_sync::<crate::surf::SurfIntensity>();
    }
}
