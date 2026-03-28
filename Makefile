.PHONY: check fmt clippy test audit deny bench coverage build doc clean

# Run all CI checks locally
check: fmt clippy test audit

# Format check
fmt:
	cargo fmt --all -- --check

# Lint (zero warnings)
clippy:
	cargo clippy --all-features --all-targets -- -D warnings

# Run test suite
test:
	cargo test --all-features
	cargo test --no-default-features

# Security audit
audit:
	cargo audit

# Supply-chain checks (cargo-deny)
deny:
	cargo deny check

# Run benchmarks
bench:
	cargo bench

# Generate coverage report
coverage:
	cargo llvm-cov --all-features --html --output-dir coverage/
	@echo "Coverage report: coverage/html/index.html"

# Build release
build:
	cargo build --release --all-features

# Generate documentation
doc:
	RUSTDOCFLAGS="-D warnings" cargo doc --no-deps --all-features

# Clean build artifacts
clean:
	cargo clean
	rm -rf coverage/
