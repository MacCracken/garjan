# ADR-004: Deterministic Synthesis

## Status

Accepted (v0.1)

## Context

Game engines need reproducible audio for replays, debugging, and testing. Non-deterministic synthesis (using system entropy, thread-local state, or wallclock time) makes bugs unreproducible.

## Decision

All randomness in garjan comes from the internal PCG32 PRNG (`src/rng.rs`), seeded with hardcoded constants per synthesizer type. There is no use of `std::time`, `thread_local!`, `rand` crate, or OS entropy anywhere in the crate.

### Seed assignments

Each synthesizer uses a unique prime or memorable seed (e.g., Rain=7919, Thunder=1337, Fire=6661). The `generate_modes` function derives its seed from the material's resonance frequency: `(f0 * 1000.0) as u64`.

## Consequences

- Given identical constructor parameters, `synthesize(duration)` produces bit-identical output every time. Verified by `test_deterministic_replay_all_synths`.
- Different instances of the same type with the same parameters produce identical output (same seed). If the caller needs variation, they should use different constructor params (e.g., slightly different distance for Thunder).
- Serde round-tripping preserves RNG state, allowing mid-synthesis save/restore.
- The naad-backend path's determinism depends on naad's noise generators also being seed-deterministic (they are).
