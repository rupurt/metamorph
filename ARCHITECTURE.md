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
- The library crate also owns deterministic cache identity, source acquisition reporting, reusable-output validation, and publish planning.
- The CLI crate owns argument parsing, human-readable output, and orchestration.
- Nix owns packaging and toolchain setup, not product behavior.

Moving boundaries:

- The internal module layout of the library is still early and will likely grow toward `source`, `format`, `plan`, `transform`, `validate`, `cache`, and `publish`.
- Concrete remote fetchers and remote upload implementations are still being defined.

## Key Components

- Domain surface: `Format`, `Source`, `Target`, `InspectReport`, `ConvertRequest`, `ConversionPlan`, `CacheIdentity`, `SourceAcquisitionReport`, `ValidationReport`, `PublishPlan`, and `MetamorphError` in `crates/metamorph/src/lib.rs`.
- Inspection and planning: `inspect()` infers a format from local or Hugging Face style inputs; `plan()` turns a request into an explicit conversion plan and enforces lossy opt-in.
- Cache and acquisition: `cache_identity()` and `acquire_source()` expose deterministic local cache paths plus explicit reuse, materialization, cache-hit, and cache-miss outcomes.
- Execution surface: `convert()` executes the first local `gguf -> hf-safetensors` backend and resolves GGUF inputs through the acquisition layer.
- Validation and publish surface: `validate()` marks reusable outputs explicitly; `plan_publish()` and `publish()` expose a preview-first upload path with explicit execution gating and credential checks.
- CLI surface: `crates/metamorph-cli/src/main.rs` exposes `inspect`, `convert`, `validate`, `upload`, and `cache`, with `cache source` and `upload` now rendering acquisition and publish preflight details directly.

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
