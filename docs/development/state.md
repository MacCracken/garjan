# garjan — Current State

> Refreshed every release. CLAUDE.md is preferences/process/procedures
> (durable); this file is **state** (volatile).

## Version

**2.6.2** (2026-11-07). Per-release detail is in the
[CHANGELOG](../../CHANGELOG.md); this file is current state only.

The 2.x line so far: the Rust→Cyrius port (2.0.0), a toolchain and dependency
refresh (2.0.1), a P-1 security audit and its repairs (2.0.2-2.0.4), hot-path
optimization (2.0.5, 2.3.0), parity completion — integration suite, benchmarks,
examples (2.1.0) — boundary validation (2.2.0, 2.4.0), serde live-state parity
(2.5.0, the last outstanding divergence from the Rust oracle), a second full
audit sweep (2.5.1), toolchain moves to cyrius 6.6.3 (2.5.2), 6.6.20 (2.5.3), 6.7.9
(2.6.0) and 6.7.18 (2.6.1), and the roadmap's quick repairs (2.6.2).

## Toolchain

- **Cyrius pin**: `6.7.18` (in `cyrius.cyml [package].cyrius`). It and every dependency
  below were the newest published tags at 2.6.1.

## Source

- Rust reference: 6,186 lines at `rust-old/` (frozen).
- Cyrius port: **32 modules** in `src/*.cyr` — foundations (error, logging,
  dsp, rng, lod, material, modal, contact, aero, creature, voice) + 19
  synthesizers (fire, weather=thunder/rain/wind, water, precipitation, bubble,
  impact, footstep, friction, creak, rolling, foliage, whoosh, whistle, cloth,
  insect, wingflap, underwater, surf, texture) + builder + bridge. Entry
  `src/main.cyr`; distlib bundle `dist/garjan.cyr`.

## Tests

- **33 module suites** in `tests/*.tcyr`, **847 assertions, all green**.
  `tests/garjan.tcyr` is the cross-module integration suite (367 of those),
  ported from `rust-old/tests/integration.rs`.
  Covers parity (incl. bit-exact PCG32), synthesis finiteness/energy, and
  serde roundtrips. `cyrius test` runs them; each also builds standalone.
- Cleanliness: `cyrius audit` exits 0 (fmt, lint, docs and tests all clean; CI
  does not run it), `cyrius build` warning-free, `cyrius vet` 0 missing (it lists
  `dist/garjan.cyr` as untrusted, as it did on 6.6.3), `cyrius distlib --check` in
  sync, 0 lines > 120 chars in `src/` or `tests/`.
- **Fuzz**: `tests/garjan.fcyr` drives **19,603 hostile documents** through all 22
  `*_from_json_str` entry points in about 12 s on a quiet machine (under a minute at a
  load average of 40), each in a forked child with a time and memory limit. CI runs it, and releases gate on CI. It covers JSON only: constructors
  and setters are still swept by hand, as
  [`dependency-watch.md`](dependency-watch.md) describes.
- ⚠ **Verify formatting by running the formatter and diffing, not with
  `cyrius fmt --check`** — `--check` has reported a file clean that the
  formatter then rewrote (that false negative is what broke the `cyrius audit`
  fmt gate at 2.0.2). Diff against the toolchain's `cyrfmt <file>`: at 6.6.20
  `cyrius fmt <file>` prints nothing to stdout. Formatting follows the 6.5.36
  canon: continuation lines indent 2 spaces per open paren, 4 also accepted.
- ⚠ **`cyrius build` does not regenerate `dist/`** — it compiles the stale
  bundle and still reports `OK`. Run `cyrius distlib` after any `src/` edit.
- Benchmarks: **26 ops**, `cyrius bench tests/garjan.bcyr` — see
  [`../../BENCHMARKS.md`](../../BENCHMARKS.md). `insect` (swarm of 8) is the
  slowest at ~29x real-time.
- Examples: 5 runnable programs in [`../examples/`](../examples/).
- Every `process_block` allocates nothing, pinned by test, and a `*_synthesize`
  retains only its buffer and a small header (352,952 B per second of 44.1 kHz audio).

## Dependencies

Declared in `cyrius.cyml`:

- **stdlib** — syscalls, string, alloc, str, fmt, vec, io, args, assert, math,
  ganita, bayan, bench. bayan 1.11.1 and goonj pull in `slice`, `hashseed`, `simd`,
  `chrono` and `lib/unicode/` through their bundles.
- **naad** 2.3.3 (noise/filter/LFO via the dist bundle)
- **hisab** 3.4.1, **goonj** 2.3.1 (naad's transitive deps, resolved here)
- **sakshi** 2.5.10 (logging — `src/logging.cyr`)

These tags are what the dependencies themselves pin, so the flattened graph is
consistent: naad 2.3.3 → hisab 3.4.1 + goonj 2.3.1; goonj 2.3.1 → hisab 3.4.1;
hisab 3.4.1 → sakshi 2.5.10, which is also the sakshi folded into cyrius 6.7.18.
`lib/` is 45 of 48 files byte-identical to the 6.7.18 snapshot, plus the naad,
hisab and goonj bundles byte-identical to their tags. `lib/boxed.cyr`,
`lib/callback.cyr` and `lib/tagged.cyr` are vendored but no longer referenced.
Symbol-collision audit: garjan's 426 top-level `fn`/`var` symbols intersect every
vendored file at zero — re-run it on any dep bump, since Cyrius gives no
diagnostic for a duplicate top-level `var`.

## Consumers

- **dhvani** pins garjan 2.5.2 on cyrius 6.6.10, and calls it from
  `src/environment.cyr`. Its `tests/environment.tcyr` passes all 10 on the 2.6.2 bundle,
  and all 21 synths round-trip under its toolchain and tags, so it can take 2.6.2 without
  moving anything else. Adopting garjan's own dep set would need cyrius 6.7.2. dhvani's
  pins are why `garjan_json_get` and `whistle_svf_band_pass` exist and why `src/` stays
  off 6.7.x syntax: see [`dependency-watch.md`](dependency-watch.md).

Expected: kiran, joshua, and any AGNOS component needing environmental audio.

## Next

**2.6.3, the P(-1) audit, hardening, refactor and optimization sweep**, is scheduled
next. Its method and open items are in [`roadmap.md`](roadmap.md), as is everything after it.

## Port completeness

Verified and re-verified 2026-08-30: **30 of 34** modules ported; **all 106 enum
variants, 103 preserving Rust's exact discriminant** (the 3 exceptions are
`GarjanError`, deliberately remapped); ~223 of 242 public items compared. Also
dropped deliberately: ~232 lines of non-naad fallback across 19 modules.

Re-run with [the method](../guides/port-completeness.md) — the enum-discriminant check is
the one that matters, since a missing `pub fn` fails the build but a mis-valued
enum variant fails silently.
