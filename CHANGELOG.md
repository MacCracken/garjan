# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [2.4.0]

Bounds the size parameters that drive allocation. See
[ADR-0007](docs/adr/0007-bounded-duration-and-sample-rate.md).

### Fixed — unbounded duration / sample rate

`rust-old/src/dsp.rs:39` validated a duration as **positive and finite** and
nothing more, and the port copied that faithfully — so `1e8` seconds passed
validation, then sized a **4.4-trillion-sample** buffer and killed the process
allocating it. The 2.0.2 audit found this and deliberately left it, because
capping is a divergence from the oracle rather than a bug fix.

Three bounds now, in `src/dsp.cyr`:

```cyrius
var GARJAN_MIN_SAMPLE_RATE_HZ = 1;          # below this is not audio
var GARJAN_MAX_SAMPLE_RATE_HZ = 768000;     # highest standard hi-res rate
var GARJAN_MAX_DURATION_S     = 600;        # 10 minutes per synthesize call
var GARJAN_MAX_SAMPLES        = 44100000;   # 1000 s at 44.1 kHz
```

**The sample count is the binding constraint, and it needs its own check** —
neither input alone is sufficient. 600 s is unremarkable at 44.1 kHz (26.5 M
samples) and ruinous at 768 kHz (461 M), so a pair of individually-legal inputs
could still produce an illegal buffer. A new `garjan_validate_sample_count` is
applied at all 20 `*_synthesize` sites right after the count is computed;
`garjan_validate_sample_rate` and `garjan_validate_duration` gained range checks
so every constructor and synthesize inherits them.

`GARJAN_MAX_SAMPLES` was chosen against the allocator rather than by taste:
44.1 M f64 samples is ~353 MB ideally and ~1.05 GB as actually built, because
`*_synthesize` grows its vector by doubling from capacity 16 (~3x overhead,
measured — blocked on a missing stdlib `vec_with_capacity`). That leaves
headroom inside `ALLOC_MAX` (2 GiB).

Deliberately generous — real calls are seconds, not minutes. The bounds are
named constants and the tests reference the names, not literals, so tightening
them later is a one-line change.

**12 new assertions** (791 total), including the composite case: 600 s at
768 kHz is rejected although both inputs are individually in range.

## [2.3.0]

Performance: the `insect` hot spot the 2.1.0 benchmark expansion exposed.
**Bit-identical output** — verified across all 21 synths and, additionally, all
3 insect types at every swarm count 1..8.

### Changed — insect per-sample loop

`insect_process_block` runs its inner loop once per swarm voice: 352,800 times
per second of audio at swarm 8. Three layers of redundant work removed, all of
it pure hoisting — no floating-point reassociation.

- **Per-voice invariants hoisted out of the sample loop.** `freq` and
  `phase_offset` depend only on the voice index, yet were recomputed every
  sample — including an 8-branch `insect_detune` chain. Worse, the CicadaDrone
  branch never reads `freq` at all, so that was pure waste on the slowest type.
  Now computed once per block into two 8-slot stack buffers.
- **The insect type is branched on once, outside the loop**, into three
  specialized loops. This removes a per-voice-per-sample function call and lets
  each branch hoist its own accessor reads (`insect_type`, `mod_rate`,
  `chirp_rate`, `noise_gen`, `shape_filter`).
- **`TAU * mod_rate * t` is voice-invariant** but was rebuilt for all 8 voices
  every sample; likewise `chirp_rate * t` in the cricket branch. Computed once
  per sample now.

| type (swarm 8) | before | after | change |
|---|---|---|---|
| wing buzz | 36.4 ms | 27.4 ms | −24.7% |
| cricket chirp | 24.3 ms | 18.4 ms | −24.3% |
| cicada drone | 49.6 ms | 42.5 ms | −14.3% |

Cicada gains least because it is the type dominated by naad rather than by
garjan's arithmetic.

### Measured, not guessed

