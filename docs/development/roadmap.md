# garjan — Roadmap

> Sequencing for the 2.x line: what ships next, in what order, against what gates.
> Live state is in [`state.md`](state.md); completed detail is in the
> [CHANGELOG](../../CHANGELOG.md). This file carries only what is still open, plus
> enough context to pick each item up cold.

| Release | Theme | Gate |
|---|---|---|
| 2.8.0 | Close the port's last two records | A decision |
| 2.9.x | Performance that changes samples | A decision, then an ADR |
| 2.x.x | Watch items | Upstream, or a new consumer |
| 3.0.0 | Allocator ownership | Breaking, if it happens |

This sets an order, not a schedule. Every item waits on a decision or on something outside
this repository, so a release ships when its gate opens, even ahead of a lower number.

---

## 2.8.0 — Close the port's last two records

**Gate: a decision on each.** The 2.7.0 audit of `rust-old/` found every module, public
function, struct and enum variant ported, except these two, which were decided at 2.0.0
but never written down as ADRs.

- **Standalone `ModalBank` and `Exciter` codecs.** Rust derived `Serialize` and
  `Deserialize` on both. The port carries their live state only inside a host synth's
  document (ADR-0008), and the 2.0.0 CHANGELOG says they are "rebuilt from their inputs
  rather than serialized". `test_serde_roundtrip_modal_bank` and
  `test_serde_roundtrip_exciter` are the only `integration.rs` tests still unported.
  Either:
  - **Port them.** `modal_bank_to_json` and `modal_bank_from_json_str`, the same for the
    exciter, two fuzz entry points, and a loader that refuses coefficients no
    `modal_bank_new` produces (ADR-0010). The coefficients are the hard part: a document
    can carry a pole outside the unit circle.
  - **Or record the divergence** in an ADR. The header of `src/modal.cyr` already says
    what is true today.
- **Write down that garjan's documents are not Rust's.** Enums serialize as integers, not
  variant names, and array fields are flattened (ADR-0012), so no `rust-old` document
  loads here. That is implied by the overview but stated in no ADR.

---

## 2.9.x — Performance that changes samples

**Gate: a decision, then an ADR.** `insect` is the slowest synth at ~29x real-time, and
measurement at 2.3.0 says most of what remains is irreducible: per 352,800 voice-calls,
biquad 11.9 ms, `f64_sin` 10.6 ms and noise 7.1 ms. Going further means a recurrence
oscillator in place of a `sin` per voice per sample, which is **not** bit-exact. The first
question is whether ~29x is already enough for any consumer.

`whistle` is the other candidate. It re-derives its filter every sample through
`filter_svf_set_params`, which was 89 of its ~231 ns per sample at 2.6.3, as `rust-old`
does. A control-rate update would save about a third, and it changes samples. Since 2.7.0
it uses naad's own `filter_svf_process_sample_into`, which also computes the two outputs
whistle discards. That costs 3.6%, the price of dropping the copied filter step.

2.6.3 measured naad's buffer forms and found nothing worth taking. A synth interleaves
its components per sample, with parameters that change per sample, so the call overhead a
buffer form saves is a few ns against 45 to 700 ns per sample.

---

## 2.x.x — Watch items

- **dhvani moving its garjan pin.** dhvani pins garjan 2.6.3. The 2.7.0 bundle passes
  dhvani's environment, bundle and integration suites under dhvani's own pins (cyrius
  6.7.18, naad 2.3.3, hisab 3.4.1, goonj 2.3.1), so the move needs nothing else.
- **New consumers.** kiran and joshua are expected, and soorat is the natural consumer of
  the 2.7.0 field types once its port finishes. Each adds a pin to the oldest-consumer
  check in [`dependency-watch.md`](dependency-watch.md). soorat may want a flat f32
  particle buffer rather than a vec of particle pointers: add it then (ADR-0012).
- **Upstream allocation defects U2 to U4.** The derive reader, the stdlib builder and
  bayan, listed in [`dependency-watch.md`](dependency-watch.md). naad fixed U1 and U1b in
  2.3.4, and garjan's constructor sweep went from 312 faults in 650 refusals to none. Re-run
  the full sweep from [the 2.6.3 audit](../audit/2026-11-07-audit-3.md) as each of the
  others is fixed.
- **`vec_with_capacity` in the Cyrius stdlib.** `garjan_vec_zeros` writes `lib/vec.cyr`'s
  header directly, as `naad_vec_zeros` does. When a capacity constructor lands, use it and
  drop that coupling.
- **`dist/garjan.deps` names only `bayan`.** The bundle also calls sakshi, which became a
  stdlib leaf at 2.7.0. Every consumer so far includes sakshi itself, so nothing breaks,
  but a new one that relies on the sidecar would miss it. Check what `cyrius distlib`
  counts before the next consumer arrives.
- **ADR-0010's two gaps**, for a consumer that needs a stricter document:
  - **A missing field loads as 0.** The derive reader defaults it. To refuse it,
    `garjan_json_open` needs each synth's list of expected keys, or the reader needs to
    report which fields it found. The three soorat loaders already refuse one
    (ADR-0012).
  - **Configuration copies are trusted beyond finiteness.** Water's `lfo_rate`, and the
    per-type tables a synth copies at construction, take any finite value from a
    document. Re-deriving them, as 2.6.3 did for bubble, precipitation and foliage, closes
    each one.
- **6.7.x syntax in `src/`** is allowed since 2.7.0: `const` for tables, `bool`, and `!`.
  Adopt it where code is touched anyway. A sweep for its own sake would move nothing a
  consumer can see.
- **Benchmarks are not like-for-like with `rust-old`'s.** Several ops run other settings
  than `rust-old/benches/benchmarks.rs` (thunder at 2 km, not 500 m; an insect swarm of 8
  cicadas, not 5 wing-buzzes; impact through `process_block`). Align them, or keep the
  note in [`BENCHMARKS.md`](../../BENCHMARKS.md), before anyone compares the two tables.

---

## 3.0.0 — breaking, if it happens

- **Allocator ownership.** The arena never frees, and `alloc_reset()` invalidates *every*
  outstanding pointer, so it is only usable at a clean epoch boundary. A caller that
  constructs and discards synths in a loop still grows memory. Fixing it properly means a
  lifetime model in the public API.

---

## Shipped

2.0.0 through 2.7.0: the port, three audits and their repairs, hot-path optimization,
parity completion, boundary validation, serde live-state parity, the fuzz harness, the
moves to cyrius 6.7.18, and at 2.7.0 the last Rust module (soorat's visualization data,
ungated), the items that waited on dhvani's pins, the newest naad, hisab and goonj, and the
rename of the DC blocker struct that collided with prani's in dhvani's closure.
All 2.x maturity criteria are met, the last at 2.6.2, when dhvani's environment suite was
run against the bundle. Per-release detail is in the [CHANGELOG](../../CHANGELOG.md).
