# garjan — Roadmap

> Sequencing for the 2.x line: what ships, in what order, against what gates.
> Live state is in [`state.md`](state.md); completed detail is in the
> [CHANGELOG](../../CHANGELOG.md) — this file carries only what is still open,
> plus enough context to pick each item up cold.

## Next up

### serde live-state round-trip parity

**The last open parity divergence, and the next thing to build.**

Rust derived `Serialize`/`Deserialize` over *all* fields, naad components
included, so a save/restore resumed with filter history intact. The port drops
those fields from `*Params` and rebuilds the components from scratch, so a synth
saved mid-stream resumes with **zeroed filter state** — audible as a
discontinuity. Full background:
[architecture note 001](../architecture/001-deserialize-does-not-restore-dsp-state.md).

**Decided: implement full parity.** Verbose but mechanical, and it needs no
hand-rolled JSON — the `*Params` structs are flat `#derive(Serialize)` types and
every piece of naad state is a plain `f64`/`i64`.

State to carry, all reachable through naad's existing accessors:

| Component | Fields | Notes |
|---|---|---|
| `BiquadFilter` | `z1`, `z2` | coefficients are re-derived from the params already stored |
| `StateVariableFilter` | `ic1eq`, `ic2eq` | |
| `Lfo` | `phase`, `sh_value` | |
| `NoiseGenerator` | rng state, `pink_counter`, `pink_running_sum`, `brown_prev`, 16 × `pink_octaves` | 20 fields; garjan uses white ×11, pink ×9, brown ×5 |

Plan:

1. Shared state-transfer helpers, one pair per component type.
2. Prove it end-to-end on **one** synth with a round-trip test that asserts
   *continued output after restore* matches the un-saved original — not just
   that the JSON round-trips, which is what the current serde tests check and
   why this gap survived.
3. Roll out to the remaining 18.

It **changes the JSON format**, so it lands as a minor bump with an ADR.

---

## Open, blocked upstream

Neither can be fixed inside garjan without coupling to a dependency's internals.

- **`vec_with_capacity` in the Cyrius stdlib.** Every `*_synthesize` builds its
  output by pushing from capacity 16. Measured: **1,048,472 bytes retained per
  second of audio against an ideal 352,824** — ~3x overhead, never reclaimed.
  `lib/vec.cyr` exposes only `vec_new()`, which hardcodes capacity 16;
  hand-building the vector header would couple garjan to its private
  `[data][len][cap]` layout, and `CLAUDE.md` forbids modifying `lib/`.
- **`filter_svf_process_sample_bandpass` in naad.** `whistle_process_block`
  allocates **32 B/sample** because naad's `filter_svf_process_sample` returns a
  heap `SvfOutput`; naad has a non-allocating `_lowpass` variant but no
  band-pass one. That is 1.41 MB/s, **5.08 GB/hour** — the 2 GiB arena is gone
  in ~25 minutes of continuous whistle. Pinned by a test so it cannot worsen,
  and called out in the README and integration guide.

- **Port `integration/soorat.rs`** (315 lines: `PrecipitationField`,
  `FireEmitter`, `WindField`). Blocked — soorat has not landed in Cyrius. It was
  feature-gated in Rust and is the one genuinely unported *feature*.

---

## Open, unscheduled

- **Performance beyond 2.3.0.** `insect` is still the slowest at ~23x real-time,
  but measurement says most of what remains is irreducible: per 352,800
  voice-calls, biquad 11.9 ms + `f64_sin` 10.6 ms + noise 7.1 ms accounts for
  ~30 of its 42.6 ms. Going further needs an algorithmic change — a recurrence
  oscillator instead of a `sin` per voice per sample — which would **not** be
  bit-exact and so needs an ADR. Smaller garjan-side wins remain in `bubble`,
  `foliage`, `precipitation`, `whoosh`, `impact`.
- **A downstream consumer.** The last unmet maturity criterion.

## 3.0.0 — breaking, if it happens

- **Allocator ownership.** The arena never frees, and `alloc_reset()`
  invalidates *every* outstanding pointer, so it is only usable at a clean epoch
  boundary. A caller that constructs and discards synths in a loop still grows
  memory. Fixing it properly means a lifetime model in the public API.

---

## 2.x maturity criteria

- [x] Rust → Cyrius surface parity verified (function-level diff against `rust-old/`)
- [x] Test coverage adequate for the surface area — 33 suites, 797 assertions,
      including a 288-assertion cross-module suite
