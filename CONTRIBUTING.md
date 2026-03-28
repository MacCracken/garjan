# Contributing to garjan

Thank you for your interest in contributing to garjan.

## Development Workflow

1. Fork and clone the repository
2. Create a feature branch from `main`
3. Make your changes
4. Run the cleanliness check (see below)
5. Open a pull request

## Prerequisites

- Rust stable (MSRV 1.89)
- Components: `rustfmt`, `clippy`
- Optional: `cargo-audit`, `cargo-deny`

## Cleanliness Check

Every change must pass:

```bash
cargo fmt --check
cargo clippy --all-features --all-targets -- -D warnings
cargo test --all-features
cargo test --no-default-features
RUSTDOCFLAGS="-D warnings" cargo doc --all-features --no-deps
cargo audit
cargo deny check
```

## Code Conventions

- `#[non_exhaustive]` on all public enums
- `#[must_use]` on all pure functions
- `#[inline]` on hot-path sample processing functions
- Serde (`Serialize + Deserialize`) on all public types
- Zero `unwrap`/`panic` in library code
- `no_std` compatible — use `alloc` not `std` collections
- Feature-gate all `naad` usage behind `#[cfg(feature = "naad-backend")]`
- DC blocking filter on all synthesis outputs
- `validate_sample_rate` in all constructors, `validate_duration` in all `synthesize` methods

## Adding a New Synthesizer

1. Create `src/my_synth.rs` following the pattern in existing modules
2. Add shared enums to the appropriate types module (`contact.rs`, `aero.rs`, `creature.rs`)
3. Register in `lib.rs`: module declaration, prelude export, Send+Sync assertion
4. Add integration tests: all variants, zero-intensity silence, serde roundtrip
5. Add a criterion benchmark
6. Check scope boundaries — does this belong in garjan or a sibling crate?

## Scope Boundaries

Before adding new sound categories, check whether the sound belongs in garjan or a sibling crate. See [docs/architecture/adr-003-scope-boundaries.md](docs/architecture/adr-003-scope-boundaries.md).

## License

By contributing, you agree that your contributions will be licensed under GPL-3.0-only.
