# garjan — Roadmap

> Sequencing for the 2.x line: what ships, in what order, against what gates.
> Live state is in [`state.md`](state.md); completed detail is in the
> [CHANGELOG](../../CHANGELOG.md) — this file carries only what is still open,
> plus enough context to pick each item up cold.

## Open

The parity work is complete and nothing is queued. What remains splits three
ways: two upstream API gaps, one dependency that does not exist yet, and work
that needs a decision rather than an implementation.

### Blocked on an upstream API

Neither is fixable inside garjan without coupling to a dependency's internals,
and `CLAUDE.md` forbids modifying `lib/`.

- **`vec_with_capacity` in the Cyrius stdlib.** Every `*_synthesize` builds its
  output by pushing from capacity 16. Measured: **1,048,472 bytes retained per
  second of audio against an ideal 352,824** — ~3x overhead, never reclaimed.
  `lib/vec.cyr` exposes only `vec_new()`, which hardcodes 16; hand-building the
  vector header would couple garjan to its private `[data][len][cap]` layout.
- **`filter_svf_process_sample_bandpass` in naad.** `whistle_process_block`
  allocates **32 B/sample** because naad's `filter_svf_process_sample` returns a
  heap `SvfOutput`; naad has a non-allocating `_lowpass` variant but no
  band-pass one. That is 1.41 MB/s, **5.08 GB/hour** — the 2 GiB arena is gone
  in ~25 minutes of continuous whistle. Pinned by a test so it cannot worsen,
  and flagged in the README and integration guide.

### Blocked on a dependency that does not exist

- **Port `integration/soorat.rs`** (315 lines: `PrecipitationField`,
  `FireEmitter`, `WindField`) — soorat has not landed in Cyrius. It was
  feature-gated in Rust and is the one genuinely unported *feature*.

### Needs a decision, not an implementation

- **Performance beyond 2.3.0.** `insect` is still the slowest at ~23x real-time,
  but measurement says most of what remains is irreducible: per 352,800
  voice-calls, biquad 11.9 ms + `f64_sin` 10.6 ms + noise 7.1 ms accounts for
  ~30 of its 42.6 ms. Going further needs an algorithmic change — a recurrence
  oscillator instead of a `sin` per voice per sample — which would **not** be
  bit-exact and so needs an ADR. Smaller garjan-side wins remain in `bubble`,
  `foliage`, `precipitation`, `whoosh`, `impact`.
- **A downstream consumer.** The last unmet maturity criterion — and the only
  one that cannot be closed from inside this repository.

---

## 3.0.0 — breaking, if it happens

- **Allocator ownership.** The arena never frees, and `alloc_reset()`
  invalidates *every* outstanding pointer, so it is only usable at a clean epoch
  boundary. A caller that constructs and discards synths in a loop still grows
  memory. Fixing it properly means a lifetime model in the public API.

---

## 2.x maturity criteria

- [x] Rust → Cyrius surface parity verified (function-level diff against `rust-old/`)
- [x] Test coverage adequate for the surface area — 33 suites, 836 assertions,
      including a 288-assertion cross-module suite
- [x] Benchmarks captured — 26 ops in [`BENCHMARKS.md`](../../BENCHMARKS.md)
- [x] Security audit written up — [2.0.2](../audit/2026-08-30-audit.md) and [2.5.1](../audit/2026-08-30-audit-2.md)
- [x] CHANGELOG complete from 2.0.0 onward
- [ ] **At least one downstream consumer green** — none yet

## Shipped

2.0.0 through 2.5.1 — the port itself, two security audits and their repairs,
hot-path optimization, parity completion, boundary validation, and serde
live-state parity. Per-release detail is in the
[CHANGELOG](../../CHANGELOG.md); it is not duplicated here.

---

## Port completeness

The verified result lives in [`state.md`](state.md#port-completeness). What
belongs here is the **method**, because it must be re-run after any upstream
change and because the first pass got it wrong — it asserted completeness it had
not established.

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

**6. Check properties, not diffs.** Every defect in the 2.5.1 sweep came from
asking a question across the whole surface — *"does every deserialize entry
point validate its sample rate?"*, *"does every vector-building function have
the size guard?"* — rather than from re-reading code already reviewed. A
scripted rollout is only as good as its pattern: 2.4.0's size guard missed
`impact_synthesize_velocity` because its operands were reversed.
