---
# system-managed
id: VFloln5yM
status: done
created_at: 2026-04-03T15:38:03
updated_at: 2026-04-03T15:56:34
# authored
title: Keep New Conversion Outputs Validation-Backed
type: feat
operator-signal:
scope: VFg70aqT7/VFlohna1u
index: 4
started_at: 2026-04-03T15:43:48
completed_at: 2026-04-03T15:56:34
---

# Keep New Conversion Outputs Validation-Backed

## Summary

Keep the new conversion paths honest by validating outputs before success and by ensuring failed attempts do not silently claim reusable results.

## Acceptance Criteria

- [x] [SRS-NFR-01/AC-01] Newly promoted relayout and bundle-materialization paths validate their targets before returning success. <!-- verify: cargo test --workspace, SRS-NFR-01:start:end, proof: ac-1.log-->
- [x] [SRS-NFR-02/AC-01] Representative failure cases show invalid outputs are rejected instead of being marked reusable. <!-- verify: cargo test --workspace, SRS-NFR-02:start:end, proof: ac-2.log-->
