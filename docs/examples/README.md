# Examples

Runnable Cyrius programs, ported from `rust-old/examples/`. Each includes the
dist bundle, so build them like any other program:

```sh
cyrius build docs/examples/<name>.cyr build/<name> && ./build/<name>
```

| Example | Shows |
|---|---|
| [`weather_scene.cyr`](weather_scene.cyr) | Layering thunder + rain + wind; peak/RMS of the mix |
| [`forest_ambience.cyr`](forest_ambience.cyr) | Continuous layers (texture, foliage, insects) plus a one-shot wing flap offset to t=2s |
| [`combat_impacts.cyr`](combat_impacts.cyr) | Impact synthesis, material interaction, velocity sensitivity, and the voice pool |
| [`error_handling.cyr`](error_handling.cyr) | The pointer-or-negative-code convention, `garjan_is_err` / `garjan_err_name`, and propagation |
| [`logging.cyr`](logging.cyr) | Runtime log gating via `sakshi_set_level` |

Mixing in these examples is a plain weighted sum, for illustration only. Mixing,
buses and scheduling belong to **dhvani**; garjan only produces the sources —
see [ADR-0003](../adr/0003-scope-boundaries.md).

Two things differ from the Rust originals by design:

- **Errors are integer codes, not `Result<T, GarjanError>`.** Constructors
  return a heap pointer or a negative code; test with `garjan_is_err`.
- **Logging has no feature gate.** Rust put it behind `logging` and required the
  application to install a `tracing` subscriber. The port always compiles
  `src/logging.cyr` (sakshi) in and gates at runtime with `sakshi_set_level`.
