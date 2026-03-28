# garjan Roadmap — v0.1 to v1.0

> Environmental and nature sound synthesis for AGNOS.
> Pure procedural — no samples, no assets, pure math.

## v0.1.0 (current) — Initial Scaffold

- [x] Weather: Thunder, Rain (4 intensities), Wind
- [x] Impact: 10 materials, 4 impact types, material-specific resonance
- [x] Water: Stream, Drip, Splash, Waves
- [x] Fire: Crackle + Roar
- [x] Ambient textures: 6 environments (Forest, City, Ocean, Cave, Desert, Night)
- [x] PCG32 PRNG with Poisson distribution
- [x] `no_std` support, Send + Sync, serde on all types
- [x] Criterion benchmarks

---

## v0.2.0 (current) — naad Integration & Core DSP

- [x] Integrate naad biquad/SVF filters into all synthesis modules
- [x] Replace noise-averaging with proper filter chains (LP, HP, BP, SVF)
- [x] Use naad noise generators (white, pink, brown) as noise sources
- [x] DC blocking filter on all synthesis outputs
- [x] Block-based streaming API: `process_block(&mut self, output: &mut [f32])`
- [x] Dual code paths: naad-backend (default) + manual fallback (no_std)
- [x] Sample rate validation on all constructors
- [ ] Parameter smoothing (one-pole filter on control parameters) — deferred to v0.9
- [ ] Use hisab easing functions for envelope shaping — deferred to v0.3
- [ ] Fade-in/fade-out on all continuous synthesizers — deferred to v0.9

---

## v0.3.0 (current) — Modal Synthesis & Enhanced Impact

- [x] Modal synthesis engine: bank of N parallel damped complex resonators
- [x] 5 mode patterns: Harmonic, Beam, Plate, StiffString, Damped
- [x] Material-to-mode mapping for all 10 materials with frequency-dependent damping
- [x] Upgrade Impact to modal response (multi-mode resonance replaces single sin())
- [x] Material interaction: `new_interaction(striker, surface)` constructor
- [x] Velocity-sensitive impacts: `synthesize_velocity(type, velocity)`
- [x] Excitation models: Impulse, NoiseBurst, HalfSine
- [x] Shatter redesign: primary impact + debris cascade (3-8 events)
- [ ] Destruction/fracture: sustained collapse sequences — deferred to v0.4+
- [ ] Karplus-Strong for metallic ping and plucked-string transients — deferred to v0.4+

---

## v0.4.0 (current) — Contact & Surface

- [x] **Footsteps**: 8 terrains (Gravel, Sand, Mud, Snow, Wood, Metal, Tile, Wet)
- [x] Footstep movement types: Walk, Run, Sneak, JumpLand + `trigger_step()`
- [x] **Friction**: Scrape, Slide, Grind with velocity + pressure control
- [x] **Rolling**: Ball, Wheel, Boulder, Barrel with rotation bumps + hollow resonance
- [x] **Foliage**: LeafRustle, GrassSwish (wind-driven + stochastic), BranchSnap (one-shot)
- [x] **Creaking**: Door, Hinge, Rope, WoodStress with tension + speed control
- [x] Shared contact types module with terrain-to-material mapping

---

## v0.5.0 (current) — Aerodynamic

Mechanical sounds → ghurni crate. Doppler math → goonj crate.

- [x] **Whoosh**: Swing, Projectile, Vehicle, Throw — speed-dependent envelope + brightness
- [x] **Wind whistle**: Gap, Pipe, Bottle, Wire — narrow-band SVF resonance + pitch wobble
- [x] **Cloth flapping**: Flag, Cape, Sail, Tarp — Poisson flap events + modal resonance

---

## v0.6.0 (current) — Creature & Fluid

Physical creature sounds and fluid dynamics. Vocal synthesis → prani/svara.

