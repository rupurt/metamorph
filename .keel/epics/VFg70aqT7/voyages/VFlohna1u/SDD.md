# Execute Local Relayout And Bundle Materialization - Software Design Description

> Execute local relayout and safetensors-to-bundle flows with reusable-output validation.

**SRS:** [SRS.md](SRS.md)

## Overview

This voyage implements the new local backends as structural artifact transforms. Each backend copies or rewrites local files into the requested target layout, then validates the target before returning success.

## Context & Boundaries

In scope: local file-system execution for relayout and metadata-backed bundle materialization.

Out of scope: new tensor math, new remote acquisition, and publish behavior.

```
┌─────────────────────────────────────────┐
│              This Voyage                │
│                                         │
│  ┌──────────────┐  ┌─────────────────┐ │
│  │ Relayout     │  │ Bundle          │ │
│  │ backends     │  │ materializer    │ │
│  └──────────────┘  └─────────────────┘ │
└─────────────────────────────────────────┘
         ↑                    ↑
   Local source path      Validation gate
```

## Dependencies

| Dependency | Type | Purpose | Version/API |
|------------|------|---------|-------------|
| `std::fs` | Standard library | Local copy and directory creation | Rust stable |
| `validate.rs` | Internal module | Reusable-output gate | current workspace |
| `cache.rs` | Internal module | Local acquisition report for conversion output | current workspace |

## Key Decisions

| Decision | Choice | Rationale |
|----------|--------|-----------|
| Relayout implementation | File copy or directory copy with validation | The promoted paths are structural layout changes, not tensor transforms |
| HF bundle materialization contract | Require one safetensors artifact plus HF metadata sidecars; synthesize only `generation_config.json` when absent | This keeps the first bundle path useful without inventing core model metadata |
| Source preservation | Keep source local artifacts untouched and validate the target separately | Failures should not corrupt or silently mutate the source |

## Architecture

`convert()` dispatches to new backend helpers in `transform.rs`. Each helper acquires the local source, checks source-shape prerequisites, writes the target layout, and calls `validate()` on the target before reporting success.

## Components

- local safetensors relayout helper
  Purpose: copy one or more safetensors artifacts into the requested local target layout.
- local HF bundle relayout helper
  Purpose: copy a reusable HF bundle, including auxiliary files, into a new local bundle target.
- metadata-backed bundle materializer
  Purpose: create a reusable HF bundle from a local safetensors source plus required sidecars.

## Interfaces

- `convert(&ConvertRequest) -> ConversionReport`
- private source-resolution helpers in `transform.rs`
- `validate(path, Some(format)) -> ValidationReport`

## Data Flow

1. Acquire or resolve a local source path.
2. Inspect the source shape required by the selected backend.
3. Create the local output path and copy or rename artifacts into the target layout.
4. Write any allowed synthesized metadata file.
5. Validate the target format and return the output path plus acquisition report.

## Error Handling

| Error Condition | Detection | Response | Recovery |
|-----------------|-----------|----------|----------|
| Missing required sidecars | Source inspection cannot find `config.json` or `tokenizer.json` | Abort before writing success | Supply the missing metadata files and retry |
| Unsupported source shape | Source directory contains zero or multiple safetensors artifacts for bundle materialization | Abort with explicit error | Reduce the source to one representative safetensors artifact or use a different path |
| Invalid copied output | `validate()` fails on the target layout | Fail conversion and surface the validation reason | Fix the source layout or target contract, then rerun |
