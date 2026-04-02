# Metamorph Code Walkthrough

This document orients contributors and agents to the source layout, key abstractions, and data flows in the Metamorph codebase. For governance philosophy see [CONSTITUTION.md](CONSTITUTION.md); for architectural contracts see [ARCHITECTURE.md](ARCHITECTURE.md).

## Repository Layout

- `Cargo.toml` defines a small Rust workspace.
- `crates/metamorph/` is the library crate and current domain center.
- `crates/metamorph-cli/` is the binary crate that exposes the `metamorph` CLI.
- `flake.nix`, `flake.lock`, `nix/metamorph.nix`, and `rust-toolchain.toml` define the Nix and Rust toolchain surfaces.
- `justfile` provides common wrapper commands.
- `.keel/` plus the root Markdown docs define planning, governance, and project workflow.

## Key Abstractions

The library currently centers on a small set of explicit domain types in `crates/metamorph/src/lib.rs`:

- `Format`: the artifact representation such as `gguf`, `safetensors`, or `hf-safetensors`
- `Source`: where artifacts come from, currently a local path or Hugging Face-style reference
- `Target`: where converted output should go
- `InspectReport`: the result of inferring or describing a source
- `ConvertRequest`: the input contract for conversion work
- `ConversionPlan`: the explicit description of what a requested conversion would do
- `CacheIdentity` and `SourceAcquisitionReport`: the deterministic local cache contract plus source reuse/materialization report
- `ValidationReport`: the reusable-output validation result
- `PublishPlan` and `PublishRequest`: the preview-first publish contract
- `MetamorphError`: the error surface for unsupported formats, unsupported paths, lossy opt-in failures, and not-yet-implemented backends

The design intent from `README.md` is that the library grows toward separate `source`, `format`, `plan`, `transform`, `validate`, `cache`, and `publish` modules, but that decomposition is not in place yet.

## State and Lifecycle

Metamorph currently has very little runtime state. The main lifecycle is conceptual:

1. Parse a `Source`
2. Inspect the source and infer a `Format`
3. Build a `ConversionPlan`
4. Execute the conversion backend
5. Validate the output

Today, inspection, cache identity, local acquisition or reuse reporting, the first local conversion backend, reusable-output validation, and publish preflight all exist in the library. Remote fetch and remote upload execution remain explicit future seams.

## Command / Request Flow

A representative request currently flows like this:

1. User input enters through `crates/metamorph-cli/src/main.rs` via Clap.
2. The CLI parses flags into library-facing values such as `Format`, `Source`, and `ConvertRequest`.
3. `inspect()` infers a source format from a path or repo name.
4. `cache_identity()` and `acquire_source()` turn that source into a deterministic local cache contract and an explicit reuse/materialization outcome.
5. `plan()` enforces supported conversion paths and lossy opt-in before constructing a `ConversionPlan`.
6. `convert()` executes the first local GGUF backend and validates the resulting bundle.
7. `plan_publish()` and `publish()` expose a preview-first upload path that validates local bundles before any remote write would occur.
8. The CLI prints human-readable output.

## Configuration

Current configuration surfaces are mostly development and workflow oriented:

- `rust-toolchain.toml`: pins the Rust toolchain and components.
- `flake.nix`: defines the Nix dev shell, Rust toolchain provisioning, and package outputs.
- `nix/metamorph.nix`: packages the CLI with `buildRustPackage`.
- `keel.toml`: configures the Keel board location and lane defaults.

Metamorph does not yet have a project-specific runtime config file for conversion behavior.

## Where to Look

Provide a quick-reference table mapping common tasks to starting points in the code.

| I want to... | Start here |
|---------------|-----------|
| Understand the domain model | `crates/metamorph/src/lib.rs` |
| Add a new CLI command | `crates/metamorph-cli/src/main.rs` |
| Change how command output renders | `crates/metamorph-cli/src/main.rs` |
| Modify conversion validation, cache identity, or supported paths | `crates/metamorph/src/lib.rs` |
| Add a new source or target type | `crates/metamorph/src/lib.rs`, then update `README.md` |
| Change board or workflow behavior | `keel.toml`, `INSTRUCTIONS.md`, `.keel/` |
