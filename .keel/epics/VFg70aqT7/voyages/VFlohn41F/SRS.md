# Reclassify The Compatibility Matrix For Executable Relayouts - SRS

## Summary

Epic: VFg70aqT7
Goal: Replace generic compatible-path claims with truthful format-specific backend registration and blockers.

This voyage cleans up the capability contract before new execution ships:

- replace the generic `from == to` relayout promise with explicit per-format capability entries
- decide which same-format requests are truly promotable now and which should be reclassified
- expose request-specific blockers for local-only execution and missing metadata prerequisites
- keep the registry as the shared source of compatibility, planning, and execution truth

## Scope

### In Scope

- [SCOPE-01] Explicit capability registration for promoted relayout and bundle-materialization paths
- [SCOPE-02] Reclassification of unsupported same-format cases that lack a reusable-output contract
- [SCOPE-03] Compatibility blockers for local-only execution and unsupported local-vs-remote request shapes
- [SCOPE-04] Compatibility blockers for missing Hugging Face metadata prerequisites on `safetensors -> hf-safetensors`

### Out of Scope

- [SCOPE-05] Implementing the relayout backends themselves
- [SCOPE-06] CLI rendering and documentation refresh beyond the registry truth it depends on
- [SCOPE-07] New remote execution or publish behavior

## Functional Requirements

<!-- BEGIN FUNCTIONAL_REQUIREMENTS -->
| ID | Requirement | Scope | Source | Verification |
|----|-------------|-------|--------|--------------|
| SRS-01 | The registry must replace the generic same-format relayout path with explicit capabilities for the promotable format pairs and reclassify unsupported same-format cases rather than advertising one blanket placeholder. | SCOPE-01, SCOPE-02 | FR-01 | automated |
| SRS-02 | Compatibility reports must surface request-specific blockers when a promoted backend currently requires a local source path or local output target. | SCOPE-03 | FR-04 | automated |
| SRS-03 | Compatibility reports for `safetensors -> hf-safetensors` must surface missing metadata sidecars or unsupported source shapes before conversion executes. | SCOPE-04 | FR-03 | automated |
<!-- END FUNCTIONAL_REQUIREMENTS -->

## Non-Functional Requirements

<!-- BEGIN NON_FUNCTIONAL_REQUIREMENTS -->
| ID | Requirement | Scope | Source | Verification |
|----|-------------|-------|--------|--------------|
| SRS-NFR-01 | The capability registry remains the single source of truth consumed by compatibility, planning, and conversion dispatch. | SCOPE-01, SCOPE-02, SCOPE-03, SCOPE-04 | NFR-01 | automated |
| SRS-NFR-02 | Blocked or reclassified requests remain distinguishable from unsupported format pairs instead of collapsing into one generic error. | SCOPE-02, SCOPE-03, SCOPE-04 | NFR-04 | automated |
<!-- END NON_FUNCTIONAL_REQUIREMENTS -->
