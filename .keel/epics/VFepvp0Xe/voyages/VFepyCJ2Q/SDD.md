# Surface Compatibility Reports And Extension Guidance - Software Design Description

> Expose structured compatibility reporting for supported, unsupported, and lossy paths and align docs to the new extension contract.

**SRS:** [SRS.md](SRS.md)

## Overview

This voyage makes the new extensibility contract consumable:

- add a structured library report for compatibility and conversion constraints
- render that reasoning through the CLI instead of relying on opaque failures
- align docs and board artifacts with the actual module and backend surface
- keep unsupported requests actionable for operators and embeddings

## Context & Boundaries

The voyage does not add more backends. It exposes and documents the truth about the backends and capabilities delivered by the first two voyages.

```
┌─────────────────────────────────────────────────────────────┐
│                        This Voyage                         │
│                                                             │
│  source + target -> capability lookup -> compatibility      │
│                         report -> CLI/docs                  │
└─────────────────────────────────────────────────────────────┘
           ↑                                      ↑
   embedding applications                   operator workflow
```

## Dependencies

| Dependency | Type | Purpose | Version/API |
|------------|------|---------|-------------|
| Capability registry from voyage two | internal | Supplies shared support and lossy truth for reports and CLI output | local workspace |
| Existing planning surface | internal | Anchors compatibility reasoning to real convert requests | local workspace |
| Repo foundational docs | docs | Carry the operator and architecture contract for the new extension model | repository docs |

## Key Decisions

| Decision | Choice | Rationale |
|----------|--------|-----------|
| Shared truth source | Derive compatibility reporting from the same registry that planning and execution use. | Prevents explanation drift between the report surface and actual backend support. |
| CLI exposure | Extend planning-oriented CLI output instead of inventing a separate heavyweight UI. | Keeps the CLI thin and scriptable while still making reasoning visible. |
| Doc honesty | Update README, USER_GUIDE, ARCHITECTURE, and CODE_WALKTHROUGH in the same change as compatibility surfacing. | The mission explicitly rejects architecture and support drift. |
| Recovery style | Prefer actionable blockers and next steps over generic unsupported-path errors. | Integrators need operationally useful explanations, not raw rejection text alone. |

## Architecture

The compatibility layer should sit logically above capability lookup:

- source and target inputs enter the planning layer
- the capability registry determines support status, lossy status, and backend availability
- a structured compatibility report packages that truth for library consumers
- the CLI renders the same report for operators
- docs mirror the same support matrix and caveats

This keeps explanation and execution coupled to the same underlying data while leaving the CLI as a renderer rather than a rule engine.

## Components

- Compatibility report component: typed library report for support status, lossy status, and blockers
- CLI rendering component: formats compatibility reasoning for supported and unsupported requests
- Documentation component: updates the product, architecture, and walkthrough stories to match the delivered extension surface
- Recovery guidance component: maps unsupported or blocked requests to actionable next steps

## Interfaces

- Library:
  - a compatibility or assessment API derived from the same capability data the planner uses
- CLI:
  - planning-oriented output that can show supported, lossy, and blocked requests explicitly
- Docs:
  - updated architecture and user guidance that list only the currently delivered paths and seams

## Data Flow

1. The caller supplies a source and target request.
2. The planning layer consults the shared capability registry.
3. A structured compatibility report is assembled with support status, lossy state, blockers, and caveats.
4. The library returns the report to embeddings, and the CLI renders the same reasoning for operators.
5. Docs and board artifacts are updated to match the delivered support contract.

## Error Handling

| Error Condition | Detection | Response | Recovery |
|-----------------|-----------|----------|----------|
| Compatibility report diverges from planning truth | Tests compare report results with planner outcomes | Collapse the report back onto shared registry-driven logic | Keep the registry as the single source of truth |
| CLI output hides why a request is blocked | CLI proof shows only generic errors for unsupported or lossy requests | Enrich rendering with blockers and next-step guidance | Re-run planned proof until the output is actionable |
| Docs overstate support or future plugin breadth | Review finds claims beyond delivered backends | Correct the docs in the same patch | Prefer explicit planned-language over aspirational capability claims |
