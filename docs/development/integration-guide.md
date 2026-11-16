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

A `ModalBank` or `Exciter` you drive yourself saves the same way, through
`modal_bank_to_json` / `modal_bank_from_json_str` and `exciter_to_json` /
`exciter_from_json_str` (since 2.8.0, [ADR-0013](../adr/0013-modal-bank-and-exciter-serialize-standalone.md)).
None of these documents is the Rust crate's format
([ADR-0014](../adr/0014-garjan-documents-are-not-rust-old-documents.md)).

## Visualization data for soorat

Since 2.7.0 garjan also builds what a renderer draws beside a sound, from the same
physical inputs. Each constructor returns a pointer or a negative `GARJAN_ERR_*` code, and
each type has `*_to_json` / `*_from_json_str`.

```cyrius
# 500 raindrops in a 10 x 20 x 10 m box, moderate rain, 2 m/s wind along x
var rain = precipitation_field_rain(500, 0, 0, 0, f64_from(10), f64_from(20), f64_from(10),
  F64_HALF, f64_from(2), 0);
if (garjan_is_err(rain) == 1) { return rain; }
var i = 0;
while (i < precipitation_field_len(rain)) {
    var p = precipitation_field_particle(rain, i);
    # PrecipitationParticle_position_x/_y/_z, _velocity_x/_y/_z, _size, _life
    i = i + 1;
}

var fire = fire_emitter_from_intensity(0, 0, 0, F64_HALF);   # FireEmitter_flame_height, ...
var wind = wind_field_gradient(8, 8, 0, 0, 0, F64_ONE, F64_ONE, f64_from(12), F64_ONE);
# point (x, z) is index z * nx + x: wind_field_vx(wind, i), wind_field_vz(wind, i)
```

`precipitation_field_snow` takes the same arguments as `_rain`. A field holds at most
`GARJAN_MAX_FIELD_POINTS` (2^20) points, and a negative or larger count is refused
([ADR-0012](../adr/0012-soorat-visualization-data-is-ported-ungated.md)).

## Where garjan stops

garjan produces raw mono sources. Propagation is **goonj**, mixing and
scheduling **dhvani**, mechanical sound **ghurni**, vocal **prani/svara**.
See [ADR-0003](../adr/0003-scope-boundaries.md).

Runnable programs: [`docs/examples/`](../examples/).
