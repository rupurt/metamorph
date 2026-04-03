# Harden Publish Recovery Proof And Documentation - Software Design Description

> Surface partial-failure recovery, repeatable mock-provider proof, and truthful docs for executable upload behavior.

**SRS:** [SRS.md](SRS.md)

## Overview

This voyage makes executable upload operationally safe and understandable:

- classify and render recovery guidance for the main publish failure classes
- expose partial-publish truth and operator-driven retry information
- extend the controlled provider harness into end-to-end publish proof
- align the docs with the new preview and execute contract

## Context & Boundaries

```
┌───────────────────────────────────────────────────────────────┐
│                          This Voyage                          │
│                                                               │
│  recovery messaging + partial retry truth + proof + docs      │
└───────────────────────────────────────────────────────────────┘
           ↑                                           ↑
   publish execution substrate                    operator contract
```

The voyage is intentionally about hardening and clarity, not about broadening provider scope or automating repair.

## Dependencies

| Dependency | Type | Purpose | Version/API |
|------------|------|---------|-------------|
| Publish execution substrate | internal workflow | Supplies preview and execute behavior to harden with recovery truth | prior voyage output |
| CLI upload surface | binary adapter | Needs recovery and partial-result rendering | local module |
| Foundational docs | repo contract | Must stay aligned with executable upload behavior | root docs |

## Key Decisions

| Decision | Choice | Rationale |
|----------|--------|-----------|
| Failure classification | Distinguish the main publish recovery classes rather than collapsing them into generic upload errors. | Operators need actionable next steps once real remote execution exists. |
| Partial retry model | Expose remaining artifacts and retry guidance instead of promising an automatic sync engine. | Keeps the first slice explicit and bounded. |
| Proof surface | Drive preview, success, guarded refusal, and partial failure through a mock provider. | Remote publish behavior needs repeatable evidence without live services. |
| Documentation scope | Update README and foundational docs in the same change as the behavior. | Prevents the operator story from drifting immediately after launch. |

## Architecture

- Recovery classifier
  - maps provider and guard failures into operator-facing categories
- Partial-result summarizer
  - identifies uploaded versus pending artifacts for retry reporting
- Controlled e2e proof
  - drives the primary publish paths through the same library and CLI seams the product uses
- Documentation alignment
  - updates operator and architecture docs to match the real upload contract

## Components

- Recovery messaging layer
  - purpose: turn structured publish failures into actionable next steps
  - behavior: keep library and CLI terminology aligned
- Partial publish report surface
  - purpose: show what succeeded remotely and what still needs retry
  - behavior: remain explicit and operator-driven
- Provider-backed e2e tests
  - purpose: prove the workflow without live external dependencies
  - behavior: exercise preview, execute, refusal, and partial-failure paths

## Interfaces

- Library:
  - publish report fields for artifact outcomes, notes, and retry-oriented status
  - structured publish errors for the primary failure classes
- CLI:
  - `upload` output that distinguishes preview, success, partial publish, and guarded refusal
  - recovery notes explaining the next safe operator action

## Data Flow

1. A publish request reaches preview or execute mode through the library surface.
2. Guard or provider failures are classified into recovery-oriented results.
3. Partial execution records which artifacts succeeded remotely and which remain pending.
4. CLI and library proof assert the same terminology and retry semantics.
5. Docs are updated to match the implemented publish contract.

## Error Handling

| Error Condition | Detection | Response | Recovery |
|-----------------|-----------|----------|----------|
| Guarded refusal due to credentials or destination state | Preflight rejects the request | Surface explicit refusal and next action | Fix credentials or destination, then rerun intentionally |
| Partial remote publish leaves some artifacts pending | Provider reports mixed upload results | Return partial publish status plus pending artifact set | Operator retries only after reviewing the remaining work |
| Interrupted upload state is ambiguous | Outcome model cannot distinguish complete versus pending artifacts | Treat the result as partial or failed, never full success | Retry with explicit operator action once the remote state is understood |
| Docs drift from shipped behavior | Review or tests expose mismatch | Update docs in the same change as the behavior | Keep foundational docs part of story closure |
