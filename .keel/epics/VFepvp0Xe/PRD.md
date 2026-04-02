# Modular Conversion Core And Compatibility Contracts - Product Requirements

## Problem Statement

Metamorph needs to evolve beyond a monolithic prototype into a modular conversion core with explicit backend seams and compatibility reporting so new formats and runtime layouts can be added without invasive rewrites.

## Goals & Objectives

| ID | Goal | Success Metric | Target |
|----|------|----------------|--------|
| GOAL-01 | The library grows from a single-file prototype into stable modules for source, format, plan, transform, validate, cache, and publish concerns without making the CLI the source of truth. | The public workflow lives behind explicit library modules and facade re-exports while the CLI remains orchestration-only. | `crates/metamorph/src/lib.rs` becomes a facade and the existing workflow remains provably intact through tests and command evidence. |
| GOAL-02 | Adding a new conversion backend becomes incremental instead of invasive. | Planning and execution dispatch use explicit capability and backend seams, and a second backend lands through them. | A bounded `gguf -> safetensors` path ships through the new registry and dispatch contract. |
| GOAL-03 | Downstream integrators can understand compatibility, unsupported paths, and lossy edges without reverse-engineering internal match arms. | Library and CLI surfaces expose structured compatibility reasoning tied to the actual support matrix. | Supported, unsupported, and lossy requests all produce explicit reports or actionable explanations. |
| GOAL-04 | The repo's docs and board continue to describe the architecture and support surface truthfully as Metamorph becomes more extensible. | README and foundational docs track the modular architecture, extension seam, and delivered paths in the same changes that reshape them. | The extension contract is documented without overstating future plugin or backend breadth. |

## Users

| Persona | Description | Primary Need |
|---------|-------------|--------------|
| Runtime Integrator | A Rust application developer embedding Metamorph instead of carrying bespoke conversion glue. | Stable library seams plus compatibility reporting that can be trusted programmatically. |
| Metamorph Maintainer | A contributor adding a new format, layout, or backend path. | A bounded implementation surface that does not require editing unrelated CLI or cache code. |
| Local AI Operator | A developer or infra engineer driving the CLI directly. | Clear explanations of what conversions are supported, lossy, or blocked before execution starts. |

## Scope

### In Scope

- [SCOPE-01] Extract the library into stable `source`, `format`, `plan`, `transform`, `validate`, `cache`, and `publish` modules with a top-level facade.
- [SCOPE-02] Define a centralized conversion capability and backend registration contract for planning and execution.
- [SCOPE-03] Implement one additional backend, `gguf -> safetensors`, through the new seam with validation proof.
- [SCOPE-04] Expose structured compatibility reporting for supported, unsupported, and lossy paths in library and CLI surfaces.
- [SCOPE-05] Update README and foundational docs whenever the architecture or support contract changes.

### Out of Scope

- [SCOPE-06] Dynamic plugin loading, marketplace-style backend discovery, or runtime-loaded conversion modules.
- [SCOPE-07] Broad multi-format expansion beyond the first additional backend required to prove the seam.
- [SCOPE-08] New remote fetch or publish side effects unrelated to the extensibility contract.

## Requirements

### Functional Requirements

<!-- BEGIN FUNCTIONAL_REQUIREMENTS -->
| ID | Requirement | Goals | Priority | Rationale |
|----|-------------|-------|----------|-----------|
| FR-01 | The library must expose stable module boundaries for source, format, plan, transform, validate, cache, and publish concerns while preserving a top-level facade for the current public workflow. | GOAL-01 | must | The mission exists to turn a prototype file into an extensible library surface without pushing logic into the CLI. |
| FR-02 | Planning and execution must derive supported conversion behavior from an explicit capability and backend registry rather than scattered hard-coded branches. | GOAL-02, GOAL-03 | must | New paths cannot stay incremental if support truth is duplicated across planner and executor match arms. |
| FR-03 | Metamorph must add a second backend, `gguf -> safetensors`, through the new seam and prove that it can be planned, executed, and validated. | GOAL-02 | must | The new seam needs a real delivered backend, not only a refactor promise. |
| FR-04 | The library and CLI must expose structured compatibility reporting that explains supported, unsupported, and lossy requests clearly enough for embedding and operator use. | GOAL-03 | must | Integrators need more than raw error strings to build against an extensible platform. |
| FR-05 | README, foundational docs, and board artifacts must be updated when architecture or support surfaces change. | GOAL-04 | must | Extensibility work creates drift quickly unless docs and planning artifacts move in the same patch. |
<!-- END FUNCTIONAL_REQUIREMENTS -->

