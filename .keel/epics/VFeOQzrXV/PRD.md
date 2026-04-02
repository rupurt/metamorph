# Artifact Operations Pipeline - Product Requirements

## Problem Statement

Metamorph needs deterministic cache, validation, and publish flows so converted bundles become reusable operator-managed artifacts instead of one-shot outputs.

## Goals & Objectives

| ID | Goal | Success Metric | Target |
|----|------|----------------|--------|
| GOAL-01 | An operator can acquire a source artifact into a deterministic local cache and reuse it without guessing what Metamorph will fetch or where it will land. | Cache identity, reuse behavior, and source acquisition proof exist for representative local and `hf://` inputs. | One repeatable cache or reuse flow is proven through tests or command evidence. |
| GOAL-02 | Converted bundles are validated before they are treated as reusable or publishable outputs. | Validation rejects malformed bundles and accepts valid ones with explicit reports. | Positive and negative validation proof exists for the primary Hugging Face-style safetensors bundle shape. |
| GOAL-03 | An operator can plan and optionally publish a validated bundle to a destination such as Hugging Face without hidden network side effects. | Publish surfaces expose explicit destination, dry-run or preview behavior, and guarded execution semantics. | One safe publish or mirror workflow is proven without implying automatic uploads. |
| GOAL-04 | Operators can recover from cache, validation, and publish failures without reverse-engineering internal state. | CLI and docs explain failure causes and next actions for the primary pipeline paths. | README and workflow docs stay aligned with the shipped behavior and recovery paths. |

## Users

| Persona | Description | Primary Need |
|---------|-------------|--------------|
| Runtime Integrator | A Rust application developer embedding Metamorph into a local AI workflow. | Deterministic cache and validation behavior that can be trusted programmatically. |
| Local AI Operator | A developer or infra engineer using the CLI directly. | A repeatable fetch, validate, and publish workflow with explicit side effects. |
| Model Infrastructure Engineer | An engineer mirroring converted artifacts for a team or org. | Guarded publishing behavior and clear recovery steps when remote operations fail. |

## Scope

### In Scope

- [SCOPE-01] Deterministic source acquisition and cache layout for representative local and `hf://` inputs.
- [SCOPE-02] Validation of converted Hugging Face-style safetensors bundles and associated metadata before reuse or publish.
- [SCOPE-03] Explicit publish or mirror planning surfaces with dry-run or preview-first behavior.
- [SCOPE-04] Operator-visible recovery guidance for cache, validation, and publish failures.

### Out of Scope

- [SCOPE-05] Automatic background uploads, mirrors, or cache mutation without explicit user intent.
- [SCOPE-06] Multi-registry publishing beyond the first deliberate destination workflow.
- [SCOPE-07] Cache garbage collection, eviction strategy, or broad artifact lifecycle automation beyond the first deterministic reuse path.

## Requirements

### Functional Requirements

<!-- BEGIN FUNCTIONAL_REQUIREMENTS -->
| ID | Requirement | Goals | Priority | Rationale |
|----|-------------|-------|----------|-----------|
| FR-01 | The library and CLI must define deterministic cache identity and acquisition behavior for representative local and `hf://` sources. | GOAL-01 | must | Operators need a stable reuse contract before converted artifacts become operationally useful. |
| FR-02 | Metamorph must validate converted bundles and required metadata before reuse or publish paths report success. | GOAL-02 | must | Reusability is meaningless if malformed bundles can silently pass through the pipeline. |
| FR-03 | Publish or mirror flows must expose an explicit destination, preview or dry-run step, and guarded execution behavior. | GOAL-03 | must | Network-side effects must stay deliberate and auditable. |
| FR-04 | Cache, validation, and publish failures must produce actionable recovery guidance in the CLI and supporting docs. | GOAL-04 | must | This mission is operational, not just structural, so recovery needs to be first-class. |
<!-- END FUNCTIONAL_REQUIREMENTS -->

### Non-Functional Requirements

<!-- BEGIN NON_FUNCTIONAL_REQUIREMENTS -->
| ID | Requirement | Goals | Priority | Rationale |
|----|-------------|-------|----------|-----------|
| NFR-01 | The library and CLI must present the same truth about cache identity, validation outcomes, and publish prerequisites. | GOAL-01, GOAL-02, GOAL-03 | must | Prevents operational drift between embedded and command-line use. |
| NFR-02 | All new workflow paths must keep network and publish side effects explicit rather than inferred or automatic. | GOAL-01, GOAL-03 | must | Silent external mutation violates the repository policy contract. |
| NFR-03 | Story closure must include executable proof and doc updates whenever user-visible behavior changes. | GOAL-01, GOAL-02, GOAL-03, GOAL-04 | must | The repo requires evidence-backed delivery and zero drift. |
<!-- END NON_FUNCTIONAL_REQUIREMENTS -->

## Verification Strategy

| Area | Method | Evidence |
|------|--------|----------|
| Cache identity and acquisition | Unit or integration tests plus CLI proof | Captured `inspect`, `convert --plan-only`, cache, or reuse command evidence for representative local and `hf://` sources |
| Validation gates | Automated positive and negative validation tests plus CLI proof | Tests and command logs showing malformed bundles fail and valid bundles pass |
| Publish planning and guarded execution | Automated planning tests plus dry-run or controlled command proof | Story evidence showing explicit destination, preview output, and refusal of unsafe publish attempts |
| Recovery path clarity | Docs review plus command evidence | README and foundational doc updates in the same change as any new recovery behavior |

## Assumptions

| Assumption | Impact if Wrong | Validation |
|------------|-----------------|------------|
| A deterministic cache contract can be defined without implementing every future source type up front. | Cache work could sprawl into a generic storage subsystem too early. | Limit the first slice to representative local and `hf://` inputs during voyage planning. |
| The first publish destination can be treated as an explicitly named remote such as Hugging Face. | Publish planning may become too abstract to implement or verify. | Keep voyage scope destination-specific enough to prove behavior. |
| Recovery guidance can be expressed through command output and repo docs without a separate UI layer. | Failure handling may remain too opaque for operators. | Require CLI and docs updates in the recovery stories. |

## Open Questions & Risks

| Question/Risk | Owner | Status |
|---------------|-------|--------|
| What cache key and directory layout are stable enough for repeatable reuse without locking Metamorph into a premature global cache design? | Epic owner | Open |
| What is the minimal validated artifact contract for a publishable Hugging Face-style safetensors bundle? | Epic owner | Open |
| Which publish interactions can be proven locally via dry-run versus requiring controlled remote verification? | Epic owner | Open |
| Where should human review gates appear when licensing or redistribution expectations are unclear? | Epic owner | Open |

## Success Criteria

<!-- BEGIN SUCCESS_CRITERIA -->
- [ ] Operators can identify what Metamorph will cache, reuse, and validate for the primary local and `hf://` workflows.
- [ ] Converted bundles must pass explicit validation before they are considered reusable or publishable.
- [ ] Publish behavior remains previewable and explicitly gated rather than automatic.
- [ ] README, foundational docs, and the implemented workflow tell the same operational story.
<!-- END SUCCESS_CRITERIA -->
