# Migration Guide: garjan 0.x to 1.0

## From 0.1 to 0.2 (Breaking)

### Constructor changes

All synthesizer constructors now take `sample_rate: f32` and return `Result<Self>`:

```rust
// Before (0.1)
let mut thunder = Thunder::new(2000.0);
let samples = thunder.synthesize(44100.0, 3.0).unwrap();

// After (0.2+)
let mut thunder = Thunder::new(2000.0, 44100.0).unwrap();
let samples = thunder.synthesize(3.0).unwrap();
```

### synthesize() signature

`synthesize(sample_rate, duration)` became `synthesize(duration)` — sample rate is set at construction.

### Impact::synthesize

`Impact::synthesize(impact_type, sample_rate)` became `Impact::synthesize(impact_type)`.

## From 0.2 to 0.3

No breaking changes. Added `modal` module, `Impact` now uses modal synthesis internally (output sounds different but API is identical).

## From 0.3 to 1.0

No further breaking changes. All additions (v0.4–v0.9) are purely additive:
new modules, new types, new methods. Existing code compiles without modification.

## Feature flag changes

- `naad-backend` now implies `std` (since v0.2)
- `logging` feature removed (since v0.4 — tracing dependency dropped)
- `hisab` dependency removed (since v0.4 — was unused)
- `science` feature was considered but not added — bridge module is always available

## New modules by version

| Version | New modules |
|---------|-------------|
| 0.2 | `dsp` (internal) |
| 0.3 | `modal` |
| 0.4 | `contact`, `footstep`, `friction`, `creak`, `rolling`, `foliage` |
| 0.5 | `aero`, `whoosh`, `whistle`, `cloth` |
| 0.6 | `creature`, `insect`, `wingflap`, `bubble`, `bridge` |
| 0.7 | `precipitation`, `underwater`, `surf` |
| 0.8 | (hardening — no new synths) |
| 0.9 | `voice`, `lod`, `builder` |
