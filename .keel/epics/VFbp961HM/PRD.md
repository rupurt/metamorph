# First Candle Conversion Path - Product Requirements

## Problem Statement

Metamorph needs a concrete vertical slice that turns README promises into a usable model conversion workflow for Candle-oriented runtimes.

## Goals & Objectives

| ID | Goal | Success Metric | Target |
|----|------|----------------|--------|
| GOAL-01 | A Rust application or operator can inspect a source model and understand what format Metamorph believes it is handling. | Representative local and Hugging Face inputs return explicit format reports through the library and CLI. | Inspection proof exists for at least one local path and one `hf://...` source. |
| GOAL-02 | An operator can run the first Candle-oriented `gguf -> hf-safetensors` path with explicit lossy acknowledgment. | The CLI can plan and execute the path without silent network or conversion-side effects. | One end-to-end CLI flow works against a representative fixture or controlled source. |
| GOAL-03 | A downstream runtime can trust the resulting bundle shape. | Validation rejects malformed output and accepts a valid Candle-style bundle. | Validation proof exists for both failure and success cases. |

## Users

| Persona | Description | Primary Need |
|---------|-------------|--------------|
| Runtime Integrator | A Rust application developer embedding Metamorph. | A reusable library surface that can inspect, plan, convert, and validate model artifacts. |
| Local AI Operator | A developer or infra engineer using the CLI directly. | A dependable command path to turn an upstream artifact into a Candle-loadable bundle. |

## Scope

### In Scope

- [SCOPE-01] Source inspection for local paths and Hugging Face-style references.
- [SCOPE-02] A first `gguf -> hf-safetensors` conversion backend aimed at Candle-style bundle output.
- [SCOPE-03] Validation of the expected Candle-friendly output layout and required metadata files.
- [SCOPE-04] Tests and CLI proof that make the path trustworthy.

### Out of Scope

- [SCOPE-05] Additional runtime targets such as MLX-native or llama.cpp-native output flows.
- [SCOPE-06] Generalized plugin or backend systems beyond what the first path needs.
- [SCOPE-07] Automatic public publishing or redistribution workflows.

## Requirements

### Functional Requirements

<!-- BEGIN FUNCTIONAL_REQUIREMENTS -->
| ID | Requirement | Goals | Priority | Rationale |
|----|-------------|-------|----------|-----------|
| FR-01 | The library and CLI must inspect local and `hf://` sources and return an explicit detected format or an explicit inability to infer one. | GOAL-01 | must | Inspection is the first operator decision point and drives every later step. |
| FR-02 | The planner must reject unsupported conversions and require explicit opt-in for lossy `gguf -> hf-safetensors` behavior. | GOAL-02 | must | The product promise depends on truthful conversion semantics. |
| FR-03 | The first execution backend must materialize a Candle-friendly Hugging Face-style bundle shape for the supported path. | GOAL-02, GOAL-03 | must | The mission exists to prove a real downstream-consumable workflow. |
| FR-04 | Validation must verify presence and structure of the expected bundle artifacts such as `config.json`, `tokenizer.json`, `generation_config.json`, and safetensors output. | GOAL-03 | must | A written bundle is not enough if the runtime cannot consume it. |
<!-- END FUNCTIONAL_REQUIREMENTS -->

### Non-Functional Requirements

<!-- BEGIN NON_FUNCTIONAL_REQUIREMENTS -->
| ID | Requirement | Goals | Priority | Rationale |
|----|-------------|-------|----------|-----------|
| NFR-01 | The CLI and library must present the same conversion truth, especially around lossy behavior and unsupported paths. | GOAL-01, GOAL-02 | must | Prevents product drift between embedded and operational surfaces. |
| NFR-02 | The first path should favor deterministic outputs and explicit file layout over premature optimization. | GOAL-02, GOAL-03 | should | Stable bundles make validation, caching, and downstream use tractable. |
| NFR-03 | Tests and command-level proof must exist for the shipped path. | GOAL-01, GOAL-02, GOAL-03 | must | This repo’s proof bar requires executable evidence. |
<!-- END NON_FUNCTIONAL_REQUIREMENTS -->

## Verification Strategy

| Area | Method | Evidence |
|------|--------|----------|
| Inspection | Unit tests plus CLI command proof | `cargo test` coverage for detection logic and captured `metamorph inspect` runs |
| Conversion planning and execution | Unit tests plus CLI proof for planning and execution | Tests for lossy gating and one reproducible CLI path |
| Validation | Positive and negative validation tests plus CLI proof | Tests showing malformed bundles fail and valid bundles pass |

## Assumptions

| Assumption | Impact if Wrong | Validation |
|------------|-----------------|------------|
| A Candle-oriented bundle can be represented as a Hugging Face-style layout for the first path. | The first vertical slice may target the wrong output contract. | Validate against the README and downstream runtime expectations during voyage planning. |
| A representative GGUF fixture can be used for tests or controlled execution without requiring large external downloads in every proof path. | The epic may become too heavyweight for repeatable local verification. | Decide fixture strategy during the first voyage. |

## Open Questions & Risks

| Question/Risk | Owner | Status |
|---------------|-------|--------|
| What is the minimal fixture or artifact strategy for proving the first backend without making CI or local tests brittle? | Epic owner | Open |
| Which metadata transformations are required to produce a truly Candle-loadable bundle rather than a merely well-shaped directory? | Epic owner | Open |

## Success Criteria

<!-- BEGIN SUCCESS_CRITERIA -->
- [ ] A local and remote source can be inspected with clear format reporting.
- [ ] The first Candle-oriented `gguf -> hf-safetensors` path can be planned and executed with explicit lossy opt-in.
- [ ] Validation can prove the resulting bundle shape is acceptable for the intended runtime contract.
- [ ] README, foundational docs, and shipped behavior still agree after delivery.
<!-- END SUCCESS_CRITERIA -->