The first attempt — hoisting per-voice invariants alone — bought only 2-3%, so
the components were profiled directly before going further. Per 352,800 calls,
against an empty-loop baseline of 0.88 ms: **biquad 11.9 ms, `f64_sin`
10.6 ms, noise 7.1 ms, one accessor read 0.54 ms, and constant construction
~0 ms.**

Two things fell out of that:

- **cycc already constant-folds** `f64_div(f64_from(6), f64_from(10))` — it
  costs the same as an empty loop. Hoisting constants for speed is pointless;
  hoisting accessor reads and invariant sub-expressions is not. This is recorded
  in `BENCHMARKS.md` so the next optimization pass does not repeat it.
- ~30 of the remaining 42.6 ms is per-voice DSP the algorithm genuinely
  requires. Further gains need an algorithmic change (a recurrence oscillator
  rather than a `sin` per voice per sample), which would not be bit-exact.

## [2.2.0]

First of the deliberate-divergence items: out-of-range enum ids are now
rejected instead of silently absorbed. See
[ADR-0006](docs/adr/0006-out-of-range-enum-ids-are-rejected.md).

### Fixed — an invalid enum id silently selected the last variant

Rust's enums made an invalid variant **unrepresentable**; the port carries them
as module-prefixed integers and dispatches with `if`/`elif` chains ending in a
bare `else`. So an out-of-range id did not fail — it returned the **last
variant's table**. `material_properties(99)` returned Ceramic.
`surf_new(0.5)` returned Storm. No error, no log, quietly wrong audio.

**This was live in this repository's own harnesses.** The 2.1.0 benchmark suite,
the `scripts/audio-hash.cyr` bit-exactness oracle and the cross-module
integration suite all passed an f64 where `surf_new` / `underwater_new` expect
an enum id (`F64_HALF`, and a depth in metres). An f64's *bit pattern* is a huge
integer, so every call fell through:

- the "surf renders at every intensity" sweep tested **Storm four times**;
- the "underwater renders at every depth" sweep tested **Shallow three times**;
- the published `surf` and `underwater` benchmark figures measured the wrong
  configuration. Corrected in `BENCHMARKS.md`.

The per-module suites (`tests/surf.tcyr`, `tests/underwater.tcyr`) had it right
all along — the defect was confined to harnesses added during 2.0.5/2.1.0.

### Added

- **`garjan_enum_invalid(id, max_id)`** in `src/error.cyr`, applied at
  **21 public constructors and entry points** and **10 pointer-returning table
  dispatchers**. Bounds are written as the *named* maximum variant
  (`MATERIAL_CERAMIC`, not `9`) so adding a variant cannot silently narrow the
  accepted range. All 30 dispatcher call sites already checked `garjan_is_err`
  (a consequence of the 2.0.3 allocation guard), so no caller changes were
  needed.
- Regression tests pinning the contract, including the specific
  f64-where-an-id-belongs case. **779 assertions** across 33 suites (was 764).

Deliberately not covered: dispatchers returning a bare `f64`
(`lod_mode_factor`, `rain_intensity_amplitude`, `impact_type_force`,
`friction_filter_freq`, `creak_shape_filter_freq`) — a negative return is a
valid value there, so an error code would be ambiguous, and they are reachable
only through an already-validated constructor.

### Verified

`scripts/audio-hash.cyr` shows **19 of 21 synths bit-identical**; the two that
changed are `surf` and `underwater`, which now receive the configuration they
were always labelled with. The guards altered no valid-path behavior.

## [2.1.0]

Parity completion — everything Rust shipped that the port had not. Additive
only: no behavior change to any existing API, and `scripts/audio-hash.cyr`
confirms every synthesizer's output is bit-identical to 2.0.5.

### Added — cross-module integration suite

`tests/garjan.tcyr` was a **two-assertion placeholder** (`assert(1, "true is
true")`) standing in for Rust's 134 integration tests. It is now a real suite:
**2 → 288 assertions**, taking the project from 478 to **764**.

