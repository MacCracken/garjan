# ADR-002: Dual Code Paths (naad + fallback)

## Status

Accepted (v0.2)

## Context

garjan targets both `std` environments (game engines with full OS support) and `no_std` environments (embedded, WASM). The naad crate provides high-quality DSP primitives (filters, noise generators, LFOs) but requires `std`.

## Decision

Every synthesizer has two code paths, selected at compile time via `#[cfg(feature = "naad-backend")]`:

1. **naad path** (default): Uses `naad::noise::NoiseGenerator` (White/Pink/Brown), `naad::filter::BiquadFilter` / `StateVariableFilter`, and `naad::modulation::Lfo` for proper spectral shaping.
2. **Fallback path**: Uses the internal `Rng` for noise (raw or averaged for crude filtering) and `math::f32::sin()` for modulation. Lower quality but zero external dependencies.

The `naad-backend` feature implies `std`.

## Consequences

- `no_std` builds work with degraded but functional audio quality.
- Every synthesizer struct has `#[cfg(feature = "naad-backend")]` fields for naad types.
- Serde format differs between features (naad fields present or absent) — serialized state is not cross-feature compatible.
- Code duplication is minimal: the cfg blocks are typically 5-10 lines each, not full method duplicates.
