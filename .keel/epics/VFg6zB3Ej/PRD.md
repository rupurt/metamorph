# Guarded Hugging Face Publish Execution - Product Requirements

## Problem Statement

Metamorph can preview a publish today, but `upload --execute` still stops before a real remote write, so operators cannot complete a validated publish flow through the product.

## Goals & Objectives

| ID | Goal | Success Metric | Target |
|----|------|----------------|--------|
| GOAL-01 | An operator can publish a validated local `hf-safetensors` bundle to a supported Hugging Face destination through an explicit execution path. | Controlled proof shows `upload --execute` or the library publish surface carrying a validated bundle through a real or mock-backed remote write instead of a not-implemented stop. | One repeatable existing-repo publish flow is proven through tests and CLI evidence. |
| GOAL-02 | Publish execution remains explicit, guarded, and credential-aware rather than turning into an accidental network side effect. | Preview-first behavior remains intact, `--execute` is required for mutation, and credential or destination preflight failures are surfaced before unsafe execution. | Library and CLI proof exists for dry-run, explicit execute, and guarded refusal cases. |
| GOAL-03 | Operators and embedders can inspect publish outcomes, partial-failure states, and retry guidance without reverse-engineering remote state. | Publish reports and CLI output distinguish success, partial upload, destination rejection, and retryable failure paths. | Controlled proof exists for success plus representative failure and retry surfaces. |
| GOAL-04 | The README and foundational docs describe the upload contract truthfully, including what is executable, what still requires prior setup, and where human review remains required. | Shipped docs match the implemented publish behavior and preconditions. | README and foundational docs are updated in the same change set as the executable upload slice. |

## Users

| Persona | Description | Primary Need |
|---------|-------------|--------------|
| Runtime Integrator | A Rust developer embedding Metamorph into a local AI workflow or artifact pipeline. | A library-owned publish surface with explicit outcomes that can be inspected programmatically. |
| Local AI Operator | A developer or infra engineer using the CLI directly. | A real `upload --execute` path that keeps remote mutation explicit and recoverable. |
| Model Infrastructure Engineer | An engineer mirroring validated bundles into existing model repositories for a team or org. | Publish execution, partial-failure visibility, and retry guidance that do not depend on manual remote guesswork. |

## Scope

### In Scope

- [SCOPE-01] A library-owned Hugging Face publish executor seam for the first supported remote destination workflow.
- [SCOPE-02] Real remote write execution for validated local `hf-safetensors` bundles into an explicitly named existing Hugging Face repository.
- [SCOPE-03] Preview-first, credential-aware, and policy-gated CLI and library execution semantics that preserve explicit user intent.
- [SCOPE-04] Structured publish outcome reporting for successful writes, partial failures, destination rejections, and retry guidance.
- [SCOPE-05] Controlled mock-provider or equivalent proof for publish execution without depending on uncontrolled live remote state.
- [SCOPE-06] README and foundational doc updates reflecting the executable upload contract.

### Out of Scope

- [SCOPE-07] Automatic background publishing, implicit synchronization, or any remote write without explicit operator intent.
- [SCOPE-08] Multi-registry or non-Hugging-Face publish targets in the same mission.
- [SCOPE-09] Automatic repository creation, model-card generation, or destination bootstrap beyond the first explicit existing-repo execution path.
- [SCOPE-10] Legal, licensing, redistribution, or governance decisions that require human judgment before public publication.

## Requirements

### Functional Requirements

<!-- BEGIN FUNCTIONAL_REQUIREMENTS -->
| ID | Requirement | Goals | Priority | Rationale |
|----|-------------|-------|----------|-----------|
| FR-01 | The library must execute a supported Hugging Face publish flow for a validated local `hf-safetensors` bundle instead of stopping at a not-implemented error. | GOAL-01 | must | This is the mission's core user-visible gap. |
| FR-02 | Publish execution must remain preview-first and require explicit execute intent, validation preflight, destination checks, and credentials before remote mutation begins. | GOAL-02 | must | Network-side effects must stay deliberate and auditable. |
| FR-03 | Publish reports must describe which artifacts were uploaded, which were skipped or updated, and whether the result was complete, partial, or failed. | GOAL-03 | must | Operators need actionable remote outcome truth, not a boolean success guess. |
| FR-04 | Publish failures must surface actionable recovery guidance for missing credentials, missing or unsupported destinations, permission failures, interrupted transfers, and partial uploads. | GOAL-03 | must | Real execution is not usable if recovery remains opaque. |
| FR-05 | The mission must include controlled proof for preview, success, partial-failure, and guarded-refusal flows without relying on live remote state. | GOAL-01, GOAL-02, GOAL-03 | must | Repeatable verification is required for a networked workflow. |
| FR-06 | README and operator-facing docs must explain the executable upload contract, including what requires an existing repo and what still remains human-sensitive. | GOAL-04 | must | User-visible behavior and docs must move together. |
<!-- END FUNCTIONAL_REQUIREMENTS -->