- [x] Benchmarks captured — 26 ops in [`BENCHMARKS.md`](../../BENCHMARKS.md)
- [x] Security audit written up — [`docs/audit/`](../audit/2026-08-30-audit.md)
- [x] CHANGELOG complete from 2.0.0 onward
- [ ] **At least one downstream consumer green** — none yet

## Shipped

| | |
|---|---|
| 2.0.0 | Full Rust→Cyrius port, 32 modules |
| 2.0.1 | Toolchain 6.3.44 → 6.5.36; all deps to latest |
| 2.0.2 | P-1 audit: 3 JSON-reachable defects; `cyrius audit` fmt gate |
| 2.0.3 | Allocation-failure guard ([ADR-0005](../adr/0005-allocation-failure-is-an-error-code-not-an-abort.md)) |
| 2.0.4 | Arena lifetime: per-block allocations → zero |
| 2.0.5 | Per-sample hot paths; bit-exact |
| 2.0.6 | Docs accuracy (untagged); ADRs relocated to `docs/adr/` |
| 2.1.0 | Integration suite (2 → 288 assertions), 26 benchmarks, 5 examples, audit write-up |
| 2.2.0 | Out-of-range enum ids rejected ([ADR-0006](../adr/0006-out-of-range-enum-ids-are-rejected.md)) |
| 2.3.0 | `insect` hot spot: 49.9 → 42.6 ms, bit-exact |
| 2.4.0 | Duration / sample-rate / sample-count caps ([ADR-0007](../adr/0007-bounded-duration-and-sample-rate.md)) |

---

## Port completeness

Verified 2026-08-30, then **re-verified** after the first pass was found to have
asserted things it had not checked. Re-run these after any upstream change.

- **30 of 34** `rust-old/src` modules have a `.cyr` counterpart.
- **All 106 enum variants ported, 103 preserving Rust's exact discriminant.**
  The 3 exceptions are `GarjanError`, deliberately remapped to negative codes.
- **~223 of 242** public items compared. The remainder is `math.rs` (superseded
  — [note 002](../architecture/002-where-the-transcendentals-come-from.md)) and
  `integration/soorat.rs` (deferred).
- Correctly not ported: `lib.rs` (module declarations, a prelude, and a
  `#[cfg(test)]` Send+Sync assertion — no behavior), `math.rs`, and
  `integration/mod.rs` (7 lines).
- **Also dropped deliberately: ~232 lines of non-naad fallback across 19 ported
  modules** ([ADR-0002](../adr/0002-dual-code-paths.md)). The module count alone
  overstates completeness.
- One API-shape gap: `VoicePool::active_voices` returned an iterator; Cyrius has
  none. Reachable by looping `voice_pool_slot` + `VoiceSlot_active`.

### Method

**1. Module level** — note `rglob`, not `glob`; the first pass used `glob` and
silently skipped `rust-old/src/integration/`.

```sh
find rust-old/src -name '*.rs' | while read f; do
  b=$(basename "$f" .rs); [ -f "src/$b.cyr" ] || echo "NO .cyr: $f"; done
```

**2. Enum discriminants — the check that matters.** A missing `pub fn` fails the
build at its call site; a missing or mis-valued enum *variant* does not, because
ported enums are plain integers. Compare Rust's declaration order against the
Cyrius constant's value, and pin families whose variant names collide
(`Small`/`Medium`/`Large`, `Moderate`/`Heavy`) by exact prefix, not suffix match.

**3. Surface beyond fn/struct/enum.** Confirm these stay empty:

```sh
grep -rnE '^\s*pub\s+(const|static|trait|type)\s' rust-old/src/
grep -rn ' for ' rust-old/src/ | grep -E '^\S+:[0-9]+:\s*impl'   # manual trait impls
grep -rn 'derive(.*Default' rust-old/src/                        # non-zero defaults
```

**4. Account for the dropped fallback**, which the module count hides:

```sh
grep -rc 'cfg(not(feature = "naad-backend"))' rust-old/src/
```

**5. "A symbol exists" is not "it does the same thing."** Verify formulas and
constants — `DcBlocker`'s clamp bounds were checked as bit patterns
(`0x3FECCCCCCCCCCCCD` == 0.9), and the transcendentals measured against libm
rather than assumed.
