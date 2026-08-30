# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [2.0.3]

Completes the allocation-failure hardening deferred from 2.0.2. No behavior
change for valid input — all 33 suites pass unchanged, and the only new failure
mode is one that previously corrupted memory instead.

### Fixed — a failed allocation was indistinguishable from success

The stdlib `alloc` signals exhaustion by returning `0`, `garjan_is_err` treats
only `< 0` as an error, and `GARJAN_OK` **is** `0` — so `garjan_is_err(alloc(n))`
was `0` on failure and every error check passed. Constructors write through
their allocation on the very next line, so an OOM was a store to address 0.

- **New `garjan_alloc(size)` guard** in `src/error.cyr`, mapping `alloc`'s `0`
  onto the new `GARJAN_ERR_ALLOCATION` (`-4`) so the existing
  `if (garjan_is_err(p) == 1) { return p; }` idiom catches it. Deliberately does
  not log: `error.cyr` is L0 and the `aero`/`creature`/`rng`/`voice` test entries
  include it *without* `logging.cyr`, and a reachable undefined function is a
  hard build error since cyrius 6.5.36.
- **All 85 direct allocation sites** routed through the guard and checked.
- **~70 further sites** where a pointer-returning helper was consumed unchecked
  — `rng_new`, `garjan_dcblocker_new`, `voice_slot_new`, `modal_modespec_new`,
  `exciter_new`, `modal_bank_new`, `texture_band_mix`, `generate_modes` and the
  `*_config` table builders. Nested uses like `Fire_set_rng(self, rng_new(6661))`
  and `vec_push(slots, voice_slot_new())` were hoisted into checked locals; a
  partial guard would have been worse than none, since it looks safe while OOM
  still crashes elsewhere. 132 guards inserted in all.
- `material.cyr`'s table dispatchers needed no change — they are bare
  `return material_*_new(...)`, which already propagate.
- `src/main.cyr` checks and reports rather than returning a raw negative code,
  which would have become a bogus process exit status.

**Invariant to preserve:** `src/*.cyr` contains no raw `alloc(` outside
`garjan_alloc` itself. See ADR-0005 for the full rationale, including why
aborting (which is what Rust actually does on OOM) was rejected.

### Added

- **ADR-0005** — *Allocation failure is an error code, not an abort*, in
  `docs/adr/`, per CLAUDE.md's rule that divergence from the Rust oracle needs
  one. Rust's allocator aborts the process and `GarjanError` has exactly three
  variants, so adding a fourth is a real divergence rather than a bug fix.
- Regression tests in `tests/error.tcyr` pinning the distinction between raw
  `alloc` (undetectable failure) and `garjan_alloc` (detectable).
  **472 assertions** across 33 suites (was 466).

### Note on verification

The guard is verified directly — `alloc(-1)` returns `0` with
`garjan_is_err == 0`, while `garjan_alloc(-1)` returns `-4` with
`garjan_is_err == 1` — and coverage is verified mechanically (no unguarded
allocation or helper result remains in `src/`). End-to-end propagation under
*real* heap exhaustion was **not** exercised, since inducing it would mean
exhausting the machine's memory.

## [2.0.2]

Security and hardening release from a P-1 audit sweep. The deserialization
surface (`*_from_json_str`) was the weak point: it is the one set of entry
points that takes fully attacker-controlled input, and it was skipping the
validation its sibling constructors perform. No behavior changes for valid
input — all 33 suites pass unchanged.

### Fixed — security (all reachable from untrusted JSON)

- **`*_from_json_str` discarded the `*_build_naad` error code — 20 of 21 sites.**
  The `*_new` path has always checked it (`var b = X_build_naad(self); if
  (garjan_is_err(b) == 1) { return b; }`); the deserialize path called it bare.
  `*_build_naad` returns *before* assigning the naad component pointers when a
  constructor rejects its input, and `*_from_json_str` never validates the
  JSON-supplied `sample_rate` the way `*_new` does. So a well-formed document
  with `"sample_rate":0.0` returned a **non-negative pointer** whose component
  fields were left 0 — the caller's `garjan_is_err` check passed, and the first
  `process_block` dereferenced null. Confirmed as a SIGSEGV against 2.0.1 and
  re-confirmed as a clean `GARJAN_ERR_SYNTHESIS_FAILED` after the fix.
  `foliage` was the single site that already did this correctly.
- **`voice_pool_from_json_str` sized the pool from `max_voices`, not the slots
  array.** Rust derives `Deserialize` on `slots: Vec<VoiceSlot>`, so serde
  allocates exactly as many slots as the array carries and never calls
  `VoicePool::new`. The port passed the raw `max_voices` scalar into
  `voice_pool_new`, whose push loop allocates one `VoiceSlot` per unit against
  an arena that never frees — so `{"max_voices":2000000000,"slots":[]}`, about
  40 bytes, exhausted memory. Now bounded by the array length, which is a no-op
  on any well-formed round-trip and restores parity with Rust.
- **`insect_from_json_str` restored `swarm_count` without its 1..8 clamp.**
  Every other path enforces that invariant, and the eight `det0..det7` fields
  physically encode it. `swarm_count` bounds the per-sample inner voice loop, so
  an unclamped value made one `process_block` effectively unbounded — with
  `insect_detune` saturating every index >= 7 to `det7` instead of trapping, so
  nothing downstream noticed.

### Fixed — build hygiene

- **`cyrius audit` failed its fmt gate.** Root cause: `cyrius fmt --check`
  **false-negatives**. It reported `src/builder.cyr` clean while the formatter
  itself rewrote 20 lines of it. Reformatted (whitespace only). Note for future
  work: verify formatting by running the formatter into a scratch copy and
  diffing, not by trusting `--check`.

### Added

- Regression tests for all three security fixes, in `tests/fire.tcyr`,
  `tests/voice.tcyr` and `tests/insect.tcyr`, each driving a genuinely hostile
  document rather than a round-tripped one. **466 assertions** across 33 suites
  (was 460).

### Known, deliberately not fixed here

- **A failed allocation is indistinguishable from success.** `alloc` returns `0`
  on exhaustion (`lib/alloc.cyr`), but `garjan_is_err` only treats `< 0` as an
  error and `GARJAN_OK` *is* `0` — so `garjan_is_err(alloc(...)) == 0` on
  failure. All ~85 `alloc(sizeof(X))` sites are unchecked and constructors write
  through the result immediately, making OOM a null-pointer store rather than a
  clean error. Reachability is low (it needs genuine memory exhaustion), and the
  fix touches every constructor, so it is deferred rather than bundled into a
  patch release. Tracked for 2.1.0.
- **No upper bound on `duration` or `sample_rate`.** `validate_duration` accepts
  any positive finite value, so 1e8 seconds sizes a 4.4-trillion-sample buffer.
  This is *faithful* to the oracle — `rust-old/src/dsp.rs:39` validated only
  positive-and-finite too — so adding a cap is a deliberate divergence and needs
  an ADR rather than a unilateral patch.
- **Per-sample hot-path hoisting** (loop-invariant constants rebuilt per sample
  across ~17 modules, and the second DC-blocking pass in the block synths).
  Behavior-preserving but broad; it belongs in its own change with before/after
  benchmarks, not in a security release.

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
