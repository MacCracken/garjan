# garjan — Roadmap

> Milestone plan for the 2.x line. State lives in [`state.md`](state.md);
> this file is the **sequencing** — what ships, in what order, against what
> dependency gates.
>
> **Numbering note.** The pre-port file planned a v0.1.0 → v1.0 line. The
> Rust→Cyrius port took a major bump to **2.0.0** (the public surface moved
> language), so that scaffolding no longer described this project. It has been
> replaced with the 2.x arc below. v1.0 criteria are retained, re-scoped, as
> **2.x maturity criteria** — they were never met and are still the right bar.

## Where the port actually stands

Verified 2026-08-30 and **re-verified 2026-08-30** after the first pass was
found to have asserted things it had not checked. Method in
[Port completeness](#port-completeness).

- **30 of 34** `rust-old/src` modules have a `.cyr` counterpart.
- **All 106 enum variants are ported, and 103 of them preserve Rust's exact
  discriminant value.** This is the check that matters: Rust enums became
  module-prefixed ints, so a variant that survived by *name* but landed on the
  wrong *value* would silently return the wrong material/terrain/intensity
  table with no compile error. The 3 exceptions are `GarjanError`'s variants,
  deliberately remapped from 0/1/2 to negative codes (`GARJAN_ERR_*`) because
  the port signals failure by sign.
  - Five enums share variant names (`Small`/`Medium`/`Large`,
    `Moderate`/`Heavy`), so `StoneSize`, `UnderwaterDepth`, `BirdSize`,
    `SurfIntensity`, `RainIntensity` and `Material` were additionally pinned
    against their exact constant prefix — a name-suffix match alone could have
    matched the wrong family.
- **~223 of 242** public `fn`/`struct`/`enum` items were compared. The
  remainder is `math.rs` (10 — superseded, see architecture note
  [002](../architecture/002-where-the-transcendentals-come-from.md)) and
  `integration/soorat.rs` (9 — the known deferral).
  - ⚠ The first pass reported "219 items" and implied full coverage. Its
    extractor used `glob('*.rs')` rather than `rglob`, so it **silently skipped
    `integration/` altogether**. soorat was known to be unported only from a
    separate file-existence check. Use `rglob`.
- **No `pub const`, `pub static`, `pub trait`, or manual `impl … for …`** exist
  in `rust-old/src`, and no `Default` derive — so the behavioral surface really
  is fns, structs and enums. One `pub type` (`Result<T>`), replaced by integer
  error codes.
- Not ported, **correctly**:
  - `lib.rs` — verified to contain only `mod`/`pub mod` declarations, a
    `prelude` of re-exports, and a `#[cfg(test)]` Send+Sync assertion. No
    behavior. (Note `dsp`, `math` and `rng` were *private* modules in Rust; the
    port exposes their functions in the flat namespace, so the port's surface
    is slightly *wider* than the crate's.)
  - `math.rs` — an f32 `sin/cos/exp/sqrt/powf` shim. Superseded by cycc's f64
    intrinsics plus ganita's `f64_pow`; accuracy measured, see note 002.
  - `integration/mod.rs` — 7 lines: a doc comment and a `#[cfg]`-gated
    `pub mod soorat`.
- **Also dropped, deliberately: ~232 lines of non-naad fallback code across 19
  ported modules.** Rust carried dual code paths ([ADR-0002](../adr/0002-dual-code-paths.md));
  the port always uses the naad backend and drops the `#[cfg(not(feature =
  "naad-backend"))]` branch. The module-level "30 of 34" figure does not capture
  this — the port is "30 of 34 files, minus the fallback branch in 19 of them".
  Only 7 of those 19 modules say so in their header; the rest state it only
  indirectly.
- **`VoicePool::active_voices` has no direct equivalent.** Rust returned
  `impl Iterator<Item = (usize, &VoiceSlot)>` filtered to active slots; Cyrius
  has no iterators. The capability is reachable by looping `voice_pool_slot(p, i)`
  and testing `VoiceSlot_active`, and `voice_pool_active_count` gives the count —
  but it is an API-shape gap, not a like-for-like port. `slot_mut` *is* covered:
  `voice_pool_slot` is bounds-checked and returns 0 for out-of-range, matching
  `.get_mut()`'s `None`, and Cyrius pointers are mutable.
- Outstanding: `integration/soorat.rs` (315 lines), and — until 2.1.0 —
  `examples/` and `tests/integration.rs`.

## 2.x maturity criteria

- [x] Rust → Cyrius surface parity verified (function-level diff against `rust-old/`)
- [x] Test coverage adequate for the surface area — 2.1.0 replaced the
      two-assertion placeholder with a 288-assertion cross-module suite (764
      total), and benchmarks now cover all 26 ops rather than 5
- [x] Benchmarks captured — in [`BENCHMARKS.md`](../../BENCHMARKS.md) at the
      repo root (the old criterion said `docs/benchmarks.md`; the root file is
      the real one and the criterion was corrected, not the file moved)
- [ ] At least one downstream consumer green — **none yet**
- [x] CHANGELOG complete from 2.0.0 onward
- [x] Security audit written up — [`docs/audit/2026-08-30-audit.md`](../audit/2026-08-30-audit.md)

---

## 2.0.x — stabilization ✅ mostly shipped

Patch line. No API changes, no audio changes.

| Version | Status | Content |
|---|---|---|
| 2.0.0 | ✅ 2026-07-03 | Full Rust→Cyrius port; 32 modules |
| 2.0.1 | ✅ 2026-08-29 | Toolchain 6.3.44 → 6.5.36; deps to latest |
| 2.0.2 | ✅ 2026-08-30 | P-1 audit: 3 JSON-reachable defects; `cyrius audit` fmt gate |
| 2.0.3 | ✅ 2026-08-30 | Allocation-failure guard ([ADR-0005](../adr/0005-allocation-failure-is-an-error-code-not-an-abort.md)) |
| 2.0.4 | ✅ 2026-08-30 | Arena lifetime: per-block allocations → zero |
| 2.0.5 | ✅ 2026-08-30 | Per-sample hot paths; bit-exact (`scripts/audio-hash.cyr`) |

### 2.0.6 — documentation accuracy ✅ (docs only, shipped untagged)

Nothing here changed behavior, so it rides along with 2.0.5 rather than taking
a tag of its own.

- ✅ **Removed the false `#[serde(skip)]` premise** from 35 comment sites across
  17 modules. `rust-old/src` contains zero `serde(skip)`; Rust derived
  serialization over *all* fields, naad components included. The authoritative
  explanation now lives in architecture note
  [001](../architecture/001-deserialize-does-not-restore-dsp-state.md), and the
  comments point at it instead of restating a false premise 35 times.
- ✅ **Filled `CLAUDE.md`'s `## Goal`** — grounded in
  [ADR-0003](../adr/0003-scope-boundaries.md)'s boundary table rather than
  invented: garjan owns environmental/nature sound *sources*; propagation is
  goonj, mixing dhvani, vocal prani/svara, mechanical ghurni.
- ✅ **Populated `docs/architecture/README.md`**'s empty index with note 001.
- ✅ **Resolved the ADR location split** — `adr-001`..`adr-004` moved from
  `docs/architecture/` to `docs/adr/0001`..`0004`, the home both `CLAUDE.md`
  and the two READMEs already declared. Renamed to the four-digit convention;
  **not** renumbered and **not** reformatted (they predate the template and
  reference the pre-port Rust version line — rewriting them would blur what was
  decided when). Inbound links updated; all 34 relative doc links verified.

---

## 2.1.x — parity completion ✅ shipped 2.1.0

Everything Rust shipped that the port had not. Additive; no behavior change to
existing APIs.

- ✅ **Ported `tests/integration.rs` → `tests/garjan.tcyr`.** The placeholder
  (`assert(1, "true is true")`) is replaced by a real cross-module suite:
  **2 → 288 assertions**, taking the project from 478 to **764** overall.
  Scoped deliberately: Rust's 134 tests include many per-type serde
  round-trips and per-synth constructor checks the 33 per-module suites
  already cover, so re-porting them verbatim would duplicate rather than add.
  This file owns what no per-module suite *can* assert —
  the uniform validation contract across all 21 constructors, exhaustive enum
  variant sweeps (all 10 materials, all 32 terrain × movement pairs, every
  intensity/type/size), cross-synth relative invariants (heavier rain louder
  than light; closer thunder louder than distant), the uniform silence gates,
  empty-buffer and multi-block streaming, determinism replay, builder-vs-direct
  equivalence, bridge→synth wiring, LOD monotonicity, and voice-pool stealing.
  The variant sweeps matter most: Rust enums became module-prefixed ints, so a
  dropped variant produces no compile error — it silently falls into the final
  `else`.
- ✅ **Ported `examples/` (5 programs) → `docs/examples/`** with a README —
  `weather_scene`, `forest_ambience`, `combat_impacts`, `error_handling`,
  `logging`. All build and run.
- ✅ **Expanded `tests/garjan.bcyr` from 5 to all 26 benchmarks.** This
  overturned the optimization priority: **`insect` (swarm of 8) is the hot spot
  at ~20x real-time**, five times slower than the next synth, while `wind` —
  which the old five-benchmark set named as the worst target — is mid-table.
  It also revealed the old `cloth` number was measuring a silent fast-path.
- ✅ **Wrote [`docs/audit/2026-08-30-audit.md`](../audit/2026-08-30-audit.md)**,
  including the refuted findings.
- ⛔ **Pre-size `*_synthesize` output vectors — BLOCKED UPSTREAM.** Measured:
  1,048,472 bytes retained per second of audio against an ideal 352,824 (~3x
  overhead, ~695 KB wasted per call, never reclaimed). Cannot be fixed from
  garjan: the Cyrius stdlib has no `vec_with_capacity` / `vec_reserve`, and
  `vec_new()` hardcodes capacity 16. Hand-building the vector header would
  couple garjan to `lib/vec.cyr`'s private `[data][len][cap]` layout — silent
  corruption on any upstream change — and `CLAUDE.md` forbids modifying `lib/`.
  **Needs a stdlib addition.** Moved to the gated section.

---

## 2.2.x — deliberate divergences _(each needs an ADR)_

Behavior changes where the port must knowingly differ from the oracle. Per
`CLAUDE.md`, none of these may land without an ADR.

- **serde live-state round-trip parity.** Rust serialized the naad component
  state (`BiquadFilter`, `NoiseGenerator`) as ordinary derived fields; the port
  drops them from `*Params` and rebuilds fresh. A save/restore mid-stream
  therefore loses filter and noise-generator state that Rust preserved —
  audible on restore. Decide: reproduce Rust (serialize the state, requires the
  naad types to expose it) or keep reconstruct-and-document. **Changes the JSON
  format**, so it cannot be a patch.
- **Upper bounds on `duration` / `sample_rate`.** `validate_duration` accepts
  any positive finite value, so 1e8 s sizes a 4.4-trillion-sample buffer.
  `rust-old/src/dsp.rs:39` validated only positive-and-finite too, so a cap is a
  divergence and needs both an ADR and an agreed limit.
- **Out-of-range enum ids.** Rust's exhaustive `match` over closed enums is
  ported as `if/elif` chains whose final `else` silently absorbs any invalid
  integer, yielding the last variant's table rather than an error. Decide
  whether the Cyrius surface should reject unknown ids.

---

## 2.3.x — performance

- **`insect` swarm is the real hot spot** — 50.3 ms/s of audio, ~20x
  real-time, five times slower than anything else. Its per-sample loop runs
  once per swarm voice, so cost scales linearly with `swarm_count` (max 8).
  This only became visible when the benchmark set went from 5 ops to 26 in
  2.1.0. Start here, not with `wind`.
- **naad-level per-sample work.** 2.0.5 took the garjan-side hoisting wins.
  Profiling points into the naad bundle's per-sample noise generation and
  biquad/SVF filtering. Needs upstream or algorithmic work, not more hoisting.
- Remaining garjan-side per-event conversions in `bubble`, `foliage`,
  `precipitation`, `insect`, `whoosh`, `impact` — small, individually.
- Gate every change on `scripts/audio-hash.cyr` staying bit-identical.

---

## Gated — not scheduled

- **`vec_with_capacity` in the Cyrius stdlib.** Without it, every
  `*_synthesize` wastes ~695 KB per second of audio to doubling reallocation,
  permanently. garjan cannot fix this without coupling to `lib/vec.cyr`'s
  private layout. Blocked on an upstream addition; see the 2.1.x entry for the
  measurement.
- **Port `integration/soorat.rs`** (315 lines: `PrecipitationField`,
  `FireEmitter`, `WindField` — visualization data for downstream rendering).
  **Blocked**: soorat has not landed in Cyrius. Feature-gated `soorat-compat`
  in Rust, so it was never part of the default surface. Schedule when the
  dependency exists; until then this is the one genuinely unported *feature*.

## 3.0.0 — breaking, if it happens

- **Allocator ownership.** The arena never frees. `alloc_reset()` invalidates
  *every* outstanding pointer, so it is only usable at a clean epoch boundary —
  a caller that constructs and discards synths in a loop still grows memory.
  Fixing this properly means an ownership/lifetime model in the public API.
- Any surface change falling out of the 2.2.x decisions.

---

## Port completeness

Method, so it can be re-run after any upstream change. The first pass used a
weaker version of this and reported completeness it had not established, so use
these, not a name-existence check.

**1. Module level** — note `rglob`, not `glob`; the first pass used `glob` and
silently skipped `rust-old/src/integration/`:

```sh
find rust-old/src -name '*.rs' | while read f; do
  b=$(basename "$f" .rs); [ -f "src/$b.cyr" ] || echo "NO .cyr: $f"; done
```

**2. Enum discriminants — the check that matters.** A missing `pub fn` fails
the build at its call site. A missing enum *variant* does not: Rust enums became
module-prefixed ints, so a variant that is absent, or present but bound to the
wrong value, silently returns a different table entry. Compare Rust's
declaration order (implicit discriminants 0,1,2… unless explicit) against the
Cyrius constant's value, and pin families whose variant names collide
(`Small`/`Medium`/`Large`, `Moderate`/`Heavy`) by exact prefix rather than
suffix match.

**3. Surface beyond fn/struct/enum.** Confirm these stay empty, or the
extractor is under-counting:

```sh
grep -rnE '^\s*pub\s+(const|static|trait|type)\s' rust-old/src/
grep -rn ' for ' rust-old/src/ | grep -E '^\S+:[0-9]+:\s*impl'   # manual trait impls
grep -rn 'derive(.*Default' rust-old/src/                          # non-zero defaults
```

**4. Account for the dropped fallback.** The module count alone overstates
completeness: 19 modules also dropped their `#[cfg(not(feature =
"naad-backend"))]` branch (~232 lines). Re-measure rather than trusting that
number:

```sh
grep -rc 'cfg(not(feature = "naad-backend"))' rust-old/src/
```

**5. Do not confuse "a symbol exists" with "it does the same thing."** Verify
formulas and constants, not just names — e.g. `DcBlocker`'s clamp bounds were
checked as bit patterns (`0x3FECCCCCCCCCCCCD` == 0.9, `0x3FEFFF2E48E8A71E` ==
0.9999) rather than read as decimals, and the transcendentals were measured
against libm rather than assumed (note
[002](../architecture/002-where-the-transcendentals-come-from.md)).
