# 0010 — A document restores only states the public API can produce

**Status**: Accepted
**Date**: 2026-11-07

## Context

`*_from_json_str` takes fully attacker-controlled input and writes it straight into a
synth's fields and its naad components' live state
([ADR-0008](0008-serde-carries-live-dsp-state.md)). Before 2.6.3 it validated the
sample rate and the enum ids (the [2.5.1 audit](../audit/2026-08-30-audit-2.md)) and
nothing else, so a document could set states that no sequence of public calls reaches.
The 2.6.3 audit found that several of those states break the synth:

| State only a document could set | Effect |
|---|---|
| An even PCG increment (`rng_inc`) | With an increment divisible by 4 the generator can sit at a fixed point. Where that point draws `0xFFFFFFFF`, `rng_poisson` never returns, and the audit's `cloth` document hangs. |
| A NaN or infinite value in filter memory (`"dsp"`) | Output is non-finite from then on. One NaN bit pattern in a modal bank's state is enough. |
| A zero xorshift state in an LFO or noise generator | The generator is stuck at a fixed point. |
| A negative sample position | Output is non-finite from then on. |
| A voice slot with `active` 2, a priority of 256, a negative age or a tag above 2^32 | The pool steals the wrong voice. |
| An intensity, speed or distance outside the range its setter clamps to | Run time grows with the value. Each event costs more, and some loops run quadratically. |
| A derived field out of step with the one it is derived from (bubble's decay time, precipitation's `decay_samples`, foliage's event length) | Hours of work for one call at the 600 s duration cap. |
| A `"dsp"` that is not an object, or a document the tree parser refuses but the field reader accepts | The synth loads without its live state, and the caller is not told. |

In Rust, serde's types refused part of this. A `usize` refuses a negative number, a `bool`
refuses 2, a `u8` refuses 256, and an `f32` refuses `null` or `1e999`. **Rust accepted the
rest**: an even `u64` increment, an unclamped `f32` and a stale derived field all load,
and the even increment hangs `rust-old` in the same loop.

## Decision

**A document restores only a state that some sequence of public calls could produce.
Anything else is refused with `GARJAN_ERR_INVALID_PARAMETER`, or put back in range the
way the matching setter would.**

1. **The document must parse.** Every `*_from_json_str` starts with `garjan_json_open`,
   which parses with bayan's tree parser and refuses whatever that parser refuses. That
   means malformed JSON and, from bayan 1.11 (garjan's pin), also duplicate keys, leading
   zeros and nesting deeper than 128. The root must be an object, `"dsp"` an object and
   `"slots"` an array. A parse failure is now an error instead of a load without live
   state. Under an older bayan, such as the one bundled with dhvani's cyrius 6.6.10, a
   duplicate key still loads, but both copies go through the checks below.
2. **Every value must be one Rust's type would have taken.** `null`, a non-finite number,
   a string and a bool are refused at the top level. 13 fields that were unsigned in
   `rust-old` refuse a negative value or a float. 5 fields that were `bool` refuse
   anything but 0 or 1.
3. **Live state must be reachable.** The `"dsp"` loaders refuse a non-finite value, a
   zero xorshift state, a negative position, an exciter whose `active` is not 0 or 1, and
   an even PCG increment. `rng_restore` enforces the last for the synths' own generators
   as well. The voice pool refuses slots outside their Rust types.
4. **Clamped fields are clamped again.** The 22 fields that a constructor or setter
   clamps, such as intensity, speed and distance, get the same clamp on load.
5. **Derived fields are derived again.** bubble's base frequency, event rate, spread and
   decay time come from its type. precipitation's rate, amplitude and `decay_samples` come
   from its stone size and sample rate. foliage's event rate and event length come from its
   type and sample rate. No public call sets any of these on its own, so the document's
   copy is ignored.

This diverges from `rust-old` where Rust accepted the state: the even increment, the
clamps and the re-derived fields. Every document garjan itself writes still loads, to the
same state, unless it carries a NaN that the API stored (see Consequences). For the same
state, the writer's output is byte-identical to 2.6.2 apart from water's new `lfo_rate`
member. Every round trip in the suite survives, and `scripts/audio-hash.cyr` is unchanged
across all 21 synths. The one change in audio is water's, recorded in
[ADR-0011](0011-water-stream-lfo-follows-the-raw-intensity.md).

## Consequences

- **Positive** — every crash, hang and non-finite output that the 2.6.3 audit reached
  through JSON is closed. One rule covers them, and it can be checked: "could the API get
  here?"
- **Positive** — a document that fails to parse is an error. A load can no longer drop the
  live state silently.
- **Negative** — `null` no longer restores as NaN. A NaN that the API stored, for example
  through `fire_new(NaN, sr)`, is written as `null` and so does not round-trip.
  [ADR-0009](0009-a-nan-event-rate-means-no-events.md) keeps such a synth from hanging.
  It does not make NaN a value worth saving.
- **Negative** — a hand-edited document with an out-of-range field loads with that field
  clamped rather than refused. This matches the setter, but the caller is not told.
- **Neutral** — the rule is applied where 2.6.3 found harm. **Two gaps remain**, recorded
  in the roadmap:
  - Other stored copies of configuration are trusted beyond finiteness. Water's
    `lfo_rate` is one, and so are the per-type tables a synth copies at construction.
    A document can set them to any finite value the API would not.
  - A missing field still loads as 0, because the derived field reader defaults it.
    Refusing a missing field needs the reader to say which fields it found.

## Alternatives considered

- **Refuse anything out of range instead of clamping.** Stricter, but it would refuse
  documents that the setters accept from the same numbers. A clamp keeps "what the API
  takes" and "what a document takes" the same.
- **Trust the document, and make every consumer of a field defensive.** That is the
  approach ADR-0009 took for one loop. Applied everywhere, it spreads checks through every
  hot path, and it cannot see harm that comes from two fields in combination.
- **Validate each field by hand in each loader.** That needs about 300 checks across 22
  loaders. Most of what goes wrong is a JSON type or sign mistake that a single pass over
  the root object catches. The checks in `garjan_json_open` and the shared `"dsp"`
  loaders cover all 22 at once.
