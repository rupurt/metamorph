set shell := ["bash", "-euo", "pipefail", "-c"]

default:
  @just --list --unsorted

setup:
  cargo install --locked --force cargo-nextest cargo-llvm-cov

build:
  @just build-debug

build-debug:
  cargo build --workspace
  mkdir -p target/debug target/release
  cp -f "${CARGO_TARGET_DIR:-target}/debug/metamorph" target/debug/metamorph

build-release:
  cargo build --workspace --release
  mkdir -p target/debug target/release
  cp -f "${CARGO_TARGET_DIR:-target}/release/metamorph" target/release/metamorph

test *args:
  cargo nextest run --workspace {{args}}

doctest:
  cargo test --doc

quality:
  cargo fmt --all -- --check
  cargo clippy --workspace --all-targets --all-features -- -D warnings

coverage args="":
  mkdir -p coverage
  if [[ -n "{{args}}" ]]; then cargo llvm-cov nextest {{args}}; else cargo llvm-cov nextest --lcov --output-path ./coverage/lcov.info; fi

metamorph *args:
  cargo run -p metamorph-cli -- {{args}}

pre-commit: quality test doctest
  @echo "✓ All pre-commit checks passed"
