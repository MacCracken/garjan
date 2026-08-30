# garjan — Current State

> Refreshed every release. CLAUDE.md is preferences/process/procedures
> (durable); this file is **state** (volatile).

## Version

**2.0.3** (2026-08-30) — completes the allocation-failure hardening deferred
from 2.0.2. `alloc` returns `0` on exhaustion and `garjan_is_err` only detects
`< 0`, so an OOM was a store to address 0; all allocations now route through
`garjan_alloc` -> `GARJAN_ERR_ALLOCATION` (`-4`), with 132 guards across the 85
direct sites and every unchecked helper result. See
[ADR-0005](../adr/0005-allocation-failure-is-an-error-code-not-an-abort.md).

**2.0.2** (2026-08-30) — security/hardening from a P-1 audit sweep. Closed
three defects reachable from untrusted JSON: `*_from_json_str` discarded
`*_build_naad`'s error code (20 of 21 sites; a null-deref SIGSEGV, confirmed and
re-confirmed fixed), `voice_pool_from_json_str` sized the pool from `max_voices`
instead of the slots array, and `insect_from_json_str` dropped swarm_count's
1..8 clamp. Also fixed the `cyrius audit` fmt gate. No behavior change for valid
input.

**2.0.1** (2026-08-29) — maintenance: toolchain 6.3.44 → 6.5.36, all four
first-party deps bumped to latest, vendored stdlib re-synced. Two mechanical
renames forced by upstream (`FILTER_*` → `NAAD_FILTER_*`,
`bayan_json_v_parse_str` → `_buf`); no behavior change.

**2.0.0** — full Rust→Cyrius port complete (2026-07-03). Major bump: the public
API moves from Rust to Cyrius (breaking for all consumers). The 6,186-line Rust
crate is preserved at `rust-old/` for parity reference (frozen, do not edit).

## Toolchain

- **Cyrius pin**: `6.5.36` (in `cyrius.cyml [package].cyrius`)

## Source

- Rust reference: 6,186 lines at `rust-old/` (frozen).
- Cyrius port: **32 modules** in `src/*.cyr` — foundations (error, logging,
  dsp, rng, lod, material, modal, contact, aero, creature, voice) + 19
  synthesizers (fire, weather=thunder/rain/wind, water, precipitation, bubble,
  impact, footstep, friction, creak, rolling, foliage, whoosh, whistle, cloth,
  insect, wingflap, underwater, surf, texture) + builder + bridge. Entry
  `src/main.cyr`; distlib bundle `dist/garjan.cyr`.

## Tests

- **33 module suites** in `tests/*.tcyr`, **472 assertions, all green**.
  Covers parity (incl. bit-exact PCG32), synthesis finiteness/energy, and
  serde roundtrips. `cyrius test` runs them; each also builds standalone.
- Cleanliness: `cyrius lint` 0 warnings (33 modules), `cyrius vet` clean,
  `cyrfmt --check` clean (33/33), `cyrius distlib --check` in sync;
  `tests/garjan.fcyr` fuzz harness passes; 0 lines > 120 chars in `src/`.
- Formatting follows the 6.5.36 canon: continuation lines indent 2 spaces per
  open paren (4 also accepted), replacing the port's original paren-aligned
  style. Adopted as a whitespace-only 48-line reindent across 17 `src/` files;
  build and all 460 assertions unchanged. CI does not gate on fmt.
- Benchmarks: `cyrius bench tests/garjan.bcyr` — see [`../../BENCHMARKS.md`](../../BENCHMARKS.md).

## Dependencies

Declared in `cyrius.cyml`:

- **stdlib** — syscalls, string, alloc, str, fmt, vec, io, args, assert, math,
  ganita, bayan, bench
- **naad** 2.2.2 (noise/filter/LFO via the dist bundle)
- **hisab** 2.11.2, **goonj** 2.0.4 (naad's transitive deps, resolved here)
- **sakshi** 2.4.11 (logging — `src/logging.cyr`)

These tags are what the dependencies themselves pin, so the flattened graph is
consistent: naad 2.2.2 → hisab 2.11.2 + goonj 2.0.4; goonj 2.0.4 → hisab
2.11.2; hisab 2.11.2 → sakshi 2.4.11. `lib/tagged.cyr` + `lib/callback.cyr`
joined the vendored set as new leaf requirements. Symbol-collision audit:
garjan's 399 top-level `fn`/`var` symbols intersect every dependency bundle at
zero — re-run it on any dep bump, since Cyrius gives no diagnostic for a
duplicate top-level `var`.

## Consumers

_None yet (kiran, joshua, dhvani + any AGNOS component needing environmental audio)._

## Next

- **Arena lifetime.** Nothing is ever freed, so long-running streams grow
  without bound — `texture_band_mix` allocates per `texture_process_block`, and
  `impact_process_block` builds a fresh `Exciter` (and `Rng`) per call. 2.0.3
  fixed OOM *detection*; this is the separate problem of not reaching OOM.
- **Duration / sample-rate upper bounds — needs an ADR.** Currently unbounded
  (1e8 s sizes a 4.4-trillion-sample buffer). Faithful to `rust-old`, so
  capping is a deliberate divergence, not a bug fix.
- Optimize per-sample hot paths (the f64 port calls the naad bundle
  unoptimized). The audit catalogued loop-invariant constants rebuilt per sample
  across ~17 modules plus a redundant second DC-blocking pass in the block
  synths; `wind` (11.0 ms/s of audio) is the highest-value target.
- Port `integration/soorat.rs` (visualization data) once soorat lands in Cyrius.
