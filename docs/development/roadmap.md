# garjan — Roadmap

> Sequencing for the 2.x line: what ships next, in what order, against what gates.
> Live state is in [`state.md`](state.md); completed detail is in the
> [CHANGELOG](../../CHANGELOG.md). This file carries only what is still open, plus
> enough context to pick each item up cold.

| Release | Theme | Gate |
|---|---|---|
| 2.7.x | Follow dhvani onto current dependencies | dhvani moves its pins |
| 2.8.x | Performance that changes samples | A decision and an ADR |
| 2.9.x | soorat integration | soorat lands in Cyrius |
| 2.x.x | Watch items | Upstream, or a new consumer |
| 3.0.0 | Allocator ownership | Breaking, if it happens |

This sets an order, not a schedule. Every gate sits outside this repository, so a release
ships when its gate opens, even ahead of a lower number. Nothing is scheduled with no gate:
the 2.6.3 sweep closed the last of that work.

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

`whistle` is the other candidate. It re-derives its filter every sample through
`filter_svf_set_params`, which is 89 of its ~231 ns per sample at 2.6.3, as `rust-old`
does. A control-rate update would save about a third, and it changes samples.

2.6.3 measured naad's buffer forms and found nothing worth taking. A synth interleaves
its components per sample, with parameters that change per sample, so the call overhead a
buffer form saves is a few ns against 45 to 700 ns per sample.

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
- **Upstream allocation defects.** The 2.6.3 fault sweep left 380 segfaults and 274 exits,
  all in naad, the `#derive` reader, the stdlib builder or bayan. They are listed in
  [`dependency-watch.md`](dependency-watch.md) as U1 to U4. File them, and re-run the
  sweep from [the audit](../audit/2026-11-07-audit-3.md) as each is fixed. garjan needs no
  change for U1: `garjan_noise_new` is ready for it.
- **ADR-0010's two gaps**, for a consumer that needs a stricter document:
  - **A missing field loads as 0.** The derive reader defaults it. To refuse it,
    `garjan_json_open` needs each synth's list of expected keys, or the reader needs to
    report which fields it found.
  - **Configuration copies are trusted beyond finiteness.** Water's `lfo_rate`, and the
    per-type tables a synth copies at construction, take any finite value from a
    document. Re-deriving them, as 2.6.3 did for bubble, precipitation and foliage, closes
    each one.

---

## 3.0.0 — breaking, if it happens

- **Allocator ownership.** The arena never frees, and `alloc_reset()` invalidates *every*
  outstanding pointer, so it is only usable at a clean epoch boundary. A caller that
  constructs and discards synths in a loop still grows memory. Fixing it properly means a
  lifetime model in the public API.

---

## Shipped

2.0.0 through 2.6.3: the port, three audits and their repairs, hot-path optimization,
parity completion, boundary validation, serde live-state parity, the fuzz harness, and the
moves to cyrius 6.7.18. All 2.x maturity criteria are met, the last at 2.6.2, when dhvani's
environment suite was run against the bundle. Per-release detail is in the
[CHANGELOG](../../CHANGELOG.md).
