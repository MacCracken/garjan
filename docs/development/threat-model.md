# Threat Model

## Trust Boundaries

garjan operates at the **library boundary**. It trusts the calling application to:
- Provide valid numeric inputs (validated at entry points, but not on every setter call)
- Handle `Result` errors appropriately
- Initialize a tracing subscriber if logging output is desired
- Mix and spatialize audio outputs (garjan produces raw mono samples)

garjan does NOT trust:
- Sample rate values (validated in all constructors)
- Duration values (validated in all `synthesize` methods)
- Deserialized data (enum validation via serde derive; parameters clamped on use)

## Attack Surface

### Input validation
All public entry points validate critical parameters:
- `validate_sample_rate`: rejects ≤0, NaN, Infinity
- `validate_duration`: rejects ≤0, NaN, Infinity
- Real-time setters: `.clamp(0.0, 1.0)` on all intensity/velocity/pressure/speed values
- Modal bank: modes outside [20 Hz, Nyquist] are silently excluded
- Poisson rate: clamped to [0, 30] to prevent excessive iteration

### Numerical stability
- Modal resonator radius clamped to [0.0, 0.9999] — prevents blowup
- DC blocker coefficient clamped to [0.9, 0.9999] — prevents oscillation
- All synthesis outputs are finite (verified by test suite)

### Memory safety
- Zero `unsafe` code in the entire crate
- No raw pointer manipulation
- `alloc::format!` used only in error paths, never in `process_block` hot path
- Impact excitation buffer pre-allocated to avoid hot-path allocation

### Denial of service
- Very large `duration` values in `synthesize` will allocate proportionally large buffers. Callers should bound duration to reasonable values (e.g., ≤60 seconds).
- `process_block` with caller-provided buffers has no allocation risk.
- All loops are bounded (no infinite loops possible with clamped inputs).

## Dependency Risk

| Dependency | Type | unsafe | I/O | Risk |
|---|---|---|---|---|
| serde | Serialization | No (derive) | No | Low |
| thiserror | Error derive | No (proc macro) | No | Minimal |
| libm | Math fallback | No | No | Minimal |
| naad (opt) | DSP primitives | No | No | Low |
| tracing (opt) | Logging facade | No | No | Minimal |
