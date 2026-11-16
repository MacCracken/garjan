# garjan — Current State

> Refreshed every release. CLAUDE.md is preferences/process/procedures
> (durable); this file is **state** (volatile).

## Version

**2.7.0** (2026-11-16). Per-release detail is in the
[CHANGELOG](../../CHANGELOG.md); this file is current state only.

The 2.x line so far: the Rust→Cyrius port (2.0.0), a toolchain and dependency
refresh (2.0.1), a P-1 security audit and its repairs (2.0.2-2.0.4), hot-path
optimization (2.0.5, 2.3.0), parity completion — integration suite, benchmarks,
examples (2.1.0) — boundary validation (2.2.0, 2.4.0), serde live-state parity
(2.5.0), a second full audit sweep (2.5.1), toolchain moves to cyrius 6.6.3 (2.5.2),
6.6.20 (2.5.3), 6.7.9 (2.6.0) and 6.7.18 (2.6.1), the roadmap's quick repairs (2.6.2), a
third full audit sweep (2.6.3), and port closure (2.7.0): the last Rust module, the items
that waited on dhvani's pins, the Rust tests the port had dropped, and the newest naad,
hisab and goonj.

## Toolchain

- **Cyrius pin**: `6.7.18` (in `cyrius.cyml [package].cyrius`), still the newest release
  at 2.7.0, as is every dependency below.
- **Floor**: garjan's dep set builds and passes from cyrius 6.7.2. 6.7.1 fails inside
  naad's bundle. Re-measured at 2.7.0.
- **6.7.x syntax** is allowed in `src/` since 2.7.0, because the oldest consumer pin is
  6.7.18.

## Source

- Rust reference: 6,186 lines at `rust-old/` (frozen).
- Cyrius port: **33 modules** in `src/*.cyr` — foundations (error, logging,
  dsp, rng, lod, material, modal, contact, aero, creature, voice) + 19
  synthesizers (fire, weather=thunder/rain/wind, water, precipitation, bubble,
  impact, footstep, friction, creak, rolling, foliage, whoosh, whistle, cloth,
  insect, wingflap, underwater, surf, texture) + bridge, builder and soorat (visualization
  data). Entry `src/main.cyr`; distlib bundle `dist/garjan.cyr`, 10,093 lines.

## Tests

- **35 suites** in `tests/*.tcyr`, **1,120 assertions, all green**.
  `tests/garjan.tcyr` is the cross-module integration suite (506 of those),
  ported from `rust-old/tests/integration.rs`. `tests/soorat.tcyr` (90) ports
  `integration/soorat.rs`'s ten tests and pins the port's own choices.
  `tests/naad_failure.tcyr` replaces naad's `noise_new` for its whole binary, so it is a
  suite of its own. Covers parity (incl. bit-exact PCG32), synthesis finiteness, energy
  and audibility of every enum variant, and serde roundtrips. `cyrius test` runs them;
  each also builds standalone.
- Cleanliness: `cyrius audit` exits 0 (fmt, lint, docs and tests all clean; CI
  does not run it), `cyrius build` warning-free, `cyrius vet` 0 missing (it lists
  `dist/garjan.cyr` as untrusted, as it did on 6.6.3), `cyrius distlib --check` in
  sync, 0 lines > 120 chars in `src/` or `tests/`.
