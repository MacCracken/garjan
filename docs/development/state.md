# garjan — Current State

> Refreshed every release. CLAUDE.md is preferences/process/procedures
> (durable); this file is **state** (volatile).

## Version

**1.1.0** — full Rust→Cyrius port complete (2026-07-03). The 6,186-line Rust
crate is preserved at `rust-old/` for parity reference (frozen, do not edit).

## Toolchain

- **Cyrius pin**: `6.3.44` (in `cyrius.cyml [package].cyrius`)

## Source

- Rust reference: 6,186 lines at `rust-old/` (frozen).
- Cyrius port: **32 modules** in `src/*.cyr` — foundations (error, logging,
  dsp, rng, lod, material, modal, contact, aero, creature, voice) + 19
  synthesizers (fire, weather=thunder/rain/wind, water, precipitation, bubble,
  impact, footstep, friction, creak, rolling, foliage, whoosh, whistle, cloth,
  insect, wingflap, underwater, surf, texture) + builder + bridge. Entry
  `src/main.cyr`; distlib bundle `dist/garjan.cyr`.

## Tests

- **32 module suites** in `tests/*.tcyr`, **425 assertions, all green**.
  Covers parity (incl. bit-exact PCG32), synthesis finiteness/energy, and
  serde roundtrips. `cyrius test` runs them; each also builds standalone.
- Cleanliness: `cyrfmt --check`, `cyrius lint`, `cyrius vet` all clean;
  0 lines > 120 chars.
- Benchmarks: `cyrius bench tests/garjan.bcyr` — see [`../../BENCHMARKS.md`](../../BENCHMARKS.md).

## Dependencies

Declared in `cyrius.cyml`:

- **stdlib** — syscalls, string, alloc, str, fmt, vec, io, args, assert, math,
  ganita, bayan, bench
- **naad** 2.1.0 (noise/filter/LFO via the dist bundle)
- **hisab** 2.6.7, **goonj** 2.0.0 (naad's transitive deps, resolved here)
- **sakshi** 2.4.3 (logging — `src/logging.cyr`)

## Consumers

_None yet (kiran, joshua, dhvani + any AGNOS component needing environmental audio)._

## Next

- Port `integration/soorat.rs` (visualization data) once soorat lands in Cyrius.
- Optimize per-sample hot paths (the f64 port calls the naad bundle unoptimized).
