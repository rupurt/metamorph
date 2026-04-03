# Prove And Document Compatible Path Promotion - SRS

## Summary

Epic: VFg70aqT7
Goal: Prove the new executable matrix through CLI evidence and documentation.

This voyage makes the promotion legible to operators and embedders:

- render the new backend and blocker truth through `metamorph convert`
- add automated proof for successful relayouts and representative blocked cases
- update README and foundational docs so the shipped matrix matches the code

## Scope

### In Scope

- [SCOPE-01] CLI rendering for promoted backends and request-specific blockers
- [SCOPE-02] Automated proof for successful relayout and metadata-gated bundle materialization
- [SCOPE-03] Automated proof for representative blocked or reclassified cases
- [SCOPE-04] README and foundational doc updates for the promoted conversion matrix

### Out of Scope

- [SCOPE-05] Additional backend implementation beyond what voyage 2 ships
- [SCOPE-06] New publish or remote cache features
- [SCOPE-07] Narrative docs unrelated to the conversion matrix contract

## Functional Requirements

<!-- BEGIN FUNCTIONAL_REQUIREMENTS -->
| ID | Requirement | Scope | Source | Verification |
|----|-------------|-------|--------|--------------|
| SRS-01 | `metamorph convert` must render the promoted backend labels, compatibility status, and request-specific blockers for the new local relayout flows. | SCOPE-01 | FR-05 | automated |
| SRS-02 | The mission must include automated CLI or library proof for successful local relayout and successful metadata-backed `safetensors -> hf-safetensors`. | SCOPE-02 | FR-05 | automated |
| SRS-03 | The mission must include automated proof for representative blocked or reclassified requests such as remote-only attempts, missing metadata sidecars, or unsupported same-format cases. | SCOPE-03 | FR-05 | automated |
| SRS-04 | README and foundational docs must distinguish executable local relayouts from blocked or unsupported cases in operator and integration language. | SCOPE-04 | FR-06 | automated |
<!-- END FUNCTIONAL_REQUIREMENTS -->

## Non-Functional Requirements

<!-- BEGIN NON_FUNCTIONAL_REQUIREMENTS -->
| ID | Requirement | Scope | Source | Verification |
|----|-------------|-------|--------|--------------|
| SRS-NFR-01 | CLI messaging must stay aligned with library-owned compatibility and conversion truth instead of introducing CLI-specific policy. | SCOPE-01, SCOPE-02, SCOPE-03 | NFR-01 | automated |
| SRS-NFR-02 | Story closure must rely on automated proof rather than chat-only claims for promoted and blocked paths. | SCOPE-02, SCOPE-03, SCOPE-04 | NFR-04 | automated |
| SRS-NFR-03 | The documentation must stay explicit about local-only execution and required metadata sidecars. | SCOPE-04 | NFR-04 | automated |
<!-- END NON_FUNCTIONAL_REQUIREMENTS -->
