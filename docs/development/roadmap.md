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

Verified 2026-08-30 by diffing `rust-old/` against `src/` (see
[Port completeness](#port-completeness) for method):

- **30 of 34** `rust-old/src` modules ported.
- **219** public Rust items checked; **106 of 106** enum variants have a Cyrius
  constant (`GarjanError::ComputationError` → `GARJAN_ERR_COMPUTATION` is a
  name shortening, not a gap).
- Not ported, **correctly**: `lib.rs` (crate root — 62 `use`/`mod` lines plus a
  `Send + Sync` assertion test; Cyrius has a flat namespace and no module
  system) and `math.rs` (an f32 `sin/cos/exp/sqrt/powf` shim over std/libm,
  superseded by ganita's f64 transcendentals).
- Not ported, **outstanding** — these are scheduled below:
  `integration/soorat.rs` (315 lines), `examples/` (5 programs),
  `tests/integration.rs` (134 tests), and most of `benches/benchmarks.rs`
  (26 benches vs 5 ported).

## 2.x maturity criteria

- [x] Rust → Cyrius surface parity verified (function-level diff against `rust-old/`)
- [ ] Test coverage adequate for the surface area — **not met**: the
      cross-module suite `tests/garjan.tcyr` is a **two-assertion** placeholder
      against Rust's 134 integration tests
- [x] Benchmarks captured — in [`BENCHMARKS.md`](../../BENCHMARKS.md) at the
      repo root (the old criterion said `docs/benchmarks.md`; the root file is
      the real one and the criterion was corrected, not the file moved)
- [ ] At least one downstream consumer green — **none yet**
- [x] CHANGELOG complete from 2.0.0 onward
- [ ] Security audit written up under `docs/audit/` — the sweep was **done**
      (2.0.2/2.0.3, three JSON-reachable defects plus the allocation guard) but
      never written up as an audit document

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

### 2.0.6 — documentation accuracy _(next patch)_

Nothing here changes behavior. All of it is wrong-or-absent documentation found
during the 2.0.x work.

- **Remove the false `#[serde(skip)]` premise.** Roughly eight comments across
  `bubble/friction/insect/impact/texture/cloth/fire/…` state the port "mirrors
  Rust's skip+reconstruct". **`rust-old/src` contains zero `serde(skip)`
  attributes.** Rust derived `Serialize`/`Deserialize` over *all* fields,
  naad components included. The reconstruct-on-deserialize behavior is a
  deliberate divergence, not a mirror — say so, and cross-reference the parity
  item scheduled for 2.2.0.
- Fill in the `## Goal` section of `CLAUDE.md` (currently a `_TODO:_` stub —
  what does garjan own in the stack?).
- Populate `docs/architecture/README.md`'s empty index, or state that the
  architecture notes are intentionally empty.
- Decide the ADR location split: `adr-001`..`adr-004` live in
  `docs/architecture/` under a pre-conventions `adr-NNN-` scheme, while
  `docs/adr/README.md` declares `docs/adr/` with four-digit names as the home
  and 0005 was filed there. Either relocate 1–4 or record the exception.

---

## 2.1.x — parity completion

Everything Rust shipped that the port has not. Additive; no behavior change to
existing APIs.

- **Port `tests/integration.rs` → `tests/garjan.tcyr`.** Rust has **134**
  cross-module integration tests; the Cyrius equivalent is a two-assertion
  placeholder (`assert(1, "true is true")`). This is the single largest
  coverage gap and it blocks a maturity criterion. The 33 per-module suites
  (478 assertions) are real, but nothing covers cross-module interaction.
- **Port `examples/` (5 programs) → `docs/examples/`**, which currently holds
  only `.gitkeep` while `CLAUDE.md` advertises it: `forest_ambience`,
  `weather_scene`, `combat_impacts`, `error_handling`, `logging`.
- **Expand `tests/garjan.bcyr`** from 5 benchmarks toward Rust's 26, so
  optimization work has coverage beyond the five currently-measured synths.
  2.0.5 could only measure a fifth of the surface it changed.
- **Write up `docs/audit/2026-08-30-audit.md`** from the 2.0.2/2.0.3 findings —
  satisfies the maturity criterion and records the refuted findings too.
- **Pre-size `*_synthesize` output vectors.** Each builds its buffer by pushing
  from capacity 16 — about a dozen doubling reallocations per second of audio,
  every intermediate permanently retained by the non-freeing arena.
  Behavior-neutral.

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

- **naad-level per-sample work.** 2.0.5 took the garjan-side hoisting wins
  (−5.6% on the worst synth). Profiling now points into the naad bundle's
  per-sample noise generation and biquad/SVF filtering. Needs upstream work or
  an algorithmic change, not more hoisting.
- Remaining garjan-side per-event conversions in `bubble`, `foliage`,
  `precipitation`, `insect`, `whoosh`, `impact` — small, individually.
- Gate every change on `scripts/audio-hash.cyr` staying bit-identical.

---

## Gated — not scheduled

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

Method, so it can be re-run after any upstream change:

```sh
# module-level: every rust-old/src module has a .cyr counterpart
find rust-old/src -name '*.rs' | while read f; do
  b=$(basename "$f" .rs); [ -f "src/$b.cyr" ] || echo "NO .cyr: $f"; done

# symbol-level: enum variants must all have Cyrius constants, since Rust enums
# were ported to module-prefixed ints and a dropped variant is invisible
grep -rn 'pub enum' rust-old/src/
grep -rhE '^var [A-Z][A-Z0-9_]*' src/*.cyr
```

The enum check is the one that matters: a missing `pub fn` fails the build at
its call site, but a missing enum *variant* just silently narrows the surface.