Scoped deliberately rather than transcribing all 134. Many of Rust's tests are
per-type serde round-trips and per-synth constructor checks that the 33
per-module suites already cover; re-porting those would duplicate, not add.
This file owns what no per-module suite *can* assert:

- the **uniform validation contract** across all 21 constructors — a synth that
  forgets `garjan_validate_sample_rate` is invisible to its own module suite if
  that suite only tests the happy path;
- **exhaustive enum variant sweeps** — all 10 materials, all 32 terrain ×
  movement pairs, every intensity / type / size. This is the highest-value part:
  Rust enums became module-prefixed ints, so a dropped variant produces **no
  compile error**, it silently falls into the final `else`;
- **cross-synth relative invariants** — heavier rain louder than light, closer
  thunder louder than distant;
- uniform silence gates, empty-buffer and multi-block streaming, determinism
  replay, builder-vs-direct equivalence, bridge→synth wiring, LOD monotonicity,
  and voice-pool stealing.

### Added — benchmarks, 5 → 26

`tests/garjan.bcyr` now mirrors `rust-old/benches/benchmarks.rs` in full. The
expanded set **overturns the previous optimization priority**:

- **`insect` (swarm of 8) is the hot spot at 50.3 ms/s of audio (~20x
  real-time)** — five times slower than the next-worst synth. The old
  five-benchmark set named `wind` as the worst target; `wind` is mid-table.
- **The old `cloth` figure was measuring near-silence.** Cloth takes a silent
  fast-path until `cloth_set_wind_speed` is called, which the previous harness
  never did — 1.02 ms was an early return, not synthesis. With wind blowing it
  is 2.02 ms. The harness now sets every event-driven synth's driving parameter
  before timing.

See [`BENCHMARKS.md`](BENCHMARKS.md) for the full table.

### Added — examples and audit write-up

- **Five runnable examples** in `docs/examples/`, which previously held only
  `.gitkeep` while `CLAUDE.md` advertised it: `weather_scene`,
  `forest_ambience`, `combat_impacts`, `error_handling`, `logging`, plus a
  README. All build and run.
- **[`docs/audit/2026-08-30-audit.md`](docs/audit/2026-08-30-audit.md)** — the
  2.0.2/2.0.3 sweep written up, satisfying a maturity criterion. It records the
  **refuted** findings too (no undersized stack buffers exist; zero symbol
  collisions with the dependency bundles; no API drift across the 2.0.1 bump),
  since "we checked and it was fine" is worth keeping.

### Fixed — log level semantics were inverted

`sakshi_set_level` is a verbosity **ceiling**: sakshi emits when
`event_level <= configured_level`, and the levels run `SK_FATAL=0` … `SK_TRACE=5`,
so **lower is quieter**. Four call sites used `sakshi_set_level(5)` intending
"quiet" and were in fact selecting the *loudest* setting. Corrected to
`SK_ERROR` in `tests/garjan.tcyr`, `tests/garjan.bcyr`,
`scripts/audio-hash.cyr` and the `error_handling` example; the `logging`
example, which had the semantics backwards in its narrative, was rewritten to
demonstrate them correctly.

Also fixed: `scripts/audio-hash.cyr` passed a `MATERIAL_*` id where
`footstep_new` takes a `TERRAIN_*`. Only the footstep hash changes; the other
20 synths are unaffected.

### Fixed — the math-library attribution was wrong

The port audit recorded that `rust-old/src/math.rs` (an f32 `sin/cos/exp/sqrt/
powf` shim) was "superseded by ganita's f64 transcendentals". That was asserted
from reading `math.rs`'s surface, never verified, and is **wrong**: five of the
six float functions garjan calls — `f64_sin`, `f64_cos`, `f64_exp`, `f64_sqrt`,
`f64_abs` — are **cycc intrinsics**, defined nowhere in `lib/`. ganita supplies
only `f64_pow`, one of its 133 functions.

ganita is still a required dependency, but for the *deps*: hisab needs 19 of its
symbols (including the whole `mat_*` linear-algebra family), goonj 5, naad 2.
Dropping it breaks the build through them, not through garjan.

