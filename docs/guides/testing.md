# Testing Guide

## Running Tests

```bash
# Default features (std + naad-backend)
cargo test

# All features including logging
cargo test --all-features

# No default features (no_std fallback path)
cargo test --no-default-features

# Specific test
cargo test test_impact_all_materials
```

## Test Categories

### Synthesis correctness (per synthesizer)
- All enum variants produce finite output
- Zero-intensity/velocity produces silence
- Output energy ordering (Heavy > Light, Crash > Tap)
- Deterministic replay (same params = identical output)

### Serde roundtrips (per public type)
- Serialize → deserialize → re-serialize produces identical JSON
- Covers all enums and all synthesizer structs

### Parameter validation
- Invalid sample rate (0, negative, NaN, Infinity) → `Err`
- Invalid duration (0, negative, NaN, Infinity) → `Err`

### Infrastructure
- Send + Sync compile-time assertions on all public types
- DC blocker correctness
- Modal bank: impulse response, Nyquist guard, reset
- Voice pool: allocation, stealing, priority, aging
- LOD: mode scaling, rate scaling
- Bridge: conversion accuracy

### process_block streaming
- 512-sample blocks produce finite output
- Empty buffers don't panic

## Writing New Tests

Follow the existing pattern in `tests/integration.rs`:

```rust
#[test]
fn test_my_synth_all_variants() {
    for variant in &[MyType::A, MyType::B, MyType::C] {
        let mut s = MySynth::new(*variant, SR).unwrap();
        let samples = s.synthesize(0.5).unwrap();
        assert!(samples.iter().all(|s| s.is_finite()), "NaN for {:?}", variant);
        assert!(samples.iter().any(|&s| s.abs() > 0.001), "silent for {:?}", variant);
    }
}

#[test]
fn test_my_synth_zero_intensity_is_silent() {
    let mut s = MySynth::new(MyType::A, SR).unwrap();
    s.set_intensity(0.0);
    let samples = s.synthesize(0.1).unwrap();
    assert!(samples.iter().all(|&s| s.abs() < 0.001));
}

#[test]
fn test_serde_roundtrip_my_synth() {
    let s = MySynth::new(MyType::A, SR).unwrap();
    let json = serde_json::to_string(&s).unwrap();
    let s2: MySynth = serde_json::from_str(&json).unwrap();
    let json2 = serde_json::to_string(&s2).unwrap();
    assert_eq!(json, json2);
}
```

## Benchmarks

```bash
# Run all benchmarks
cargo bench

# Run a specific benchmark
cargo bench -- thunder_1s
```

All 25 synthesizer types have criterion benchmarks in `benches/benchmarks.rs`.
