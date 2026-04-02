---
# system-managed
id: VFg8FN0q1
status: backlog
created_at: 2026-04-02T16:17:47
updated_at: 2026-04-02T16:18:57
# authored
title: Materialize Remote GGUF Artifacts Into Deterministic Cache Entries
type: feat
operator-signal:
scope: VFg6yYH7e/VFg7mjSPU
index: 2
---

# Materialize Remote GGUF Artifacts Into Deterministic Cache Entries

## Summary

Turn the provider results into deterministic managed cache entries for representative remote GGUF sources, including revision-aware manifest state and defensive handling for incomplete materialization.

## Acceptance Criteria

- [ ] [SRS-02/AC-01] A representative remote GGUF source materializes into the deterministic cache path derived from its source identity, with revision-aware metadata or manifest state persisted alongside the fetched artifact. <!-- verify: cargo test --workspace, SRS-02:start:end, proof: ac-1.log-->
- [ ] [SRS-03/AC-01] Partial, interrupted, or malformed remote materialization is not treated as a reusable cache hit. <!-- verify: cargo test --workspace, SRS-03:start:end, proof: ac-2.log-->