- **Fuzz**: `tests/garjan.fcyr` runs **26,927 cases** over **25 entry points** in about
  10 s on a quiet machine, each in a forked child with a time and memory limit. Hostile
  documents go through all 25 `*_from_json_str` entry points (21 synths, the voice pool
  and soorat's three fields), including malformed numbers and duplicate keys. An API stage
  feeds hostile f64 values and ids through every constructor, setter, synthesize duration,
  particle count and grid size. CI runs it, and releases gate on CI.
- **Allocation failure**: at 2.7.0 a sweep refused each allocation of every synth
  constructor in turn, 650 cases, and every one ended in `GARJAN_ERR_ALLOCATION`.
  The sweep harness is not committed; the 2.6.3 audit has the method.
- ⚠ **Verify formatting by running the formatter and diffing, not with
  `cyrius fmt --check`** — `--check` has reported a file clean that the
  formatter then rewrote (that false negative is what broke the `cyrius audit`
  fmt gate at 2.0.2). Diff against the toolchain's `cyrfmt <file>`, which prints the
  canonical text: at 6.7.18 `cyrius fmt <file>` rewrites the file in place and prints
  nothing. Formatting follows the 6.5.36 canon: continuation lines indent 2 spaces per
  open paren, 4 also accepted.
- ⚠ **`cyrius build` does not regenerate `dist/`** — it compiles the stale
  bundle and still reports `OK`. Run `cyrius distlib` after any `src/` edit.
- Benchmarks: **26 ops**, `cyrius bench tests/garjan.bcyr` — see
  [`../../BENCHMARKS.md`](../../BENCHMARKS.md). `insect` (swarm of 8) is the
  slowest at ~30x real-time.
- Examples: 5 runnable programs in [`../examples/`](../examples/).
- Every `process_block` allocates nothing, pinned by test, and a `*_synthesize`
  retains only its buffer and a small header (352,952 B per second of 44.1 kHz audio).

## Dependencies

Declared in `cyrius.cyml`:

- **stdlib** — syscalls, string, alloc, str, fmt, vec, io, args, assert, math,
  ganita, bayan, bench, sakshi. bayan 1.11.1 and goonj pull in `slice`, `hashseed`,
  `simd`, `chrono` and `lib/unicode/` through their bundles. sakshi (logging,
  `src/logging.cyr`) is a stdlib leaf since 2.7.0, the 2.5.10 the toolchain folds.
- **naad** 2.3.5 (noise/filter/LFO via the dist bundle)
- **hisab** 3.5.0, **goonj** 2.3.2 (naad's transitive deps, resolved here)

These tags are what the dependencies themselves pin, so the flattened graph is
consistent: naad 2.3.5 → hisab 3.5.0 + goonj 2.3.2; goonj 2.3.2 → hisab 3.5.0; all three
pin cyrius 6.7.18. `lib/` is 45 files: 42 byte-identical to the 6.7.18 snapshot, plus the
naad, hisab and goonj bundles byte-identical to their tags.
Symbol-collision audit: garjan's 534 top-level names (fn, var, const, struct) intersect
every vendored file at zero. Against dhvani's whole closure there is one collision, prani's
`struct DcBlocker`, which the roadmap fixes. Re-run both on any dep bump, since Cyrius gives
no diagnostic for a duplicate top-level `var`.

## Consumers

- **dhvani** pins garjan 2.6.3 on cyrius 6.7.18, naad 2.3.3, hisab 3.4.1 and goonj 2.3.1,
  and calls it from `src/environment.cyr`. The 2.7.0 bundle passes its environment (10),
  bundle, bundle_synth, integration (41) and integration_advanced (125) suites under those
  pins, and its main binary builds with no undefined function. So it can take 2.7.0 without
  moving anything else.

Expected: kiran, joshua, soorat (for the 2.7.0 field types, once its port finishes), and
any AGNOS component needing environmental audio.

## Next

2.7.1 is ready to start: rename garjan's internal `DcBlocker`, which collides with prani's in
dhvani's closure. After it, 2.8.0 needs a decision on standalone `ModalBank` and `Exciter`
codecs. Everything else waits on a decision or on something outside this repository:
[`roadmap.md`](roadmap.md). The upstream allocation defects still open are U2 to U4, in
[`dependency-watch.md`](dependency-watch.md).

## Port completeness

Re-verified 2026-11-16 by an independent audit of `rust-old/`:

- **Modules:** every one of the 34 `.rs` files has a Cyrius home. 31 map to a `.cyr` of
  the same name, `integration/soorat.rs` included since 2.7.0. `lib.rs` (declarations, a
  prelude) and `integration/mod.rs` are replaced by the flat namespace and the `[lib]`
  list, and `math.rs` by cycc intrinsics plus ganita's `f64_pow` (architecture note 002).
- **Surface:** 161 of 162 `pub` and `pub(crate)` functions outside `math.rs` have a
  counterpart, and the last, `VoicePool::active_voices`, is `voice_pool_next_active` since
  2.7.0. All 34 structs and 27 enums are ported, and all 109 enum variants, **106
  preserving Rust's exact discriminant**. The 3 exceptions are `GarjanError`, remapped
  deliberately (ADR-0005).
- **Tests:** of `integration.rs`'s 134 tests, 26 are not applicable by design (enum serde
  round-trips, since enums are integers). Every other test has a counterpart except
  `test_serde_roundtrip_modal_bank` and `test_serde_roundtrip_exciter`: the port has no
  standalone codec for either, and the roadmap's 2.8.0 decides whether it should.
- **Benches and examples:** 26 of 26 and 5 of 5, though several bench settings differ from
  Rust's (see the roadmap).
- Also dropped deliberately: ~232 lines of non-naad fallback across 19 modules
  (ADR-0002).

Re-run with [the method](../guides/port-completeness.md) — the enum-discriminant check is
the one that matters, since a missing `pub fn` fails the build but a mis-valued
enum variant fails silently.
