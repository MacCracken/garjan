//! Science crate bridges — convert physical simulation outputs to garjan parameters.
//!
//! This module provides conversion functions that map outputs from AGNOS science
//! crates (badal, pavan, goonj, ushma, vanaspati) to garjan synthesizer parameters.
//! Enabled by the `science` feature flag.
//!
//! # Architecture
//!
//! ```text
//! badal (weather)    ──┐
//! pavan (aerodynamics) ┤
//! goonj (acoustics)    ┼──> bridge ──> garjan synthesizer parameters
//! ushma (thermo)       ┤
//! vanaspati (botany)   ┘
//! ```
//!
//! The game engine calls science crate functions to compute physical state,
//! then passes results through these bridges to drive garjan synthesis.

use crate::weather::RainIntensity;

// ---------------------------------------------------------------------------
// Weather bridges (badal)
// ---------------------------------------------------------------------------

/// Converts a rain rate (mm/hr) to a garjan `RainIntensity`.
#[must_use]
pub fn rain_intensity_from_rate(rate_mm_hr: f64) -> Option<RainIntensity> {
    if rate_mm_hr <= 0.0 {
        None
    } else if rate_mm_hr < 2.5 {
        Some(RainIntensity::Light)
    } else if rate_mm_hr < 7.5 {
        Some(RainIntensity::Moderate)
    } else if rate_mm_hr < 50.0 {
        Some(RainIntensity::Heavy)
    } else {
        Some(RainIntensity::Torrential)
    }
}

/// Converts a rain rate (mm/hr) to a snow amplitude scale factor.
///
/// Higher snow-liquid ratio (colder, drier snow) produces quieter sounds.
#[must_use]
pub fn snow_amplitude_scale(snow_liquid_ratio: f64) -> f32 {
    (1.0 / snow_liquid_ratio.max(1.0)) as f32
}

/// Converts wind speed in m/s to the 0.0-1.0 range used by `Wind::new`.
///
/// The `Wind` synth normalizes internally via `speed / 30.0`, so this
/// just clamps to a sane range. Pass the raw m/s value to `Wind::new`.
#[must_use]
pub fn wind_speed_raw(speed_ms: f64) -> f32 {
    speed_ms.max(0.0) as f32
}

/// Converts wind speed in m/s to the normalized 0.0-1.0 range used by
/// `Foliage::set_wind_speed`, `Cloth::set_wind_speed`, `Whistle::set_wind_speed`.
///
/// Reference: 20 m/s (~Beaufort 8, gale) maps to 1.0.
#[must_use]
pub fn wind_speed_normalized(speed_ms: f64) -> f32 {
    (speed_ms / 20.0).clamp(0.0, 1.0) as f32
}

/// Converts a Beaufort scale number (0-12) to normalized wind speed 0.0-1.0.
#[must_use]
pub fn wind_from_beaufort(beaufort: u8) -> f32 {
    (beaufort as f32 / 12.0).clamp(0.0, 1.0)
}

/// Converts thermal wind shear (m/s per km) to gustiness 0.0-1.0.
///
/// Strong vertical shear (~20 m/s/km) produces very gusty conditions.
#[must_use]
pub fn gustiness_from_shear(shear_ms_per_km: f64) -> f32 {
    (shear_ms_per_km / 20.0).clamp(0.0, 1.0) as f32
}

/// Converts an atmospheric stability class to a gustiness modifier.
///
/// Unstable atmosphere increases gustiness, stable decreases it.
/// Add this to the base gustiness value.
#[must_use]
pub fn gustiness_stability_modifier(unstable: bool) -> f32 {
    if unstable { 0.3 } else { -0.2 }
}

/// Maps a severe weather threat level (0-5 scale) to thunder/wind parameters.
///
/// Returns `(thunder_distance_m, wind_speed_ms, rain_intensity)`.
/// Scale: 0=None, 1=Marginal, 2=Slight, 3=Enhanced, 4=Moderate, 5=High.
#[must_use]
pub fn weather_from_threat_level(level: u8) -> (f32, f32, Option<RainIntensity>) {
    match level {
        0 | 1 => (f32::MAX, 5.0, None),
        2 => (3000.0, 8.0, Some(RainIntensity::Moderate)),
        3 => (1500.0, 14.0, Some(RainIntensity::Heavy)),
        4 => (500.0, 22.0, Some(RainIntensity::Torrential)),
        _ => (100.0, 30.0, Some(RainIntensity::Torrential)),
    }
}

