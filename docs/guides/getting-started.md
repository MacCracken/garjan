# Getting started with garjan

## Build

```sh
cyrius deps                              # resolve + vendor dependencies into lib/
cyrius distlib                           # regenerate dist/garjan.cyr from src/
cyrius build src/main.cyr build/garjan   # compile the smoke entry
cyrius test                              # run tests/*.tcyr
```

> **`cyrius build` does not regenerate `dist/`.** `src/main.cyr` includes
> `dist/garjan.cyr`, so building after editing `src/` compiles the **stale**
> bundle and still reports `OK`. Run `cyrius distlib` after any source edit;
> `cyrius distlib --check` reports staleness correctly.

## Layout

- `src/*.cyr` — the library: 32 modules, listed in dependency order in
  `cyrius.cyml`'s `[lib] modules`. Modules do not `include` each other; the
  entry or test harness sets the order.
- `src/main.cyr` — a small **smoke entry**, not the library. It includes the
  dist bundle and exercises a few paths so the bundle is known to compile and
  link.
- `dist/garjan.cyr` — the generated single-file bundle consumers include.
- `tests/` — 33 `.tcyr` suites (auto-discovered), plus `garjan.bcyr`
  (benchmarks) and `garjan.fcyr` (fuzz).
- `scripts/audio-hash.cyr` — the bit-exactness oracle for DSP changes.
- `rust-old/` — the original Rust source, frozen as the parity oracle. **Do not
  modify it.**
- `lib/` — vendored stdlib and dependency bundles, managed by `cyrius deps`.
  **Do not modify these either.**

## Using garjan from another project

Include the bundle and declare garjan's dependencies in your own `cyrius.cyml`,
exactly as this repo does. Constructors return a heap pointer or a negative
`GARJAN_ERR_*` code — check with `garjan_is_err`:

```cyrius
include "dist/garjan.cyr"

fn main() {
    alloc_init();
    var fire = fire_new(F64_HALF, f64_from(44100));
    if (garjan_is_err(fire) == 1) { return 1; }
    var samples = fire_synthesize(fire, F64_HALF);
    if (garjan_is_err(samples) == 1) { return 1; }
    return 0;
}
var r = main();
syscall(60, r);
```

Five worked programs are in [`../examples/`](../examples/). For real-time use,
error handling and persistence, see the
[integration guide](../development/integration-guide.md).

## Adding a module

1. Create `src/my_module.cyr`. It must be self-contained — no `include` lines.
2. Register it in `cyrius.cyml`'s `[lib] modules`, **after** anything it depends
   on.
3. Cross-check behaviour against `rust-old/`. Diverging needs an ADR.
4. Add `tests/my_module.tcyr`, and extend the cross-module sweeps in
   `tests/garjan.tcyr`.
5. `cyrius distlib && cyrius test`.
6. Bump `VERSION` and add a CHANGELOG entry before tagging.

See [`../adr/template.md`](../adr/template.md) when a design choice deserves an
ADR, and [CONTRIBUTING](../../CONTRIBUTING.md) for the conventions the code
follows — error codes over `Result`, `garjan_alloc` over raw `alloc`, and enum
id validation at every boundary.
