# Metamorph Architecture

This document describes the implemented technical shape of Metamorph, with emphasis on the integration boundary and the CLI contract.

## System Map

Metamorph is a Rust workspace plus a Nix development shell.

- `crates/metamorph/` is the source of truth for inspection, planning, conversion, validation, cache identity, acquisition, and publish execution
- `crates/metamorph-cli/` is a thin renderer and argument-parsing layer over the library
- `flake.nix`, `flake.lock`, and `nix/` define the build and toolchain environment
- `.keel/` plus the root governance docs define planning and operational workflow

## Layer Model

| Layer | Owns | Current truth |
| --- | --- | --- |
| `source.rs` | `Source`, `Target`, source parsing, local inspection, remote-source description | Local inspection is filesystem-based; Hugging Face inspection is heuristic and naming-based |
| `format.rs` | `Format` parsing and display | Formats currently modeled are `gguf`, `safetensors`, `hf-safetensors`, and `mlx` |
| `plan.rs` | `ConvertRequest`, `CompatibilityReport`, `ConversionPlan` | Compatibility and planning are registry-driven and explicit about blockers |
| `transform.rs` | capability registry and backend dispatch | GGUF execution plus local safetensors relayout and metadata-backed bundle materialization are wired through shared registry metadata |
| `validate.rs` | reusable-output verification | Validation is implemented for `safetensors` and `hf-safetensors` contracts |
| `cache.rs` | deterministic source cache identity, acquisition reporting, refresh control, and cache manifests | Local reuse/materialization and representative remote GGUF fetch/reuse/refresh are implemented |
| `remote.rs` | Hugging Face acquisition seam and mock-backed fetch proof surface | The default provider uses `hf-hub`; the mock provider drives deterministic fetch tests |
| `remote_publish.rs` | Hugging Face publish seam and mock-backed upload proof surface | The default provider negotiates preupload/LFS/commit execution; the mock provider drives deterministic upload proof |
| `publish.rs` | publish preflight, report modeling, and execution dispatch | Publish preview and existing-repo execute flows are library-owned and explicit about guarded refusal, partial, and failed outcomes |
| `crates/metamorph-cli/src/main.rs` | CLI argument parsing and human-readable rendering | The CLI calls the library directly and should not drift from library behavior |

## Stable Integration Contract

The main public integration surface is the library facade in `crates/metamorph/src/lib.rs`.

It re-exports the workflow types and functions that integrators should build around:

- `Source`, `Target`, `Format`
- `InspectReport`, `inspect()`
- `ConvertRequest`, `CompatibilityReport`, `compatibility()`
- `ConversionPlan`, `plan()`
- `ConversionReport`, `convert()`
- `ValidationReport`, `validate()`
- `CacheIdentity`, `cache_identity()`
- `SourceAcquisitionOptions`, `SourceAcquisitionReport`, `acquire_source()`, `acquire_source_with_options()`
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
- `safetensors -> safetensors`
- `hf-safetensors -> hf-safetensors`
- `safetensors -> hf-safetensors`

Current reclassification:

- generic same-format relayout is no longer advertised as one blanket placeholder
- unsupported same-format requests such as `gguf -> gguf` are reported as `unsupported` until they have a real reusable-output contract

Important consequence:

- compatibility should be interpreted together with blockers, not as a binary yes/no
- a backend can exist while a request is still blocked by local-only execution limits or missing metadata sidecars

## Transport vs Transformation

Metamorph separates source transport from model transformation.

Planning can succeed for a remote source even when execution cannot yet fetch it. That is intentional.
Planning can also stay purely descriptive even though acquisition now supports a bounded remote fetch slice.

Current behavior by stage:

- `inspect()` can describe local sources and infer a likely format for `hf://...` references
- `compatibility()` and `plan()` can reason about those sources without downloading anything
- `convert()` resolves the source through acquisition and can fetch a representative remote GGUF source on demand into managed cache
- remote refresh remains explicit through `SourceAcquisitionOptions` and `ConvertRequest { refresh_remote: true, .. }`
- `publish()` validates the bundle first, then either returns preview state or executes an explicit existing-repo upload flow

This boundary keeps the conversion core reusable while making network side effects explicit and auditable.

## CLI Boundary

The CLI in `crates/metamorph-cli/` should stay thin.

Its responsibilities are:

- parse flags with Clap
- construct library request types
- print compatibility, plan, validation, cache, and publish reports
- print acquisition outcomes such as fetched, reused, or refreshed without re-implementing fetch policy
- return errors from the library without re-implementing domain rules

Its responsibilities are not:

- deciding which paths are supported
- bypassing lossy opt-in
- inventing publish or fetch behavior the library does not implement

## Operational Seams

These are the main seams where future work can expand behavior without distorting the current design:

- broader remote fetch coverage for more `hf://...` layouts
- additional conversion backends in the registry
- broader validation contracts
- broader publish targets, branch control, and repo bootstrap

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
