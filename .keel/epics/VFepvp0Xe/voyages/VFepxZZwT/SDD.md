# Add Backend Registry And Second Path - Software Design Description

> Route planning and execution through explicit conversion capability seams, then deliver a second gguf-to-safetensors backend through that contract.

**SRS:** [SRS.md](SRS.md)

## Overview

This voyage converts the extracted module boundaries into a reusable extension contract:

- define a centralized capability registry that describes supported paths and lossy edges
- drive planning and execution from that shared capability data
- register the current backend and add `gguf -> safetensors` without invasive edits
- capture the guardrails that keep future backend additions bounded and explicit

## Context & Boundaries

The voyage is not a plugin system. It is a bounded in-process registration model that proves new backends can be added incrementally.

```
┌──────────────────────────────────────────────────────────────┐
│                         This Voyage                          │
│                                                              │
│  ConvertRequest -> capability registry -> backend dispatch   │
│                                 -> gguf->hf or gguf->safe    │
└──────────────────────────────────────────────────────────────┘
           ↑                                      ↑
     planning / CLI                         validation surface
```

## Dependencies

| Dependency | Type | Purpose | Version/API |
|------------|------|---------|-------------|
| Voyage-one module seams | internal | Provide `plan` and `transform` homes for the registry and dispatch code | local workspace |
| `candle_core` GGUF and safetensors support | crate | Reuse GGUF reading and safetensors writing logic for both backends | workspace dependency |
| Existing validation surface | internal | Validate the additional `gguf -> safetensors` output contract | local workspace |

## Key Decisions

| Decision | Choice | Rationale |
|----------|--------|-----------|
| Capability source of truth | One centralized registry describes supported paths, lossy status, and backend identity. | Prevents drift between planning, compatibility reporting, and execution dispatch. |
| Backend model | Use an explicit in-process backend seam rather than dynamic plugins. | The mission needs bounded extensibility, not runtime module loading. |
| Proof backend | Add `gguf -> safetensors` as the second delivered path. | The route already appears in the support matrix, and it reuses the current GGUF materialization work without forcing a new remote contract. |
| Guardrails | Keep lossy opt-in, deterministic outputs, and no new network behavior. | Extensibility cannot weaken the repo's safety and proof invariants. |

## Architecture

The voyage adds two core design pieces:

- capability registry in the planning layer, describing what paths exist and what caveats they carry
- backend dispatch in the transform layer, keyed by the same capability data

The registry should answer:

- is this source-to-target pair supported?
- is it lossy?
- which backend, if any, can execute it?

The dispatcher should answer:

- given a planned request, which backend executes it?
- does the request satisfy the capability's preconditions?
- how should the output be validated after execution?

## Components

- Capability registry component: shared table or descriptors for supported paths and their semantics
- Execution dispatch component: routes a request to the registered backend implementation
- Existing GGUF backend: preserved and registered through the new seam
- New GGUF-to-safetensors backend: emits validated safetensors output through the same transform boundary
- Guardrail evidence component: tests and docs showing the bounded touch surface for backend additions

## Interfaces

- Planning:
  - A capability lookup surface inside `plan` determines whether a request is supported and lossy
- Execution:
  - Transform dispatch resolves a planned request to a backend implementation
- Validation:
  - The added `gguf -> safetensors` backend must produce output accepted by the existing validation surface

## Data Flow

1. A request enters planning.
2. The planner queries the capability registry for the requested source and target path.
3. The planner enforces lossy opt-in and returns an explicit plan referencing the supported capability.
4. Execution dispatch resolves that capability to a registered backend.
5. The backend materializes output for either `gguf -> hf-safetensors` or `gguf -> safetensors`.
6. Validation checks the result against the requested target contract.

## Error Handling

| Error Condition | Detection | Response | Recovery |
|-----------------|-----------|----------|----------|
| Planner and executor disagree about supported paths | Tests compare registry-driven plan and dispatch behavior | Treat as a bug and collapse both onto the shared capability source | Keep the registry the single source of truth |
| The new backend bypasses lossy opt-in | Planning or CLI tests show execution can proceed implicitly | Block execution until lossy semantics are enforced through the registry | Re-run with explicit opt-in once fixed |
| `gguf -> safetensors` output is not valid or deterministic enough for proof | Validation or repeatability tests fail | Tighten output naming, validation, or backend behavior | Keep the proof path bounded until repeatable results pass |