### Non-Functional Requirements

<!-- BEGIN NON_FUNCTIONAL_REQUIREMENTS -->
| ID | Requirement | Goals | Priority | Rationale |
|----|-------------|-------|----------|-----------|
| NFR-01 | The CLI must remain orchestration-only and must not become the source of truth for capability, validation, or backend selection rules. | GOAL-01, GOAL-03 | must | The repo contract keeps product logic in the library so embeddings and CLI users receive the same truth. |
| NFR-02 | Existing inspect, convert, cache, validate, and publish workflows must remain behaviorally stable or be migrated explicitly during the modularization work. | GOAL-01 | must | A cleaner architecture is not acceptable if it regresses the working vertical slices already shipped. |
| NFR-03 | New capability and backend seams must preserve explicit lossy semantics and deterministic enough behavior for repeatable proof capture. | GOAL-02, GOAL-03 | must | Extensibility cannot weaken the repo's lossiness and proof contracts. |
| NFR-04 | Story closure must include executable proof and doc updates whenever architectural or user-visible behavior changes. | GOAL-01, GOAL-02, GOAL-03, GOAL-04 | must | Zero-drift delivery depends on evidence-backed closure rather than chat-only claims. |
<!-- END NON_FUNCTIONAL_REQUIREMENTS -->

## Verification Strategy

| Area | Method | Evidence |
|------|--------|----------|
| Module extraction | Workspace tests plus command proof | Passing tests and CLI examples showing the current workflow still works after modules are extracted |
| Backend registry and second path | Unit or integration tests plus CLI proof | Tests showing capability lookup, backend dispatch, and `gguf -> safetensors` execution and validation |
| Compatibility reporting | Unit tests plus CLI proof | Structured report assertions and command output for supported, unsupported, and lossy requests |
| Doc and board truthfulness | Docs review plus command evidence | README and foundational doc updates in the same change as architecture or support-surface changes |

## Assumptions

| Assumption | Impact if Wrong | Validation |
|------------|-----------------|------------|
| The current single-file implementation can be split into modules without requiring a breaking rewrite of the user-facing workflow. | The epic could balloon into a full public API redesign. | Preserve the top-level facade and require tests to stay green during voyage one. |
| `gguf -> safetensors` is a meaningful second backend for proving the new seam without requiring broad new metadata contracts. | The second-path proof could become too vague or too heavyweight. | Keep the backend scoped to a validated safetensors artifact contract during voyage two. |
| Compatibility reasoning can be derived from a shared capability registry rather than a separate policy engine. | Planning and reporting may drift apart. | Tie voyage three requirements directly to the registry introduced in voyage two. |

## Open Questions & Risks

| Question/Risk | Owner | Status |
|---------------|-------|--------|
| How much of the current public Rust surface should remain as stable re-exports versus becoming module-qualified imports over time? | Epic owner | Open |
| Does the `gguf -> safetensors` backend need additional metadata or layout guidance beyond a validated weights artifact to stay useful and honest? | Epic owner | Open |
| Should compatibility reporting be introduced as a new library API, richer planning output, or both? | Epic owner | Open |
| Where should future plugin or dynamic-backend ideas stop so this mission stays bounded? | Epic owner | Open |

## Success Criteria

<!-- BEGIN SUCCESS_CRITERIA -->
- [ ] `crates/metamorph` is organized around stable modules with a top-level facade and a thin CLI.
- [ ] A second backend, `gguf -> safetensors`, lands through explicit capability and backend seams rather than invasive edits.
- [ ] Supported, unsupported, and lossy requests can be explained through structured compatibility reporting.
- [ ] README, foundational docs, and board artifacts still describe the shipped architecture and support surface truthfully.
<!-- END SUCCESS_CRITERIA -->
