//! Math compatibility layer for `no_std` support.
//!
//! When `std` is available, delegates to the standard library's `f32` methods.
//! Without `std`, uses `libm` for transcendental functions.

/// f32 math operations (sin, exp).
#[cfg(feature = "std")]
pub(crate) mod f32 {
    #[inline(always)]
    pub fn sin(x: f32) -> f32 {
        x.sin()
    }
    #[inline(always)]
    pub fn exp(x: f32) -> f32 {
        x.exp()
    }
}

#[cfg(not(feature = "std"))]
pub(crate) mod f32 {
    #[inline(always)]
    pub fn sin(x: f32) -> f32 {
        libm::sinf(x)
    }
    #[inline(always)]
    pub fn exp(x: f32) -> f32 {
        libm::expf(x)
    }
}