Accuracy is now measured rather than assumed — 401 trig points over `[0, 4π]`
and 241 `exp` points over `[-12, 0]`, against correctly-rounded libm:
**`f64_exp` and `f64_sqrt` are bit-exact**, `f64_cos` 1.4e-20, `f64_sin`
2.8e-17, `f64_pow` ≤4 ULP. The worst deviation is ~4 billion times smaller than
a 24-bit audio LSB. Recorded in architecture note
[002](docs/architecture/002-where-the-transcendentals-come-from.md), along with
the parity consequence it makes explicit: **the port can never be bit-identical
to `rust-old`**, because Rust computed in f32 (epsilon 1.2e-7) and the port in
f64 (2.2e-16). Parity with the oracle is structural, not sample-for-sample —
which is why `GARJAN_EPSILON` is deliberately `f32::EPSILON`.

Corrected in `cyrius.cyml`, `docs/development/roadmap.md` and
`docs/development/state.md`.

### Fixed — port-audit claims re-verified

Prompted by the ganita finding, every claim in the port audit was re-checked.
Most held; three did not.

- **Enum parity is now verified by VALUE, not just by name.** The original check
  only confirmed a constant existed whose name ended with the variant name — it
  never compared discriminants, and for five enums with colliding variant names
  (`Small`/`Medium`/`Large`, `Moderate`/`Heavy`) it could have matched the wrong
  family entirely. Re-checked: **all 106 variants ported, 103 preserving Rust's
  exact discriminant**; the 3 exceptions are `GarjanError`, deliberately
  remapped to negative codes. `StoneSize`, `UnderwaterDepth`, `BirdSize`,
  `SurfIntensity`, `RainIntensity` and `Material` were pinned by exact prefix.
- **The item-level audit silently skipped `integration/`.** Its extractor used
  `glob('*.rs')` rather than `rglob`, so the reported "219 public items checked"
  never covered `integration/soorat.rs`. Actual coverage is ~223 of 242; soorat
  was known to be unported only from a separate file-existence check.
- **The module count omitted the dropped fallback.** Beyond the 4 unported
  files, **~232 lines of `#[cfg(not(feature = "naad-backend"))]` fallback across
  19 ported modules** were also dropped — deliberately
  ([ADR-0002](docs/adr/0002-dual-code-paths.md)), but "30 of 34 modules" implied
  more completeness than that.
- **`VoicePool::active_voices` is a genuine API-shape gap**, not the clean
  equivalence originally implied: Rust returned an iterator of active
  `(index, &VoiceSlot)` pairs and Cyrius has no iterators. Reachable by looping
  `voice_pool_slot` + `VoiceSlot_active`. (`slot_mut` *is* properly covered —
  `voice_pool_slot` is bounds-checked and returns 0, matching `.get_mut()`'s
  `None`.)

Confirmed sound on re-check: `lib.rs` contains no behavior (module declarations,
a prelude, and a `#[cfg(test)]` Send+Sync assertion); `integration/mod.rs` is 7
trivial lines; soorat is genuinely absent from the port; there are no
`pub const`/`static`/`trait`, no manual trait impls and no `Default` derive, so
the behavioural surface really is fns/structs/enums; and `DcBlocker`'s ported
formula and clamp bounds are bit-exact (`0x3FECCCCCCCCCCCCD` == 0.9,
`0x3FEFFF2E48E8A71E` == 0.9999).

