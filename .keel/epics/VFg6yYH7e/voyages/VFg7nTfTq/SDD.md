# Wire Remote Fetch Into Cache And Conversion Flows - Software Design Description

> Make `cache source`, `acquire_source`, and `convert` fetch remote sources on demand while preserving explicit acquisition outcomes and thin CLI behavior.

**SRS:** [SRS.md](SRS.md)

## Overview

This voyage connects the remote fetch substrate to the workflows operators already use:

- `acquire_source()` becomes the single acquisition entry point for local reuse, remote fetch, and cached remote reuse
- `cache source` renders the acquisition result directly
- supported remote conversion uses the same acquisition path before backend execution
- the CLI remains an adapter over library request and report types

## Context & Boundaries

```
┌───────────────────────────────────────────────────────────────┐
│                          This Voyage                          │
│                                                               │
│  cache source / convert -> acquire_source -> fetch or reuse   │
│                                      -> resolved local path   │
└───────────────────────────────────────────────────────────────┘
           ↑                                           ↑
     local acquisition truth                    remote fetch substrate
```

The voyage is workflow integration, not a new provider implementation. It assumes the fetch substrate from voyage one exists.

## Dependencies

| Dependency | Type | Purpose | Version/API |
|------------|------|---------|-------------|
| `crates/metamorph/src/cache.rs` | module | Own source acquisition reporting and cache identity | local module |
| `crates/metamorph/src/transform.rs` | module | Keep conversion execution routing behind the library seam | local module |
| `crates/metamorph-cli/src/main.rs` | binary adapter | Render acquisition outcomes without owning transport rules | local module |

## Key Decisions

| Decision | Choice | Rationale |
|----------|--------|-----------|
| Acquisition truth | Route both CLI cache inspection and conversion through `acquire_source()`. | Preserves one source of truth for fetched versus reused outcomes. |
| Remote convert behavior | Allow supported remote conversion to trigger fetch on miss rather than requiring manual cache seeding. | This is the user-visible outcome promised by the mission. |
| CLI role | Keep the CLI focused on formatting reports from the library. | Avoids duplicating acquisition policy in command handlers. |

## Architecture

- Acquisition layer
  - returns deterministic cache identity, fetched or reused outcome, resolved local path, and notes
- CLI cache surface
  - prints acquisition reports for local and remote inputs
- Conversion preflight
  - resolves remote inputs through acquisition before backend-specific execution

## Components

- `acquire_source()` integration
  - purpose: centralize fetch or reuse behavior
  - behavior: unify local reuse, remote cache hit, and remote fetch on miss
- CLI renderer
  - purpose: surface acquisition status and notes to operators
  - behavior: reuse the same report fields for local and remote inputs
- Conversion source resolver
  - purpose: supply an executable local path to current backends
  - behavior: fetch remote sources when needed, then continue with existing backend logic

## Interfaces

- Library:
  - `acquire_source(source, from, ...) -> SourceAcquisitionReport`
  - conversion entry points that rely on acquisition outcomes rather than manual cache assumptions
- CLI:
  - `cache source ...`
  - `convert --input hf://...`

## Data Flow

1. The CLI or embedding creates a `Source`.
2. `acquire_source()` checks cache identity and remote cache state.
3. On miss, the remote fetch substrate materializes the source into managed storage.
4. The acquisition report returns fetched or reused outcome plus the resolved local path.
5. `cache source` prints the report or `convert` passes the resolved path into the execution backend.

## Error Handling

| Error Condition | Detection | Response | Recovery |
|-----------------|-----------|----------|----------|
| Remote source still unavailable after fetch attempt | Acquisition fails with remote error | Return the structured failure to CLI or caller | Operator corrects source or retries once the remote issue is resolved |
| CLI and library drift in outcome rendering | Tests show divergent output semantics | Move shared logic back into the library and keep CLI as renderer | Re-run acquisition and command tests |
| Remote integration breaks local behavior | Existing local cache or convert tests fail | Stop and repair the integration before widening scope | Preserve local-path compatibility as a hard constraint |
