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
//! | `logging` | No | Structured logging via tracing-subscriber |

#![cfg_attr(not(feature = "std"), no_std)]

extern crate alloc;

mod dsp;
pub mod error;
pub mod fire;
pub mod impact;
pub mod material;
mod math;
pub mod modal;
mod rng;
pub mod texture;
pub mod water;
pub mod weather;

/// Convenience re-exports for common usage.
pub mod prelude {
    pub use crate::error::{GarjanError, Result};
    pub use crate::fire::Fire;
    pub use crate::impact::{Impact, ImpactType};
    pub use crate::material::Material;
    pub use crate::modal::{ExcitationType, Exciter, ModalBank, ModePattern, ModeSpec};
    pub use crate::texture::{AmbientTexture, TextureType};
    pub use crate::water::{Water, WaterType};
    pub use crate::weather::{Rain, RainIntensity, Thunder, Wind};
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
    }
}
