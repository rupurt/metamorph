# Add Backend Registry And Second Path - SRS

## Summary

Epic: VFepvp0Xe
Goal: Route planning and execution through explicit conversion capability seams, then deliver a second gguf-to-safetensors backend through that contract.

This voyage turns the structural refactor into an extensibility proof:

- describe supported conversion paths in one capability registry instead of scattered planner and executor branches
- dispatch the existing backend through a registered backend seam
- add a second backend, `gguf -> safetensors`, through the same contract
- capture the guardrails that keep extension work bounded, explicit, and policy-safe

## Scope

### In Scope

- [SCOPE-01] A centralized capability registry for supported source-to-target paths, lossy semantics, and execution metadata
- [SCOPE-02] Backend dispatch that routes the existing `gguf -> hf-safetensors` path through an explicit seam
- [SCOPE-03] A second backend, `gguf -> safetensors`, planned, executed, and validated through the same seam
- [SCOPE-04] Tests or design guidance that show adding a backend now touches a bounded implementation surface

### Out of Scope

- [SCOPE-05] Dynamic plugin loading, backend marketplaces, or runtime-loaded modules
- [SCOPE-06] More than one additional backend beyond the explicit `gguf -> safetensors` proof path
- [SCOPE-07] New remote fetch or publish side effects as part of the registry work

## Functional Requirements

<!-- BEGIN FUNCTIONAL_REQUIREMENTS -->
| ID | Requirement | Scope | Source | Verification |
|----|-------------|-------|--------|--------------|
| SRS-01 | Supported conversion paths, lossy semantics, and backend selection metadata must be defined in one capability registry that planning can query. | SCOPE-01 | FR-02 | automated |
| SRS-02 | Execution dispatch must resolve the existing `gguf -> hf-safetensors` path through a registered backend seam rather than open-coded top-level branching. | SCOPE-02 | FR-02 | automated |
| SRS-03 | Metamorph must plan, execute, and validate `gguf -> safetensors` through the same registered backend seam with explicit lossy opt-in. | SCOPE-03 | FR-03 | automated |
| SRS-04 | The extension contract must be explicit enough that adding a backend touches bounded registry and backend modules rather than unrelated CLI or cache code. | SCOPE-04 | FR-02 | automated |
<!-- END FUNCTIONAL_REQUIREMENTS -->

## Non-Functional Requirements

<!-- BEGIN NON_FUNCTIONAL_REQUIREMENTS -->
| ID | Requirement | Scope | Source | Verification |
|----|-------------|-------|--------|--------------|
| SRS-NFR-01 | Capability and backend registration must preserve explicit lossy opt-in behavior across library and CLI surfaces. | SCOPE-01, SCOPE-02, SCOPE-03 | NFR-03 | automated |
| SRS-NFR-02 | The new `gguf -> safetensors` path must be deterministic enough for repeatable local validation and proof capture. | SCOPE-03 | NFR-03 | automated |
| SRS-NFR-03 | The extensibility work must not introduce dynamic plugin loading or implicit network behavior. | SCOPE-01, SCOPE-02, SCOPE-03, SCOPE-04 | NFR-01 | automated |
<!-- END NON_FUNCTIONAL_REQUIREMENTS -->
