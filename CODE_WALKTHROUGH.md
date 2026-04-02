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

The library now exposes a facade in `crates/metamorph/src/lib.rs` and keeps the implementation in explicit module files:

- `Format`: the artifact representation such as `gguf`, `safetensors`, or `hf-safetensors`
- `Source`: where artifacts come from, currently a local path or Hugging Face-style reference
- `Target`: where converted output should go
- `InspectReport`: the result of inferring or describing a source
- `ConvertRequest`: the input contract for conversion work
- `CompatibilityReport`: the explicit description of whether a request is executable, planned-only, unsupported, or missing source-format information
- `ConversionPlan`: the explicit description of what a requested conversion would do
- `CacheIdentity` and `SourceAcquisitionReport`: the deterministic local cache contract plus source reuse/materialization report
- `ValidationReport`: the reusable-output validation result
- `PublishPlan` and `PublishRequest`: the preview-first publish contract
- `MetamorphError`: the error surface for unsupported formats, unsupported paths, lossy opt-in failures, and not-yet-implemented backends

The main code now lives in:

- `crates/metamorph/src/source.rs`
- `crates/metamorph/src/format.rs`
- `crates/metamorph/src/plan.rs`
- `crates/metamorph/src/transform.rs`
- `crates/metamorph/src/validate.rs`
- `crates/metamorph/src/cache.rs`
- `crates/metamorph/src/publish.rs`

## State and Lifecycle

Metamorph currently has very little runtime state. The main lifecycle is conceptual:

1. Parse a `Source`
2. Inspect the source and infer a `Format`
3. Assess compatibility and build a `ConversionPlan`
4. Dispatch through the registered conversion backend when one exists
5. Validate the output

Today, inspection, cache identity, local acquisition or reuse reporting, compatibility assessment, two local GGUF execution backends, reusable-output validation, and publish preflight all exist in the library. Remote fetch and remote upload execution remain explicit future seams.

## Command / Request Flow

A representative request currently flows like this:

1. User input enters through `crates/metamorph-cli/src/main.rs` via Clap.
2. The CLI parses flags into library-facing values such as `Format`, `Source`, and `ConvertRequest`.
3. `inspect()` infers a source format from a path or repo name.
4. `compatibility()` consults the shared capability registry and reports whether the requested path is executable, planned-only, unsupported, or blocked by lossy opt-in.
5. `cache_identity()` and `acquire_source()` turn the source into a deterministic local cache contract and an explicit reuse/materialization outcome.
6. `plan()` uses the same registry-driven truth to construct a `ConversionPlan`.
7. `convert()` dispatches through the registered local GGUF backends and validates the resulting bundle.
8. `plan_publish()` and `publish()` expose a preview-first upload path that validates local bundles before any remote write would occur.
9. The CLI prints human-readable output.

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
| Understand the public domain model | `crates/metamorph/src/lib.rs`, then the relevant module file |
| Add a new CLI command | `crates/metamorph-cli/src/main.rs` |
| Change how command output renders | `crates/metamorph-cli/src/main.rs` |
| Modify compatibility assessment or supported paths | `crates/metamorph/src/plan.rs`, `crates/metamorph/src/transform.rs` |
| Modify conversion validation or cache identity | `crates/metamorph/src/validate.rs`, `crates/metamorph/src/cache.rs` |
| Add a new source or target type | `crates/metamorph/src/source.rs`, then update `README.md` |
| Change board or workflow behavior | `keel.toml`, `INSTRUCTIONS.md`, `.keel/` |
