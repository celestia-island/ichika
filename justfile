# ichika — thread-pool pipeline helper built on flume.

set shell := ["bash", "-c"]

import "./celestia-devtools.just"

default:
    @just --list

# Format all sources.
fmt:
    cargo fmt --all

# Check formatting without writing.
fmt-check:
    cargo fmt --all -- --check

# Type-check all targets and features.
check:
    cargo check --workspace --all-targets --all-features

# Clippy with -D warnings.
clippy:
    cargo clippy --workspace --all-targets --all-features -- -D warnings

# Run the test suite.
test:
    cargo test --workspace --all-features

# Build all features.
build:
    cargo build --workspace --all-features

# One-shot local gate: fmt-check + clippy + test.
ci:
    just fmt-check
    just clippy
    just test
