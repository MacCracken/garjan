# 0013 — ModalBank and Exciter serialize standalone; a bank's document carries its modes

**Status**: Accepted
**Date**: 2026-11-16

## Context

Rust derived `Serialize` and `Deserialize` on `ModalBank` and `Exciter`, so a caller could
save either on its own. Through 2.7.0 the port could not. It carried their live state
only inside a host synth's `"dsp"` member (ADR-0008), and the 2.0.0 CHANGELOG said they
were "rebuilt from their inputs rather than serialized". That was decided but never
recorded as an ADR. It left `test_serde_roundtrip_modal_bank` and
`test_serde_roundtrip_exciter` as the last two unported `integration.rs` tests.

Porting them meets ADR-0010: a document restores only a state some sequence of public
calls could produce.

- **A bank's coefficients are derived values.** `modal_bank_new` computes each pole as
  `r·e^(iθ)` from a mode's frequency and decay, with `θ = 2π·f/sr`,
  `r = clamp(exp(−6.908 / (max(decay, 0.001)·sr)), 0, 0.9999)`, and modes below 20 Hz, at
  or above Nyquist, or with no decay dropped. Rust serialized `coeff_re` and `coeff_im`
  directly. A document carrying coefficients could set a pole outside the unit circle, or
  at an angle no frequency in range gives. A loader could bound-check the radius and the
  angle, but only to a tolerance, and never prove that a constructor made them.
- **The bank did not keep its modes.** After construction only the coefficients and
  amplitudes remained, so a writer had nothing else to write.
- **An exciter's fields are coupled.** While active, its position never passes its
  duration, and an impulse is spent after its first sample. An impulse marked active at a
  later position would report active forever. Rust's `usize` duration and position could
  not be negative, but `exciter_new` stored a negative duration as given.

## Decision

**Both types get a standalone codec. A bank's document carries its modes, not its
coefficients, and a load rebuilds the bank through `modal_bank_new`.**

- **`ModalBank` keeps its inputs.** It gains `frequency` and `decay` vecs, parallel to
  `amplitude`, holding each surviving mode's values as given. Construction pushes two more
  values per mode. Processing is unchanged.
- **`modal_bank_to_json`** writes
  `{"sample_rate":..,"modes":[{"frequency":..,"amplitude":..,"decay":..},..],"dsp":{"mb":[..]}}`.
  The modes go through the derived float writer, which is shortest-round-trip exact,
  including `-0.0` and denormals. The resonator state goes under `"dsp"` as raw bit
  patterns, the form a host synth's `"mb"` already takes.
- **`modal_bank_from_json_str`** rebuilds through `modal_bank_new`, so every coefficient is
  bit-identical and producible by construction. It refuses a mode the constructor would
  drop, because the writer never writes one. It refuses a state that is missing, not two
  integers per mode, or not finite, and a missing or non-finite member anywhere.
- **`exciter_to_json`** writes the seven fields flat: type, duration, amplitude, position,
  active flag, and the Rng's state and increment.
- **`exciter_from_json_str`** refuses a missing member, an out-of-range type, a negative
  duration or position, an active flag other than 0 or 1, an even Rng increment, and an
  active exciter whose position is past its duration (for an impulse, past 0). An inactive
  exciter may sit at any position, because `exciter_trigger` resets it and an inactive
  exciter yields 0.
- **`exciter_new` and `exciter_reset` refuse a negative duration**, so the API builds
  nothing the loader refuses. Every synth computes its duration from positive inputs, so
  no synth changes.

Both loaders are fuzz entry points, and both Rust tests are ported under their own names.

## Consequences

**Positive**

- Every `integration.rs` test now has a counterpart, and every Rust type that serialized
  serializes here.
- A loaded bank cannot hold a pole the constructor would not build, and the check is
  exact rather than a tolerance.
- An exciter cannot be loaded into the stuck-active state.

**Negative**

- Each bank holds two more vecs, a few words per mode. No synth builds banks per block.
- `exciter_new(type, -1, amp)` used to succeed and now returns
  `GARJAN_ERR_INVALID_PARAMETER`.
- The document shapes differ from Rust's: no `coeff_re` or `coeff_im`, and a flattened Rng
  (ADR-0014).

**Neutral**

- A host synth's `"dsp"` still carries only the bank's state and the exciter's position,
  flag and Rng, because the synth re-derives the rest from its own configuration. The
  standalone codecs leave that path alone.

## Alternatives considered

- **Serialize the coefficients, as Rust did, and bound-check them on load.** Rejected: the
  radius and angle checks need a tolerance around `exp`, `cos` and `sin` rounding, so they
  would accept poles no constructor builds, and could refuse ones it does.
- **Recover frequency and decay from the coefficients** with `atan2` and `ln`. Rejected:
  the round trip through transcendentals is not bit-exact, so a restored bank would not
  continue sample for sample (ADR-0008).
- **Keep the 2.0.0 decision and record it.** Rejected by the maintainer on 2026-11-16 in
  favour of closing the port.
