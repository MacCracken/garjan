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

## v0.5.0 — Aerodynamic & Mechanical

Motion through air, machines.

- [ ] **Whoosh**: object pass-by synthesis (sword swing, projectile, vehicle)
- [ ] Doppler-aware whoosh (pitch contour from approach/retreat)
- [ ] Wind whistling through gaps/openings (waveguide model)
- [ ] Flag/cloth flapping
- [ ] **Engine**: combustion cycle synthesis, RPM-driven, exhaust resonance
- [ ] **Motor**: electric hum, servo whine
- [ ] **Gears**: clicking, meshing, grinding
- [ ] **Steam**: hissing, venting, pressure release
- [ ] **Electrical**: arc, transformer hum, buzzing

---

## v0.6.0 — Creature & Organic

Living sounds.

- [ ] **Insects**: wing buzz (frequency-modulated), chirping (crickets, cicadas)
- [ ] Insect swarm (granular: many overlapping micro-events)
- [ ] **Birds**: FM chirps, trills, frequency-modulated songs, wing flaps
- [ ] **Vocalization**: glottis-based model for growls, howls, purrs, roars
- [ ] Vocal parameter space: size, tension, breathiness, pitch
- [ ] **Bubbles**: underwater, boiling, viscous fluid, pouring

---

## v0.7.0 — Enhanced Weather & Water

Deeper physical models for existing modules.

- [ ] Rain surface interaction: splatter character varies by surface material
- [ ] Hail synthesis
- [ ] Wind: turbulence model with spectral variation by terrain
- [ ] Waveguide wind for pipes, gaps, building edges
- [ ] Snow/ice cracking and crunching
- [ ] Underwater ambience (muffled, resonant, pressure-dependent)
- [ ] Improved waves: surf zone model with breaking wave phases
- [ ] Thunder: multi-bolt sequences, rolling echo from terrain

---

## v0.8.0 — Real-Time & Performance

Production-ready runtime behavior.

- [ ] Voice management: priority system, voice stealing, max polyphony
- [ ] Virtual voices: track inaudible sources without synthesizing
- [ ] LOD: simplified synthesis models for distant/quiet sources
- [ ] Pre-allocated buffer pools: zero allocation during synthesis
- [ ] Object pool for transient events (drops, crackles, debris)
- [ ] Deterministic replay: same seed + parameters = identical output (verify)
- [ ] SIMD-friendly buffer layouts (leverage naad's vectorization)
- [ ] Benchmark all new modules, profile hot paths

---

## v0.9.0 — API Hardening & Polish

Ergonomics, safety, completeness.

- [ ] Builder pattern constructors for all synthesizers
- [ ] Comprehensive parameter validation at construction time
- [ ] Graceful degradation: reduce quality under CPU pressure, never panic
- [ ] Full serde save/restore of mid-synthesis state
- [ ] Crossfade utilities: equal-power transitions between sound states
- [ ] RTPC-style parameter mapping: game value -> synthesis parameters
- [ ] Event system: trigger one-shot sounds, schedule sequences
- [ ] Complete documentation with acoustic rationale for each model

---

## v1.0.0 — Release

- [ ] API freeze — no breaking changes until v2
- [ ] All public types: `#[non_exhaustive]`, `#[must_use]`, serde, Send + Sync
- [ ] Full rustdoc with examples for every public item
- [ ] Example programs: weather scene, forest ambience, combat impacts, vehicle
- [ ] Performance optimization pass: all benchmarks baselined
- [ ] `cargo fuzz` targets for all public API entry points
- [ ] Migration guide from 0.x series
- [ ] Audit: `cargo audit`, `cargo deny`, `cargo clippy`, zero warnings

---

## Design Principles (all versions)

- **No samples**: every sound is synthesized from math
- **No allocations on hot path**: pre-allocate at construction, stream into caller's buffer
- **Deterministic**: seeded RNG, reproducible output
- **`no_std` compatible**: `libm` fallback, `alloc` only
- **Composable**: synthesizers are independent, caller mixes
- **Physical grounding**: models rooted in acoustics, not arbitrary DSP chains
- **Leverage dependencies**: hisab for math, naad for audio primitives — don't reinvent

## Non-Goals (garjan scope)

These belong in **dhvani** (audio engine) or **kiran** (game engine):

- Spatial audio / 3D positioning / HRTF
- Reverb zones and room acoustics simulation
- Audio bus architecture and mixing
- Compression / limiting on master output
- Speaker layout and format conversion
- Asset management and sound banks
