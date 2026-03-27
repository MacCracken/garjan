//! Error types for the garjan crate.

use alloc::string::String;
use serde::{Deserialize, Serialize};

/// Errors that can occur during environmental sound synthesis.
#[derive(Debug, Clone, Serialize, Deserialize, thiserror::Error)]
#[non_exhaustive]
pub enum GarjanError {
    /// A synthesis parameter is out of valid range.
    #[error("invalid parameter: {0}")]
    InvalidParameter(String),

    /// A synthesis operation failed.
    #[error("synthesis failed: {0}")]
    SynthesisFailed(String),

    /// A numeric computation produced an invalid result.
    #[error("computation error: {0}")]
    ComputationError(String),
}

/// Convenience type alias for garjan results.
pub type Result<T> = core::result::Result<T, GarjanError>;
