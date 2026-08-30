# garjan — Claude Code Instructions

> **Core rule**: this file is **preferences, process, and procedures** —
> durable rules that change rarely. Volatile state (current version,
> module line counts, port progress, test counts, consumers) lives in
> [`docs/development/state.md`](docs/development/state.md).
> Do not inline state here.

## Project Identity

**garjan** — Cyrius port of a Rust project (6186 lines preserved at `rust-old/`).

- **Type**: Port (Rust → Cyrius)
- **License**: GPL-3.0-only
- **Language**: Cyrius (toolchain pinned in `cyrius.cyml [package].cyrius`)
- **Version**: `VERSION` at the project root is the source of truth — do not inline the number here
- **Standards**: [First-Party Standards](https://github.com/MacCracken/agnosticos/blob/main/docs/development/applications/first-party-standards.md) · [First-Party Documentation](https://github.com/MacCracken/agnosticos/blob/main/docs/development/applications/first-party-documentation.md)

## Goal

garjan owns **environmental and nature sound *source* generation** for the
AGNOS stack — what a thing sounds like at the point it is made: weather
(thunder, rain, wind), fire, water and fluids, impacts and surface contact
(footsteps, friction, rolling, creak), aerodynamics (whoosh, whistle, cloth),
foliage, and creature *mechanisms* (insect stridulation, wing flaps).

It deliberately owns nothing downstream of the source. Propagation belongs to
**goonj**, mixing and scheduling to **dhvani**, RTPC mapping to
**dhvani/kiran**, mechanical sounds to **ghurni**, and anything vocal — speech,
bird song, growls — to **prani/svara**. garjan therefore carries no dependency
on a sibling audio crate; the `bridge` module converts physical units at the
boundary using primitive values only. The full boundary table, including the
Doppler and creature-sound edge cases, is
[ADR-0003](docs/adr/0003-scope-boundaries.md).

## Current State

> Volatile state lives in [`docs/development/state.md`](docs/development/state.md) —
> port progress, surface parity, in-flight work. Refreshed every release.

This file (`CLAUDE.md`) is durable rules.

## Scaffolding

Project was scaffolded with `cyrius port`. Original Rust at `rust-old/` is the reference oracle — do not modify it; cross-check the port against it.

## Quick Start

```sh
cyrius deps                              # resolve dependencies
cyrius build src/main.cyr build/garjan    # compile
cyrius test                              # run tests/*.tcyr
```

## Key Principles

- **Cross-check against `rust-old/`** — the port's correctness bar is "matches what Rust did". Diverge only with an ADR.
- **Correctness over cleverness** — if the Cyrius behavior diverges silently from Rust, the bugs win
- Test after every change, not after the feature is "done"
- ONE change at a time — never bundle unrelated changes
- Build with `cyrius build`, not raw `cat file | cc5` — the manifest auto-resolves deps
- Source files only need project includes — stdlib auto-resolves from `cyrius.cyml`
- `var buf[N]` = N **bytes**, not N entries

## Rules (Hard Constraints)

- **Do not commit or push** — the user handles all git operations
- **Never use `gh` CLI** — use `curl` to the GitHub API if needed
- Do not modify `rust-old/` — it's the parity oracle
- Do not skip tests before claiming changes work
- Do not modify `lib/` files (vendored stdlib / dep symlinks)
- Do not hardcode toolchain versions in CI YAML — `cyrius = "X.Y.Z"` in `cyrius.cyml` is the source of truth

## Documentation

- [`docs/adr/`](docs/adr/) — Architecture Decision Records (*why X over Y?*)
- [`docs/architecture/`](docs/architecture/) — Non-obvious constraints
- [`docs/guides/`](docs/guides/) — Task-oriented how-tos
- [`docs/examples/`](docs/examples/) — Runnable examples
- [`docs/development/state.md`](docs/development/state.md) — Live state
- [`docs/development/roadmap.md`](docs/development/roadmap.md) — Milestones through v1.0

