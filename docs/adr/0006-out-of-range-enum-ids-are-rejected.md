# 0006 — Out-of-range enum ids are rejected, not absorbed

**Status**: Accepted
**Date**: 2026-08-30

## Context

Rust's enums made an invalid variant **unrepresentable**. `Impact::new` took a
`Material`, so `Material::from(99)` never existed to be passed; the exhaustive
`match` inside `material_properties` was total by construction, and the compiler
enforced it at every call site.

The port carries enums as module-prefixed integers (`MATERIAL_METAL = 0` …
`MATERIAL_CERAMIC = 9`) and dispatches with `if`/`elif` chains. Those chains end
in a bare `else`, so an out-of-range id does not fail — it **silently selects
the last variant's table**. `material_properties(99)` returned Ceramic.
`surf_new(0.5)` returned Storm. No error, no log, just quietly wrong audio.

This is not theoretical. During 2.0.5 and 2.1.0 work, the benchmark harness, the
`scripts/audio-hash.cyr` bit-exactness oracle, and the cross-module integration
suite all passed an f64 where `surf_new` and `underwater_new` expect an enum id
— `F64_HALF`, and a depth in metres. An f64's *bit pattern* is a huge integer,
so every one of those calls fell through to the final `else`:

- the "surf renders at every intensity" sweep tested **Storm four times**;
- the "underwater renders at every depth" sweep tested **Shallow three times**;
- the published `surf` and `underwater` benchmark figures were measuring the
  wrong configuration entirely.

Nothing caught it. The suite passed, the benchmarks reported plausible numbers,
and the audit that verified all 106 enum variants had the correct discriminant
never checked that callers were *passing* ids at all.

## Decision

Reject out-of-range ids at the public boundary, returning
`GARJAN_ERR_INVALID_PARAMETER`.

A predicate in the L0 module, `src/error.cyr`:

```cyrius
#must_use
fn garjan_enum_invalid(id, max_id) {
    if (id < 0) { return 1; }
    if (id > max_id) { return 1; }
    return 0;
}
```

applied at two layers:

1. **21 public constructors and entry points** — every `*_new` taking an enum
   id, plus `impact_generate`'s per-call `impact_type` and `voice_pool_new`'s
   `steal_policy`.
2. **10 pointer-returning table dispatchers** — `material_properties`,
   `material_mode_config`, `terrain_noise_config`, `movement_config`,
   `stone_size_config`, and the five `aero_*`/`creature_*` config builders.
   These already returned "pointer or negative code", and all 30 of their call
   sites already check `garjan_is_err` (a consequence of the 2.0.3 allocation
   guard), so the code propagates cleanly with no caller changes.

The bound is written as the **named** maximum variant (`MATERIAL_CERAMIC`, not
`9`), so adding a variant does not silently narrow the accepted range.

Deliberately **not** covered: dispatchers returning a bare `f64`
(`lod_mode_factor`, `rain_intensity_amplitude`, `impact_type_force`,
`friction_filter_freq`, `creak_shape_filter_freq`). A negative return is a
legitimate value there, so an error code would be ambiguous. They are reachable
only with an id that the constructor layer has already validated.

`garjan_enum_invalid` does not log, for the same reason `garjan_alloc` does not:
`error.cyr` is L0, and the `aero`/`creature`/`rng`/`voice` test entries include
it *without* `logging.cyr`, where a reachable undefined function is a hard build
error since cyrius 6.5.36.

## Consequences

**Positive**

- The failure mode changes from *silently wrong audio* to a checked error at the
  call site — restoring, by validation, the guarantee Rust got from its types.
- Callers already using `garjan_is_err` need no changes.
- It immediately caught a real defect in this project's own test and benchmark
  harnesses (above), which had gone unnoticed through three releases.

**Negative**

- A divergence from the oracle in the strict sense: Rust had no runtime path for
  an invalid id because it had no invalid id. The port now has one, and returns
  an error Rust never could. This is what makes it an ADR rather than a fix.
- Callers that were relying — knowingly or not — on the `else` fallthrough to
  mean "default variant" will now get an error. There are no external consumers
  yet, so the blast radius is zero today.
- A small per-call cost on the dispatchers (two integer compares), off the
  per-sample path.

**Neutral**

- The guarantee holds only where the guard is applied. A new constructor that
  takes an enum id without calling `garjan_enum_invalid` reopens the hole
  silently. The regression tests in `tests/garjan.tcyr` pin the current surface,
  including the specific f64-where-an-id-belongs case that motivated this.
