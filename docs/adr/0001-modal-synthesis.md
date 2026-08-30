# ADR-001: Modal Synthesis for Impact Sounds

## Status

Accepted (v0.3)

## Context

The v0.1 impact module used a single sinusoidal oscillator for resonance, producing unrealistic "pure tone" impacts. Real materials produce complex spectra with multiple decaying resonant modes.

## Decision

Implement a modal synthesis engine using a bank of N parallel damped complex resonators. Each mode is defined by frequency, amplitude, and decay time. The bank processes excitation signals (noise bursts, impulses) and produces the sum of all mode outputs.

### Key choices

- **Complex resonator** over biquad: 4 muls + 3 adds per mode per sample, numerically stable, easy coefficient computation from physical parameters.
- **SoA (Structure of Arrays)** layout over AoS: parallel arrays for `state_re`, `state_im`, `coeff_re`, `coeff_im`, `amplitude` enable SIMD auto-vectorization.
- **Material-to-mode mapping**: each of 10 materials has a `ModePattern` (Harmonic, Beam, Plate, StiffString, Damped) and per-material mode count and damping factor.
- **Frequency-dependent damping**: higher modes decay faster, controlled by a damping factor per material.

## Consequences

- Impact sounds are significantly more realistic (multi-mode spectra vs single tone).
- CPU cost scales linearly with mode count (4-12 modes typical, ~6M muls/sec at 48kHz for 32 modes — negligible).
- The modal bank is reused by footstep, creak, foliage (BranchSnap), and cloth (Sail) synthesizers.
- SoA layout means serde serializes 5 parallel arrays instead of N structs — slightly larger JSON but identical binary size.
