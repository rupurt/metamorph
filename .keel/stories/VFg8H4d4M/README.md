---
# system-managed
id: VFg8H4d4M
status: done
created_at: 2026-04-02T16:17:53
updated_at: 2026-04-02T17:01:42
# authored
title: Add Explicit Refresh Control For Remote Sources
type: feat
operator-signal:
scope: VFg6yYH7e/VFg7oHwbx
index: 1
started_at: 2026-04-02T16:59:18
completed_at: 2026-04-02T17:01:42
---

# Add Explicit Refresh Control For Remote Sources

## Summary

Add an explicit refresh control for representative remote sources so operators can deliberately replace cached remote state without deleting cache directories by hand.

## Acceptance Criteria

- [x] [SRS-01/AC-01] The library and CLI expose an explicit refresh control for representative remote sources. <!-- verify: cargo test --workspace, SRS-01:start:end, proof: ac-1.log-->
- [x] [SRS-NFR-01/AC-01] Refresh remains opt-in and operator-visible rather than turning into hidden background mutation. <!-- verify: cargo test --workspace, SRS-NFR-01:start:end, proof: ac-2.log-->
