# garjan — Roadmap

> Sequencing for the 2.x line: what ships next, in what order, against what gates.
> Live state is in [`state.md`](state.md); completed detail is in the
> [CHANGELOG](../../CHANGELOG.md). This file carries only what is still open, plus
> enough context to pick each item up cold.

| Release | Theme | Gate |
|---|---|---|
| **2.6.3** | **P(-1) audit, hardening, refactor and optimization sweep** | **None. Next.** |
| 2.7.x | Follow dhvani onto current dependencies | dhvani moves its pins |
| 2.8.x | Performance that changes samples | A decision and an ADR |
| 2.9.x | soorat integration | soorat lands in Cyrius |
| 2.x.x | Watch items | Upstream, or a new consumer |
| 3.0.0 | Allocator ownership | Breaking, if it happens |

2.7 onward sets an order, not a schedule. Each of those gates sits outside this
repository, so a release ships when its gate opens, even ahead of a lower number.

---

## 2.6.3 — P(-1) sweep

**Find what the suite cannot.** All 847 assertions passed while non-object documents
crashed all 21 synths, a JSON `null` hung synthesis, and a benchmark measured the wrong
surface. Those were found by asking new questions, not by reading the same code again.

Output on valid input does not move, unless an ADR says so. Every fix gets a test shown
to fail without it.

### Method

1. **Differential against `rust-old/`.** Build the Rust oracle from a scratch copy, leaving
   `rust-old/` untouched, and drive a matching harness on each side: every generator, each
   numeric parameter across hostile values. Compare against an f32 tolerance, not bits
   ([note 002](../architecture/002-where-the-transcendentals-come-from.md)).
2. **Line by line against `rust-old/`**, by module cluster, reproducing each defect
   before listing it.
3. **Fault injection.** Refuse the Nth allocation in every constructor and synthesize
   path. [ADR-0005](../adr/0005-allocation-failure-is-an-error-code-not-an-abort.md)
   promises an error code, and nothing has tested that promise end to end.
4. **All-zero sweep.** Call every public fn with 0 for every argument.
5. **Property sweeps.** Whole-surface questions, not diffs:
   [the method](../guides/port-completeness.md).
6. **Independent review** of the merged diff, by a reader who saw none of the work.
7. **Revert proof.** Revert a sample of guards on the merged tree. Each must fail a test.

### Audit: open questions

- [ ] **A failed tree parse is silent.** `garjan_dsp_node` cannot tell "no `dsp`" from
      "parse failed", so a document the tree parser refuses and the field reader accepts
      loads without its live state and with no error. Decide whether it should refuse.
- [ ] **Non-finite values through JSON.** `null` restores as NaN and `1e999` as +Inf, and the
      writer emits `null` for both, so +Inf comes back as NaN. ADR-0009 made one loop safe.
      Find the other NaN-sensitive paths, and decide whether deserialize refuses these.
- [ ] **Wrong-enum calls.** 2.6.2 found `MATERIAL_*` passed as a terrain in the benchmark, the
      audio oracle and the suite, the second bug of the kind
      [ADR-0006](../adr/0006-out-of-range-enum-ids-are-rejected.md) records. Check every
      enum-typed argument in `src/`, `tests/`, `scripts/` and `docs/examples/`.

### Hardening

- [ ] **Malformed numbers and duplicate keys in `tests/garjan.fcyr`.** The 5,835-document
      sweep from 2.6.1 ran by hand and is not in the repository.
- [ ] **Constructors and setters in the harness.** Hostile values reach them only through
      the manual cross-toolchain sweep today, and that is how the NaN hang stayed hidden.
- [ ] **`garjan_vec_zeros`'s error path.** It is the first buffer path that can return
      `GARJAN_ERR_ALLOCATION` rather than abort, and no test reaches it yet.

### Refactor

- [ ] **Three copied families.** 21 `*_from_json_str`, 21 `*_to_json` and 21 `*_synthesize`
      share their opening by copy, which is how the 2.4.0 rollout missed a site. Factor out
      what the language allows, and keep the bundle compiling under dhvani's pins.
