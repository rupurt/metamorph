# Inspect Convert And Validate Candle Bundle - SRS

## Summary

Epic: VFbp961HM
Goal: Prove the first end-to-end inspect, convert, and validate workflow for a Candle-oriented bundle.

This voyage covers the smallest real delivery slice promised by the README:

- inspect local and `hf://` sources
- run a first `gguf -> hf-safetensors` path with explicit lossy opt-in
- validate that the output looks like a Candle-friendly Hugging Face-style bundle

## Scope

### In Scope

- [SCOPE-01] Inspecting local paths and Hugging Face-style references through the library and CLI
- [SCOPE-02] Planning and executing the first `gguf -> hf-safetensors` conversion path
- [SCOPE-03] Validating the expected output layout for Candle-oriented consumption
- [SCOPE-04] Tests and command-level proof for the shipped path

### Out of Scope

- [SCOPE-05] Additional runtime targets or model-family-specific optimizations
- [SCOPE-06] Automatic upload or publish flows
- [SCOPE-07] A generalized backend plugin system

## Functional Requirements

<!-- BEGIN FUNCTIONAL_REQUIREMENTS -->
| ID | Requirement | Scope | Source | Verification |
|----|-------------|-------|--------|--------------|
| SRS-01 | The library and CLI must inspect local paths and `hf://` references and return an explicit detected format or an explicit unknown result. | SCOPE-01 | FR-01 | automated |
| SRS-02 | The planner and first execution path must support `gguf -> hf-safetensors` with explicit lossy opt-in and clear errors for unsupported or unsafe requests. | SCOPE-02 | FR-02 | automated |
| SRS-03 | Validation must verify that the output bundle contains the expected Candle-friendly files and fail clearly when required artifacts are missing. | SCOPE-03 | FR-04 | automated |
<!-- END FUNCTIONAL_REQUIREMENTS -->

## Non-Functional Requirements

<!-- BEGIN NON_FUNCTIONAL_REQUIREMENTS -->
| ID | Requirement | Scope | Source | Verification |
|----|-------------|-------|--------|--------------|
| SRS-NFR-01 | The library and CLI must stay behaviorally aligned on inspection, lossy gating, and validation outcomes for this path. | SCOPE-04 | NFR-01 | automated |
<!-- END NON_FUNCTIONAL_REQUIREMENTS -->
