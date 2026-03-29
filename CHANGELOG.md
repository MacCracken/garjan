# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [1.1.0]

### Added
- **integration/soorat** — feature-gated `soorat-compat` module with visualization data structures: `PrecipitationField` (rain/snow particle positions, velocities, sizes), `FireEmitter` (position, intensity, color temperature, flame height, ember rate), `WindField` (2D velocity grid with uniform and gradient constructors)

### Updated
- zerocopy 0.8.47 -> 0.8.48

## [1.0.0] - 2026-03-28

Initial release of garjan — environmental and nature sound synthesis for Rust.

### Added

#### Weather
- **Thunder**: distance-based crack + rumble with atmospheric filtering
- **Rain**: Poisson-distributed stochastic drops at 4 intensities (Light, Moderate, Heavy, Torrential)
- **Wind**: pink noise through state variable filter with LFO gust modulation
- **Precipitation**: hail (modal impacts on surfaces), snow (muffled noise), surface rain (terrain-dependent splatter) with 3 stone sizes
- **Surf**: breaking wave cycle with 3-phase model (approach, crash, wash) at 4 intensity levels
- **Underwater**: submerged ambience at 3 depths with rumble, surface noise, and stochastic bubbles

#### Impact & Contact
- **Impact**: 10 materials with modal synthesis (4-12 resonant modes per material), 4 impact types, velocity-sensitive synthesis, material interaction, shatter with debris cascade
- **Footstep**: 8 terrains (Gravel, Sand, Mud, Snow, Wood, Metal, Tile, Wet), 4 movement types, auto-timed with jitter, game-driven `trigger_step()` API
- **Friction**: stick-slip model (Scrape, Slide, Grind) with real-time velocity/pressure control
- **Creak**: low-frequency stick-slip (Door, Hinge, Rope, WoodStress) with tension/speed control
- **Rolling**: continuous contact (Ball, Wheel, Boulder, Barrel) with rotation bumps and hollow resonance
- **Foliage**: LeafRustle, GrassSwish (continuous + stochastic micro-events), BranchSnap (one-shot)

#### Aerodynamic
- **Whoosh**: object pass-by (Swing, Projectile, Vehicle, Throw) with speed-dependent envelope
- **Whistle**: wind through openings (Gap, Pipe, Bottle, Wire) with SVF resonance and pitch wobble
- **Cloth**: fabric flapping (Flag, Cape, Sail, Tarp) with Poisson-scheduled flap events

#### Creature & Fluid
- **Insect**: WingBuzz, CricketChirp, CicadaDrone with swarm mode (1-8 detuned voices)
- **WingFlap**: bird wing synthesis for Small, Medium, Large birds
- **Bubble**: Underwater, Boiling, Viscous, Pouring using Minnaert resonance model

#### Ambient
- **AmbientTexture**: 6 environments (Forest, City, Ocean, Cave, Desert, Night) with multi-band spectral shaping
- **Fire**: crackle (stochastic impulses) + roar (broadband noise), intensity-scaled

#### Engine
- **Modal synthesis**: `ModalBank` with N parallel damped complex resonators, SoA layout for SIMD auto-vectorization, 5 mode patterns (Harmonic, Beam, Plate, StiffString, Damped)
- **Voice management**: `VoicePool` with priority-based polyphony (Oldest, LowestPriority, None steal policies)
- **LOD**: `Quality` enum (Full, Reduced, Minimal) for CPU scaling of distant sources
- **Science bridge**: 18 dependency-free conversion functions mapping badal/pavan/goonj/ushma/vanaspati outputs to garjan parameters
- **Builder pattern**: `PrecipitationBuilder`, `FootstepBuilder`, `FrictionBuilder`

#### Infrastructure
- `process_block()` streaming API on all 25 synthesizers
- DC blocking filter on all synthesis outputs
- Dual code paths: `naad-backend` (proper filters/noise) + manual fallback (`no_std`)
- Duration and sample rate validation on all public entry points
- Deterministic synthesis: seeded PCG32 PRNG, bit-identical replay guaranteed
- Zero hot-path heap allocations in `process_block`
- `no_std` support via `libm` + `alloc`
- Serde (Serialize + Deserialize) on all public types
- Send + Sync on all public types
- `#[non_exhaustive]` on all public enums

[1.0.0]: https://github.com/MacCracken/garjan/releases/tag/v1.0.0
