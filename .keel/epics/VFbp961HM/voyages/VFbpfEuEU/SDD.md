# Inspect Convert And Validate Candle Bundle - Software Design Description

> Prove the first end-to-end inspect, convert, and validate workflow for a Candle-oriented bundle.

**SRS:** [SRS.md](SRS.md)

## Overview

This voyage keeps the product architecture simple:

- the library owns source parsing, format inspection, conversion planning, execution, and validation
- the CLI parses arguments, calls the library, and renders results
- the first execution path targets a Candle-friendly Hugging Face-style bundle layout

## Context & Boundaries

The voyage is limited to the first usable vertical slice. It should not generalize beyond what is required to inspect, convert, and validate one Candle-oriented path.

```
┌───────────────────────────────────────────────────────┐
│                    This Voyage                        │
│                                                       │
│  CLI -> Source/Format Inspection -> Plan -> Execute  │
│                       -> Validate                     │
└───────────────────────────────────────────────────────┘
         ↑                              ↑
   Local / hf:// input            Candle-style bundle
```

## Dependencies

| Dependency | Type | Purpose | Version/API |
|------------|------|---------|-------------|
| `clap` | crate | CLI argument parsing | workspace dependency |
| `serde` | crate | typed serialization support | workspace dependency |
| `thiserror` | crate | explicit error surface | workspace dependency |

## Key Decisions

| Decision | Choice | Rationale |
|----------|--------|-----------|
| Product seam | Keep conversion logic in `crates/metamorph` and keep `crates/metamorph-cli` thin. | Prevents CLI-specific drift and keeps the library reusable. |
| First output contract | Emit a Hugging Face-style bundle shaped for Candle-oriented runtime expectations. | This is the README’s first real operator target. |
| Lossy behavior | Require explicit `allow_lossy` opt-in for `gguf -> hf-safetensors`. | The user must understand when representation changes are not lossless. |
| Validation timing | Validate as part of the core workflow, not as an optional post-step. | A written directory is not enough if the runtime cannot load it. |

## Architecture

The voyage extends the current early library surface rather than introducing a new abstraction layer:

- `Source` and `Format` continue to model inputs explicitly
- `inspect()` remains the source-format inference seam
- `plan()` remains the enforcement seam for supported paths and lossy opt-in
- `convert()` becomes the first concrete execution seam
- validation logic should be exposed from the library and surfaced by the CLI

## Components

- Inspection component: infers formats from local layouts, file extensions, and `hf://` naming hints
- Planning component: enforces supported path rules and constructs explicit conversion steps
- Execution component: materializes the first bundle layout for the supported path
- Validation component: checks required files and reports actionable errors
- CLI presentation component: prints inspect, plan, convert, and validate results for operators

## Interfaces

- Library:
  - `inspect(&Source) -> Result<InspectReport>`
  - `plan(&ConvertRequest) -> Result<ConversionPlan>`
  - `convert(&ConvertRequest) -> Result<()>`
- CLI:
  - `metamorph inspect <input>`
  - `metamorph convert --input ... --output ... --to hf-safetensors --allow-lossy`
  - `metamorph validate <path> --format hf-safetensors`

## Data Flow

1. The operator or embedding application supplies a `Source`.
2. The library inspects and infers the format.
3. The planner validates the requested source-to-target path.
4. The execution path materializes the target bundle.
5. Validation checks the resulting layout and required files.
6. The CLI or embedding application reports success or actionable failure.

## Error Handling

| Error Condition | Detection | Response | Recovery |
|-----------------|-----------|----------|----------|
| Source format cannot be inferred | `inspect()` returns unknown or error | Surface explicit message | Operator supplies `--from` or corrects the input |
| Unsupported conversion path | `plan()` rejects the request | Return `UnsupportedConversionPath` | Choose a supported target or extend the library |
| Lossy path requested without opt-in | `plan()` rejects the request | Return `LossyConversionRequiresOptIn` | Re-run with explicit opt-in after review |
| Output bundle is missing required files | validation fails | Return actionable validation error | Fix execution logic or bundle contents and rerun |
