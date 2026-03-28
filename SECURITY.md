# Security Policy

## Scope

garjan is a pure sound synthesis library. It performs no I/O, no network access, and contains no `unsafe` code. All synthesis is deterministic from seeded PRNG state.

## Attack Surface

| Area | Risk | Mitigation |
|------|------|------------|
| Sample rate validation | Division by zero, NaN propagation | `validate_sample_rate` rejects ≤0, NaN, Infinity |
| Duration validation | Allocation panic on infinite duration | `validate_duration` rejects ≤0, NaN, Infinity |
| Poisson distribution | Near-infinite loop on large rate | Rate clamped to 0–30 in `Rng::poisson` |
| Modal bank coefficients | Numerical instability if radius ≥ 1 | Radius clamped to [0.0, 0.9999] |
| DC blocker coefficient | Oscillation at very low sample rates | R clamped to [0.9, 0.9999] |
| Serde deserialization | Crafted JSON with extreme values | Enum validation via serde derive; parameters clamped on use |
| Buffer lengths | Mismatched excitation/output buffers | `debug_assert_eq!` in ModalBank; `min()` fallback in release |
| `alloc::format!` in errors | Allocation in error paths | Only in constructors and `synthesize`, not in `process_block` hot path |

## Reporting Vulnerabilities

Report security issues to the repository maintainer via GitHub Security Advisories. Do not file public issues for security vulnerabilities.

## Dependencies

| Dependency | Purpose | Risk |
|---|---|---|
| `serde` | Serialization | Widely audited, no unsafe in derive |
| `thiserror` | Error derive | Proc macro only, no runtime code |
| `libm` | `no_std` math | Pure Rust, no unsafe |
| `naad` (optional) | DSP primitives | Filters, noise generators; no I/O |
| `tracing` (optional) | Structured logging | No I/O; subscriber is caller's responsibility |
