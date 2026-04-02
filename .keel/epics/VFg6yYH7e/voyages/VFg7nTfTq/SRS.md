# Wire Remote Fetch Into Cache And Conversion Flows - SRS

## Summary

Epic: VFg6yYH7e
Goal: Make `cache source`, `acquire_source`, and `convert` fetch remote sources on demand while preserving explicit acquisition outcomes and thin CLI behavior.

This voyage makes the new fetch substrate user-visible and operational:

- integrate remote fetch into the main source-acquisition workflow
- let `cache source` show fetched versus reused outcomes instead of only misses
- let remote conversion execution succeed after fetch instead of requiring manual cache seeding
- keep the CLI a thin renderer over the library's acquisition policy

## Scope

### In Scope

- [SCOPE-01] Integrating on-demand remote fetch into `acquire_source()` for representative `hf://` inputs
- [SCOPE-02] Updating `cache source` to surface fetched or reused remote outcomes and the resolved local path
- [SCOPE-03] Updating remote `convert` flows so a cache miss triggers fetch rather than immediate failure for supported remote sources
- [SCOPE-04] Preserving the same acquisition truth across the library and CLI

### Out of Scope

- [SCOPE-05] Explicit refresh controls
- [SCOPE-06] Remote publish execution
- [SCOPE-07] Broader remote source families beyond the representative Hugging Face path for current GGUF-oriented execution

## Functional Requirements

<!-- BEGIN FUNCTIONAL_REQUIREMENTS -->
| ID | Requirement | Scope | Source | Verification |
|----|-------------|-------|--------|--------------|
| SRS-01 | `acquire_source()` must fetch a representative remote source on cache miss and return an explicit fetched or reused outcome with the resolved local path. | SCOPE-01 | FR-02 | automated |
| SRS-02 | `metamorph cache source hf://...` must surface the same remote fetch, reuse, and resolved-path truth that the library reports. | SCOPE-02, SCOPE-04 | FR-03 | automated |
| SRS-03 | Supported remote conversion execution must fetch the source on demand rather than failing until the cache is manually pre-populated. | SCOPE-03 | FR-01 | automated |
| SRS-04 | Remote acquisition behavior must remain library-owned so CLI orchestration does not duplicate fetch or cache policy. | SCOPE-04 | FR-03 | automated |
<!-- END FUNCTIONAL_REQUIREMENTS -->

## Non-Functional Requirements

<!-- BEGIN NON_FUNCTIONAL_REQUIREMENTS -->
| ID | Requirement | Scope | Source | Verification |
|----|-------------|-------|--------|--------------|
| SRS-NFR-01 | Local and remote acquisition flows must remain behaviorally aligned across the library and CLI, including outcome naming and resolved-path reporting. | SCOPE-01, SCOPE-02, SCOPE-03, SCOPE-04 | NFR-01 | automated |
| SRS-NFR-02 | Existing local execution behavior must remain intact while remote fetch is integrated. | SCOPE-01, SCOPE-03 | NFR-04 | automated |
| SRS-NFR-03 | Network side effects must remain legible in command output by distinguishing fetch from cache reuse. | SCOPE-01, SCOPE-02, SCOPE-03 | NFR-02 | automated |
<!-- END NON_FUNCTIONAL_REQUIREMENTS -->
