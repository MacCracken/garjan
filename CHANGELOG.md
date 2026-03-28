# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.9.0] - 2026-03-28

### Added

- **LOD (Level of Detail)** (`src/lod.rs`): `Quality` enum (Full/Reduced/Minimal) with `scale_modes()` and `scale_rate()` helpers for CPU scaling of distant/quiet sources
- **Builder pattern** (`src/builder.rs`): `PrecipitationBuilder`, `FootstepBuilder`, `FrictionBuilder` for ergonomic construction of complex synthesizers
- **SIMD-friendly ModalBank**: restructured from AoS (Array of Structs) to SoA (Struct of Arrays) — parallel `state_re`, `state_im`, `coeff_re`, `coeff_im`, `amplitude` arrays enable compiler auto-vectorization across 4+ modes per SIMD lane
- 6 new tests: LOD scaling, builder pattern for Precipitation/Footstep/Friction, Quality serde roundtrip

## [0.8.0] - 2026-03-28

### Fixed

- **Impact hot-path allocation eliminated**: excitation buffer is now pre-allocated in the struct. `process_block` no longer allocates on the heap.
- **Duration validation on all synthesize methods**: negative, zero, NaN, and infinite durations now return `GarjanError::InvalidParameter` instead of silently failing or panicking.

### Added

- `validate_duration()` helper in `dsp.rs`, applied to all 20 `synthesize(duration)` methods
- 3 missing benchmarks: `precipitation_hail_1s`, `underwater_medium_1s`, `surf_moderate_2s`
- Deterministic replay test: verifies 9 synthesizers produce bit-identical output across two runs with same params
- Invalid duration test: verifies proper error handling for negative/zero/NaN/infinite
- **Voice management** (`src/voice.rs`): `VoicePool` with priority-based polyphony, 3 steal policies (Oldest, LowestPriority, None), slot allocation/release, age tracking via `tick()`, active voice iteration
- 10 voice management tests

## [0.7.0] - 2026-03-28

### Added

- **Enhanced Weather & Water** — 3 new synthesizer modules:
  - **Precipitation** (`src/precipitation.rs`): Hail (modal impacts on terrain surfaces), Snow (muffled filtered noise), SurfaceRain (terrain-dependent splatter character). StoneSize: Small/Medium/Large. Uses Terrain enum for surface interaction.
  - **Underwater** (`src/underwater.rs`): Submerged ambience at Shallow/Medium/Deep depth. Low-frequency rumble + filtered surface noise + stochastic bubble events. Boundary note: garjan = source generation, goonj = propagation.
  - **Surf** (`src/surf.rs`): Breaking wave cycle with 3-phase model — approach rumble, crash break, receding wash. SurfIntensity: Calm/Moderate/Heavy/Storm with period/amplitude scaling.
- 13 new tests: all type/depth/intensity variants, zero-intensity silence, terrain interaction, serde roundtrips
- New enums: `PrecipitationType`, `StoneSize`, `UnderwaterDepth`, `SurfIntensity`

## [0.6.0] - 2026-03-28

### Added

- **Creature & Fluid sound synthesis** — 3 new synthesizer modules:
  - **Insect** (`src/insect.rs`): WingBuzz (AM tone), CricketChirp (stridulation pulse train), CicadaDrone (broadband rattle). Swarm mode (1-8 detuned voices). `set_intensity()` control.
  - **WingFlap** (`src/wingflap.rs`): Bird wing flap synthesis for Small, Medium, Large birds. Periodic filtered noise bursts at size-dependent rate. `set_intensity()` control.
  - **Bubble** (`src/bubble.rs`): Underwater, Boiling, Viscous, Pouring — Poisson-scheduled decaying sinusoids (Minnaert resonance model) with onset pop noise. `set_intensity()` control.
- **Shared creature types** (`src/creature.rs`): `InsectType`, `BubbleType` enums
- `BirdSize` enum in `wingflap.rs`
- 13 new tests: all type variants, swarm mode, zero-intensity silence, serde roundtrips
- 3 new benchmarks: insect_swarm_5_1s, wingflap_medium_1s, bubble_boiling_1s
- **Science bridge** (`src/bridge.rs`): conversion functions mapping physical simulation outputs to garjan parameters — rain rate (mm/hr) → RainIntensity, wind speed (m/s) → normalized 0-1, Beaufort scale, thermal shear → gustiness, threat level → thunder/wind/rain preset, flash time + temperature → thunder distance, flame temperature → fire intensity, convective flux → fire intensity, seasonal growth → foliage contact, Shannon diversity → foliage density, Reynolds number → WhooshType. All dependency-free (takes primitives, returns garjan types).
- 6 new bridge tests

## [0.5.0] - 2026-03-28

### Added

