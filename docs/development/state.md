# garjan — Current State

> Refreshed every release. CLAUDE.md is preferences/process/procedures
> (durable); this file is **state** (volatile).

## Version

**2.0.5** (2026-08-30) — per-sample hot-path optimization: loop-invariant work
hoisted across 9 modules, and the redundant second DC-blocking pass folded into
generation for `wind` and `texture`. Bit-exact, verified with
[`scripts/audio-hash.cyr`](../../scripts/audio-hash.cyr). wind −5.6%, thunder
−3.3%, fire −2.2%. Most remaining time is inside naad, not garjan.

**2.0.4** (2026-08-30) — arena lifetime: the two streaming paths that allocated
per block (`impact_generate`'s fresh `Exciter`, 64 B; `texture_process_block`'s
band-mix buffer, 24 B) now allocate **zero**, via `exciter_reset` + a stack
buffer. Bit-identical audio, asserted directly. ~27 MB/hour of growth removed.

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

- **33 module suites** in `tests/*.tcyr`, **478 assertions, all green**.
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

- **Arena lifetime, remaining.** 2.0.4 fixed the two per-block streaming
  allocations. Still outstanding: every `*_synthesize` grows its output vec from
  capacity 16 (~12 doubling reallocations per second of audio, all retained),
  and the allocator has no free at all, so constructing and discarding synths in
  a loop still grows the arena. `alloc_reset()` invalidates *every* outstanding
  pointer, so it is only usable at a clean epoch boundary.
- **Duration / sample-rate upper bounds — needs an ADR.** Currently unbounded
  (1e8 s sizes a 4.4-trillion-sample buffer). Faithful to `rust-old`, so
  capping is a deliberate divergence, not a bug fix.
- **Further perf needs naad-level work.** 2.0.5 took the hoisting wins
  (−5.6% on the worst synth); profiling now points into the naad bundle's
  per-sample noise generation and biquad/SVF filtering, not garjan's own
  arithmetic. Remaining garjan-side candidates are small: per-event conversions
  in `bubble`/`foliage`/`precipitation`/`insect`/`whoosh`/`impact`.
- Port `integration/soorat.rs` (visualization data) once soorat lands in Cyrius.
