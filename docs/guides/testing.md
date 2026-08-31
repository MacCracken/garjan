# Testing Guide

garjan is a Cyrius project. There is no cargo — `cyrius test` auto-discovers
`tests/*.tcyr`.

## Running Tests

```sh
cyrius test                      # all 33 suites
cyrius test tests/fire.tcyr      # one suite
cyrius fuzz                      # tests/garjan.fcyr
cyrius bench tests/garjan.bcyr   # 26 benchmarks
cyrius audit                     # fmt / lint / docs / tests / bench
```

**Run `cyrius distlib` first if you have edited `src/`.** The suites include
`src/*.cyr` directly, but `src/main.cyr` and the benchmarks include
`dist/garjan.cyr`, so a stale bundle silently tests old code.

## How the suites are split

- **33 per-module suites** (`tests/<module>.tcyr`) — each includes the `src/`
  modules it needs and tests one module against itself: construction,
  validation, per-variant behaviour, serde round-trips.
- **`tests/garjan.tcyr` — the cross-module integration suite**, ported from
  `rust-old/tests/integration.rs`. It owns what no per-module suite *can*
  assert: the uniform validation contract across all 21 constructors,
  exhaustive enum-variant sweeps, relative invariants *between* synths (heavier
  rain is louder than light), the uniform silence gates, determinism replay,
  builder-vs-direct equivalence, and the hot-path allocation contract.

Total: **797 assertions**.

## What the suite does NOT check

**Exact sample values.** Assertions cover finiteness, energy ordering, silence
gates and JSON round-trips — never the waveform itself. An optimization can pass
every suite while changing the audio.

Bit-exactness is enforced separately, by
[`scripts/audio-hash.cyr`](../../scripts/audio-hash.cyr):

```sh
cyrius build scripts/audio-hash.cyr build/audio-hash && ./build/audio-hash > before.txt
# change something, then: cyrius distlib && rebuild
./build/audio-hash > after.txt && diff before.txt after.txt
```

It folds the raw f64 bit patterns of every synth over three successive blocks,
so any last-ulp or ±0.0 change shows up. Three blocks, not one, because a broken
filter/RNG carry-over only appears from block 2.

## Writing a new test

Follow an existing suite. The shape is:

```cyrius
include "src/error.cyr"
include "src/logging.cyr"
include "src/dsp.cyr"
include "src/rng.cyr"
include "src/my_synth.cyr"

fn main() {
    alloc_init();
    sakshi_set_level(SK_ERROR);      # lower = quieter; SK_TRACE(5) is loudest

    test_group("my_synth_new");
    var s = my_synth_new(MY_TYPE_A, f64_from(44100));
    assert(garjan_is_err(s) == 0, "constructs");
    assert(garjan_is_err(my_synth_new(MY_TYPE_A, 0)) == 1, "rejects sr=0");
    assert(garjan_is_err(my_synth_new(99, f64_from(44100))) == 1, "rejects bad id");

    return assert_summary();
}
var exit_code = main();
syscall(60, exit_code);
```

Three things that have caused real defects here:

- **Pass enum ids, not floats.** `surf_new` and `underwater_new` take
  `SURF_*` / `UNDERWATER_DEPTH_*` ids. An f64's *bit pattern* is a huge integer,
  so before 2.2.0 passing `F64_HALF` silently selected the last variant — the
  benchmark suite and the hash oracle both did this for three releases.
- **Excite event-driven synths before asserting.** whoosh, whistle, cloth,
  friction, creak, rolling, foliage, insect, wingflap and bubble take a silent
  fast-path until their driving parameter is set. An unexcited synth outputs
  silence, so the test passes vacuously.
- **`sakshi_set_level` is a verbosity ceiling** — it emits when
  `event_level <= configured`, so **lower is quieter**. `SK_ERROR` (1)
  suppresses the validator warnings a deliberately-bad-input test provokes.

## Coverage areas

- Per-synth: all variants finite, zero-intensity silence, energy ordering,
  deterministic replay, serde round-trip
- Validation: sample rate (0, negative, NaN, inf, out of 1 Hz–768 kHz),
  duration (0, negative, NaN, inf, over 600 s), sample count over 44.1 M,
  out-of-range enum ids
- Infrastructure: DC blocker, modal bank (impulse response, Nyquist guard,
  reset), voice pool (allocation, stealing, priority, aging), LOD scaling,
  bridge conversions, `garjan_alloc` failure detection
- Streaming: empty buffers, multi-block continuity, zero per-block allocation
