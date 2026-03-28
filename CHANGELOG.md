# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.2.0] - 2026-03-28

### Changed

- **Breaking**: All synthesizer constructors now take `sample_rate: f32` and return `Result<Self>`
- **Breaking**: `synthesize(sample_rate, duration)` is now `synthesize(duration)` — sample rate set at construction
- **Breaking**: `Impact::synthesize(impact_type, sample_rate)` is now `Impact::synthesize(impact_type)`
- **naad integration**: All modules use naad filters, noise generators, and LFOs when `naad-backend` feature is enabled
  - **Thunder**: Brown noise + LowPass filter for rumble (cutoff varies by distance), White noise for crack
  - **Rain**: White noise generator + BandPass filter (~3kHz) for drop character
  - **Wind**: Pink noise + State Variable Filter (cutoff varies by speed) + LFO for gust modulation
  - **Fire**: Brown noise + LowPass for roar, White noise + HighPass for crackle
  - **Water/Stream**: Pink noise + BandPass (800Hz) + LFO modulation
  - **Water/Splash**: White noise + LowPass (4kHz)
  - **Water/Waves**: Brown noise + LowPass (500Hz) + LFO for wave rhythm
  - **AmbientTexture**: 3-band synthesis (Brown/Pink/White noise + LP/BP/HP filters) + LFO
  - **Impact**: White noise + BandPass at material resonance for transient
- `naad-backend` feature now implies `std` (naad requires std)

### Added

- `process_block(&mut self, output: &mut [f32])` streaming API on all synthesizers
- DC blocking filter on all synthesis outputs (removes offset drift)
- Sample rate validation — constructors reject zero, negative, NaN, and infinite values
- `TextureType` re-exported in prelude
- Fallback code path when `naad-backend` is disabled — original manual DSP preserved
- 9 new tests: process_block streaming, parameter validation, empty buffer handling
- `process_block_wind_512` benchmark for streaming performance
- `src/dsp.rs` shared DSP module with `DcBlocker`

### Removed

- Unused `f64` math module and unused `f32::{cos, sqrt, sinh}` functions

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
