# Promote Compatible Paths Into Executable Flows - Product Requirements

## Problem Statement

Metamorph already distinguishes executable, planned-only, and unsupported paths, but some compatible paths remain registry entries without execution backends, validation coverage, or delivery proof.

## Goals & Objectives

| ID | Goal | Success Metric | Target |
|----|------|----------------|--------|
| GOAL-01 | Local operators and embedders can execute non-lossy safetensors relayout workflows instead of stopping at planned-only compatibility. | Controlled proof shows local `safetensors -> safetensors` and `hf-safetensors -> hf-safetensors` requests executing through real backends and producing reusable outputs. | At least two relayout paths are executable with test and CLI proof. |
| GOAL-02 | `safetensors -> hf-safetensors` becomes a truthful workflow surface instead of an ambiguous planned-only promise. | The path either executes for a clearly defined local source contract or surfaces explicit blockers and source-shape requirements before conversion. | Metadata-backed local bundle promotion is proven, and blocked cases are explicit. |
| GOAL-03 | Compatibility and planning stay truthful when a backend exists but a specific request is still gated by source locality, target support, or missing metadata. | Compatibility reports distinguish executable backend class from request-specific blockers without overclaiming remote or unsupported execution. | Automated proof exists for promoted, blocked, and reclassified requests. |
| GOAL-04 | README and foundational docs describe the promoted backend matrix in terms a CLI operator or integrator can act on immediately. | Shipped docs list executable, blocked, and unsupported paths with the actual local and metadata constraints. | README and foundational docs are updated in the same change set as the new backends. |

## Users

| Persona | Description | Primary Need |
|---------|-------------|--------------|
| Local AI Operator | A developer using `metamorph convert` directly on local artifact trees. | Real relayout and bundle-materialization paths that do not stop at planned-only. |
| Runtime Integrator | A Rust developer embedding compatibility, plan, and convert flows. | A truthful registry and blocker model they can branch on programmatically. |
| Model Artifact Maintainer | An engineer normalizing local model drops before validation or publish. | Clear rules for which local safetensors layouts can be promoted into reusable bundles. |

## Scope

### In Scope

- [SCOPE-01] Replacing the generic same-format relayout promise with explicit, format-specific capability registration.
- [SCOPE-02] Executable local relayout backends for `safetensors -> safetensors` and `hf-safetensors -> hf-safetensors`.
- [SCOPE-03] A first local `safetensors -> hf-safetensors` execution path for sources that provide the required Hugging Face metadata sidecars.
- [SCOPE-04] Request-specific compatibility blockers for local-only execution, unsupported targets, and missing bundle metadata.
- [SCOPE-05] Automated library and CLI proof for promoted backends, blocked cases, and any explicit reclassification.
- [SCOPE-06] README and foundational doc updates that reflect the promoted backend truth.

### Out of Scope

- [SCOPE-07] New lossy conversion families or silent dequantize/requantize behavior.
- [SCOPE-08] Broad Hugging Face remote fetch expansion beyond the already shipped representative GGUF slice.
- [SCOPE-09] Dynamic backend loading, plugin registries, or a general-purpose format-matrix explosion.
- [SCOPE-10] Sharded safetensors bundle synthesis, repo bootstrap, or remote publish policy changes.

## Requirements

### Functional Requirements

<!-- BEGIN FUNCTIONAL_REQUIREMENTS -->
| ID | Requirement | Goals | Priority | Rationale |
|----|-------------|-------|----------|-----------|
| FR-01 | The registry must replace the generic same-format relayout promise with explicit capabilities and reclassification for formats that do not yet have a reusable-output contract. | GOAL-01, GOAL-03 | must | Compatibility should stop implying that every `from == to` path is equally meaningful or executable. |
| FR-02 | The library must execute local `safetensors -> safetensors` and local `hf-safetensors -> hf-safetensors` relayout requests through real backends that produce reusable outputs. | GOAL-01 | must | These are the most direct planned-only paths to promote safely. |
| FR-03 | The library must either execute `safetensors -> hf-safetensors` for a clearly defined local metadata-backed source contract or reject the request with explicit blockers before conversion. | GOAL-02, GOAL-03 | must | The current planned-only label is too vague to guide operators or integrators. |
| FR-04 | Compatibility reports must expose request-specific blockers for local-only backends, unsupported conversion targets, and missing metadata prerequisites. | GOAL-02, GOAL-03 | must | A backend class existing is not the same as the current request being runnable. |
| FR-05 | CLI and library proof must cover successful relayout, successful metadata-backed bundle promotion, and representative blocked or reclassified cases. | GOAL-01, GOAL-02, GOAL-03 | must | The conversion matrix is only credible if the promoted and blocked cases are both proven. |
| FR-06 | README and foundational docs must list the promoted backend matrix and the source-contract limits truthfully. | GOAL-04 | must | Operators and embedders need the contract in terms they can actually act on. |
<!-- END FUNCTIONAL_REQUIREMENTS -->

