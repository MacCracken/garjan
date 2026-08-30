# 0007 — Duration and sample rate are bounded

**Status**: Accepted
**Date**: 2026-08-30

## Context

`rust-old/src/dsp.rs:39` validated a duration as **positive and finite**, and
nothing more. `validate_sample_rate` did the same. The port copied that
faithfully — which means `1e8` seconds passed validation, then sized a
**4.4-trillion-sample** buffer and killed the process allocating it.

The 2.0.2 audit found this and deliberately left it alone: it is what the oracle
did, so capping is a divergence, not a bug fix. naad does not bound them either.
goonj does cap by sample count (`MAX_IR_SAMPLES = 115200000`), which is the
useful precedent — the thing that actually drives allocation is neither input
alone but their **product**.

That distinction matters. 600 s is unremarkable at 44.1 kHz (26.5 M samples) and
ruinous at 768 kHz (461 M). Bounding duration and sample rate independently
would still let a legal pair produce an illegal buffer.

## Decision

Bound all three, in `src/dsp.cyr`:

```cyrius
var GARJAN_MIN_SAMPLE_RATE_HZ = 1;          # below this is not audio
var GARJAN_MAX_SAMPLE_RATE_HZ = 768000;     # highest standard hi-res rate
var GARJAN_MAX_DURATION_S     = 600;        # 10 minutes per synthesize call
var GARJAN_MAX_SAMPLES        = 44100000;   # 1000 s at 44.1 kHz
```

- `garjan_validate_sample_rate` gains the min/max range check, so every
  constructor inherits it.
- `garjan_validate_duration` gains the max check, so every `*_synthesize`
  inherits it.
- A new `garjan_validate_sample_count(num)` guards the product, applied at all
  20 `*_synthesize` sites immediately after the count is computed.

`GARJAN_MAX_SAMPLES` is the binding constraint and was chosen against the
allocator, not by taste: 44.1 M samples is ~353 MB of f64 ideally, and about
1.05 GB as actually built, because `*_synthesize` grows its vector by doubling
from capacity 16 (~3x overhead — measured, and blocked on a missing stdlib
`vec_with_capacity`). That leaves headroom inside `ALLOC_MAX` (2 GiB).

These are deliberately generous. Real synthesis calls are seconds, not minutes.
**Tighten freely** — the constants are named, the tests reference the names
rather than literals, so lowering them is a one-line change.

## Consequences

**Positive**

- The unbounded-allocation DoS is closed: a hostile or careless duration now
  returns `GARJAN_ERR_INVALID_PARAMETER` instead of killing the process.
- The sample-count guard catches the composite case that neither individual
  bound can see.
- Bounds are named constants, so tuning does not touch call sites or tests.

**Negative**

- A divergence from the oracle. Rust accepted any positive finite duration; the
  port now rejects some inputs Rust would have attempted. A caller synthesizing
  more than 10 minutes in one call, or above 768 kHz, must now chunk — which is
  what a caller should be doing at that size anyway.
- The upper sample-rate bound rejects genuinely exotic rates. 768 kHz is the
  highest standard hi-res audio rate; anything beyond is not an audio use case.

**Neutral**

- `GARJAN_MAX_SAMPLES` is derived from `ALLOC_MAX` **and** from the vector
  doubling overhead. If `vec_with_capacity` lands upstream and `*_synthesize`
  stops wasting ~3x, this cap could roughly triple. It is a measurement, and
  like every dependency-derived constant it goes stale silently — re-derive it
  on a toolchain bump rather than trusting the number.