- **Aerodynamic sound synthesis** — 3 new synthesizer modules:
  - **Whoosh** (`src/whoosh.rs`): object pass-by / swing (Swing, Projectile, Vehicle, Throw), speed-dependent brightness and envelope, `trigger()` + streaming
  - **Whistle** (`src/whistle.rs`): wind through openings (Gap, Pipe, Bottle, Wire), narrow-band tonal resonance with SVF, pitch wobble LFO, `set_wind_speed()` control
  - **Cloth** (`src/cloth.rs`): fabric flapping (Flag, Cape, Sail, Tarp), Poisson-scheduled flap events, Sail uses Fabric modal bank for heavy resonance, `set_wind_speed()` control
- **Shared aero types** (`src/aero.rs`): `WhooshType`, `WhistleSource`, `ClothType` enums
- 12 new tests: all type variants, trigger APIs, zero-wind silence, serde roundtrips
- 3 new benchmarks: whoosh_swing, whistle_pipe_1s, cloth_flag_1s

### Changed

- Roadmap: mechanical sounds (engines, gears, motors, steam, electrical) moved to ghurni crate. Doppler math deferred to goonj crate.

## [0.4.0] - 2026-03-28

### Added

- **Contact & Surface synthesis** — 5 new synthesizer modules:
  - **Footstep** (`src/footstep.rs`): terrain-aware step sequences on 8 surfaces (Gravel, Sand, Mud, Snow, Wood, Metal, Tile, Wet) with 4 movement types (Walk, Run, Sneak, JumpLand), auto-timed with jitter, `trigger_step()` for game-driven timing
  - **Friction** (`src/friction.rs`): stick-slip model for Scrape, Slide, Grind with real-time `set_velocity()` and `set_pressure()` control, material modal resonance
  - **Creak** (`src/creak.rs`): low-frequency stick-slip for Door, Hinge, Rope, WoodStress with `set_tension()` (pitch) and `set_speed()` (amplitude) control
  - **Rolling** (`src/rolling.rs`): continuous surface contact for Ball, Wheel, Boulder, Barrel with rotation bumps and hollow body resonance (Barrel uses Wood modal bank)
  - **Foliage** (`src/foliage.rs`): LeafRustle and GrassSwish (continuous filtered noise bed + Poisson micro-events), BranchSnap (one-shot through Wood modal bank), `set_wind_speed()` and `set_contact_intensity()` control
- **Shared contact types** (`src/contact.rs`): `Terrain`, `MovementType`, `FrictionType`, `RollingBody`, `FoliageType`, `CreakSource` enums with terrain-to-material mapping
- 19 new tests covering all terrain/movement/type combinations, zero-velocity silence, trigger APIs, serde roundtrips
- 5 new benchmarks for all contact synthesizers

### Fixed (from v0.3 audit)

- `powf` now in `math.rs` compat layer — `no_std` builds work correctly
- Removed phantom dependencies `hisab` and `tracing` (10 fewer crate deps)
- Shatter debris now builds excitation buffer first, processes linearly through modal bank (no state corruption)
- Exciter Impulse deactivates immediately after emitting
- `Rng::poisson` rate clamped to 0–30 (prevents near-infinite loop)
- Added `PartialEq` to `MaterialProperties`, `MaterialModeConfig`, `ModeSpec`
- Water drip frequency is now deterministic across `process_block` calls
- DC blocker R coefficient clamped to [0.9, 0.9999] for low sample rates
- `ModalBank::process_block` has `debug_assert_eq!` on buffer lengths
- Added serde roundtrip tests for `Exciter` and `MaterialModeConfig`

## [0.3.0] - 2026-03-28

### Added

- **Modal synthesis engine** (`src/modal.rs`):
  - `ModalBank`: bank of N parallel damped complex resonators
  - `ModeSpec`: user-facing mode specification (frequency, amplitude, decay)
  - `ModePattern` enum: `Harmonic`, `Beam`, `Plate`, `StiffString`, `Damped`
  - `generate_modes()`: generates mode specs from material properties with frequency-dependent damping
  - `Exciter` with `ExcitationType`: `Impulse`, `NoiseBurst`, `HalfSine`
  - Pre-computed free-free beam ratios for physically accurate wood/marimba modes
- **Material mode configuration**: `Material::mode_config()` maps each of 10 materials to its modal pattern, mode count, and damping factor
- **Enhanced Impact synthesis**:
  - Now uses `ModalBank` for resonance instead of single sinusoid
  - `Impact::new_interaction(striker, surface, sample_rate)` for material-on-material impacts
  - `Impact::synthesize_velocity(impact_type, velocity)` for velocity-sensitive impacts
  - Redesigned Shatter: primary impact + 3-8 debris cascade events within 200ms
  - Per-ImpactType excitation: Tap=HalfSine, Strike=NoiseBurst(3ms), Crash=NoiseBurst(1ms)
- Re-added `cos` and `sqrt` to math compatibility layer (needed for mode coefficients)
- 15 new tests: modal bank, exciter, interaction, velocity, determinism, serde roundtrips
- 3 new benchmarks: `modal_bank_8_modes_512`, `impact_metal_strike`, `impact_glass_shatter`

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
