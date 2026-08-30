# garjan — Benchmarks

Run: `cyrius bench tests/garjan.bcyr`

26 benchmarks, mirroring `rust-old/benches/benchmarks.rs`. Each `_1s` op is a
`process_block` over one second of audio (44,100 samples) at 44.1 kHz, so the
figure is the wall-clock time to synthesize 1 s of sound. Measured on x86_64
Linux; 20 iterations per synth (200 for the 512-sample ops).

| Synthesizer                | Time / 1 s audio | Real-time factor |
|----------------------------|------------------|------------------|
| precipitation (hail/metal) | 1.88 ms          | ~530x            |
| cloth (Flag, 12 m/s)       | 2.02 ms          | ~495x            |
| bubble (Boiling)           | 2.64 ms          | ~378x            |
| wingflap (Medium)          | 3.11 ms          | ~322x            |
| rain (Moderate)            | 3.86 ms          | ~259x            |
| fire                       | 4.24 ms          | ~236x            |
| thunder (2 km)             | 5.27 ms          | ~190x            |
| whoosh (Swing)             | 5.86 ms          | ~171x            |
| impact (glass shatter)     | 6.19 ms          | ~162x            |
| water (Waves)              | 7.74 ms          | ~129x            |
| impact (metal strike)      | 7.76 ms          | ~129x            |
| footstep (gravel walk)     | 8.77 ms          | ~114x            |
| rolling (wheel on wood)    | 10.39 ms         | ~96x             |
| wind (15 m/s)              | 10.44 ms         | ~96x             |
| water (Stream)             | 11.00 ms         | ~91x             |
| foliage (rustle)           | 11.04 ms         | ~91x             |
| friction (scrape metal)    | 12.49 ms         | ~80x             |
| underwater (Medium depth)  | 14.20 ms         | ~70x             |
| impact (wood strike)       | 14.67 ms         | ~68x             |
| whistle (Pipe)             | 15.33 ms         | ~65x             |
| surf (Moderate, 2 s op)    | 15.67 ms         | ~64x             |
| texture (Forest)           | 16.77 ms         | ~60x             |
| creak (Door)               | 18.68 ms         | ~54x             |
| **insect (swarm of 8)**    | **42.57 ms**     | **~23x**         |

Sub-block ops:

| Op                        | Time      |
|---------------------------|-----------|
| `process_block` wind, 512 | 119.6 us  |
| modal bank, 8 modes, 512  | 138.5 us  |

Last measured on Cyrius 6.5.36 with naad 2.2.2 / hisab 2.11.2 / goonj 2.0.4.
Run-to-run spread is roughly 3%, so treat single-run differences under that as
noise.

> **Corrected in 2.2.0.** The `surf` and `underwater` rows previously measured
> the wrong configuration. `surf_new` and `underwater_new` take **enum ids**
> (`SURF_*`, `UNDERWATER_DEPTH_*`), and the harness was passing an f64 —
> `F64_HALF`, and a depth in metres. An f64's bit pattern is a huge integer, so
> both calls fell through their dispatch chain's final `else`, silently
> benchmarking **Storm** and **Shallow** while labelled Moderate and 25 m.
> The rows above are the labelled configurations. ADR-0006 now rejects such ids
> outright.

## Notes

- **`insect` (swarm of 8) remains the hot spot at ~23x real-time**, though
  2.3.0 took it from 49.9 ms to 42.6 ms (−15%; wing-buzz and cricket gained
  ~25%). Its per-sample loop runs once per swarm voice, so cost scales linearly
  with `swarm_count` (capped at 8): measured 7.5 / 12.7 / 23.5 / 42.7 ms at
  swarm 1 / 2 / 4 / 8.
  **Most of what remains is irreducible.** Measured per 352,800 voice-calls —
  one second of audio at swarm 8 — against an empty-loop baseline of 0.88 ms:
  biquad 11.9 ms, `f64_sin` 10.6 ms, noise 7.1 ms. That is ~30 of the 42.6 ms
  in per-voice DSP the algorithm genuinely requires. Further gains need an
  algorithmic change (e.g. a recurrence oscillator instead of a `sin` per voice
  per sample), which would **not** be bit-exact.
  This was invisible before 2.1.0: the benchmark set covered only fire, thunder,
  rain, wind and cloth, and named `wind` as the worst target. It is not —
  **`wind` is mid-table.**
- **The old `cloth` figure (1.02 ms) was measuring near-silence.** Cloth takes
  the silent fast-path until `cloth_set_wind_speed` is called, which the
  previous harness never did. With wind actually blowing it is 2.02 ms. Any
  event-driven synth benchmarked without its driving parameter measures an
  early return; the harness now sets velocity / pressure / tension / wind speed
  / intensity before timing.
- The Cyrius port widened `f32` → `f64` throughout and calls the naad DSP
  bundle without the Rust crate's inlining/vectorization, so per-sample cost is
  higher than the original Rust numbers — still comfortably real-time
  everywhere, though `insect` at ~20x has the least headroom.
- Most remaining time is inside naad, in per-sample noise generation and
  biquad/SVF filtering, not in garjan's own arithmetic. 2.0.5 took the
  garjan-side hoisting wins; further gains need naad-level or algorithmic work.
- **Do not hoist constants for speed.** cycc already constant-folds
  `f64_div(f64_from(6), f64_from(10))` — measured at the same cost as an empty
  loop. Hoist *accessor reads* (~0.54 ms per 352,800 calls) and per-voice or
  per-sample invariant sub-expressions instead.
- Benchmark an event-driven synth with its **enum id**, not a plausible-looking
  float. Nothing in the toolchain distinguishes them, and before 2.2.0 a wrong
  id produced believable numbers for the wrong configuration.
- Every optimization must keep [`scripts/audio-hash.cyr`](scripts/audio-hash.cyr)
  bit-identical. The test suite asserts finiteness and energy, **not** exact
  sample values, so it will not catch an optimization that changes the audio.
- Benchmarks are a release gate per the project process.
