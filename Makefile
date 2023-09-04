# AWT Makefile

# Misc tasks
.PHONY: fmt lint test check-all check

default: check

fmt:
	cargo +nightly fmt --check

lint:
	cargo clippy --workspace  --all-features --all-targets 

test:
	cargo test --workspace

check-all: test lint fmt

check: check-all

# Run examples
.PHONY: example-full example-simple

example-full:
	cargo run -- examples/config.toml

example-simple:
	cargo run -- examples/simple.toml
