//! Integration APIs for downstream consumers.
//!
//! Feature-gated modules that expose visualization-ready data structures
//! for rendering engines and other consumers.

#[cfg(feature = "soorat-compat")]
pub mod soorat;