### Non-Functional Requirements

<!-- BEGIN NON_FUNCTIONAL_REQUIREMENTS -->
| ID | Requirement | Goals | Priority | Rationale |
|----|-------------|-------|----------|-----------|
| NFR-01 | The registry remains the single source of truth for compatibility, planning, and execution dispatch while the CLI stays a thin rendering layer. | GOAL-01, GOAL-02, GOAL-03, GOAL-04 | must | Backend truth should not fork across library and CLI code. |
| NFR-02 | Newly executable outputs must pass validation before they are reported as reusable conversion results. | GOAL-01, GOAL-02, GOAL-03 | must | Promotion is only useful if the outputs remain safely reusable. |
| NFR-03 | No newly promoted path may hide lossy behavior, implicit remote fetch, or implicit publish side effects. | GOAL-01, GOAL-02, GOAL-03 | must | The repo's explicitness guarantees must hold as the matrix expands. |
| NFR-04 | Reclassified or blocked requests must remain legible to both CLI operators and library consumers. | GOAL-03, GOAL-04 | must | Truthful degradation matters as much as new execution. |
<!-- END NON_FUNCTIONAL_REQUIREMENTS -->

## Verification Strategy

| Area | Method | Evidence |
|------|--------|----------|
| Registry truth | Unit tests plus compatibility-report proof | Evidence showing promoted backends, request blockers, and reclassified paths |
| Local relayout execution | Library and CLI tests | Evidence showing reusable `safetensors` and `hf-safetensors` relayout outputs |
| Metadata-backed bundle promotion | Library and CLI tests | Evidence showing `safetensors -> hf-safetensors` success with required sidecars and explicit blocking without them |
| Docs alignment | Review plus command evidence | README and foundational docs updated with the shipped backend matrix |

## Assumptions

| Assumption | Impact if Wrong | Validation |
|------------|-----------------|------------|
| Local relayout and metadata-backed bundle promotion are the highest-value safe promotions before broader format expansion. | The epic might optimize low-leverage paths first. | Keep voyage scope tied to currently advertised planned-only paths. |
| A reusable `hf-safetensors` bundle can be constructed from plain safetensors only when the local source also provides Hugging Face metadata sidecars. | The bundle materialization path could either over-promise or under-deliver. | Require explicit blockers and proof for both the satisfied and unsatisfied source contracts. |
| Unsupported same-format cases such as `gguf -> gguf` can be reclassified without harming the operator contract if the reasoning is documented. | The matrix could regress in clarity or surprise users. | Update compatibility tests and docs in the same change set as the reclassification. |

## Open Questions & Risks

| Question/Risk | Owner | Status |
|---------------|-------|--------|
| What is the smallest truthful local source contract for `safetensors -> hf-safetensors` without inventing synthetic model metadata? | Epic owner | Open |
| Should partial Hugging Face metadata sidecars continue to inspect as plain safetensors until the bundle is complete? | Epic owner | Open |
| Which same-format requests should be reclassified instead of kept as planned-only placeholders? | Epic owner | Open |

## Success Criteria

<!-- BEGIN SUCCESS_CRITERIA -->
- [ ] Local `safetensors -> safetensors` and `hf-safetensors -> hf-safetensors` relayout requests execute through real backends and validate as reusable outputs.
- [ ] `safetensors -> hf-safetensors` is either executable for a defined local metadata-backed source contract or explicitly blocked with actionable reasons.
- [ ] Compatibility reports distinguish backend availability from request-specific blockers such as local-only execution and missing metadata.
- [ ] Promoted, blocked, and reclassified paths all have automated proof through library or CLI tests.
- [ ] README and foundational docs describe the shipped conversion matrix truthfully.
<!-- END SUCCESS_CRITERIA -->
