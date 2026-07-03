# garjan — Benchmarks

Run: `cyrius bench tests/garjan.bcyr`

Each op is a `<synth>_process_block` over one second of audio (44,100 samples)
at 44.1 kHz, so the figure is the wall-clock time to synthesize 1 s of sound.
Measured on x86_64 Linux; 20 iterations per synth.

| Synthesizer        | Time / 1 s audio | Real-time factor |
|--------------------|------------------|------------------|
| cloth (Flag)       | 1.03 ms          | ~970x            |
| rain (Moderate)    | 3.84 ms          | ~260x            |
| fire               | 4.94 ms          | ~200x            |
| thunder (2 km)     | 5.65 ms          | ~180x            |
| wind (15 m/s)      | 11.6 ms          | ~85x             |

Notes:

- The Cyrius port widened `f32` → `f64` throughout and calls the naad DSP
  bundle without the Rust crate's inlining/vectorization, so per-sample cost is
  higher than the original Rust numbers — still comfortably real-time.
- The harness lives in [`tests/garjan.bcyr`](tests/garjan.bcyr); extend it with
  more synths as needed. Benchmarks are a release gate per the project process.
