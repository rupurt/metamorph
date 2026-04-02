# Remote Hugging Face Acquisition And Cache Materialization - Product Requirements

## Problem Statement

Metamorph can reason about `hf://` sources today, but it still cannot fetch them on demand; operators must pre-populate the cache manually before remote execution can work.

## Goals & Objectives

| ID | Goal | Success Metric | Target |
|----|------|----------------|--------|
| GOAL-01 | An operator can point Metamorph at a representative `hf://repo[@revision]` source and have the required artifact fetched into managed cache storage on demand. | `cache source` or conversion proof shows a remote source being fetched without manual cache prepopulation. | One repeatable remote fetch flow is proven through tests and CLI evidence. |
| GOAL-02 | Remote fetch preserves deterministic cache semantics and makes reuse or refresh behavior explicit rather than surprising. | Cache identity, fetch or reuse outcome, and any refresh behavior are visible through the library and CLI. | Representative fetch, reuse, and refresh proof exists for revision-aware `hf://` inputs. |
| GOAL-03 | Remote fetch failures are actionable for both operators and embedders. | Auth, missing revision, interrupted transfer, and invalid remote layout errors surface recovery guidance rather than generic cache-miss messaging. | Tests and command evidence show distinct recovery output for the primary failure classes. |
| GOAL-04 | The README and workflow docs describe remote acquisition truthfully, including what Metamorph now does automatically versus what still remains explicit. | Shipped docs match the new remote fetch contract and recovery path. | README and foundational docs are updated in the same change set as the new behavior. |

## Users

| Persona | Description | Primary Need |
|---------|-------------|--------------|
| Runtime Integrator | A Rust application developer embedding Metamorph into a local AI workflow. | Programmatic source acquisition that can fetch, reuse, and diagnose remote model sources predictably. |
| Local AI Operator | A developer or infra engineer using the CLI directly. | A direct `hf://` workflow that no longer requires manual cache seeding before conversion. |
| Model Infrastructure Engineer | An engineer managing shared local model mirrors or build pipelines. | Deterministic remote acquisition semantics and clear refresh or recovery behavior for automated pipelines. |

## Scope

### In Scope

- [SCOPE-01] On-demand acquisition of representative Hugging Face model sources into deterministic managed cache paths.
- [SCOPE-02] Revision-aware cache identity, reuse, and explicit refresh semantics for remote sources.
- [SCOPE-03] Integration of remote fetch behavior into source acquisition, cache inspection, and remote conversion execution paths.
- [SCOPE-04] Operator-visible recovery guidance for auth failures, missing revisions, interrupted fetches, and remote layout mismatches.
- [SCOPE-05] Test harnesses and mock-provider proofs for remote acquisition flows without relying on uncontrolled live network state.
- [SCOPE-06] README and foundational doc updates reflecting the remote fetch contract.

### Out of Scope

- [SCOPE-07] General support for non-Hugging-Face registries or arbitrary remote storage backends.
- [SCOPE-08] Background cache refresh daemons, eviction policies, or generalized artifact lifecycle automation.
- [SCOPE-09] Remote publish execution, which remains covered by a separate mission.
- [SCOPE-10] Silent lossy fallback behavior or changes to conversion gating unrelated to transport.

## Requirements

### Functional Requirements

<!-- BEGIN FUNCTIONAL_REQUIREMENTS -->
| ID | Requirement | Goals | Priority | Rationale |
|----|-------------|-------|----------|-----------|
| FR-01 | The library and CLI must fetch representative `hf://repo[@revision]` sources on demand into deterministic managed cache paths instead of requiring manual cache prepopulation. | GOAL-01, GOAL-02 | must | This is the mission's core user-visible capability. |
| FR-02 | Source acquisition must report whether a remote source was fetched, reused from cache, or explicitly refreshed, and must surface the resolved local path that subsequent steps will use. | GOAL-02 | must | Operators and embeddings need to trust cache semantics rather than infer them. |
| FR-03 | `cache source`, conversion execution, and other acquisition-facing flows must share the same remote fetch behavior and error surface. | GOAL-01, GOAL-02, GOAL-03 | must | Prevents drift between inspection, conversion, and operational tooling. |
| FR-04 | Remote fetch failures must produce actionable recovery guidance for credentials, missing revisions, interrupted downloads, and unsupported or malformed remote artifact layouts. | GOAL-03 | must | Replacing cache-miss-only behavior is not enough unless recovery becomes legible. |
| FR-05 | The mission must include a mock-provider or equivalent controlled test surface that proves remote acquisition behavior without depending on live external state. | GOAL-01, GOAL-02, GOAL-03 | must | This keeps the proof bar repeatable and protects the repo from flaky network-bound verification. |
| FR-06 | README and operator-facing docs must explain the remote fetch contract, including what remains explicit versus automatic. | GOAL-04 | must | The repo contract requires user-visible behavior and docs to move together. |
<!-- END FUNCTIONAL_REQUIREMENTS -->

