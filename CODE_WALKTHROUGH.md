# Metamorph Code Walkthrough

This document maps the user-facing workflows in Metamorph to the code that implements them.

For product-facing usage, read [README.md](README.md) and [USER_GUIDE.md](USER_GUIDE.md). For boundary rules, read [ARCHITECTURE.md](ARCHITECTURE.md).

## Repository Layout

- `crates/metamorph/` is the library crate and the real domain center
- `crates/metamorph-cli/` is the binary crate for the `metamorph` command
- `flake.nix`, `flake.lock`, `nix/metamorph.nix`, and `rust-toolchain.toml` define the build environment
- `.keel/` plus the root docs define planning and workflow

## Public Workflow Model

The facade in `crates/metamorph/src/lib.rs` re-exports the workflow types and functions that matter to both the CLI and library users.

Core types:

- `Format`
- `Source`
- `Target`
- `InspectReport`
- `ConvertRequest`
- `CompatibilityReport`
- `ConversionPlan`
- `ValidationReport`
- `CacheIdentity`
- `SourceAcquisitionReport`
- `PublishPlan`
- `PublishReport`

Core functions:

- `inspect()`
- `compatibility()`
- `plan()`
- `convert()`
- `validate()`
- `cache_identity()`
- `acquire_source()`
- `plan_publish()`
- `publish()`

## Command To Code Map

| CLI command | Primary library entry points | Main files |
| --- | --- | --- |
| `metamorph inspect` | `Source::from_str`, `inspect()` | `crates/metamorph-cli/src/main.rs`, `crates/metamorph/src/source.rs` |
| `metamorph convert --plan-only` | `compatibility()`, `plan()` | `crates/metamorph-cli/src/main.rs`, `crates/metamorph/src/plan.rs`, `crates/metamorph/src/transform.rs` |
| `metamorph convert` | `compatibility()`, `plan()`, `convert()` | `crates/metamorph-cli/src/main.rs`, `crates/metamorph/src/transform.rs`, `crates/metamorph/src/validate.rs` |
| `metamorph validate` | `validate()` | `crates/metamorph-cli/src/main.rs`, `crates/metamorph/src/validate.rs` |
| `metamorph cache dir` | `cache_dir()` | `crates/metamorph-cli/src/main.rs`, `crates/metamorph/src/cache.rs` |
| `metamorph cache source` | `acquire_source()` | `crates/metamorph-cli/src/main.rs`, `crates/metamorph/src/cache.rs`, `crates/metamorph/src/source.rs` |
| `metamorph upload` | `plan_publish()`, `publish()` | `crates/metamorph-cli/src/main.rs`, `crates/metamorph/src/publish.rs`, `crates/metamorph/src/validate.rs` |

## Module Responsibilities

### `source.rs`

Start here when changing how Metamorph understands inputs.

This module owns:

- parsing `Source` from local paths and `hf://...` references
- formatting `Source` and `Target` for display
- local filesystem inspection
- heuristic remote format inference

If a CLI or integration example says “Metamorph can inspect this source,” the behavior lives here.

### `format.rs`

This is the small format vocabulary layer.

It defines:

- the `Format` enum
- string parsing such as `hf-safetensors`
- display formatting used across reports and CLI output

### `plan.rs`

This is the main reasoning layer for integrations.

It defines:

- `ConvertRequest`
- `CompatibilityStatus`
- `CompatibilityReport`
- `ConversionPlan`

Key behaviors:

- inspect the source first
- combine inferred and explicit source format
- look up a capability in the shared registry
- surface blockers such as missing lossy opt-in or planned-only execution

If you want to change how Metamorph answers “can this request run?”, start here and in `transform.rs`.

### `transform.rs`

This module owns the capability registry and actual execution backends.

Today it contains:

- the `ExecutionSupport` enum
- `ConversionCapability`
- `find_capability()`
- `convert()`
- the concrete `gguf -> hf-safetensors` backend
- the concrete `gguf -> safetensors` backend

This file is the highest-value starting point when you are:

- adding a new executable conversion path
- changing the step list shown in plan output
- changing backend labels shown to operators and integrations

### `validate.rs`

This module answers “is this output reusable?”

It owns:

- `ValidationReport`
- `validate()`
- format-specific validation helpers

Current contracts:

- `safetensors`
- `hf-safetensors`

### `cache.rs`

This module owns deterministic source cache behavior.

It defines:

- `CacheIdentity`
- `SourceAcquisitionOutcome`
- `SourceAcquisitionReport`
- `cache_dir()`
- `cache_identity()`
- `acquire_source()`

Important current distinction:

- local materialization is implemented
- remote acquisition is reported as cache hit or cache miss
- remote fetching itself is not implemented yet

### `publish.rs`

This module owns publish planning and execution gating.

It defines:

- `PublishPlan`
- `PublishRequest`
- `PublishReport`
- `plan_publish()`
- `publish()`

Current behavior:

- validates a local `hf-safetensors` bundle
- lists the artifacts that would be published
- enforces destination validation
- requires `HF_TOKEN` for the execute path
- still returns a not-yet-implemented error for actual remote write execution

## End-To-End Request Flows

### Inspect flow

1. CLI parses a string into `Source`.
2. `inspect()` checks the local filesystem or applies remote naming heuristics.
3. The CLI renders detected format plus notes.

### Plan-only conversion flow

1. CLI builds `ConvertRequest`.
2. `compatibility()` inspects the source and queries the shared capability registry.
3. `plan()` turns that same decision into ordered conversion steps.
4. The CLI prints compatibility status, backend, blockers, and steps.

### Conversion execution flow

1. `convert()` calls `plan()` first.
2. The selected capability determines whether execution is allowed.
3. The backend resolves the source through acquisition.
4. The backend writes local output artifacts.
5. `validate()` is called on the produced output.

### Cache source flow

1. CLI parses the input into `Source`.
2. `acquire_source()` computes deterministic cache identity.
3. Local inputs are reused or materialized.
4. Remote inputs report cache hit or cache miss.
5. The CLI prints the cache key, path, status, and resolved path.

### Upload flow

1. CLI calls `plan_publish()`.
2. The input bundle is validated as `hf-safetensors`.
3. The publish artifact list is collected and rendered.
4. `publish()` either returns a dry-run report or enforces explicit execute gating.

## Where To Start For Common Changes

| I want to... | Start here |
| --- | --- |
| Change a CLI message or output field | `crates/metamorph-cli/src/main.rs` |
| Change source parsing or source detection | `crates/metamorph/src/source.rs` |
| Change supported paths or compatibility logic | `crates/metamorph/src/plan.rs`, `crates/metamorph/src/transform.rs` |
| Add an execution backend | `crates/metamorph/src/transform.rs`, then `crates/metamorph/src/validate.rs` |
| Change cache behavior | `crates/metamorph/src/cache.rs` |
| Change publish preview or execute gating | `crates/metamorph/src/publish.rs` |
| Keep the user-facing docs truthful | `README.md`, `USER_GUIDE.md`, then this file and `ARCHITECTURE.md` as needed |

## Testing Orientation

The main code-level regression coverage lives in:

- `crates/metamorph/src/tests.rs`
- `crates/metamorph-cli/tests/`

If you change:

- conversion behavior
  - update or add library and CLI tests
- CLI semantics or supported flows
  - update the foundational docs in the same patch
