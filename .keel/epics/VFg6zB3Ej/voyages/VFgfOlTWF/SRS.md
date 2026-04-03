# Harden Publish Recovery Proof And Documentation - SRS

## Summary

Epic: VFg6zB3Ej
Goal: Surface partial-failure recovery, repeatable mock-provider proof, and truthful docs for executable upload behavior.

This voyage closes the operational loop around executable upload:

- classify the main remote publish failure and partial-success states
- make retry surfaces explicit for operators and embedders
- prove preview, success, partial failure, and guarded refusal through a controlled provider
- update the README and foundational docs so the upload contract remains truthful

## Scope

### In Scope

- [SCOPE-01] Recovery messaging for missing credentials, missing or unsupported destinations, permission failures, interrupted uploads, and partial publish state
- [SCOPE-02] Explicit per-artifact retry signals and remaining-work reporting after partial remote writes
- [SCOPE-05] Controlled end-to-end proof for preview, success, guarded refusal, and representative partial-failure flows
- [SCOPE-06] README and foundational doc updates reflecting the executable upload contract

### Out of Scope

- [SCOPE-07] Automatic retry loops, resumable sync engines, or background repair behavior
- [SCOPE-08] Broader registry support or repository governance automation
- [SCOPE-09] New conversion or acquisition behavior unrelated to remote publish execution

## Functional Requirements

<!-- BEGIN FUNCTIONAL_REQUIREMENTS -->
| ID | Requirement | Scope | Source | Verification |
|----|-------------|-------|--------|--------------|
| SRS-01 | Publish failures and guarded refusals must distinguish credentials, missing destination, permission failure, interrupted transfer, and partial remote publish state in operator-facing recovery paths. | SCOPE-01 | FR-04 | automated |
| SRS-02 | Publish reports and CLI output must surface which artifacts succeeded, which remain pending, and what retry action the operator can take after a partial failure. | SCOPE-02 | FR-03 | automated |
| SRS-03 | Controlled end-to-end proof must exist for preview, successful execute, guarded refusal, and representative failure or retry flows through the primary library or CLI entry points. | SCOPE-05 | FR-05 | automated |
| SRS-04 | README, USER_GUIDE, ARCHITECTURE, and CODE_WALKTHROUGH must describe the executable upload contract truthfully, including existing-repo preconditions and human-sensitive seams. | SCOPE-06 | FR-06 | automated |
<!-- END FUNCTIONAL_REQUIREMENTS -->

## Non-Functional Requirements

<!-- BEGIN NON_FUNCTIONAL_REQUIREMENTS -->
| ID | Requirement | Scope | Source | Verification |
|----|-------------|-------|--------|--------------|
| SRS-NFR-01 | Docs and command output must use consistent terminology for preview, complete publish, partial publish, guarded refusal, and retry guidance. | SCOPE-01, SCOPE-02, SCOPE-06 | NFR-01 | automated |
| SRS-NFR-02 | Mock-provider proof must remain repeatable enough for story closure and commit-hook verification. | SCOPE-05 | NFR-03 | automated |
| SRS-NFR-03 | Retry surfaces must remain explicit and operator-driven rather than turning into hidden automatic repair behavior. | SCOPE-01, SCOPE-02, SCOPE-05 | NFR-02 | automated |
<!-- END NON_FUNCTIONAL_REQUIREMENTS -->
