# Metamorph Architecture

This document is downstream from Keel and describes the actual technical shape of **Metamorph**.

## System Map

Metamorph is a Rust workspace plus Nix development shell for building a model-format conversion library and CLI.

- `crates/metamorph/` contains the reusable library surface.
- `crates/metamorph-cli/` contains the `metamorph` binary.
- `nix/`, `flake.nix`, and `rust-toolchain.toml` define the build and development environment.
- `.keel/` and the root governance docs define planning, workflow, and repo contract.

Stable boundaries:

- The library crate owns format concepts, inspection, planning, validation, and future conversion execution.
- The CLI crate owns argument parsing, human-readable output, and orchestration.
- Nix owns packaging and toolchain setup, not product behavior.

Moving boundaries:

- The internal module layout of the library is still early and will likely grow toward `source`, `format`, `plan`, `transform`, `validate`, `cache`, and `publish`.
- Concrete conversion backends, remote fetchers, and upload implementations are still being defined.

## Key Components

- Domain surface: `Format`, `Source`, `Target`, `InspectReport`, `ConvertRequest`, `ConversionPlan`, and `MetamorphError` in `crates/metamorph/src/lib.rs`.
- Inspection and planning: `inspect()` infers a format from local or Hugging Face style inputs; `plan()` turns a request into an explicit conversion plan and enforces lossy opt-in.
- Execution surface: `convert()` exists as the future execution seam and currently returns `NotImplemented`.
- CLI surface: `crates/metamorph-cli/src/main.rs` exposes `inspect`, `convert`, `validate`, `upload`, and `cache`, with `convert` currently useful as a planning surface and `upload` still stubbed.

## Technical Boundaries

- Put new product behavior in `crates/metamorph`, not in the CLI crate.
- Keep the CLI thin. It should compose library types and report results, not hide business rules.
- Keep transport concerns separate from tensor transformation concerns.
- Preserve the distinction between:
  - source location
  - source format
  - target format
  - output layout
  - conversion plan
- Keep runtime-specific loaders and adapters at the edges so the core remains reusable across applications.

## Operational Seams

- The primary verification surfaces are `cargo test`, `cargo clippy`, `nix build`, and command-level inspection of the CLI.
- Be especially conservative around quantization semantics, metadata preservation, and any behavior that changes memory or accuracy characteristics.
- License-sensitive publishing and upload behavior is an operational seam, not just a coding task.
