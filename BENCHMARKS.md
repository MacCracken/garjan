# garjan — Benchmarks

Run: `cyrius bench tests/garjan.bcyr`

26 benchmarks, mirroring `rust-old/benches/benchmarks.rs`. Each `_1s` op is a
`process_block` over one second of audio (44,100 samples) at 44.1 kHz, so the
figure is the wall-clock time to synthesize 1 s of sound. Measured on x86_64
Linux; 20 iterations per synth (200 for the 512-sample ops).

| Synthesizer                | Time / 1 s audio | Real-time factor |
|----------------------------|------------------|------------------|
| cloth (Flag, 12 m/s)       | 1.50 ms          | ~668x            |
| precipitation (hail/metal) | 1.85 ms          | ~539x            |
| rain (Moderate)            | 1.96 ms          | ~511x            |
| wingflap (Medium)          | 2.24 ms          | ~446x            |
| bubble (Boiling)           | 2.35 ms          | ~426x            |
| fire                       | 3.43 ms          | ~292x            |
| whoosh (Swing)             | 4.04 ms          | ~248x            |
| thunder (2 km)             | 4.09 ms          | ~244x            |
| impact (glass shatter)     | 5.97 ms          | ~168x            |
| footstep (gravel walk)     | 6.11 ms          | ~164x            |
| water (Waves)              | 6.21 ms          | ~161x            |
| wind (15 m/s)              | 7.71 ms          | ~130x            |
| impact (metal strike)      | 7.86 ms          | ~127x            |
| rolling (wheel on wood)    | 7.92 ms          | ~126x            |
| foliage (rustle)           | 8.23 ms          | ~121x            |
| water (Stream)             | 8.40 ms          | ~119x            |
| whistle (Pipe)             | 10.20 ms         | ~98x             |
| underwater (Medium depth)  | 10.48 ms         | ~95x             |
| friction (scrape metal)    | 10.66 ms         | ~94x             |
| surf (Moderate, 2 s op)    | 22.71 ms         | ~88x             |
| texture (Forest)           | 12.53 ms         | ~80x             |
| impact (wood strike)       | 15.88 ms         | ~63x             |
| creak (Door)               | 16.63 ms         | ~60x             |
| **insect (swarm of 8)**    | **33.98 ms**     | **~29x**         |

Sub-block ops:

| Op                        | Time      |
|---------------------------|-----------|
| `process_block` wind, 512 | 91.4 us  |
| modal bank, 8 modes, 512  | 161.4 us  |

Last measured on Cyrius 6.7.18 with naad 2.3.3 / hisab 3.4.1 / goonj 2.3.1.
Run-to-run spread is roughly 3%, so treat single-run differences under that as
noise.

> **Moving from 6.6.3 to 6.6.20 was flat**: median +0.8% across the 26 ops, both
> built and run back to back on one machine. Most of the gap from the previous
> table is the machine, not the toolchain, so compare only same-machine runs.
> `surf` runs a 2 s buffer, so its factor is 2000 / time; the table used to divide
> 1000 by it, which understated the row by half.
>
> **Moving from 6.6.20 to 6.7.9 was flat as well**: median -1.2% across the 26 ops over
> three interleaved pairs, with ranges overlapping on 23 of them, on a machine at a load
> average near 25.
>
> **Moving from 6.7.9 to 6.7.18 sped up the noise-driven synths**: rain -28%, footstep
> -14%, whoosh, wind, rolling, water and foliage about -11%, median -3.9%, and nothing
> slower than whistle's +1.3%, on the best of 40 alternating rounds a side. naad's pink
> noise and biquad work fits that, but it was not attributed synth by synth.
>
> **Under load, wall-clock medians mislead.** Three interleaved pairs at a load average
> of 40 read thunder +77% and the median op +6%. Alternating the two binaries and taking
> each op's minimum read thunder +0.5%.

> **Corrected in 2.6.2.** The `precipitation` row measured gravel while labelled metal:
> `surface` is a `TERRAIN_*` id, the harness passed `MATERIAL_METAL`, which is 0, and 0 is
> `TERRAIN_GRAVEL`. Gravel has no resonant modes, so the row left out the modal bank. It
> now measures hail on metal, which accounts for most of its rise in the table.
>
> **Corrected in 2.2.0.** The `surf` and `underwater` rows previously measured
> the wrong configuration. `surf_new` and `underwater_new` take **enum ids**
> (`SURF_*`, `UNDERWATER_DEPTH_*`), and the harness was passing an f64 —
> `F64_HALF`, and a depth in metres. An f64's bit pattern is a huge integer, so
> both calls fell through their dispatch chain's final `else`, silently
> benchmarking **Storm** and **Shallow** while labelled Moderate and 25 m.
> The rows above are the labelled configurations. ADR-0006 now rejects such ids
> outright.

## Notes

- **`insect` (swarm of 8) remains the hot spot at ~29x real-time**, though
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
