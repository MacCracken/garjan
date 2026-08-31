# Contributing to garjan

Thank you for your interest in contributing to garjan.

garjan is a **Cyrius** project — a port of an earlier Rust crate, which is kept
frozen at [`rust-old/`](rust-old/) as the parity oracle. There is no cargo here.

## Development Workflow

1. Fork and clone the repository
2. Create a feature branch from `main`
3. Make your changes
4. Run the cleanliness check (below)
5. Open a pull request

## Prerequisites

- The Cyrius toolchain. **Do not hardcode a version** — the pin in
  `cyrius.cyml [package].cyrius` is the single source of truth, and CI installs
  from it:

  ```sh
  curl -sSf https://raw.githubusercontent.com/MacCracken/cyrius/main/scripts/install.sh \
    | CYRIUS_VERSION="$(grep '^cyrius = ' cyrius.cyml | sed 's/.*"\(.*\)"/\1/')" sh
  ```

## Cleanliness Check

```sh
cyrius deps                              # resolve + vendor dependencies
cyrius distlib                           # REGENERATE dist/ -- see the trap below
cyrius build src/main.cyr build/garjan   # must be WARNING-clean, not just OK
cyrius test                              # 33 suites
cyrius fuzz
cyrius audit                             # fmt / lint / docs / tests / bench
```

Three traps that have each bitten this project:

- **`cyrius build` does not regenerate `dist/`.** `src/main.cyr` includes
  `dist/garjan.cyr`, so a build after editing `src/` compiles the **stale**
  bundle and still reports `OK`. Always `cyrius distlib` after a source edit;
  `cyrius distlib --check` reports staleness correctly.
- **`cyrius fmt --check` false-negatives.** It has reported a file clean that
  the formatter then rewrote. Verify by copying to a scratch dir, running
  `cyrius fmt`, and diffing.
- **A warning-clean build matters more than `OK`.** An undefined *function* is
  only a hard error when the compiler can reach it from the entry point, and
  `src/main.cyr` is a small smoke harness that reaches little of the library —
  so a symbol broken by an upstream rename can degrade to a bare warning while
  the build still says `OK`.

## Code Conventions

- Constructors return a **heap pointer or a negative `GARJAN_ERR_*` code**;
  callers check with `garjan_is_err`. There is no `Result`.
- `#must_use` on pure functions; `#derive(accessors)` on structs.
- **Never call `alloc` directly** — use `garjan_alloc`, which maps allocation
  failure to a negative code. Raw `alloc` returns `0`, and `0` is `GARJAN_OK`,
  so a failure would pass every error check
  ([ADR-0005](docs/adr/0005-allocation-failure-is-an-error-code-not-an-abort.md)).
- **Validate enum ids** with `garjan_enum_invalid` in any new constructor that
  takes one. Ported Rust enums are plain integers, so an unvalidated id falls
  through an `if`/`elif` chain's final `else` and silently selects the last
  variant
  ([ADR-0006](docs/adr/0006-out-of-range-enum-ids-are-rejected.md)).
- `garjan_validate_sample_rate` in every constructor,
  `garjan_validate_duration` + `garjan_validate_sample_count` in every
  `synthesize`.
- `process_block` must not allocate. A test pins this.
- DC-block every synthesis output.
- Cross-check behaviour against `rust-old/`. Diverge only with an ADR.

## Changing DSP or Optimizing

**The test suite does not verify sample values** — it asserts finiteness,
energy and serde round-trips. An optimization can pass all 33 suites while
silently changing the audio. Use the bit-exactness oracle:

```sh
cyrius build scripts/audio-hash.cyr build/audio-hash && ./build/audio-hash > before.txt
# ...make the change, then cyrius distlib and rebuild...
./build/audio-hash > after.txt && diff before.txt after.txt
```

Hoisting loop-invariants is bit-exact. **Reassociating floating-point
arithmetic is not.** And do not hoist constants for speed — cycc already folds
them; hoist accessor reads instead.

## Adding a New Synthesizer

1. Create `src/my_synth.cyr` following an existing module.
2. Put shared enums in the appropriate types module (`contact.cyr`, `aero.cyr`,
   `creature.cyr`) as module-prefixed `var` constants, values matching the Rust
   enum's discriminants.
3. Register it in `cyrius.cyml`'s `[lib] modules` list, **in dependency order**.
4. Add `tests/my_synth.tcyr`: all variants, silence gate, serde round-trip.
5. Extend the cross-module sweeps in `tests/garjan.tcyr` and add a benchmark to
   `tests/garjan.bcyr` — with the driving parameter set, or you are timing a
   silent fast-path.
6. Check scope boundaries — does this belong in garjan or a sibling crate?

## Scope Boundaries

Before adding new sound categories, check whether the sound belongs in garjan or
a sibling crate. See [ADR-0003 — Scope boundaries](docs/adr/0003-scope-boundaries.md).

## License

By contributing, you agree that your contributions will be licensed under GPL-3.0-only.
