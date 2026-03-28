# Dependency Watch

Tracked dependency version constraints, known incompatibilities, and upgrade paths.

## naad (optional)

**Status:** Pinned to `naad` 1.x

**Note:** naad is garjan's primary DSP backend. It provides noise generators (White/Pink/Brown), biquad and state variable filters, LFOs, and delay lines. A naad 2.x upgrade would require updating all `#[cfg(feature = "naad-backend")]` blocks.

**no_std:** naad requires `std`. garjan's `naad-backend` feature implies `std`. When naad is disabled, garjan falls back to manual DSP using `Rng` and `math.rs`.

## serde

**Status:** `serde` 1.x with `derive` and `alloc` features

**Note:** All public types derive `Serialize` + `Deserialize`. The serialization format is part of the public API — changing serde attributes is a breaking change.

**no_std:** Uses `default-features = false` with `alloc` feature. Works in `no_std` environments.

## thiserror

**Status:** `thiserror` 2.x with `default-features = false`

**Note:** Used only for `GarjanError` derive. Lightweight proc macro with no runtime cost.

## libm

**Status:** `libm` 0.2

**Note:** Provides `no_std` implementations of `sinf`, `cosf`, `expf`, `sqrtf`, `powf`. Only used when `std` feature is disabled (the `math.rs` compat layer delegates to `std` methods when available).

## tracing (optional)

**Status:** `tracing` 0.1 with `default-features = false`, behind `logging` feature

**Note:** garjan emits tracing events but does NOT provide a subscriber. The consuming application must initialize a subscriber (e.g., `tracing-subscriber`) to see log output.
