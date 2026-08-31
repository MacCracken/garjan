# 001 — Deserialize does not restore DSP state (and Rust's did)

> **RESOLVED in 2.5.0** — see [ADR-0008](../adr/0008-serde-carries-live-dsp-state.md).
> Deserialize now restores the live state of every component, and the
> cross-module suite verifies it for all 21 synths by comparing *continued
> output after restore*. This note is kept for the history: what the divergence
> was, how the false `#[serde(skip)]` premise hid it, and why JSON round-trip
> tests could not see it.

Every `*_from_json_str` in `src/` rebuilds its naad components — noise
generators, biquad / state-variable filters, LFOs, modal banks, exciters —
**fresh from the scalar parameters**, rather than restoring their live state.
A synth restored mid-stream therefore resumes with zeroed filter history and a
reset noise sequence.

## This is a divergence from the oracle, not a mirror of it

Source comments in this tree used to say the port "mirrors Rust's
`#[serde(skip)]`". **That premise is false.** `rust-old/src` contains **zero**
`serde(skip)` attributes. Verify rather than trust:

```sh
grep -rn 'serde(skip' rust-old/src/    # no matches
```

Rust derived serialization over *all* fields, naad components included —
`rust-old/src/fire.rs:16`:

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Fire {
    intensity: f32,
    /* … */
    #[cfg(feature = "naad-backend")]
    roar_noise: naad::noise::NoiseGenerator,
    #[cfg(feature = "naad-backend")]
    roar_filter: naad::filter::BiquadFilter,
    /* … */
}
```

The `#[cfg]` gates whether the field *exists*, not whether it is serialized.
With `naad-backend` on (the default), Rust round-tripped the live DSP state.
The Cyrius port does not.

## What it costs

A save/restore that Rust made seamless is audible here: biquad `z1`/`z2`
history and noise-generator position are lost, so a restored synth clicks or
re-attacks where the original continued smoothly. It is invisible to the test
suite because the serde tests round-trip *parameters* and compare JSON, never
the audio after a restore.

`Rng` and `DcBlocker` state **are** preserved — they are plain scalars carried
through `*Params` — so the divergence is specific to the naad-owned components.

## How it was fixed

naad exposes accessors for all of it, so the state was reachable all along. 2.5.0
splices a `"dsp"` member into the existing params object — the derive parser
ignores unknown keys — and stores values as raw i64 bit patterns so the
round-trip is exact. Documents written before 2.5.0 still load, without live
state. See [ADR-0008](../adr/0008-serde-carries-live-dsp-state.md).

As of 2.5.0 serde round-tripping **is** session snapshotting. The lesson worth
keeping is the testing one: every serde test checked round-trip idempotence —
serialize, deserialize, re-serialize, compare strings — and that passes
perfectly while every filter is silently zeroed. The check that finds it is to
warm a synth, snapshot, restore, run **both** forward, and compare samples.
