# Wire Real Upload Execution Into Library And CLI - SRS

## Summary

Epic: VFg6zB3Ej
Goal: Make `publish()` and `upload --execute` perform explicit remote writes while preserving preview-first behavior and thin CLI orchestration.

This voyage makes the new publish substrate operational:

- connect the executor seam to the public library `publish()` workflow
- keep `plan_publish()` and preview-first behavior intact
- let `metamorph upload --execute` perform a real remote write for the supported destination path
- keep the CLI a renderer over library publish policy and outcome types

## Scope

### In Scope

- [SCOPE-01] Integrating the publish executor seam into `publish()` for validated local bundles
- [SCOPE-02] Updating `metamorph upload` to preserve preview mode and expose real `--execute` behavior
- [SCOPE-03] Guarding remote publish on validation, explicit execute intent, credentials, and supported existing destinations
- [SCOPE-04] Keeping the same publish truth across the library and CLI

### Out of Scope

- [SCOPE-05] Expanded recovery messaging and doc refresh beyond the execution path itself
- [SCOPE-06] Automatic repository creation or post-upload synchronization logic
- [SCOPE-07] Non-Hugging-Face publish targets or bulk registry management

## Functional Requirements

<!-- BEGIN FUNCTIONAL_REQUIREMENTS -->
| ID | Requirement | Scope | Source | Verification |
|----|-------------|-------|--------|--------------|
| SRS-01 | `publish()` must execute a validated local bundle through the library-owned publish substrate when `execute` is true for the supported destination path. | SCOPE-01 | FR-01 | automated |
| SRS-02 | `metamorph upload` must preserve preview-only behavior by default and render the same execution truth as the library when `--execute` is supplied. | SCOPE-02, SCOPE-04 | FR-02 | automated |
| SRS-03 | Remote publish execution must refuse requests that lack validation, explicit execute intent, credentials, or a supported existing destination before remote mutation begins. | SCOPE-03 | FR-02 | automated |
| SRS-04 | CLI execution wiring must continue to consume the library-owned publish flow instead of introducing CLI-specific upload policy. | SCOPE-04 | FR-03 | automated |
<!-- END FUNCTIONAL_REQUIREMENTS -->

## Non-Functional Requirements

<!-- BEGIN NON_FUNCTIONAL_REQUIREMENTS -->
| ID | Requirement | Scope | Source | Verification |
|----|-------------|-------|--------|--------------|
| SRS-NFR-01 | Preview, complete publish, partial publish, and guarded-refusal states must remain distinguishable in command output. | SCOPE-02, SCOPE-03, SCOPE-04 | NFR-02 | automated |
| SRS-NFR-02 | The validated local bundle and preview behavior must remain intact while real remote execution is added. | SCOPE-01, SCOPE-02, SCOPE-03 | NFR-04 | automated |
| SRS-NFR-03 | The library and CLI must stay behaviorally aligned on publish prerequisites and outcome reporting. | SCOPE-01, SCOPE-02, SCOPE-03, SCOPE-04 | NFR-01 | automated |
<!-- END NON_FUNCTIONAL_REQUIREMENTS -->