### Non-Functional Requirements

<!-- BEGIN NON_FUNCTIONAL_REQUIREMENTS -->
| ID | Requirement | Goals | Priority | Rationale |
|----|-------------|-------|----------|-----------|
| NFR-01 | The library remains the source of truth for publish policy and remote execution while the CLI stays a thin rendering layer. | GOAL-01, GOAL-02, GOAL-03, GOAL-04 | must | The repo architecture should not regress while adding network behavior. |
| NFR-02 | Remote writes must stay explicit enough that operators can distinguish preview, complete publish, partial publish, and guarded refusal. | GOAL-02, GOAL-03 | must | Hidden or ambiguous mutation would violate the repo's operational rules. |
| NFR-03 | Story closure must include automated or mock-backed publish proof for each new execution and recovery surface. | GOAL-01, GOAL-02, GOAL-03 | must | Networked behavior needs stronger proof than chat-only claims. |
| NFR-04 | Remote publish failures must not corrupt the validated local bundle or misreport local state after a failed remote attempt. | GOAL-02, GOAL-03 | must | Operators need confidence that retry decisions can be made from stable local artifacts. |
<!-- END NON_FUNCTIONAL_REQUIREMENTS -->

## Verification Strategy

| Area | Method | Evidence |
|------|--------|----------|
| Publish execution substrate | Unit or integration tests against a controlled provider plus library proof | Story evidence showing a validated bundle uploaded through the library-owned executor |
| Preview and guarded execution | Automated tests plus CLI proof | Evidence showing preview-only behavior, explicit execute behavior, and refusal when credentials or destination preconditions are missing |
| Outcome reporting and recovery | Negative tests and CLI proof | Evidence for success, partial upload, destination rejection, and retry guidance |
| Docs alignment | Review plus command evidence | README and foundational docs updated in the same change set as the new execution behavior |

## Assumptions

| Assumption | Impact if Wrong | Validation |
|------------|-----------------|------------|
| The first executable publish slice can target an explicitly named existing Hugging Face repository without solving repo bootstrap in the same mission. | The epic could sprawl into repo provisioning and policy automation too early. | Keep voyage scope tied to existing-repo execution and explicit failure guidance for missing destinations. |
| A controlled mock provider can model the required publish behaviors closely enough to prove success, partial failure, and retry surfaces. | Verification may become flaky or dependent on live remote services. | Require mock-backed publish proof during voyage planning. |
| Per-artifact outcome reporting is sufficient for the first retry surface even if resumable sync is deferred. | Retry guidance may be too weak for operators who hit partial failures. | Keep the first slice explicit about what uploaded and what still needs retry. |

## Open Questions & Risks

| Question/Risk | Owner | Status |
|---------------|-------|--------|
| Which Hugging Face repository interaction surface is minimal but sufficient for the first controlled publish execution path? | Epic owner | Open |
| Should the first publish slice only target existing model repos, or is controlled repo creation required for a credible operator workflow? | Epic owner | Open |
| What per-artifact outcome model is sufficient to describe partial publishes and retries without promising a full sync engine? | Epic owner | Open |
| Which destination or permission failures require a hard stop versus retry guidance? | Epic owner | Open |

## Success Criteria

<!-- BEGIN SUCCESS_CRITERIA -->
- [ ] An operator can execute a validated publish to a supported Hugging Face destination instead of stopping at a not-implemented error.
- [ ] Preview-first and guarded execution semantics remain explicit and credential-aware.
- [ ] Publish reports distinguish complete, partial, and failed remote outcomes with actionable retry guidance.
- [ ] Controlled mock-provider proof exists for preview, success, and representative failure flows.
- [ ] README and foundational docs describe the shipped upload contract truthfully.
<!-- END SUCCESS_CRITERIA -->
