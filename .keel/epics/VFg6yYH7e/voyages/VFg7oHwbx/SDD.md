# Harden Refresh Recovery And Documentation - Software Design Description

> Expose explicit refresh control, actionable recovery messaging, mock-provider proof, and aligned docs for remote acquisition behavior.

**SRS:** [SRS.md](SRS.md)

## Overview

This voyage makes the new remote fetch behavior operationally safe and understandable:

- add an explicit refresh control to remote acquisition paths
- classify and render recovery guidance for the main failure cases
- extend the controlled provider harness into end-to-end proof
- align the docs with the new command and library contract

## Context & Boundaries

```
┌───────────────────────────────────────────────────────────────┐
│                          This Voyage                          │
│                                                               │
│  refresh control + recovery messaging + proof + docs          │
└───────────────────────────────────────────────────────────────┘
           ↑                                           ↑
   fetch substrate and integration                operator contract
```

The voyage is intentionally about hardening and clarity, not about broadening provider scope.

## Dependencies

| Dependency | Type | Purpose | Version/API |
|------------|------|---------|-------------|
| Remote fetch substrate | internal workflow | Supplies fetched and reused acquisition behavior to extend with refresh | local voyage output |
| CLI cache and convert surfaces | binary adapter | Need refresh and recovery rendering | local module |
| Foundational docs | repo contract | Must stay aligned with new network behavior | root docs |

## Key Decisions

| Decision | Choice | Rationale |
|----------|--------|-----------|
| Refresh semantics | Make refresh an explicit operator request rather than an automatic freshness policy. | Preserves deterministic and auditable cache behavior. |
| Failure classification | Distinguish the main recovery classes instead of collapsing them into cache-miss or generic I/O errors. | Operators need actionable next steps once real network fetch exists. |
| Proof surface | Extend mock-provider coverage into e2e-style command and library flows. | Remote behavior needs repeatable proof that does not depend on live services. |
| Documentation scope | Update README and foundational docs in the same change as the behavior. | Prevents the operator story from drifting immediately after launch. |

## Architecture

- Refresh control
  - optional explicit input on the acquisition path that bypasses reusable remote cache state
- Recovery classifier
  - maps fetch and cache-state failures into operator-facing notes or errors
- Controlled e2e proof
  - drives fetched, reused, refreshed, and failed flows through the same acquisition seams the product uses
- Documentation alignment
  - updates operator and architecture docs to match the real contract

## Components

- Refresh option surface
  - purpose: let operators deliberately re-fetch a remote source
  - behavior: remain opt-in and visible in outcome reporting
- Recovery messaging layer
  - purpose: turn structured remote acquisition failures into actionable guidance
  - behavior: keep library and CLI terminology aligned
- Provider-backed e2e tests
  - purpose: prove the workflow without live external dependencies
  - behavior: exercise the same paths operators use

## Interfaces

- Library:
  - explicit refresh flag or equivalent acquisition option
  - structured recovery or error surfaces for the primary failure classes
- CLI:
  - refresh-aware `cache source` or related workflow entry point
  - command output that distinguishes fetched, reused, refreshed, and failed paths

## Data Flow

1. An operator or embedding requests remote acquisition.
2. The acquisition path decides whether to reuse or refresh based on explicit intent and cache state.
3. Remote errors are classified into recovery-oriented results.
4. Tests and docs assert the same terminology and workflow semantics.

## Error Handling

| Error Condition | Detection | Response | Recovery |
|-----------------|-----------|----------|----------|
| Refresh request still fails due to auth or revision error | Provider returns classified failure | Surface refresh-specific failure and next action | Fix credentials or source revision, then retry |
| Stale or invalid cached state blocks reuse | Cache metadata or artifact validation fails | Refuse silent reuse and instruct the operator to refresh | Re-run with explicit refresh |
| Docs drift from shipped behavior | Review or tests expose mismatch | Update the docs in the same change | Keep foundational docs part of story closure |
