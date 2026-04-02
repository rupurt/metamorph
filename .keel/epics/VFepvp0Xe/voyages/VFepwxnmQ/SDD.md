# Extract Stable Library Modules - Software Design Description

> Split the monolithic library into stable source, format, plan, transform, validate, cache, and publish modules while preserving the current public surface and thin CLI.

**SRS:** [SRS.md](SRS.md)

## Overview

This voyage changes source layout, not product intent:

- `crates/metamorph/src/lib.rs` becomes a facade that re-exports the public workflow surface
- product logic moves into the modules already promised by the repo docs
- the current `gguf -> hf-safetensors` backend is extracted behind a transform-specific boundary
- the CLI continues to parse arguments and render reports, but conversion truth remains inside the library

## Context & Boundaries

The voyage is intentionally architectural. It should make later backend growth cheaper without broadening runtime or network behavior on its own.

```
┌──────────────────────────────────────────────────────────┐
│                        This Voyage                       │
│                                                          │
│  lib.rs facade -> modules -> current backend seam        │
│       source / format / plan / transform / validate      │
│                     / cache / publish                    │
└──────────────────────────────────────────────────────────┘
          ↑                                         ↑
   current library callers                    thin CLI surface
```

## Dependencies

| Dependency | Type | Purpose | Version/API |
|------------|------|---------|-------------|
| Rust module system | language | Extract the monolith into stable module files and re-exports | Rust 2024 workspace |
| `thiserror` | crate | Preserve the current error surface while modules move | workspace dependency |
| `serde` / `serde_json` | crate | Keep existing domain reports serializable across module boundaries | workspace dependency |
| `clap` | crate | Ensure the CLI remains an adapter over the library surface | workspace dependency |

## Key Decisions

| Decision | Choice | Rationale |
|----------|--------|-----------|
| Public entry point | Keep `lib.rs` as a facade with re-exports for the current top-level workflow. | Minimizes churn for existing callers while still letting the implementation become modular. |
| Module vocabulary | Use the product terms already promised in the repo docs: `source`, `format`, `plan`, `transform`, `validate`, `cache`, `publish`. | Prevents architecture drift between planning, docs, and code. |
| Backend isolation | Move the existing `gguf -> hf-safetensors` execution logic behind the `transform` seam now. | Voyage two depends on an execution seam that already carries the current backend. |
| CLI constraint | Keep product logic in the library and let the CLI remain presentation and orchestration only. | The repo policy treats the library as the source of truth. |

## Architecture

After extraction, the shape should look like:

- `lib.rs`: top-level facade and re-exports
- `source`: source parsing, identification helpers, and acquisition-facing source types
- `format`: format descriptors and parsing/display helpers
- `plan`: request types, planning logic, and later compatibility reporting hooks
- `transform`: execution dispatch plus backend-specific implementations
- `validate`: output validation reports and checks
- `cache`: deterministic cache identity and acquisition behavior
- `publish`: preview or execution planning for publish surfaces

The voyage does not require a full API redesign. It mainly relocates responsibility and boundaries so later work can extend through modules instead of growing the monolith.

## Components

- Facade component: re-exports the stable public types and functions the CLI and embeddings already use
- Domain modules: own format, source, planning, validation, cache, and publish logic independently
- Transform module: owns execution dispatch and the current backend-specific code
- CLI adapter component: consumes the facade, formats results, and stays free of conversion rules

## Interfaces

- Library:
  - Preserve the current public workflow around `inspect`, `plan`, `convert`, `validate`, `cache_identity`, `acquire_source`, `plan_publish`, and `publish`
  - Re-export the core domain types from the facade
- CLI:
  - Continue using the library entry points already exposed from `crates/metamorph`
  - Avoid owning supported-path or validation rules locally

## Data Flow

1. The CLI or embedding application calls the facade in `lib.rs`.
2. The facade forwards to the appropriate module: source inspection, planning, cache, validation, publish, or transform.
3. The transform module routes execution to the existing backend seam for `gguf -> hf-safetensors`.
4. Results and errors return through the same public surface the CLI already renders.

## Error Handling

| Error Condition | Detection | Response | Recovery |
|-----------------|-----------|----------|----------|
| Module extraction accidentally changes current behavior | Existing tests or CLI proofs fail after the move | Stop and repair the moved logic before expanding scope | Keep the facade stable and re-run the existing workflow tests |
| Refactor leaks product logic into the CLI | Code review or clippy/tests expose duplicated rules | Move the rule back into the library module | Preserve the library-as-source-of-truth boundary |
| Backend code becomes harder to locate after extraction | Review of transform layout shows the path is still scattered | Consolidate backend-specific code under the transform seam | Use voyage-one docs and walkthrough updates to keep the layout legible |
