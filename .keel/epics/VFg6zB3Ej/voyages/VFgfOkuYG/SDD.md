# Wire Real Upload Execution Into Library And CLI - Software Design Description

> Make `publish()` and `upload --execute` perform explicit remote writes while preserving preview-first behavior and thin CLI orchestration.

**SRS:** [SRS.md](SRS.md)

## Overview

This voyage connects the publish executor substrate to the workflows operators and embedders actually use:

- `publish()` becomes the single execution entry point for preview and remote upload
- preview-first behavior remains the default path
- `upload --execute` becomes the explicit remote-mutation trigger
- the CLI continues to format library reports instead of owning publish business rules

## Context & Boundaries

```
┌───────────────────────────────────────────────────────────────┐
│                          This Voyage                          │
│                                                               │
│  upload / publish -> plan_publish -> preview or execute       │
│                                   -> publish report           │
└───────────────────────────────────────────────────────────────┘
           ↑                                           ↑
   validated local bundle                      publish executor substrate
```

The voyage is workflow integration, not a new provider implementation. It assumes the executor seam from voyage one exists.

## Dependencies

| Dependency | Type | Purpose | Version/API |
|------------|------|---------|-------------|
| `crates/metamorph/src/publish.rs` | module | Own publish planning, guarded execution, and outcome reporting | local module |
| `crates/metamorph-cli/src/main.rs` | binary adapter | Render preview and execution reports without owning publish policy | local module |
| Validation surface | internal workflow | Keep upload gated on reusable local bundles | existing library contract |

## Key Decisions

| Decision | Choice | Rationale |
|----------|--------|-----------|
| Execution truth | Route both CLI upload and embedding workflows through `publish()`. | Preserves one source of truth for preview and execute semantics. |
| Preview preservation | Keep preview as the default behavior and treat `--execute` as the only mutation trigger. | This is the operational contract already promised in the README. |
| Guard model | Refuse unsafe requests before remote mutation begins whenever validation, credentials, or destination preflight fails. | Operators need a deliberate upload path, not best-effort mutation. |
| CLI role | Keep the CLI focused on rendering publish plans and reports. | Avoids duplicating upload policy in command handlers. |

## Architecture

- Publish entry point
  - validates the request, builds or reuses a publish plan, and chooses preview or execute mode
- Execution preflight
  - enforces validation, explicit execute, credentials, and supported destination checks
- CLI upload surface
  - prints the publish plan and execution report returned by the library

## Components

- `publish()` integration
  - purpose: centralize preview and execution behavior
  - behavior: return preview notes by default and execute only under explicit guarded conditions
- CLI renderer
  - purpose: surface artifact lists, execution state, and notes to operators
  - behavior: reuse report fields instead of re-deriving outcome semantics
- Execution guard layer
  - purpose: stop invalid or unsafe requests before mutation starts
  - behavior: preserve preview-first and validated-bundle semantics

## Interfaces

- Library:
  - `plan_publish(input, repo) -> PublishPlan`
  - `publish(request) -> PublishReport`
- CLI:
  - `metamorph upload --input <PATH> --repo <OWNER/NAME>`
  - `metamorph upload --input <PATH> --repo <OWNER/NAME> --execute`

## Data Flow

1. An operator or embedding provides a validated local bundle path and a Hugging Face destination.
2. `plan_publish()` derives the artifact set and explicit plan.
3. If execute is false, `publish()` returns a preview report without remote mutation.
4. If execute is true, guarded preflight runs before the executor uploads artifacts.
5. The resulting publish report returns to the CLI or embedding for rendering.

## Error Handling

| Error Condition | Detection | Response | Recovery |
|-----------------|-----------|----------|----------|
| Bundle fails validation or publish preconditions | Planning or validation fails before execution | Refuse execution and return the validation or planning error | Repair the local bundle, then retry |
| Credentials or destination preflight fails | Guard layer rejects the request before upload | Return guarded refusal without remote mutation | Set credentials or correct the destination, then retry |
| CLI and library drift on publish semantics | Tests show different preview or outcome truth | Move shared logic back into the library and keep CLI thin | Re-run upload tests before widening scope |
| Real execution breaks existing preview behavior | Preview tests regress | Stop and repair the integration before shipping | Preserve preview-first behavior as a hard constraint |
