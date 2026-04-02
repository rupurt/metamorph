---
# system-managed
id: VFg8FNhqy
status: backlog
created_at: 2026-04-02T16:17:47
updated_at: 2026-04-02T16:19:16
# authored
title: Render Remote Fetch And Reuse Outcomes In Cache Source
type: feat
operator-signal:
scope: VFg6yYH7e/VFg7nTfTq
index: 2
---

# Render Remote Fetch And Reuse Outcomes In Cache Source

## Summary

Update the `cache source` CLI path so operators can see whether a remote source was fetched or reused and which local path subsequent workflow steps will consume.

## Acceptance Criteria

- [ ] [SRS-02/AC-01] `metamorph cache source hf://...` renders the same fetched or reused outcome and resolved-path truth produced by `acquire_source()`. <!-- verify: cargo test --workspace, SRS-02:start:end, proof: ac-1.log-->
- [ ] [SRS-NFR-03/AC-01] Command output distinguishes remote fetch from cache reuse explicitly enough that network side effects stay legible. <!-- verify: cargo test --workspace, SRS-NFR-03:start, proof: ac-3.log-->
