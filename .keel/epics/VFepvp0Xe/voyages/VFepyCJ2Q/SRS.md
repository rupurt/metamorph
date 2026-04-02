# Surface Compatibility Reports And Extension Guidance - SRS

## Summary

Epic: VFepvp0Xe
Goal: Expose structured compatibility reporting for supported, unsupported, and lossy paths and align docs to the new extension contract.

This voyage makes the extensibility contract observable:

- add a structured compatibility report surface for library consumers
- render that reasoning through the CLI instead of relying on opaque planner failures
- keep README, foundational docs, and planning artifacts synchronized with the new module and backend model
- make blocked or unsupported requests actionable instead of merely rejected

## Scope

### In Scope

- [SCOPE-01] A library compatibility report for requested source and target pairs, including inferred source format, support status, lossy status, and blockers or caveats
- [SCOPE-02] CLI surfaces that render compatibility reasoning for supported, lossy, and unsupported requests
- [SCOPE-03] Documentation updates for the modular architecture, capability registry, and currently supported paths
- [SCOPE-04] Actionable next-step guidance for blocked or unsupported conversion requests

### Out of Scope

- [SCOPE-05] A separate GUI, dashboard, or policy engine for compatibility decisions
- [SCOPE-06] Additional backend delivery beyond what is already scoped in voyage two
- [SCOPE-07] Marketing future plugin systems or support matrices that are not yet implemented

## Functional Requirements

<!-- BEGIN FUNCTIONAL_REQUIREMENTS -->
| ID | Requirement | Scope | Source | Verification |
|----|-------------|-------|--------|--------------|
| SRS-01 | The library must expose a structured compatibility report for requested source and target pairs, including inferred source format, support status, lossy status, and blockers or caveats. | SCOPE-01 | FR-04 | automated |
| SRS-02 | CLI planning surfaces must render compatibility reasoning for supported, lossy, and unsupported requests without forcing operators to infer it from raw error text. | SCOPE-02 | FR-04 | automated |
| SRS-03 | `README.md`, `ARCHITECTURE.md`, `USER_GUIDE.md`, and `CODE_WALKTHROUGH.md` must explain the modular architecture, capability registry, and currently supported paths truthfully. | SCOPE-03 | FR-05 | automated |
| SRS-04 | Unsupported or blocked requests must provide actionable recovery or next-step guidance for operators and downstream integrators. | SCOPE-04 | FR-04 | automated |
<!-- END FUNCTIONAL_REQUIREMENTS -->

## Non-Functional Requirements

<!-- BEGIN NON_FUNCTIONAL_REQUIREMENTS -->
| ID | Requirement | Scope | Source | Verification |
|----|-------------|-------|--------|--------------|
| SRS-NFR-01 | Compatibility reporting must be derived from the same capability data the planner and executor use. | SCOPE-01, SCOPE-02 | NFR-03 | automated |
| SRS-NFR-02 | Docs and board artifacts must not overstate delivered backends or support levels as the extension surface evolves. | SCOPE-03 | NFR-04 | automated |
<!-- END NON_FUNCTIONAL_REQUIREMENTS -->
