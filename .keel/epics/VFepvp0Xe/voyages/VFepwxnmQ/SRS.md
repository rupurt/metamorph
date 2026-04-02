# Extract Stable Library Modules - SRS

## Summary

Epic: VFepvp0Xe
Goal: Split the monolithic library into stable source, format, plan, transform, validate, cache, and publish modules while preserving the current public surface and thin CLI.

This voyage establishes the structural seams the mission depends on:

- turn the current `lib.rs` monolith into named modules that match the product vocabulary already promised in `README.md`
- keep the public workflow usable through top-level facade re-exports rather than pushing churn onto callers or the CLI
- isolate the current `gguf -> hf-safetensors` execution path behind a transform-oriented boundary
- update docs so the architecture story matches the new source layout immediately

## Scope

### In Scope

- [SCOPE-01] Extract dedicated `source`, `format`, `plan`, `transform`, `validate`, `cache`, and `publish` modules in `crates/metamorph/src`
- [SCOPE-02] Move current inspection, planning, cache, validation, and publish logic into the matching modules without changing the shipped workflow contract
- [SCOPE-03] Isolate the existing `gguf -> hf-safetensors` execution backend behind a transform-oriented module seam
- [SCOPE-04] Preserve a thin CLI that consumes library entry points instead of duplicating product logic
- [SCOPE-05] Update `README.md`, `ARCHITECTURE.md`, and `CODE_WALKTHROUGH.md` to match the extracted module structure

### Out of Scope

- [SCOPE-06] Adding new conversion backends beyond what is needed to complete the extraction
- [SCOPE-07] Dynamic plugin loading or runtime backend discovery
- [SCOPE-08] New remote fetch or publish behavior unrelated to the modularization boundary

## Functional Requirements

<!-- BEGIN FUNCTIONAL_REQUIREMENTS -->
| ID | Requirement | Scope | Source | Verification |
|----|-------------|-------|--------|--------------|
| SRS-01 | `crates/metamorph` must expose named modules for source, format, plan, transform, validate, cache, and publish concerns while preserving a top-level facade for the current public workflow. | SCOPE-01 | FR-01 | automated |
| SRS-02 | Inspection, planning, cache or acquisition, validation, and publish logic must move out of the monolithic `lib.rs` into the corresponding modules without changing the shipped workflow behavior. | SCOPE-02 | FR-01 | automated |
| SRS-03 | The current `gguf -> hf-safetensors` execution backend must live behind a transform-oriented module seam rather than top-level monolith code. | SCOPE-03 | FR-01 | automated |
| SRS-04 | The CLI must continue to compose library entry points instead of owning conversion, validation, or backend-selection rules. | SCOPE-04 | FR-01 | automated |
<!-- END FUNCTIONAL_REQUIREMENTS -->

## Non-Functional Requirements

<!-- BEGIN NON_FUNCTIONAL_REQUIREMENTS -->
| ID | Requirement | Scope | Source | Verification |
|----|-------------|-------|--------|--------------|
| SRS-NFR-01 | Public library entry points and the current CLI workflow must remain behaviorally compatible enough for existing tests and examples to stay green after the extraction. | SCOPE-01, SCOPE-02, SCOPE-03, SCOPE-04 | NFR-02 | automated |
| SRS-NFR-02 | `README.md`, `ARCHITECTURE.md`, and `CODE_WALKTHROUGH.md` must describe the new module layout truthfully in the same change as the extraction. | SCOPE-05 | NFR-04 | automated |
<!-- END NON_FUNCTIONAL_REQUIREMENTS -->
