# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.1.0] - 2026-03-27

### Added

- Initial scaffold of the garjan crate
- **Weather**: `Thunder` (distance-based crack + rumble), `Rain` (Poisson-distributed stochastic drops, 4 intensities), `Wind` (filtered noise with gust modulation)
- **Impact**: `Impact` with 10 materials (Metal, Wood, Stone, Earth, Glass, Fabric, Leaf, Water, Plastic, Ceramic) and 4 impact types (Tap, Strike, Crash, Shatter) with material-specific resonance and decay
- **Water**: `Water` with 4 types (Stream, Drip, Splash, Waves) — stochastic and periodic water sounds
- **Fire**: `Fire` with crackle (stochastic impulses) + roar (broadband noise), intensity-scaled
- **Ambient**: `AmbientTexture` with 6 environments (Forest, City, Ocean, Cave, Desert, Night) — multi-band noise with characteristic spectral shapes
- **Material**: `Material` enum with `MaterialProperties` (resonance, bandwidth, decay, brightness, transient)
- `GarjanError` with serde roundtrip
- PCG32 PRNG with Poisson distribution for stochastic event timing
- Integration tests: all materials, all weather types, all water types, energy comparisons, serde roundtrips
- Criterion benchmarks: thunder, rain, wind, impact, fire, forest texture
- `no_std` support via `libm` + `alloc`
- Strict `deny.toml` matching hisab production patterns
- Send/Sync compile-time assertions on all public types
