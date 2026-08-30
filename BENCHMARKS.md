# garjan — Benchmarks

Run: `cyrius bench tests/garjan.bcyr`

Each op is a `<synth>_process_block` over one second of audio (44,100 samples)
at 44.1 kHz, so the figure is the wall-clock time to synthesize 1 s of sound.
Measured on x86_64 Linux; 20 iterations per synth.

| Synthesizer        | Time / 1 s audio | Real-time factor |
|--------------------|------------------|------------------|
| cloth (Flag)       | 1.03 ms          | ~970x            |
| rain (Moderate)    | 3.77 ms          | ~265x            |
| fire               | 4.43 ms          | ~225x            |
| thunder (2 km)     | 5.42 ms          | ~185x            |
| wind (15 m/s)      | 11.0 ms          | ~91x             |

Last measured on Cyrius 6.5.36 with naad 2.2.2 / hisab 2.11.2 / goonj 2.0.4
(garjan 2.0.1); figures are the mean of two runs, which agreed to within 2%.
Every synth came out marginally faster than the 6.3.44 / naad 2.1.0 numbers —
codegen and upstream DSP improvements, not a change to garjan's own hot paths.

Notes:

- The Cyrius port widened `f32` → `f64` throughout and calls the naad DSP
  bundle without the Rust crate's inlining/vectorization, so per-sample cost is
  higher than the original Rust numbers — still comfortably real-time.
- The harness lives in [`tests/garjan.bcyr`](tests/garjan.bcyr); extend it with
  more synths as needed. Benchmarks are a release gate per the project process.
