# Fetch Remote Sources Into Managed Cache - SRS

## Summary

Epic: VFg6yYH7e
Goal: Download representative `hf://` sources into deterministic managed cache entries with revision-aware metadata and controlled proof surfaces.

This voyage creates the first real remote acquisition seam for the mission:

- define how representative Hugging Face sources map into deterministic managed cache paths
- fetch remote artifacts into managed storage instead of reporting cache-miss-only outcomes
- record revision-aware metadata so reuse and later refresh decisions have explicit state
- prove the fetcher through a controlled provider surface rather than live network dependencies

## Scope

### In Scope

- [SCOPE-01] A remote fetch client or provider seam for representative `hf://repo[@revision]` inputs
- [SCOPE-02] Deterministic cache materialization for the primary remote artifact layout needed by Metamorph's current GGUF-oriented path
- [SCOPE-03] Revision-aware cache metadata or manifest state that preserves source identity and makes later reuse or refresh decisions explicit
- [SCOPE-04] Controlled proof surfaces such as a mock provider or equivalent integration harness for remote fetch behavior
- [SCOPE-05] Structured remote acquisition errors for auth, missing revision, malformed remote layout, or incomplete materialization

### Out of Scope

- [SCOPE-06] Wiring remote fetch into every CLI or conversion workflow surface
- [SCOPE-07] Explicit refresh controls
- [SCOPE-08] Remote publish execution
- [SCOPE-09] Support for non-Hugging-Face registries or arbitrary remote storage backends

## Functional Requirements

<!-- BEGIN FUNCTIONAL_REQUIREMENTS -->
| ID | Requirement | Scope | Source | Verification |
|----|-------------|-------|--------|--------------|
| SRS-01 | The library must fetch representative `hf://repo[@revision]` sources into deterministic managed cache paths instead of returning cache-miss-only behavior for every remote input. | SCOPE-01, SCOPE-02 | FR-01 | automated |
| SRS-02 | Remote materialization must persist revision-aware metadata or equivalent manifest state alongside the fetched artifact so cache identity and later reuse decisions remain explicit. | SCOPE-02, SCOPE-03 | FR-02 | automated |
| SRS-03 | Partial or malformed remote fetch results must not be treated as reusable cache entries. | SCOPE-02, SCOPE-05 | FR-04 | automated |
| SRS-04 | The remote acquisition seam must be provable through a mock provider or equivalent controlled harness that covers successful fetches and representative failure modes. | SCOPE-04 | FR-05 | automated |
| SRS-05 | Auth failures, missing revisions, and invalid remote layouts must map to structured remote-acquisition errors rather than generic cache-miss messaging. | SCOPE-05 | FR-04 | automated |
<!-- END FUNCTIONAL_REQUIREMENTS -->

## Non-Functional Requirements

<!-- BEGIN NON_FUNCTIONAL_REQUIREMENTS -->
| ID | Requirement | Scope | Source | Verification |
|----|-------------|-------|--------|--------------|
| SRS-NFR-01 | Deterministic cache identity for a representative remote source and revision must remain stable across repeated runs. | SCOPE-02, SCOPE-03 | NFR-01 | automated |
| SRS-NFR-02 | The voyage must keep real network dependencies out of the primary proof surface by relying on a controlled provider harness. | SCOPE-04 | NFR-03 | automated |
| SRS-NFR-03 | Remote fetch behavior must be implemented in the library rather than hidden inside CLI-specific code paths. | SCOPE-01, SCOPE-02, SCOPE-05 | NFR-04 | automated |
<!-- END NON_FUNCTIONAL_REQUIREMENTS -->
