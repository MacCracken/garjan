# garjan — Benchmarks

Run: `cyrius bench tests/garjan.bcyr`

26 benchmarks, mirroring `rust-old/benches/benchmarks.rs`. Each `_1s` op is a
`process_block` over one second of audio (44,100 samples) at 44.1 kHz, so the
figure is the wall-clock time to synthesize 1 s of sound. Measured on x86_64
Linux; 20 iterations per synth (200 for the 512-sample ops).

| Synthesizer                | Time / 1 s audio | Real-time factor |
|----------------------------|------------------|------------------|
| precipitation (hail/metal) | 1.26 ms          | ~792x            |
| cloth (Flag, 12 m/s)       | 1.33 ms          | ~754x            |
| bubble (Boiling)           | 2.12 ms          | ~471x            |
| wingflap (Medium)          | 2.16 ms          | ~464x            |
| rain (Moderate)            | 2.39 ms          | ~418x            |
| fire                       | 3.13 ms          | ~319x            |
| thunder (2 km)             | 3.58 ms          | ~280x            |
| whoosh (Swing)             | 4.16 ms          | ~240x            |
| impact (glass shatter)     | 5.28 ms          | ~189x            |
| water (Waves)              | 5.33 ms          | ~188x            |
| footstep (gravel walk)     | 6.23 ms          | ~161x            |
| impact (metal strike)      | 6.85 ms          | ~146x            |
| wind (15 m/s)              | 7.67 ms          | ~130x            |
| rolling (wheel on wood)    | 7.97 ms          | ~125x            |
| foliage (rustle)           | 8.22 ms          | ~122x            |
| water (Stream)             | 8.26 ms          | ~121x            |
| friction (scrape metal)    | 9.62 ms          | ~104x            |
| whistle (Pipe)             | 10.32 ms         | ~97x             |
| underwater (Medium depth)  | 10.51 ms         | ~95x             |
| surf (Moderate, 2 s op)    | 22.31 ms         | ~90x             |
| texture (Forest)           | 11.79 ms         | ~85x             |
| impact (wood strike)       | 14.05 ms         | ~71x             |
| creak (Door)               | 15.72 ms         | ~64x             |
| **insect (swarm of 8)**    | **30.56 ms**     | **~33x**         |

Sub-block ops:

| Op                        | Time      |
|---------------------------|-----------|
| `process_block` wind, 512 | 89.2 us  |
| modal bank, 8 modes, 512  | 133.7 us  |

Last measured on Cyrius 6.6.20 with naad 2.2.5 / hisab 3.3.7 / goonj 2.0.6.
Run-to-run spread is roughly 3%, so treat single-run differences under that as
noise.

> **Moving from 6.6.3 to 6.6.20 was flat**: median +0.8% across the 26 ops, both
> built and run back to back on one machine. Most of the gap from the previous
> table is the machine, not the toolchain, so compare only same-machine runs.
> `surf` runs a 2 s buffer, so its factor is 2000 / time; the table used to divide
> 1000 by it, which understated the row by half.
>
> **Moving from 6.6.20 to 6.7.9 was flat as well**: median -1.2% across the 26 ops over
> three interleaved pairs, with ranges overlapping on 23 of them. That machine sat at a
> load average near 25, about double the times above, so this table keeps 6.6.20's
> figures until a quiet run can replace them.

> **Corrected in 2.2.0.** The `surf` and `underwater` rows previously measured
> the wrong configuration. `surf_new` and `underwater_new` take **enum ids**
> (`SURF_*`, `UNDERWATER_DEPTH_*`), and the harness was passing an f64 —
> `F64_HALF`, and a depth in metres. An f64's bit pattern is a huge integer, so
> both calls fell through their dispatch chain's final `else`, silently
> benchmarking **Storm** and **Shallow** while labelled Moderate and 25 m.
> The rows above are the labelled configurations. ADR-0006 now rejects such ids
> outright.

## Notes

- **`insect` (swarm of 8) remains the hot spot at ~33x real-time**, though
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