The [verification recipe](docs/development/roadmap.md#port-completeness) was
rewritten around the checks that actually caught these.

### Deferred — output-vector pre-sizing is blocked upstream

Every `*_synthesize` builds its output by pushing from capacity 16. Measured:
**1,048,472 bytes retained to produce one second of audio, against an ideal of
352,824** — ~3x overhead, ~695 KB wasted per call, permanently (the arena never
frees). That growth *is* the entire allocation cost of a `synthesize`.

It is **not fixed here**, because it cannot be fixed cleanly from garjan. The
Cyrius stdlib has no `vec_with_capacity` or `vec_reserve` — `lib/vec.cyr`
exposes only `vec_new()`, which hardcodes capacity 16. The alternative is to
hand-construct the vector header from garjan, which means coupling to
`lib/vec.cyr`'s private `[data][len][cap]` layout; an upstream layout change
would then corrupt silently, and `CLAUDE.md` forbids modifying `lib/`. The
right fix is a stdlib addition. Recorded with the measurement so the case for
it is concrete.

## [2.0.5]

Per-sample hot-path optimization. **Every change is bit-exactness-preserving**,
verified by diffing a raw-bit hash of every synthesizer's output before and
after — not inferred from the suite passing.

### Changed — loop-invariant work hoisted out of per-sample loops

Only hoisting and redundant-pass elimination. **No floating-point
reassociation**, which would have changed results.

- **`wind_process_block`, `texture_process_block`** — folded the second
  DC-blocking pass into the generation loop. Both write each index exactly
  once, in order, so the blocker sees identical inputs in identical order;
  this removes a `vec_get` + `vec_set` per sample plus a whole extra traversal.
  The other ten synths with a separate DC pass were left alone: they accumulate
  (fire adds crackle over roar, rain adds drops), so the pass genuinely has to
  come last.
- **`thunder_process_block`** — hoisted `crack_div` and its f64 conversion,
  `-10.0`, `-2.0`, `sr*decay_time`, and the 0.8/0.6 gains. Also dropped a
  `vec_get` of a slot just written to 0, keeping it as `f64_add(0, x)` rather
  than a plain store so a `-0.0` contribution still normalises exactly as before.
- **`surf_process_block`** — eleven envelope breakpoints (0.2/0.25/0.3/0.45/0.55),
  the `-3.0` gain and the jitter range were rebuilt every sample, several twice.
  The `1/period` reciprocal is now recomputed only on a wave-cycle boundary,
  where `period` actually changes, instead of once per sample.
- **`fire`, `rain`, `cloth`** — per-event decay lengths converted to f64 once per
  event instead of once per sample of the tail; block `width` hoisted out of the
  event loop; amplitude/duration ranges and envelope constants hoisted.
- **`water_process_stream`** — the 0.7/0.3 modulator terms, matching what the
  sibling `water_process_waves` already did.
- **`rolling_process_block`** — `rotation_rate` and the phase increment are
  fixed before the loop (velocity, radius and sr are all locals), so two divides
  and a multiply per sample are gone.
- **`friction_process_block`** — `velocity`/`pressure` read once instead of
  through accessors per sample; neither is written in the loop.
- **`footstep_process_block`** — `modal_bank` read once. `footstep_fire_step`
  resets the bank's *state* but never reassigns the pointer, so it is invariant.

### Results

Median of 5 runs, against the median of the 2.0.2-2.0.4 runs:

| Synth   | Before   | After    | Change |
|---------|----------|----------|--------|
| wind    | 10.95 ms | 10.34 ms | −5.6%  |
| thunder | 5.43 ms  | 5.25 ms  | −3.3%  |
| fire    | 4.46 ms  | 4.36 ms  | −2.2%  |
| rain    | 3.74 ms  | 3.69 ms  | −1.3%  |
| cloth   | 1.02 ms  | 1.02 ms  | ~0     |

Modest, and worth stating plainly: **most of the remaining time is inside the
naad bundle** — per-sample noise generation and biquad/SVF filtering — not in
garjan's own arithmetic. The largest win (wind) came from deleting a whole pass
over the buffer, not from hoisting constants. `cloth` did not move because its
flap events are sparse, so the inner loop it optimises rarely runs. Further
gains need naad-level or algorithmic work, not more hoisting.

### Added

- **`scripts/audio-hash.cyr`** — the bit-exactness oracle used for this work.
  Prints a rolling hash of the raw f64 bit patterns of every synth's output over
  three successive blocks, so any last-ulp or ±0.0 change is visible. Capture
  before, change, rebuild, diff. Event-driven synths are excited first, because
  an unexcited synth outputs silence and hashes to 0 — which would make the
  oracle vacuously pass. Hashes are deliberately **not** committed as golden
  values: they pin one toolchain, dep set and architecture.

**478 assertions** across 33 suites, unchanged and all green.

## [2.0.4]

Arena-lifetime fix — the item 2.0.3 left open. 2.0.3 made heap exhaustion
*detectable*; this release stops the streaming paths from marching toward it.
Audio output is bit-identical, verified by direct equivalence tests rather than
inferred from the suite passing.

### Fixed — the two streaming paths that grew the arena on every block

The bump allocator never frees, so anything allocated per block accumulates for
the life of the process. Two hot paths did exactly that. Both were *faithful*
ports — Rust allocated per call too — but Rust's values were dropped, and here
they are not.

- **`impact_generate` built a fresh `Exciter` (and its `Rng`) on every call** —
  **64 bytes per block**. Mirrors `rust-old/src/impact.rs:194`
  (`let mut exciter = Exciter::new(...)` inside the generate loop). `Impact` now
  caches one exciter, built alongside the other reconstructable components in
  `impact_build_naad`, and reconfigures it per call via the new
  `exciter_reset`.
- **`texture_process_block` allocated a 24-byte band-mix buffer every block** —
  **24 bytes per block**. The table is now written through
  `texture_band_mix_into(out, texture_type)` into a stack buffer
  (`var mixbuf[24]` — 24 bytes = three f64 slots). The owning
  `texture_band_mix` is retained for API compatibility and delegates to the
  same helper, so there is one definition of the table.

Measured over 200 `process_block` calls: impact **12,800 -> 0 bytes**, texture
**4,800 -> 0 bytes**. At 44.1 kHz with 512-sample blocks (~86 blocks/s) that was
roughly 27 MB/hour between them, which reaches the 2 GiB `ALLOC_MAX` in about
three days of continuous streaming.

### Added

- **`rng_seed(self, seed)`** (`src/rng.cyr`) — re-seeds an existing `Rng` in
  place, reproducing exactly what `rng_new(seed)` would have produced. Factored
  *out of* `rng_new`, which now calls it, so there is a single definition of the
  seeding sequence and the two cannot drift.
- **`exciter_reset(self, type, duration, amplitude)`** (`src/modal.cyr`) —
  equivalent to `exciter_new` + `exciter_trigger` minus both allocations. It
  re-seeds the exciter's rng to the new `EXCITER_RNG_SEED` constant (31337,
  now shared with `exciter_new`), which is what makes reuse bit-exact:
  without it a `NOISE_BURST` excitation would drift from block 2 onward.
- **`texture_band_mix_into(out, texture_type)`** (`src/texture.cyr`).

### Verified

Bit-exactness is asserted directly, not assumed:

- `exciter_reset` reproduces `exciter_new` + `exciter_trigger` sample-for-sample
  across all three excitation types, and a *reused* exciter matches a freshly
  built one on every repeat (`tests/modal.tcyr`).
- `rng_seed(s)` reproduces `rng_new(s)`'s stream (`tests/rng.tcyr`).
- `texture_band_mix` agrees with `texture_band_mix_into` for every texture type
  (`tests/texture.tcyr`).
- 50 consecutive `process_block` calls allocate **0 bytes** on both the impact
  and texture paths (`tests/impact.tcyr`, `tests/texture.tcyr`).

**478 assertions** across 33 suites (was 472). Benchmarks unchanged within noise.

### Still open

Arena growth is fixed for the *streaming* paths only. Every `*_synthesize` still
builds its output vec by pushing from capacity 16 (about a dozen doubling
reallocations per second of audio, all of them permanently retained), and the
allocator still has no free at all — a caller that constructs and discards
synths in a loop grows the arena regardless. Both are lifetime problems, not
detection problems.

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
