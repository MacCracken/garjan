# garjan — Current State

> Refreshed every release. CLAUDE.md is preferences/process/procedures
> (durable); this file is **state** (volatile).

## Version

**2.5.0** (2026-08-30). Per-release detail is in the
[CHANGELOG](../../CHANGELOG.md); this file is current state only.

The 2.x line so far: the Rust→Cyrius port (2.0.0), a toolchain and dependency
refresh (2.0.1), a P-1 security audit and its repairs (2.0.2-2.0.4), hot-path
optimization (2.0.5, 2.3.0), parity completion — integration suite, benchmarks,
examples (2.1.0) — boundary validation (2.2.0, 2.4.0), and serde live-state parity (2.5.0) — the
last outstanding divergence from the Rust oracle.

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

- **33 module suites** in `tests/*.tcyr`, **825 assertions, all green**.
  `tests/garjan.tcyr` is the cross-module integration suite (288 of those),
  ported from `rust-old/tests/integration.rs`.
  Covers parity (incl. bit-exact PCG32), synthesis finiteness/energy, and
  serde roundtrips. `cyrius test` runs them; each also builds standalone.
- Cleanliness: `cyrius lint` 0 warnings (33 modules), `cyrius vet` clean,
  `cyrius audit` green end to end, `cyrius distlib --check` in sync,
  `tests/garjan.fcyr` fuzz passes, 0 lines > 120 chars in `src/`.
- ⚠ **Verify formatting by running the formatter and diffing, not with
  `cyrius fmt --check`** — `--check` has reported a file clean that the
  formatter then rewrote (that false negative is what broke the `cyrius audit`
  fmt gate at 2.0.2). Formatting follows the 6.5.36 canon: continuation lines
  indent 2 spaces per open paren, 4 also accepted.
- ⚠ **`cyrius build` does not regenerate `dist/`** — it compiles the stale
  bundle and still reports `OK`. Run `cyrius distlib` after any `src/` edit.
- Benchmarks: **26 ops**, `cyrius bench tests/garjan.bcyr` — see
  [`../../BENCHMARKS.md`](../../BENCHMARKS.md). `insect` (swarm of 8) is the
  slowest at ~23x real-time.
- Examples: 5 runnable programs in [`../examples/`](../examples/).
- `process_block` allocates nothing, pinned by test — **except `whistle`**,
  which allocates 32 B/sample (~5 GB/hour) via naad's heap-returning
  `filter_svf_process_sample`. Blocked on an upstream naad addition; see
  [`roadmap.md`](roadmap.md).

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

Sequencing lives in [`roadmap.md`](roadmap.md). Immediate:

- ✅ **serde live-state parity shipped in 2.5.0**
  ([ADR-0008](../adr/0008-serde-carries-live-dsp-state.md)). All 21 synths
  resume sample-identically after a restore. Nothing is queued behind it.
- **Blocked upstream**: a stdlib `vec_with_capacity` (~695 KB wasted per second
  of synthesized audio) and a naad `filter_svf_process_sample_bandpass`
  (`whistle` allocates 32 B/sample = ~5 GB/hour). Neither is fixable inside
  garjan without coupling to a dependency's internals.
- **Gated**: `integration/soorat.rs`, until soorat lands in Cyrius.

## Port completeness

Verified and re-verified 2026-08-30: **30 of 34** modules ported; **all 106 enum
variants, 103 preserving Rust's exact discriminant** (the 3 exceptions are
`GarjanError`, deliberately remapped); ~223 of 242 public items compared. Also
dropped deliberately: ~232 lines of non-naad fallback across 19 modules.

Re-run with the method in
[`roadmap.md`](roadmap.md#port-completeness) — the enum-discriminant check is
the one that matters, since a missing `pub fn` fails the build but a mis-valued
enum variant fails silently.