// ---------------------------------------------------------------------------
// Thunder bridges (goonj)
// ---------------------------------------------------------------------------

/// Computes thunder distance from time since lightning flash and temperature.
///
/// Uses the standard speed-of-sound formula: c = 331.3 + 0.606 * T.
/// Pass temperature in Celsius.
#[must_use]
pub fn thunder_distance_from_flash(time_since_flash_s: f32, temperature_celsius: f32) -> f32 {
    let speed_of_sound = 331.3 + 0.606 * temperature_celsius;
    time_since_flash_s * speed_of_sound
}

/// Computes a distance-dependent low-pass cutoff for thunder rumble.
///
/// Models atmospheric absorption: distant thunder loses high frequencies.
/// Returns a filter cutoff in Hz suitable for the thunder rumble filter.
#[must_use]
pub fn thunder_distance_cutoff(distance_m: f32) -> f32 {
    // Closer thunder has more HF content; distant thunder is pure rumble
    (5000.0 / (1.0 + distance_m * 0.002)).clamp(60.0, 5000.0)
}

/// Converts SPL drop with distance to a linear gain factor.
///
/// `distance_ref` is typically 1.0 m.
#[must_use]
pub fn gain_from_distance(distance_ref: f32, distance: f32) -> f32 {
    if distance <= distance_ref {
        return 1.0;
    }
    distance_ref / distance
}

// ---------------------------------------------------------------------------
// Fire bridges (ushma)
// ---------------------------------------------------------------------------

/// Converts a flame temperature (Kelvin) to fire intensity 0.0-1.0.
///
/// Maps 500 K (barely glowing) to 0.0, 3000 K (theoretical max) to 1.0.
/// Typical values: campfire ~1200 K → 0.28, forest fire ~1800 K → 0.52.
#[must_use]
pub fn fire_intensity_from_temperature(flame_temp_k: f64) -> f32 {
    ((flame_temp_k - 500.0) / 2500.0).clamp(0.0, 1.0) as f32
}

/// Converts convective heat transfer rate (Watts) to a fire intensity modifier.
///
/// Large convective flux drives the fire roar character.
#[must_use]
pub fn fire_intensity_from_convection(watts: f64) -> f32 {
    (watts / 50_000.0).clamp(0.0, 1.0) as f32
}

/// Combines flame temperature and convective flux into a blended fire intensity.
#[must_use]
pub fn fire_intensity_blended(flame_temp_k: f64, convection_watts: f64) -> f32 {
    let thermal = fire_intensity_from_temperature(flame_temp_k);
    let convective = fire_intensity_from_convection(convection_watts);
    (0.6 * thermal + 0.4 * convective).clamp(0.0, 1.0)
}

// ---------------------------------------------------------------------------
// Foliage bridges (vanaspati)
// ---------------------------------------------------------------------------

/// Converts a seasonal growth modifier (0.0 = winter, 1.0 = summer) to
/// foliage contact intensity.
///
/// In winter (bare branches), contact intensity is near zero.
/// In summer (full canopy), contact intensity is high.
#[must_use]
pub fn foliage_contact_from_growth(growth_modifier: f32) -> f32 {
    growth_modifier.clamp(0.0, 1.0)
}

/// Converts a Shannon diversity index to foliage contact intensity.
///
/// H ≈ 2.0 (diverse mixed-species forest) maps to 1.0.
#[must_use]
pub fn foliage_contact_from_diversity(shannon_h: f32) -> f32 {
    (shannon_h / 2.0).clamp(0.0, 1.0)
}

/// Returns whether foliage should use BranchSnap (bare) or LeafRustle (leafy).
///
/// `growth_modifier` of 0.0 (winter) → true (bare branches only).
#[must_use]
pub fn is_bare_season(growth_modifier: f32) -> bool {
    growth_modifier < 0.1
}

// ---------------------------------------------------------------------------
// Whoosh bridges (pavan)
// ---------------------------------------------------------------------------

/// Suggests a `WhooshType` based on Reynolds number turbulence.
///
/// Turbulent boundary layer → Vehicle-like broadband whoosh.
/// Laminar → Projectile-like narrow whoosh.
#[must_use]
pub fn whoosh_type_from_reynolds(reynolds: f64) -> crate::aero::WhooshType {
    if reynolds > 500_000.0 {
        crate::aero::WhooshType::Vehicle
    } else {
        crate::aero::WhooshType::Projectile
    }
}
