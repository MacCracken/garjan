# 001 — Deserialize does not restore DSP state (and Rust's did)

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

## Why it is still like this

The port had no way to read live state out of naad's bundle types when it was
written. Changing it is a real decision with a format consequence (`*Params`
would grow the component state, so stored JSON breaks), which is why it is
scheduled as a 2.2.x item requiring an ADR rather than quietly fixed — see
[`../development/roadmap.md`](../development/roadmap.md).

Until that lands, treat serde round-tripping as **parameter persistence, not
session snapshotting**.