- [x] **Insects**: WingBuzz (AM), CricketChirp (stridulation), CicadaDrone (broadband)
- [x] Insect swarm (1-8 detuned voices)
- [x] **Bird wing flaps**: Small, Medium, Large — periodic filtered noise
- [x] **Bubbles**: Underwater, Boiling, Viscous, Pouring — Minnaert resonance model

---

## v0.7.0 (current) — Enhanced Weather & Water

- [x] Rain surface interaction: SurfaceRain with Terrain-dependent splatter
- [x] Hail synthesis: modal impacts on surfaces, 3 stone sizes
- [x] Snow: muffled filtered noise with terrain interaction
- [x] Underwater ambience: Shallow/Medium/Deep with rumble + bubbles
- [x] Improved waves: Surf with 3-phase breaking cycle (approach/crash/wash)
- [ ] Wind turbulence model with terrain spectral variation — deferred to v0.8+
- [ ] Waveguide wind (extends v0.5 whistle) — deferred to v0.8+
- [ ] Thunder multi-bolt sequences — deferred to v0.8+

---

## v0.8.0 (current) — Hardening & Polish

Combined v0.8 (Real-Time) + v0.9 (API Polish). Voice audibility → dhvani.
RTPC mapping, event scheduling → dhvani/kiran.

- [x] Pre-allocated buffer: Impact excitation buffer moved to struct (zero alloc in process_block)
- [x] Duration validation: all synthesize() methods reject negative/NaN/infinite
- [x] Deterministic replay verified: 9 synths tested for bit-identical output
- [x] Benchmark coverage: all 25 synth types now benchmarked (26 benchmarks)
- [x] Voice management: VoicePool with Oldest/LowestPriority/None steal policies
- [x] LOD: Quality enum (Full/Reduced/Minimal) with scale_modes/scale_rate helpers
- [x] Builder pattern: PrecipitationBuilder, FootstepBuilder, FrictionBuilder
- [x] SIMD-friendly ModalBank: SoA layout for auto-vectorization

---

## v1.0.0 (current) — Release

- [x] API freeze — no breaking changes until v2
- [x] All public types: `#[non_exhaustive]`, serde, Send + Sync verified
- [x] Example programs: weather_scene, forest_ambience, combat_impacts
- [x] Migration guide: MIGRATION.md
- [x] Audit: cargo audit, cargo deny, cargo clippy, cargo doc — zero warnings
- [x] 137 tests, 26 benchmarks, full deterministic replay verified

---

## Design Principles (all versions)

- **No samples**: every sound is synthesized from math
- **No allocations on hot path**: pre-allocate at construction, stream into caller's buffer
- **Deterministic**: seeded RNG, reproducible output
- **`no_std` compatible**: `libm` fallback, `alloc` only
- **Composable**: synthesizers are independent, caller mixes
- **Physical grounding**: models rooted in acoustics, not arbitrary DSP chains
- **Leverage dependencies**: naad for audio primitives — don't reinvent

## Scope Boundaries (sibling crates)

| Domain | Owner | garjan's role |
|---|---|---|
| Vocal synthesis (bird song, growls, speech) | **prani** (via **svara**) | Not garjan's domain |
| Mechanical sounds (engines, gears, motors) | **ghurni** | Not garjan's domain |
| Acoustics (propagation, Doppler, reverb, RT60) | **goonj** | garjan generates source, goonj propagates |
| Audio engine (mixing, buses, scheduling, RTPC) | **dhvani** | garjan exposes params, dhvani maps them |
| Weather physics (rain rate, wind profiles) | **badal** / **pavan** | garjan consumes their outputs as params |
| Creature behavior (when/why sounds trigger) | **jantu** | jantu decides, garjan synthesizes |

## Non-Goals (garjan scope)

These belong in **dhvani** (audio engine) or **kiran** (game engine):

- Spatial audio / 3D positioning / HRTF
- Reverb zones and room acoustics simulation
- Audio bus architecture and mixing
- Compression / limiting on master output
- Speaker layout and format conversion
- Asset management and sound banks
