# 0011 — Water's Stream LFO follows the raw intensity

**Status**: Accepted
**Date**: 2026-11-07

## Context

`Water::new` in `rust-old/src/water.rs` computes the Stream modulation rate as
`0.5 + intensity * 2.0` from the intensity the caller passed, and only then clamps the
stored intensity to 0..1. The port clamped first and computed the rate from the clamped
value. The two agree for intensity in 0..1 and differ outside it:

| Intensity | rust-old | garjan 2.6.2 |
|---|---|---|
| 1.5 | LFO at 3.5 Hz | LFO at 2.5 Hz |
| below -0.25 | `Err`: the LFO refuses a negative rate | LFO at 0.5 Hz, a synth |
| `f32::MAX`, +Inf | `Err`: the LFO refuses a non-finite rate | LFO at 2.5 Hz, a synth |

The weather cluster of the 2.6.3 audit found this. Every value in the table is one that
`water_new` accepts, so the port's output differed from Rust's on valid input.

## Decision

**The Stream LFO rate comes from the raw intensity, as in `rust-old`.** `water_lfo_rate`
computes it once in `water_new`, and the synth stores it so that a document can carry it
too. A document written by 2.6.2 or earlier has no `lfo_rate` member, so the rate is
derived from the document's intensity. Those versions always wrote that intensity
clamped, so the synth resumes at the rate it had.

This moves output on valid input, which the 2.6.3 roadmap allows only with an ADR. It
moves it toward `rust-old`, the port's parity oracle.

## Consequences

- **Positive** — water matches `rust-old` for every intensity, including the ones Rust
  refuses.
- **Negative** — `water_new(WATER_STREAM, x, sr)` with intensity 1.5 sounds different
  from 2.6.2. For an intensity below -0.25 or a non-finite one, it now returns
  `GARJAN_ERR_SYNTHESIS_FAILED` and logs an error where 2.6.2 returned a synth. Callers
  that passed such values relied on behaviour Rust never had.
- **Neutral** — every intensity in 0..1, and every other water type, is unchanged.
  `scripts/audio-hash.cyr` uses 0.5, and its hashes do not move.

## Alternatives considered

- **Keep the clamped rate and record the divergence.** That would keep 2.6.2's sound,
  but the port's bar is to match `rust-old`, and nothing depends on the old rate.
- **Clamp the rate instead of refusing a negative one.** That would avoid the new error,
  but it is a divergence of its own, from a constructor Rust makes fail.
