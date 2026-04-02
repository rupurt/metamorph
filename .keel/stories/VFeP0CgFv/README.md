---
# system-managed
id: VFeP0CgFv
status: done
created_at: 2026-04-02T09:11:48
updated_at: 2026-04-02T10:38:40
# authored
title: Implement Source Acquisition And Reuse Reporting
type: feat
operator-signal:
scope: VFeOQzrXV/VFeOTEZi2
index: 2
started_at: 2026-04-02T10:33:11
completed_at: 2026-04-02T10:38:40
---

# Implement Source Acquisition And Reuse Reporting

## Summary

Implement the first acquisition or reuse slice so operators can see whether Metamorph reused an existing local artifact, copied a local source into managed storage, or fetched a remote source into the cache.

## Acceptance Criteria

- [x] [SRS-02/AC-01] The library exposes explicit acquisition or reuse outcomes for representative local and `hf://` inputs instead of hiding cache behavior behind a generic success path. <!-- verify: cargo test --workspace, SRS-02:start, proof: ac-1.log-->
- [x] [SRS-02/AC-02] The CLI reports the resulting local path and whether the source was reused or newly materialized. <!-- verify: cargo test --workspace, SRS-02:end, proof: ac-2.log-->
- [x] [SRS-NFR-01/AC-01] Library and CLI surfaces stay aligned on cache hit, miss, and reuse outcomes. <!-- verify: cargo test --workspace, SRS-NFR-01:start, proof: ac-3.log-->
