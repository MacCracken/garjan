# 0008 — serde carries live DSP state

**Status**: Accepted
**Date**: 2026-08-30
**Closes**: [architecture note 001](../architecture/001-deserialize-does-not-restore-dsp-state.md)

## Context

Rust derived `Serialize`/`Deserialize` over **all** of a synthesizer's fields,
naad components included (`rust-old/src/fire.rs:16`). A `#[cfg]` gated whether a
field *existed*, not whether it was serialized, so with `naad-backend` on — the
default — a Rust save/restore round-tripped the live DSP state and resumed
seamlessly.

The port dropped those fields from `*Params` and rebuilt the components from the
scalars, so a synth saved mid-stream resumed with **zeroed biquad memory, reset
noise position and reset LFO phase** — audible as a discontinuity.

Source comments justified this by citing Rust's `#[serde(skip)]`. There is no
`serde(skip)` anywhere in `rust-old`; that premise was invented, and was
corrected in 2.0.6.

The gap survived three releases of serde testing because **every serde test
checked JSON round-trip idempotence** — serialize, deserialize, re-serialize,
compare strings. That passes perfectly while every filter is silently zeroed.

## Decision

Carry the live state, restoring parity with the oracle.

**Transport.** A `"dsp"` member is spliced into the existing
`#derive(Serialize)` params object rather than flattening state into named
fields. Three properties make this work:

- the derive-generated parser **ignores unknown keys** (verified), so
  `*Params_from_json_str` still reads the scalars from the combined document;
- the same document is parsed by bayan's value tree to read `"dsp"`, which
  supports nested arrays — so pink noise's 16 octave values are an array, not 16
  named fields;
- state is stored as raw **i64 bit patterns** via `bayan_json_v_int`, not as
  JSON floats. In Cyrius an f64 *is* an i64 bit pattern, so the round-trip is
  exact by construction with no decimal formatting to lose. These are opaque
  machine values, never read by a human.

**Coverage.** Helpers in `src/dsp.cyr`, applied across 21 synthesizers:

| Component | State carried |
|---|---|
| `BiquadFilter` | `z1`, `z2` (coefficients re-derive from the stored params) |
| `StateVariableFilter` | `ic1eq`, `ic2eq` |
| `Lfo` | `phase`, `sh_value`, its xorshift word |
| `NoiseGenerator` | rng word, `brown_prev`, `pink_counter`, `pink_running_sum`, and — for pink only — all 16 `pink_octaves` |
| `ModalBank` | the `state_re`/`state_im` pair per surviving resonator |
| `Exciter` | `position`, `active`, and its `Rng` (`state` + `inc`) |

**Backward compatible.** A document written before 2.5.0 has no `"dsp"` member;
`garjan_dsp_node` returns 0 and the synth resumes without live state, exactly as
it did before. Nothing that parsed then fails now.

## Consequences

**Positive**

- Round-tripping is now session snapshotting, not merely parameter persistence.
  Verified for all 21 synths the only way that detects the defect: warm the
  synth, snapshot, restore, run **both** forward, and compare sample for sample.
- Parity with the oracle is restored on the last outstanding divergence.
- No behavior change to synthesis — `scripts/audio-hash.cyr` is bit-identical.

**Negative**

- Documents are larger. A `fire` snapshot goes from ~200 to ~400 bytes; a
  pink-noise carrier adds 16 integers per generator.
- `garjan_dsp_node` re-parses the document a second time on deserialize, after
  `*Params_from_json_str` has already parsed it. Deserialization is a cold path,
  and the alternative — threading one parsed tree through the derive — would
  mean abandoning `#derive(Serialize)` for hand-rolled JSON across 19 synths.

**Neutral**

- The guarantee holds only where the helpers are wired. A new synth that adds a
  stateful component without adding it to its `"dsp"` object degrades silently
  back to a zeroed restore. This is not hypothetical: `foliage` holds a modal
  bank under the field name **`snap_modal`**, which a grep for `modal_bank`
  misses — it was found only because the per-synth parity check failed. The
  cross-module suite now checks every synth.
- bayan's object API is asymmetric by design: `bayan_json_v_obj_set` takes its
  key as a **`Str`**, `bayan_json_v_obj_get` takes a **cstring**. Passing a
  cstring to `obj_set` builds a tree that segfaults when serialized.
