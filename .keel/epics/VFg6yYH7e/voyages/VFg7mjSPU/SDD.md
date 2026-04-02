# Fetch Remote Sources Into Managed Cache - Software Design Description

> Download representative `hf://` sources into deterministic managed cache entries with revision-aware metadata and controlled proof surfaces.

**SRS:** [SRS.md](SRS.md)

## Overview

This voyage introduces the remote acquisition substrate without yet widening every workflow:

- add a provider seam that can fetch representative Hugging Face artifacts
- materialize those artifacts into the existing managed cache structure
- persist explicit metadata about what was fetched and from which revision
- prevent partial remote state from looking reusable
- back the behavior with a mock-provider proof surface

## Context & Boundaries

The voyage is transport-oriented. It should make later workflow integration cheaper without yet redefining every command surface.

```
┌───────────────────────────────────────────────────────────────┐
│                          This Voyage                          │
│                                                               │
│  hf:// source -> provider seam -> cache materialization       │
│                      -> manifest/error state                  │
└───────────────────────────────────────────────────────────────┘
           ↑                                           ↑
   controlled mock proof                    later acquisition workflows
```

## Dependencies

| Dependency | Type | Purpose | Version/API |
|------------|------|---------|-------------|
| `crates/metamorph/src/source.rs` | module | Preserve `Source` parsing and revision semantics for `hf://` inputs | local module |
| `crates/metamorph/src/cache.rs` | module | Reuse deterministic cache identity and managed storage layout | local module |
| Mock provider harness | test seam | Prove remote acquisition without live network dependence | local test support |

## Key Decisions

| Decision | Choice | Rationale |
|----------|--------|-----------|
| Fetch seam | Introduce a provider abstraction inside the library rather than embedding HTTP behavior directly in CLI code. | Keeps transport behavior reusable and testable. |
| Materialization contract | Treat fetched remote artifacts as managed cache entries plus explicit manifest metadata. | Preserves deterministic cache semantics and keeps later refresh decisions grounded in stored state. |
| Partial state handling | Keep incomplete downloads or malformed layouts out of the reusable cache path. | Prevents later acquisition or conversion from mistaking broken state for a cache hit. |
| Verification strategy | Prove the fetch substrate through a mock provider. | This mission needs repeatable evidence without live network flakiness. |

## Architecture

- Provider layer: resolves a representative Hugging Face source into a controlled list of remote artifact records and download streams
- Cache materializer: writes artifacts and metadata into the deterministic cache location derived from the source identity
- Manifest state: records the revision and the fetched artifact set needed to explain reuse or recovery later
- Error mapping: translates provider failures into structured remote-acquisition errors

## Components

- Remote source resolver
  - purpose: turn `hf://repo[@revision]` into an actionable remote acquisition request
  - behavior: preserve revision-aware semantics and selected artifact layout
- Remote fetch provider
  - purpose: list and download representative remote artifacts
  - behavior: pluggable enough for a mock implementation in tests
- Cache materializer
  - purpose: stage remote artifacts into deterministic managed cache state
  - behavior: atomically or defensively handle incomplete transfer cases
- Manifest recorder
  - purpose: persist explicit source and fetched-artifact metadata for reuse or refresh decisions

## Interfaces

- Library-facing:
  - remote fetch entry point or helper invoked by acquisition code
  - structured fetch result containing fetched paths, manifest state, and remote metadata
- Test-facing:
  - mock provider surface capable of success, auth failure, missing revision, and malformed-layout cases

## Data Flow

1. A `Source::HuggingFace` value enters the remote acquisition seam.
2. The provider resolves the requested revision and artifact set.
3. Artifacts are staged into deterministic managed cache storage.
4. Manifest metadata is written only when the materialized state is coherent.
5. Structured success or failure data returns to the acquisition caller.

## Error Handling

| Error Condition | Detection | Response | Recovery |
|-----------------|-----------|----------|----------|
| Missing credentials or auth rejection | Provider returns an auth failure | Emit a structured auth error with credential guidance | Operator provides the required token or chooses a different source |
| Missing revision or repo entry | Provider cannot resolve the requested revision or artifact | Emit a structured remote-not-found error | Operator corrects the repo or revision |
| Malformed remote layout | Required representative artifact set cannot be derived | Refuse cache materialization and report layout mismatch | Operator chooses a supported remote layout |
| Interrupted or partial materialization | Transfer or staging stops before a coherent cache entry exists | Clean or quarantine incomplete state and return a fetch failure | Operator retries the fetch explicitly |