### Non-Functional Requirements

<!-- BEGIN NON_FUNCTIONAL_REQUIREMENTS -->
| ID | Requirement | Goals | Priority | Rationale |
|----|-------------|-------|----------|-----------|
| NFR-01 | Deterministic cache identity must remain stable across the library and CLI for the same remote source and revision. | GOAL-02 | must | Remote fetch should not erode the repeatability established by the existing cache contract. |
| NFR-02 | Network side effects must remain explicit enough that operators can distinguish local reuse from remote transfer or refresh. | GOAL-02, GOAL-03 | must | Hidden network mutation would violate the repo's operational guidance. |
| NFR-03 | Story closure must include automated proof and mock-provider evidence for the new remote acquisition flows. | GOAL-01, GOAL-02, GOAL-03 | must | The mission introduces network behavior and needs stronger proof than chat-only claims. |
| NFR-04 | The library remains the source of truth for remote acquisition policy and the CLI remains a thin rendering layer. | GOAL-01, GOAL-02, GOAL-03, GOAL-04 | must | The architecture contract should not regress while adding transport behavior. |
<!-- END NON_FUNCTIONAL_REQUIREMENTS -->

## Verification Strategy

| Area | Method | Evidence |
|------|--------|----------|
| Remote fetch and cache materialization | Unit or integration tests against a mock provider plus CLI proof | Story-level evidence showing a remote source fetched into managed cache on demand |
| Cache reuse and refresh semantics | Automated tests plus controlled CLI runs | Proof showing distinct fetched, reused, and refreshed outcomes for representative remote inputs |
| Failure and recovery behavior | Negative tests and CLI proof | Evidence for auth failure, missing revision, interrupted transfer, or invalid remote layout recovery messaging |
| Docs alignment | Review plus command evidence | README and foundational docs updated in the same change set as the new behavior |

## Assumptions

| Assumption | Impact if Wrong | Validation |
|------------|-----------------|------------|
| A first remote-fetch slice can focus on representative Hugging Face layouts without solving every possible repository structure. | The epic could sprawl into a generic remote artifact system too early. | Keep voyage scope tied to the primary `gguf`-oriented operator path and revision-aware cache semantics. |
| A mock provider can model the required remote behaviors well enough to prove acquisition, reuse, refresh, and recovery flows. | Verification may become flaky or dependent on live external services. | Require controlled provider-backed tests during voyage planning. |
| Explicit refresh can be exposed without introducing ambiguous background sync semantics. | Cache state could become harder to reason about than the current manual model. | Keep refresh opt-in and operator-visible rather than implicit. |

## Open Questions & Risks

| Question/Risk | Owner | Status |
|---------------|-------|--------|
| Which remote artifact subset should the first fetcher support so remote conversion works without overcommitting to every Hugging Face repository layout? | Epic owner | Open |
| What metadata should be stored alongside a fetched artifact to prove revision, freshness, and recovery state without destabilizing cache keys? | Epic owner | Open |
| How should interrupted downloads be recovered or resumed without making partial cache state look reusable? | Epic owner | Open |
| Which refresh semantics are minimal but sufficient for operators who need to update a cached remote source deliberately? | Epic owner | Open |

## Success Criteria

<!-- BEGIN SUCCESS_CRITERIA -->
- [ ] An operator can fetch a representative `hf://repo[@revision]` source on demand without manual cache seeding.
- [ ] Remote acquisition reports make fetched, reused, and refreshed outcomes explicit and deterministic.
- [ ] Remote fetch failures provide actionable recovery guidance rather than generic cache-miss output.
- [ ] Mock-provider proof exists for the primary remote acquisition flows.
- [ ] README and foundational docs describe the shipped remote fetch behavior truthfully.
<!-- END SUCCESS_CRITERIA -->
