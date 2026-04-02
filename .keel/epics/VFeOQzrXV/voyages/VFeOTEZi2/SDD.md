# Stabilize Cache And Validation Reuse Loop - Software Design Description

> Define deterministic source acquisition, cache layout, reuse semantics, and validation gates so converted artifacts can be trusted locally.

**SRS:** [SRS.md](SRS.md)

## Overview

This voyage extends Metamorph's current inspect, plan, convert, and validate pipeline into a reusable local artifact loop:

- the library owns cache identity, source acquisition or reuse, and validation gates
- the CLI remains thin and reports what source was reused, where artifacts landed, and why validation passed or failed
- converted bundles are not treated as reusable outputs until validation succeeds

## Context & Boundaries

The voyage stays focused on the first deterministic reuse contract and avoids building a fully generalized cache subsystem.

```
┌──────────────────────────────────────────────────────────────┐
│                        This Voyage                          │
│                                                              │
│  Source -> Inspect -> Cache Identity -> Acquire or Reuse    │
│                         -> Convert -> Validate -> Reuse      │
└──────────────────────────────────────────────────────────────┘
           ↑                                        ↑
     local / hf:// input                 HF-style safetensors bundle
```

## Dependencies

| Dependency | Type | Purpose | Version/API |
|------------|------|---------|-------------|
| `std::fs` / `std::path` | stdlib | Filesystem inspection, copy, and managed output layout | Rust stdlib |
| `serde` | crate | Structured metadata or report serialization where needed | workspace dependency |
| `thiserror` | crate | Explicit cache and validation error surface | workspace dependency |
| `clap` | crate | CLI exposure of cache and validation behavior | workspace dependency |

## Key Decisions

| Decision | Choice | Rationale |
|----------|--------|-----------|
| Cache ownership | Keep cache identity and acquisition behavior in `crates/metamorph`, not the CLI. | Reuse semantics are product logic and must stay aligned across embeddings and CLI usage. |
| First cache scope | Support representative local and `hf://` sources before generalizing. | This keeps the first operational slice bounded and testable. |
| Validation gate | Treat validation as a required gate before a converted bundle is considered reusable. | Prevents malformed outputs from polluting cache or downstream publish flows. |
| Recovery surface | Prefer actionable domain errors over raw filesystem or format exceptions. | The voyage is explicitly about trustworthy reuse and recovery. |

## Architecture

The voyage grows the library toward the module seams already described in `README.md` and `ARCHITECTURE.md`:

- a cache identity or acquisition seam turns a `Source` into a deterministic local artifact location
- conversion consumes the acquired source rather than re-inferring storage decisions ad hoc
- validation returns a structured success or failure result that the CLI can render clearly
- the CLI orchestrates these steps and reports outcomes without embedding cache rules

## Components

- Cache identity component: maps source, format, and revision context into a deterministic local storage identity
- Acquisition or reuse component: decides whether a source can be reused in place, copied, or fetched into managed storage
- Validation component: verifies expected output files and metadata for the primary Hugging Face-style safetensors bundle
- CLI presentation component: renders cache hits, misses, local paths, and actionable validation failures

## Interfaces

- Library:
  - `inspect(&Source) -> Result<InspectReport>`
  - `plan(&ConvertRequest) -> Result<ConversionPlan>`
  - `convert(&ConvertRequest) -> Result<()>`
  - additional cache-oriented helpers or reports exposed from `crates/metamorph` rather than the CLI
  - validation surface returning an explicit report or domain error for reusable bundle checks
- CLI:
  - `metamorph inspect <input>`
  - `metamorph convert ... --output ...`
  - `metamorph validate <path> --format hf-safetensors`
  - cache-facing output that makes reuse versus acquisition explicit

## Data Flow

1. The operator provides a local path or `hf://` source.
2. The library inspects the source and derives a deterministic cache identity.
3. Acquisition logic reuses, copies, or fetches the source into managed local storage without hiding what happened.
4. Conversion executes against the acquired local source and materializes the target bundle.
5. Validation checks the resulting layout and metadata before the workflow reports a reusable output.
6. The CLI or embedding application reports the cache decision, resulting local path, and validation result.

## Error Handling

| Error Condition | Detection | Response | Recovery |
|-----------------|-----------|----------|----------|
| Cache identity cannot be derived from the source | Source inspection or metadata normalization fails | Return explicit domain error | Operator supplies missing source context or chooses a supported input form |
| Cache acquisition fails or the source cannot be materialized locally | Filesystem or fetch step fails | Surface cache-specific failure with source context | Retry, correct permissions, or use a reachable input |
| Conversion writes a bundle that is incomplete or malformed | Validation fails after conversion | Refuse to report the bundle as reusable | Fix conversion logic or bundle contents and rerun |
| A reused cache entry is stale, missing, or structurally invalid | Reuse preflight or validation detects mismatch | Reject the reused artifact and explain why | Reacquire or rebuild the source artifact explicitly |
