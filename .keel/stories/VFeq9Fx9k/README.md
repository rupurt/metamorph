---
# system-managed
id: VFeq9Fx9k
status: done
created_at: 2026-04-02T10:59:37
updated_at: 2026-04-02T11:18:59
# authored
title: Add Structured Compatibility Reports
type: feat
operator-signal:
scope: VFepvp0Xe/VFepyCJ2Q
index: 1
started_at: 2026-04-02T11:18:57
completed_at: 2026-04-02T11:18:59
---

# Add Structured Compatibility Reports

## Summary

Add the library-facing report surface that explains whether a requested path is supported, lossy, or blocked and why.

## Acceptance Criteria

- [x] [SRS-01/AC-01] The library exposes a structured compatibility report for requested source or target pairs, including inferred source format, support status, lossy status, and blockers or caveats. <!-- verify: cargo test --workspace, SRS-01:start:end, proof: ac-1.log-->
- [x] [SRS-NFR-01/AC-01] Compatibility data is derived from the same capability registry used by planning and execution. <!-- verify: cargo test --workspace, SRS-NFR-01:start:end, proof: ac-2.log-->