- [ ] **Unused vendored files.** `lib/boxed.cyr`, `lib/callback.cyr` and `lib/tagged.cyr` are
      outside the declared footprint. Remove them with `rm <file> && cyrius deps --lock`,
      or record why they stay.

### Optimization

Bit-identical, shown by `scripts/audio-hash.cyr`, and compared the way
[`BENCHMARKS.md`](../../BENCHMARKS.md) says to under load.

- [ ] **Call naad's buffer forms.** naad 2.3.3 says the gain is in calling them, and
      `noise_fill_buffer` and `filter_biquad_process_buffer` exist in dhvani's naad too, so
      this needs no dhvani move. garjan's synths still render per sample.
- [ ] **`whistle` re-derives its filter every sample** through `filter_svf_set_params`.
      Measure what that costs, and whether a slower control rate can stay bit-identical.
- [ ] **Smaller garjan-side wins** in `bubble`, `foliage`, `precipitation`, `whoosh` and
      `impact`.

### Done means

- [ ] A write-up in `docs/audit/`, as 2.0.2 and 2.5.1 have: each finding fixed, or
      recorded as accepted with its reason.
- [ ] `cyrius audit` exits 0, the build has no warnings, `cyrius fuzz` passes, and raw
      samples match 2.6.2 except where an ADR says otherwise.
- [ ] dhvani's `tests/environment.tcyr` passes on the new bundle under its own pins.

---

## 2.7.x — Follow dhvani

**Gate: dhvani moves.** It pins cyrius 6.6.10 and naad 2.2.4, and `dist/garjan.cyr` compiles
under the consumer's toolchain, so these wait however easy they are. Each opens on its own
threshold, and dhvani can take every 2.6.x bundle without moving anything.

| dhvani reaches | Then |
|---|---|
| cyrius 6.6.13 | Replace `garjan_json_get` with `bayan_json_v_obj_get_by_cstr`, dropping a `Str` per lookup |
| naad 2.3.2 | Move `whistle` to `filter_svf_process_sample_into`, and delete `whistle_svf_band_pass` and its parity test |
| cyrius 6.7.2 | Allow 6.7.x syntax in `src/`: `const` for tables, `bool`, and `!`, which is a silent no-op below 6.7.3 |

---

## 2.8.x — Performance that changes samples

**Gate: a decision, then an ADR.** `insect` is the slowest synth at ~30x real-time, and
measurement at 2.3.0 says most of what remains is irreducible: per 352,800 voice-calls,
biquad 11.9 ms, `f64_sin` 10.6 ms and noise 7.1 ms. Going further means a recurrence
oscillator in place of a `sin` per voice per sample, which is **not** bit-exact. The first
question is whether ~30x is already enough for any consumer.

---

## 2.9.x — soorat integration

**Gate: soorat lands in Cyrius.** It is still a Rust crate. Port
`rust-old/src/integration/soorat.rs` (315 lines: `PrecipitationField`, `FireEmitter`,
`WindField`), the one unported *feature*, which was feature-gated in Rust.

---

## 2.x.x — Watch items

- **`vec_with_capacity` in the Cyrius stdlib.** `garjan_vec_zeros` closed the overhead at
  2.6.2 by writing `lib/vec.cyr`'s header directly, as `naad_vec_zeros` does. When a
  capacity constructor lands, use it and drop that coupling.
- **New consumers.** kiran and joshua are expected. Each adds a pin to the oldest-consumer
  check in [`dependency-watch.md`](dependency-watch.md).

---

## 3.0.0 — breaking, if it happens

- **Allocator ownership.** The arena never frees, and `alloc_reset()` invalidates *every*
  outstanding pointer, so it is only usable at a clean epoch boundary. A caller that
  constructs and discards synths in a loop still grows memory. Fixing it properly means a
  lifetime model in the public API.

---

## Shipped

2.0.0 through 2.6.2: the port, two security audits and their repairs, hot-path optimization,
parity completion, boundary validation, serde live-state parity, the fuzz harness, and the
moves to cyrius 6.7.18. All 2.x maturity criteria are met, the last at 2.6.2, when dhvani's
environment suite was run against the bundle. Per-release detail is in the
[CHANGELOG](../../CHANGELOG.md).
