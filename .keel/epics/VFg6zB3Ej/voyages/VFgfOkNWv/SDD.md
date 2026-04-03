# Build A Guarded Hugging Face Publish Executor - Software Design Description

> Define the library-owned publish executor seam and controlled remote write substrate for existing Hugging Face repos.

**SRS:** [SRS.md](SRS.md)

## Overview

This voyage introduces the remote publish substrate without yet widening every user-facing surface:

- add a provider seam that can target an existing Hugging Face repository
- turn a validated publish plan into explicit upload actions and results
- preserve complete, partial, and failed outcome truth at the library layer
- back the substrate with a mock-provider proof surface

## Context & Boundaries

The voyage is executor-oriented. It should make later CLI and recovery integration cheaper without solving repo bootstrap, docs, or every publish workflow in the same slice.

```
┌───────────────────────────────────────────────────────────────┐
│                          This Voyage                          │
│                                                               │
│  publish plan -> provider seam -> upload actions/results      │
│                            -> structured outcome              │
└───────────────────────────────────────────────────────────────┘
           ↑                                           ↑
    validated local bundle                      later CLI/recovery flows
```

## Dependencies

| Dependency | Type | Purpose | Version/API |
|------------|------|---------|-------------|
| `crates/metamorph/src/publish.rs` | module | Own publish planning and later execution entry points | local module |
| `crates/metamorph/src/validate.rs` | module | Keep validated local bundle gating in the publish happy path | local module |
| `crates/metamorph/src/error.rs` | module | Add structured publish failure classes | local module |
| Mock publish provider harness | test seam | Prove upload behavior without live remote dependence | local test support |

## Key Decisions

| Decision | Choice | Rationale |
|----------|--------|-----------|
| Provider seam | Introduce a publish provider abstraction inside the library rather than embedding remote write logic in CLI handlers. | Keeps publish behavior reusable and testable. |
| Destination scope | Bound the first slice to explicitly named existing Hugging Face repositories. | Delivers a credible upload path without forcing repo bootstrap or policy automation into the same voyage. |
| Outcome model | Represent per-artifact results plus an overall complete, partial, or failed state. | Operators and embedders need retryable truth, not a boolean success flag. |
| Verification strategy | Prove the substrate through a controlled provider harness. | Remote execution needs repeatable evidence without flaky live services. |

## Architecture

- Publish executor seam
  - accepts a validated publish plan and drives remote actions through a provider
- Provider implementation layer
  - performs destination checks and artifact upload operations for the first supported target
- Outcome recorder
  - aggregates per-artifact upload results into a complete, partial, or failed report
- Error mapper
  - translates provider failures into structured publish errors and retryable state

## Components

- Destination preflight
  - purpose: confirm the destination is supported and targetable before upload
  - behavior: keep the first slice bounded to existing Hugging Face repos
- Publish provider
  - purpose: upload artifacts and report remote write results
  - behavior: pluggable enough for a mock implementation in tests
- Outcome aggregator
  - purpose: capture which artifacts were uploaded, skipped, or left pending
  - behavior: preserve enough detail for later CLI rendering and retry guidance
- Substrate proof harness
  - purpose: model success and representative failure modes
  - behavior: keep proof deterministic and local

## Interfaces

- Library-facing:
  - publish execution helper or equivalent substrate invoked by `publish()`
  - structured execution report containing destination, per-artifact results, and overall status
- Test-facing:
  - mock provider capable of success, missing repo, permission failure, and interrupted upload cases

## Data Flow

1. A validated publish plan enters the publish execution seam.
2. The provider performs destination preflight against the supported Hugging Face target.
3. The executor uploads artifacts through the provider in a deterministic order.
4. Each artifact result is recorded as uploaded, updated, skipped, or failed.
5. The aggregator returns an overall complete, partial, or failed publish report.

## Error Handling

| Error Condition | Detection | Response | Recovery |
|-----------------|-----------|----------|----------|
| Missing destination or unsupported target state | Provider preflight cannot resolve the target repo | Emit a structured destination failure before upload begins | Operator creates or corrects the repo, then retries |
| Permission or auth rejection | Provider rejects the upload request | Return structured permission failure | Operator fixes token scope or destination ownership |
| Interrupted upload after some artifacts succeed | Provider fails mid-stream | Return a partial publish report instead of full success | Operator inspects the uploaded set and retries the remaining artifacts |
| Outcome aggregation loses remote truth | Tests show partial failures collapsing into generic success | Stop and repair the outcome model before CLI integration | Re-run mock-backed substrate proof |
