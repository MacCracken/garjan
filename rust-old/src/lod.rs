//! Level of Detail (LOD) for synthesis quality scaling.
//!
//! Allows callers to reduce CPU cost for distant or low-priority sources
//! by selecting a lower quality tier. Each synthesizer that supports LOD
//! checks the quality level and adjusts its processing accordingly.
//!
//! # Typical reductions per level
//!
//! | Level | Modal modes | Stochastic events | Filter passes |
//! |-------|-------------|-------------------|---------------|
//! | Full | All | All | All |
//! | Reduced | 50% | 50% | Simplified |
//! | Minimal | 25% (min 1) | 25% | Bypass |

use serde::{Deserialize, Serialize};

/// Synthesis quality level for LOD scaling.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[non_exhaustive]
pub enum Quality {
    /// Full quality — all modes, all events, all filters.
    Full,
    /// Reduced quality — half modes, half events, simplified filtering.
    Reduced,
    /// Minimal quality — quarter modes, quarter events, filter bypass.
    Minimal,
}

impl Quality {
    /// Returns the mode count multiplier (1.0, 0.5, 0.25).
    #[inline]
    #[must_use]
    pub fn mode_factor(self) -> f32 {
        match self {
            Self::Full => 1.0,
            Self::Reduced => 0.5,
            Self::Minimal => 0.25,
        }
    }

    /// Returns the stochastic event rate multiplier.
    #[inline]
    #[must_use]
    pub fn event_factor(self) -> f32 {
        match self {
            Self::Full => 1.0,
            Self::Reduced => 0.5,
            Self::Minimal => 0.25,
        }
    }

    /// Scales a mode count by the quality factor, with a minimum of 1.
    #[inline]
    #[must_use]
    pub fn scale_modes(self, count: usize) -> usize {
        ((count as f32 * self.mode_factor()) as usize).max(1)
    }

    /// Scales an event rate by the quality factor.
    #[inline]
    #[must_use]
    pub fn scale_rate(self, rate: f32) -> f32 {
        rate * self.event_factor()
    }
}
