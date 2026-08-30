# garjan — Benchmarks

Run: `cyrius bench tests/garjan.bcyr`

Each op is a `<synth>_process_block` over one second of audio (44,100 samples)
at 44.1 kHz, so the figure is the wall-clock time to synthesize 1 s of sound.
Measured on x86_64 Linux; 20 iterations per synth.

| Synthesizer        | Time / 1 s audio | Real-time factor |
|--------------------|------------------|------------------|
| cloth (Flag)       | 1.02 ms          | ~980x            |
| rain (Moderate)    | 3.69 ms          | ~271x            |
| fire               | 4.36 ms          | ~229x            |
| thunder (2 km)     | 5.25 ms          | ~190x            |
| wind (15 m/s)      | 10.34 ms         | ~97x             |

Last measured on Cyrius 6.5.36 with naad 2.2.2 / hisab 2.11.2 / goonj 2.0.4
(garjan 2.0.5); figures are the **median of 5 runs**. Run-to-run spread is
roughly 3%, so treat single-run differences under that as noise — the medians
above are what moved.

2.0.5 optimized the per-sample hot paths (loop-invariant hoisting; the redundant
second DC-blocking pass folded into generation for wind and texture), for
−5.6% on wind, −3.3% thunder, −2.2% fire, −1.3% rain, ~0 on cloth. cloth did not
move because its flap events are sparse, so the optimized inner loop rarely
runs. Every change is bit-exact — verified with
[`scripts/audio-hash.cyr`](scripts/audio-hash.cyr), not inferred from the suite
passing.

Notes:

- The Cyrius port widened `f32` → `f64` throughout and calls the naad DSP
  bundle without the Rust crate's inlining/vectorization, so per-sample cost is
  higher than the original Rust numbers — still comfortably real-time.
- **Most of the remaining time is inside naad**, in per-sample noise generation
  and biquad/SVF filtering, not in garjan's own arithmetic. The 2.0.5 pass took
  the available hoisting wins; further gains need naad-level or algorithmic
  work. Note the largest win came from deleting an entire pass over the buffer,
  not from hoisting constants.
- The harness lives in [`tests/garjan.bcyr`](tests/garjan.bcyr); extend it with
  more synths as needed. Benchmarks are a release gate per the project process.
