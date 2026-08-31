# Integration Guide

How to use garjan in a game engine or audio application.

## Basic Usage

Consumers include the bundled `dist/garjan.cyr` and declare garjan's deps
(naad / hisab / goonj / sakshi + stdlib) in their own `cyrius.cyml`.

```cyrius
include "dist/garjan.cyr"

fn main() {
    alloc_init();

    # Sample rate is fixed at construction. Returns a pointer or a NEGATIVE
    # GARJAN_ERR_* code -- there is no Result.
    var rain = rain_new(RAIN_INTENSITY_HEAVY, f64_from(44100));
    if (garjan_is_err(rain) == 1) { return 1; }

    # Option A: one-shot. Allocates the output vec.
    var samples = rain_synthesize(rain, f64_from(5));
    if (garjan_is_err(samples) == 1) { return 1; }

    # Option B: streaming into your own buffer. Allocates nothing.
    var buffer = vec_new();
    var i = 0;
    while (i < 512) { vec_push(buffer, 0); i = i + 1; }
    rain_process_block(rain, buffer);
    return 0;
}
```

## Real-Time Audio Callback

Use `process_block` on the audio thread. It performs no allocation, so it is
safe in a callback:

```cyrius
# Called ~86 times/sec at 44.1 kHz with 512-sample blocks.
fn audio_callback(wind, rain, scratch, output) {
    wind_process_block(wind, output);

    # Mix other sources additively into the same buffer.
    rain_process_block(rain, scratch);
    var i = 0;
    while (i < vec_len(output)) {
        vec_set(output, i, f64_add(vec_get(output, i),
          f64_mul(vec_get(scratch, i), F64_HALF)));   # rain at 50%
        i = i + 1;
    }
    return 0;
}
```

Allocate `scratch` **once**, outside the callback. The arena never frees, so an
allocation on the audio thread accumulates for the life of the process.

> **One exception to the no-allocation rule.** `whistle_process_block`
> allocates 32 bytes per sample — naad's `filter_svf_process_sample` returns a
> heap `SvfOutput` and exposes no non-allocating band-pass variant (it has one
> for low-pass). That is ~5 GB/hour, exhausting the 2 GiB arena in about 25
> minutes. Blocked upstream; avoid `whistle` in long-running streams for now.

## Error handling

Every constructor and `synthesize` returns a pointer/`GARJAN_OK`, or a negative
code. Check with `garjan_is_err` and name it with `garjan_err_name`:

| Code | Meaning |
|---|---|
| `GARJAN_ERR_INVALID_PARAMETER` (-1) | bad sample rate, duration, enum id, or size |
| `GARJAN_ERR_SYNTHESIS_FAILED` (-2) | a naad backend component rejected its params |
| `GARJAN_ERR_COMPUTATION` (-3) | a numeric computation went invalid |
| `GARJAN_ERR_ALLOCATION` (-4) | heap exhausted ([ADR-0005](../adr/0005-allocation-failure-is-an-error-code-not-an-abort.md)) |

Accepted input ranges: sample rate 1 Hz–768 kHz, duration ≤ 600 s, buffer
≤ 44.1 M samples, and enum ids strictly within their variant range
([ADR-0007](../adr/0007-bounded-duration-and-sample-rate.md),
[ADR-0006](../adr/0006-out-of-range-enum-ids-are-rejected.md)).

## Persistence

`<type>_to_json(p, sb)` / `<type>_from_json_str(json)` round-trip a
synthesizer's parameters, RNG and DC-blocker state.

**Since 2.5.0 this is a full session snapshot**, not just parameter
persistence: biquad and SVF memory, noise-generator position, LFO phase, modal
resonators and exciters all travel, so a synth saved mid-stream resumes
sample-identically. Documents written before 2.5.0 have no `"dsp"` member and
still load, restoring parameters only. See
[ADR-0008](../adr/0008-serde-carries-live-dsp-state.md).

## Where garjan stops

garjan produces raw mono sources. Propagation is **goonj**, mixing and
scheduling **dhvani**, mechanical sound **ghurni**, vocal **prani/svara**.
See [ADR-0003](../adr/0003-scope-boundaries.md).

Runnable programs: [`docs/examples/`](../examples/).
