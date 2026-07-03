//! Shared DSP utilities for garjan synthesis modules.

use serde::{Deserialize, Serialize};

/// DC blocking filter — removes DC offset from synthesis output.
///
/// Uses a one-pole highpass topology: `y[n] = x[n] - x[n-1] + R * y[n-1]`
/// with R chosen for a ~10 Hz cutoff at any sample rate.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct DcBlocker {
    x_prev: f32,
    y_prev: f32,
    r: f32,
}

impl DcBlocker {
    /// Creates a DC blocker for the given sample rate.
    #[inline]
    pub fn new(sample_rate: f32) -> Self {
        Self {
            x_prev: 0.0,
            y_prev: 0.0,
            r: (1.0 - (core::f32::consts::TAU * 10.0 / sample_rate)).clamp(0.9, 0.9999),
        }
    }

    /// Process a single sample, removing DC offset.
    #[inline]
    pub fn process(&mut self, x: f32) -> f32 {
        let y = x - self.x_prev + self.r * self.y_prev;
        self.x_prev = x;
        self.y_prev = y;
        y
    }
}

/// Validates that a duration is positive and finite.
#[inline]
pub(crate) fn validate_duration(duration: f32) -> crate::error::Result<()> {
    if duration <= 0.0 || !duration.is_finite() {
        let err = crate::error::GarjanError::InvalidParameter(alloc::format!(
            "duration must be positive and finite, got {duration}"
        ));
        #[cfg(feature = "logging")]
        tracing::warn!(%duration, "invalid duration");
        return Err(err);
    }
    Ok(())
}

/// Validates that a sample rate is positive and finite.
#[inline]
pub(crate) fn validate_sample_rate(sample_rate: f32) -> crate::error::Result<()> {
    if sample_rate <= 0.0 || !sample_rate.is_finite() {
        let err = crate::error::GarjanError::InvalidParameter(alloc::format!(
            "sample_rate must be positive and finite, got {sample_rate}"
        ));
        #[cfg(feature = "logging")]
        tracing::warn!(%sample_rate, "invalid sample rate");
        return Err(err);
    }
    Ok(())
}

/// Logs a naad backend initialization error with context.
///
/// Use this instead of bare `.map_err()` to get structured error messages
/// that include the synthesizer name and failed component.
#[cfg(feature = "naad-backend")]
#[allow(dead_code)]
pub(crate) fn map_naad_error(
    synth_name: &str,
    component: &str,
    err: naad::NaadError,
) -> crate::error::GarjanError {
    let msg = alloc::format!("{synth_name}: {component} init failed: {err}");
    #[cfg(feature = "logging")]
    tracing::error!(synth = synth_name, component, %err, "naad backend error");
    crate::error::GarjanError::SynthesisFailed(msg)
}
