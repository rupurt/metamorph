---
# system-managed
id: VFeP36PdD
status: done
created_at: 2026-04-02T09:11:59
updated_at: 2026-04-02T10:38:48
# authored
title: Document Cache And Validation Recovery
type: feat
operator-signal:
scope: VFeOQzrXV/VFeOTEZi2
index: 4
started_at: 2026-04-02T10:33:11
completed_at: 2026-04-02T10:38:48
---

# Document Cache And Validation Recovery

## Summary

Document the operator recovery path for cache and validation failures so the CLI, README, and foundational docs explain what to check next when local reuse fails.

## Acceptance Criteria

- [x] [SRS-04/AC-01] Cache failures describe the likely cause and an actionable next step instead of surfacing only low-level errors. <!-- verify: cargo test --workspace, SRS-04:start, proof: ac-1.log-->
- [x] [SRS-04/AC-02] Validation failures identify the missing or malformed bundle elements and direct the operator toward rerun or repair steps. <!-- verify: cargo test --workspace, SRS-04:end, proof: ac-2.log-->
