# 0009 — A NaN event rate means no events

**Status**: Accepted
**Date**: 2026-11-07

## Context

`rng_poisson` is Knuth's algorithm, ported line for line from `Rng::poisson` in
`rust-old/src/rng.rs`. It multiplies uniform draws into `p` and exits when
`p < l`, where `l` is `exp(-rate)`. The rate is clamped first, but a clamp leaves
NaN as NaN, so `l` is NaN, nothing is less than NaN, and the loop never exits.
**Rust hangs on this input too**, so a faithful port hangs.

In Rust, JSON could not reach that loop: serde_json refuses a `null` or an
out-of-range number where an `f32` belongs. In the port it can, because the
derived reader stores a JSON `null` as NaN and `1e999` parses to +Inf:

- `"crackle_rate":null` hangs `fire`. `bubble`, `cloth`, `precipitation` and
  `underwater` hang the same way through their own rate fields.
- `"wind_speed":1e999` with `"base_flap_rate":0` hangs `cloth`, because the rate is
  their product and infinity times zero is NaN. No single field spells that NaN, so
  a check that looks at one field at a time cannot see it.

`*_from_json_str` takes fully attacker-controlled input, so one edited field could
stall a thread forever. The port's API reaches the same loop too:
`fire_new(NaN, sr)`, and `bubble_set_intensity`, `cloth_set_wind_speed`,
`precipitation_set_intensity` and `underwater_set_intensity` with NaN.

`tests/garjan.fcyr` found the JSON paths. Earlier notes said JSON could not reach
this hang; they were wrong, because they looked only at how JSON spells a number.

## Decision

**`rng_poisson` returns 0 for a NaN rate, without advancing the rng.**

This is a deliberate divergence from `rust-old`, which hangs on the same input.
+Inf still clamps to the ceiling, as in Rust. Every rate that is not NaN takes the
original path, so all valid output stays bit-identical: `scripts/audio-hash.cyr`
is unchanged across all 21 synths.

## Consequences

- **Positive** — one check closes every path into the loop, from JSON and from the
  API, including products of fields that no per-field validation would catch.
- **Positive** — synthesis always terminates, which is what a caller of
  `process_block` must be able to rely on.
- **Negative** — a NaN rate produces silence rather than an error, because
  `process_block` has no error path that could carry one. A caller that passes NaN
  gets a quiet synth and no diagnostic.
- **Neutral** — NaN stays in the synth's stored parameters. This decision makes the
  loop safe; it does not make a NaN parameter valid.

## Alternatives considered

- **Refuse non-finite fields in every `*_from_json_str`.** Closer to what serde did,
  but it needs a check per f64 field across 21 documents, leaves the API paths
  hanging, and still has to reject fields that are only harmful in combination.
- **Validate in each setter and constructor.** Several sites, and it does nothing
  for a rate that is a product of two stored fields.
- **Cap the loop's iterations.** It would terminate, but it returns a meaningless
  count, and it spends the cap on every NaN block for no audible result.
