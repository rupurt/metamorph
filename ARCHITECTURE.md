# Metamorph Architecture

This document describes the implemented technical shape of Metamorph, with emphasis on the integration boundary and the CLI contract.

## System Map

Metamorph is a Rust workspace plus a Nix development shell.

- `crates/metamorph/` is the source of truth for inspection, planning, conversion, validation, cache identity, acquisition, and publish planning
- `crates/metamorph-cli/` is a thin renderer and argument-parsing layer over the library
- `flake.nix`, `flake.lock`, and `nix/` define the build and toolchain environment
- `.keel/` plus the root governance docs define planning and operational workflow

## Layer Model

| Layer | Owns | Current truth |
| --- | --- | --- |
| `source.rs` | `Source`, `Target`, source parsing, local inspection, remote-source description | Local inspection is filesystem-based; Hugging Face inspection is heuristic and naming-based |
| `format.rs` | `Format` parsing and display | Formats currently modeled are `gguf`, `safetensors`, `hf-safetensors`, and `mlx` |
| `plan.rs` | `ConvertRequest`, `CompatibilityReport`, `ConversionPlan` | Compatibility and planning are registry-driven and explicit about blockers |
| `transform.rs` | capability registry and backend dispatch | Only local GGUF execution backends are wired today |
| `validate.rs` | reusable-output verification | Validation is implemented for `safetensors` and `hf-safetensors` contracts |
| `cache.rs` | deterministic source cache identity and acquisition reporting | Local reuse and materialization work; remote fetch is still a future seam |
| `publish.rs` | publish preflight and execution gating | Publish preview works; remote write execution is intentionally not implemented |
| `crates/metamorph-cli/src/main.rs` | CLI argument parsing and human-readable rendering | The CLI calls the library directly and should not drift from library behavior |

## Stable Integration Contract

The main public integration surface is the library facade in `crates/metamorph/src/lib.rs`.

It re-exports the workflow types and functions that integrators should build around:

- `Source`, `Target`, `Format`
- `InspectReport`, `inspect()`
- `ConvertRequest`, `CompatibilityReport`, `compatibility()`
- `ConversionPlan`, `plan()`
- `convert()`
- `ValidationReport`, `validate()`
- `CacheIdentity`, `cache_identity()`
- `SourceAcquisitionReport`, `acquire_source()`
- `PublishPlan`, `plan_publish()`
- `PublishReport`, `publish()`

This is the intended contract:

- integrations reason about format and compatibility through the library
- the CLI presents those same reports in human-readable form
- business rules stay in the library, not in command rendering

## Current Execution Semantics

The registry in `transform.rs` is the single source of truth for supported paths.

Executable backends:

- `gguf -> hf-safetensors`
- `gguf -> safetensors`

Planned-only paths:

- same-format relayout
- `safetensors -> hf-safetensors`

Important consequence:

- a path can be known to the registry without being executable
- compatibility should be interpreted together with blockers, not as a binary yes/no

## Transport vs Transformation

Metamorph separates source transport from model transformation.

Planning can succeed for a remote source even when execution cannot yet fetch it. That is intentional.

Current behavior by stage:

- `inspect()` can describe local sources and infer a likely format for `hf://...` references
- `compatibility()` and `plan()` can reason about those sources without downloading anything
- `convert()` resolves the source through acquisition and only executes when a local file or cached artifact is available
- `publish()` validates and plans a remote publish before any write would happen

This boundary keeps the conversion core reusable while remote transport remains an explicit future seam.

## CLI Boundary

The CLI in `crates/metamorph-cli/` should stay thin.

Its responsibilities are:

- parse flags with Clap
- construct library request types
- print compatibility, plan, validation, cache, and publish reports
- return errors from the library without re-implementing domain rules

Its responsibilities are not:

- deciding which paths are supported
- bypassing lossy opt-in
- inventing publish or fetch behavior the library does not implement

## Operational Seams

These are the main seams where future work can expand behavior without distorting the current design:

- remote fetchers for `hf://...` sources
- additional conversion backends in the registry
- broader validation contracts
- real remote publish execution

When adding one of these, keep the shape intact:

1. model the request and report in the library
2. wire compatibility and planning from the same registry-driven truth
3. add validation for reusable outputs
4. let the CLI report the new behavior rather than own it

## Verification Surfaces

The primary confidence surfaces remain:

- `cargo test --workspace`
- `cargo clippy --workspace --all-targets --all-features -- -D warnings`
- `nix build .#metamorph`
- command-level inspection of CLI behavior

If a change affects user-visible CLI or integration behavior, the foundational docs should move in the same patch.
