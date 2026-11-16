# 0014 — garjan's documents are not `rust-old`'s

**Status**: Accepted
**Date**: 2026-11-16

## Context

`rust-old` serialized every public type through serde, so its documents had serde's
default shapes. The port's documents differ from them in several ways, each decided
separately, and until now no ADR said so as a whole. A consumer with documents saved by
the Rust crate would find out only when one failed to load. The 2.7.0 audit of `rust-old/`
listed the gap.

The differences:

| Rust (serde) | garjan |
|---|---|
| An enum as its variant name, such as `"Metal"` or `{"NoiseBurst":{"duration_samples":64}}` | An integer id, as the port carries enums (ADR-0006) |
| `[f32; N]` fields as arrays, such as `"position":[x,y,z]` | Flattened scalars, `position_x`/`_y`/`_z` (ADR-0012) |
| Nested structs: `rng`, `dc_blocker` | Flattened into the parent: `rng_state`, `rng_inc`, `dcb_x_prev` |
| No live DSP state for naad components (`#[serde(skip)]`) | A `"dsp"` member with it, as raw bit patterns (ADR-0008) |
| A modal bank's `coeff_re` and `coeff_im` | Its modes, from which a load re-derives the coefficients (ADR-0013) |
| `f32` values | `f64` values |

garjan's loaders also refuse much that serde accepted (ADR-0010), and accept some members
serde did not know, such as water's `lfo_rate` (ADR-0011).

## Decision

**garjan's documents are its own format. A `rust-old` document is not expected to load,
and a garjan document is not expected to load in `rust-old`.** No converter is provided.

Within garjan the format is stable in one direction: a document an older 2.x release
wrote still loads, unless ADR-0010 refuses a state it carries, or it carries a NaN the API
stored (written as `null`).

## Consequences

**Positive**

- Each format decision follows what the port needs: integer enums, exact live state,
  re-derived values. None is bent to match serde's shapes.
- A consumer reading this ADR knows not to migrate Rust-era documents by loading them.

**Negative**

- No Rust-era document can be carried forward without hand conversion. There are no known
  Rust-era consumers with saved documents.

**Neutral**

- If one appears, a converter belongs in that consumer's tooling, built from the table
  above.

## Alternatives considered

- **Match serde's shapes.** Rejected: string enum names would need a name table per enum,
  `[f32; N]` arrays and nested objects a hand-written reader per type, since the derive
  cannot express them, and the live state ADR-0008 requires has no serde shape at all.
- **Ship a converter.** Rejected for now: there is no consumer with Rust-era documents to
  convert.
