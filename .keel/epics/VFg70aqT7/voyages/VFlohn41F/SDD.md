# Reclassify The Compatibility Matrix For Executable Relayouts - Software Design Description

> Replace generic compatible-path claims with truthful format-specific backend registration and blockers.

**SRS:** [SRS.md](SRS.md)

## Overview

This voyage narrows the registry from a broad placeholder into a truthful capability matrix. The implementation will move backend truth into explicit entries for the local relayout and metadata-backed bundle-promotion paths, then compute request-specific blockers from that shared metadata and the concrete source/target request.

## Context & Boundaries

In scope: the capability registry, compatibility reporting, and any local source-shape inspection needed to decide whether a request is runnable.

Out of scope: the actual file-copying execution logic, CLI rendering, and documentation updates beyond what is needed to define truthful blockers.

```
┌─────────────────────────────────────────┐
│              This Voyage                │
│                                         │
│  ┌──────────────┐  ┌─────────────────┐ │
│  │ Capability   │  │ Request blocker │ │
│  │ registry     │  │ derivation      │ │
│  └──────────────┘  └─────────────────┘ │
└─────────────────────────────────────────┘
         ↑                    ↑
   Source inspection     Convert request
```

## Dependencies

| Dependency | Type | Purpose | Version/API |
|------------|------|---------|-------------|
| `crates/metamorph/src/transform.rs` | Internal module | Registry and execution metadata | current workspace |
| `crates/metamorph/src/plan.rs` | Internal module | Compatibility and plan surfaces | current workspace |
| `crates/metamorph/src/source.rs` | Internal module | Local source-format and source-shape inspection | current workspace |

## Key Decisions

| Decision | Choice | Rationale |
|----------|--------|-----------|
| Generic same-format relayout | Replace with explicit format-pair entries | Not every same-format path has a meaningful reusable-output contract today |
| Local-only gating | Represent as blockers on otherwise executable backend classes | Integrators need to know both that a backend exists and why this request is not runnable |
| Metadata-backed bundle promotion | Treat missing sidecars as request blockers, not silent synthesis | Avoid inventing model metadata or overstating what the library can infer |

## Architecture

`find_capability()` remains the authoritative registry entry point. Capability metadata is extended so `compatibility()` can derive request-specific blockers from the concrete source and target without duplicating backend truth elsewhere.

## Components

- `ConversionCapability`
  Purpose: describe executable backend class, lossy flag, steps, and request-shape limits.
- `compatibility()`
  Purpose: combine source inspection, capability lookup, and request blockers into a single report.
- local source inspection helpers
  Purpose: distinguish valid reusable HF bundles from partial sidecar layouts that are still plain safetensors inputs.

## Interfaces

- `find_capability(from, to) -> Option<ConversionCapability>`
- `compatibility(&ConvertRequest) -> CompatibilityReport`
- private helpers for request blockers and source-shape inspection

## Data Flow

1. Inspect the requested source format.
2. Resolve the explicit capability entry for the requested format pair.
3. Add blockers for lossy opt-in, local-only execution, unsupported targets, or missing sidecars.
4. Return status plus blockers without performing execution.

## Error Handling

| Error Condition | Detection | Response | Recovery |
|-----------------|-----------|----------|----------|
| Unsupported same-format request | No explicit capability entry exists | Report `unsupported` instead of generic planned-only | Choose a supported target or wait for a real backend |
| Remote request against local-only backend | Source or target shape conflicts with capability limits | Report executable backend class plus blockers | Materialize a local source path and use a local output target |
| Missing metadata sidecars | Local source inspection finds no required HF metadata | Report blockers before execution | Add `config.json` and `tokenizer.json`, then retry |
