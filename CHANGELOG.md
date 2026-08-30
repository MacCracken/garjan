# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [2.0.1]

Maintenance release: toolchain, dependency, and vendored-stdlib refresh. No
intentional behavior change — the two source edits are mechanical renames
forced by upstream, both verified value-for-value against the old bundles.

### Changed
- **Cyrius toolchain pin 6.3.44 → 6.5.36** (`cyrius.cyml [package].cyrius`).
- **Dependencies bumped to latest releases** — naad 2.1.0 → **2.2.2**,
  hisab 2.6.7 → **2.11.2**, goonj 2.0.0 → **2.0.4**, sakshi 2.4.3 → **2.4.11**.
  The graph is self-consistent at these tags: naad 2.2.2 pins hisab 2.11.2 +
  goonj 2.0.4, goonj 2.0.4 pins hisab 2.11.2, and hisab 2.11.2 pins
  sakshi 2.4.11 — so garjan's transitive re-declarations match what each
  dependency consumes upstream.
- **Vendored stdlib re-synced** to the 6.5.36 snapshot; `lib/tagged.cyr` and
  `lib/callback.cyr` are newly vendored (pulled in as leaf requirements by
  naad 2.2.2 / hisab 2.11.2).

### Fixed (upstream renames)
- **`FILTER_*` → `NAAD_FILTER_*`** (30 call sites across 17 synth modules).
  naad 2.1.3+ prefixed its filter-mode constants as part of its cross-library
  de-collision work; the bare names were a hard compile error against 2.2.2.
  Verified a pure rename: both bundles define LOWPASS/HIGHPASS/BANDPASS/NOTCH/
  ALLPASS/LOWSHELF/HIGHSHELF/PEAK as 0-7 in that order, and
  `filter_biquad_new(filter_type, sample_rate, frequency, q)` is byte-identical
  — so no filter changes type or response.
- **`bayan_json_v_parse_str` → `bayan_json_v_parse_buf`** (`src/voice.cyr`).
  The stdlib renamed the cstr+len parse entry point because Cyrius routes
  `X(str, ...)` to `X_str` when that symbol exists, which was silently
  rewriting `bayan_json_v_parse(someStr)` into a 1-arg call to the 2-arg
  function and returning 0 for valid JSON. Same signature, drop-in.

### Verified
- Build clean; **33 test suites / 460 assertions, all green**; fuzz harness
  (`tests/garjan.fcyr`) passes; benchmarks re-measured (see `BENCHMARKS.md` —
  every synth is marginally faster on the new toolchain).
- `cyrius lint` 0 warnings across all 33 modules; `cyrius vet` clean;
  `cyrius distlib --check` in sync.
- **Symbol-collision audit**: garjan's 399 top-level `fn`/`var` symbols
  intersect naad, hisab, goonj, sakshi, bayan, ganita and math at **zero** in
  all directions. This matters because Cyrius emits no diagnostic for a
  duplicate top-level `var`, so a collision would shadow silently.
- **API drift audit**: every library function garjan calls was compared
  old-bundle vs new-bundle — no arity changes, nothing removed, and the final
  build emits no undefined-symbol warnings.

## [2.0.0] - Cyrius port

Full port of the crate from Rust to the **Cyrius** language (AGNOS ecosystem
migration). Major bump: the entire public surface moves from the Rust API to
the Cyrius `fn` API (constructors return pointers / negative error codes), so
this is a breaking change for every consumer. The original Rust sources are preserved under `rust-old/`; the
Cyrius port lives in `src/*.cyr` with per-module parity suites in `tests/*.tcyr`.
Build with `cyrius build src/main.cyr build/garjan`; test with `cyrius test`.

### Ported
- **32 modules, 6,186 lines of Rust → Cyrius.** Foundations (error, dsp, rng,
  lod, material, modal, contact, aero, creature, voice) + all 19 synthesizers
  (fire, weather [thunder/rain/wind], water, precipitation, bubble, impact,
  footstep, friction, creak, rolling, foliage, whoosh, whistle, cloth, insect,
  wingflap, underwater, surf, texture) + `builder` + `bridge`.
- **`f32` → `f64` throughout** (naad/hisab are f64-only); all float ops are
  explicit calls (`f64_add`/`f64_mul`/…). Enums → module-prefixed int constants.
- **PCG32 RNG ported bit-exactly** — verified against the Rust sequence, so all
  stochastic synthesis is deterministic across the port.
- **`Result<T>` → integer error codes** (`GARJAN_OK` / negative `GARJAN_ERR_*`);
  constructors return a heap pointer or a negative code.

### Wired (deliberately kept, unlike the naad port which dropped them)
- **Logging via [sakshi](https://github.com/MacCracken/sakshi)** — `src/logging.cyr`
  wraps `sakshi_warn`/`info`/`debug`/`error`/`fatal`; the `dsp` validators emit
  through it (the Rust `tracing::warn!` sites). Always compiled, runtime-gated by
  `sakshi_set_level`.
- **Serde via `#derive(Serialize)`** (Cyrius 6.3.44) — full `to_json`/`from_json`
  roundtrip on every enum, param/config struct, all 19 synthesizers (through a
  flattened `*Params` slice with naad components reconstructed on deserialize,
  mirroring Rust's `#[serde(skip)]`), the builders, and `VoicePool` (hand-rolled
  vec-of-`VoiceSlot` codec). `ModalBank`/`Exciter` are reconstructable DSP
  components, rebuilt from their inputs rather than serialized.

### Dependencies
- naad 2.1.0, hisab 2.6.7, goonj 2.0.0, sakshi 2.4.3 (git-pinned in `cyrius.cyml`).
  garjan consumes naad's noise/filter/LFO surface from the monolithic dist bundle.

### Deferred
- `integration/soorat.rs` (visualization data, feature-gated `soorat-compat`,
  no Cyrius consumer yet) remains in `rust-old/`; port it once soorat lands.

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
