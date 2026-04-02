# Make Publish And Recovery Flows Explicit - Software Design Description

> Define explicit publish, mirror, and operator recovery surfaces so network-side effects remain deliberate and auditable.

**SRS:** [SRS.md](SRS.md)

## Overview

This voyage defines publish behavior as an explicit layer on top of a validated local bundle:

- the library constructs a publish plan and enforces prerequisites
- the CLI exposes preview or dry-run behavior before any network mutation
- execution stays guarded by explicit intent, destination choice, and human review seams where licensing or redistribution is unclear
- recovery guidance is part of the shipped surface, not an afterthought

## Context & Boundaries

The voyage is intentionally conservative. It plans for an explicit remote-delivery path without turning Metamorph into an automatic sync service.

```
┌─────────────────────────────────────────────────────────────┐
│                        This Voyage                         │
│                                                             │
│  Validated Bundle -> Publish Plan -> Dry Run -> Execute    │
│                                \-> Recovery Guidance        │
└─────────────────────────────────────────────────────────────┘
            ↑                                      ↑
     local validated output              explicit remote destination
```

## Dependencies

| Dependency | Type | Purpose | Version/API |
|------------|------|---------|-------------|
| `clap` | crate | CLI exposure for dry-run and guarded publish flags | workspace dependency |
| `serde` | crate | Structured plan or report serialization where needed | workspace dependency |
| `thiserror` | crate | Publish-specific domain errors and policy stops | workspace dependency |
| destination client | external/API | Remote repository or object-store interaction for the first supported publish target | chosen during implementation |

## Key Decisions

| Decision | Choice | Rationale |
|----------|--------|-----------|
| Publish precondition | Only publish validated local bundles. | Prevents malformed artifacts from leaving the local pipeline. |
| Side-effect control | Require explicit destination plus preview or dry-run semantics before execution. | Keeps remote mutation deliberate and auditable. |
| Human review seam | Stop for human judgment on unresolved licensing, redistribution, or attribution questions. | This is a policy requirement, not an implementation detail. |
| Local safety | Publish failures must not silently mutate or invalidate local cache state. | Recovery depends on retaining a trustworthy local source of truth. |

## Architecture

The voyage extends the library toward a `publish` seam while keeping remote specifics at the edge:

- validation remains the prerequisite gate owned by `crates/metamorph`
- publish planning translates a validated local bundle and destination into an explicit action set
- execution adapters perform remote operations only after the operator chooses to proceed
- the CLI renders the plan, dry-run output, and failure recovery guidance without embedding publish business rules

## Components

- Publish plan component: describes destination, artifact set, and prerequisite checks without performing remote mutation
- Publish executor component: performs the first supported remote action set when explicit intent is present
- Policy gate component: blocks or escalates unresolved legal, attribution, or redistribution questions
- Recovery/reporting component: turns remote or credential failures into actionable next steps for operators

## Interfaces

- Library:
  - validation or bundle-report surface used as a prerequisite for publish planning
  - publish-plan helper that returns explicit steps for a given validated local bundle and destination
  - guarded publish executor that refuses unsafe or implicit requests
- CLI:
  - `metamorph upload --input <bundle> --repo <destination>`
  - preview or dry-run output that shows what would be uploaded
  - explicit execution flow that only mutates the destination when the operator opts in

## Data Flow

1. The operator points Metamorph at a validated local bundle and an explicit destination.
2. The library checks validation state and constructs a publish or mirror plan.
3. The CLI renders a preview or dry-run so the operator can inspect the destination, artifact set, and preconditions.
4. If explicit execution is requested and policy gates pass, the executor performs the remote operations.
5. Success or failure is reported with enough detail to retry safely or stop for human review.

## Error Handling

| Error Condition | Detection | Response | Recovery |
|-----------------|-----------|----------|----------|
| Bundle has not passed validation or required metadata is missing | Validation preflight fails | Refuse to build or execute the publish plan | Repair the local bundle and rerun validation first |
| Credentials or destination access are invalid | Remote preflight or execution fails | Surface destination-specific auth error | Refresh credentials, correct the destination, or stop for human help |
| Publish request would violate policy or unresolved licensing constraints | Policy gate detects missing approval or flagged condition | Block execution and require human review | Resolve legal or operational judgment before retrying |
| Remote write partially fails | Executor returns remote-operation failure | Preserve local bundle state and explain incomplete remote result | Retry idempotently or clean up the remote destination deliberately |
