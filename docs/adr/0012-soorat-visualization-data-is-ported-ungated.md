# 0012 — soorat's visualization data is ported ungated, with bounded sizes

**Status**: Accepted
**Date**: 2026-11-16

## Context

`rust-old/src/integration/soorat.rs` (315 lines) was the last unported Rust module. It
defines three visualization structs a renderer can draw beside garjan's audio:
`PrecipitationField` (a vector of `PrecipitationParticle`), `FireEmitter` and `WindField`
(a grid of `[f32; 2]` velocities). Rust compiled it only under the `soorat-compat` feature.

Through 2.6.3 the roadmap gated the port on soorat landing in Cyrius. The module calls
nothing in soorat, though: it is plain data built from primitives, as `bridge.rs` is, and
its only dependency was serde. soorat is still mid-port, and the maintainer decided on
2026-11-16 not to gate garjan's work on it.

Porting it as written raises four questions the Rust code did not have to answer:

1. Rust's constructors cannot fail. `count`, `nx` and `nz` are `usize`, so they cannot be
   negative. `nx * nz` wraps silently in a release build, and `Vec::with_capacity(count)`
   aborts on a huge `count`. In Cyrius these are i64, and an allocation can be refused.
2. Cyrius has no fixed-size arrays in a struct or an argument list, so `[f32; 3]` and
   `[f32; 2]` need another shape.
3. serde wrote `"position":[x,y,z]` and `"dimensions":[nx,nz]`, and read any document whose
   `velocities` length disagreed with `dimensions`. A renderer indexing `z * nx + x` reads
   out of bounds on such a document.
4. The derive reader garjan uses elsewhere loads a missing member as 0 (ADR-0010's first
   known gap).

## Decision

Port the module now as `src/soorat.cyr`, compiled always, with these choices:

- **f32 → f64**, as everywhere else in the port. Values differ from Rust's in the last
  f32 bits, as every synth's do.
- **Arrays are flattened.** Struct fields become `position_x`/`_y`/`_z`,
  `wind_bias_x`/`_z`, `nx`/`nz`, `origin_x`/`_y`/`_z`. Array arguments become scalar
  arguments in the same order. `Vec<PrecipitationParticle>` becomes a stdlib vec of
  particle pointers, and `Vec<[f32; 2]>` a flat f64 vec holding vx then vz per point.
- **Sizes are bounded and refused, never wrapped.** `GARJAN_MAX_FIELD_POINTS` is 2^20 =
  1,048,576 points per field, about 72 MB of particles. A negative count or grid size, a
  count past the cap, and a grid whose `nx * nz` would pass it return
  `GARJAN_ERR_INVALID_PARAMETER`. The product is tested by division, so it cannot wrap. A
  refused allocation returns `GARJAN_ERR_ALLOCATION` (ADR-0005).
- **Non-finite inputs pass through.** A NaN direction gives NaN velocities, as in Rust and
  as in `bridge.cyr`. The writer then emits `null`, which the loader refuses, as serde did.
- **The JSON shape follows the flattened fields.** Each writer emits a derived scalar
  object and splices its array in. Each loader goes through `garjan_json_open`, then reads
  every member by hand and **refuses a missing member**, which closes ADR-0010's first gap
  for these three types. It also refuses a wind field whose velocity count is not
  `nx * nz`, and a precipitation field with more than `GARJAN_MAX_FIELD_POINTS` particles.

## Consequences

**Positive**

- Every Rust module now has a Cyrius home, and no garjan release waits on soorat.
- A hostile count, grid size or document is refused before anything is allocated, and the
  fuzz harness drives all three loaders and every constructor argument (25 entry points).
- A loaded wind field is always safe to index by `z * nx + x`.

**Negative**

- Two documents a Rust consumer could exchange are not interchangeable: the shapes differ,
  as they already did for every enum-bearing synth document.
- A field larger than 2^20 points needs splitting into several fields. Rust had no cap.
- Construction can fail where Rust's could not, so a caller checks `garjan_is_err`.

**Neutral**

- When soorat consumes these types, it may want a flat f32 buffer instead of a vec of
  particle pointers. That is a new API to add then, not a reason to wait now.

## Alternatives considered

- **Keep the gate until soorat lands.** Rejected: the module never depended on soorat, so
  the gate delayed a finished port for nothing.
- **Mirror Rust's JSON arrays** (`"position":[x,y,z]`). Rejected: the stdlib exposes no
  round-trip-exact float writer outside the derive, and garjan's documents already differ
  from Rust's for every enum.
- **Leave sizes unbounded, as Rust did.** Rejected: ADR-0007 already bounds every other
  size a caller controls, and an unbounded `count` lets one call exhaust an arena that
  never frees.
- **Store particles as one flat f64 buffer.** Rejected for now: `PrecipitationParticle`
  is a named public type in Rust, and a struct per particle keeps its accessors.
