---
# system-managed
id: VFlolnlzg
status: done
created_at: 2026-04-03T15:38:03
updated_at: 2026-04-03T15:56:35
# authored
title: Render Promoted Backends In Convert CLI Output
type: feat
operator-signal:
scope: VFg70aqT7/VFloho8ze
index: 1
started_at: 2026-04-03T15:43:48
completed_at: 2026-04-03T15:56:35
---

# Render Promoted Backends In Convert CLI Output

## Summary

Render the promoted backends and their blockers through `metamorph convert` so CLI operators see the same matrix truth that integrators get from the library.

## Acceptance Criteria

- [x] [SRS-01/AC-01] `metamorph convert` prints the promoted backend labels, compatibility status, and blockers for the new relayout paths. <!-- verify: cargo test --workspace, SRS-01:start:end, proof: ac-1.log-->
- [x] [SRS-NFR-01/AC-01] CLI rendering continues to consume library-owned compatibility and conversion truth rather than introducing CLI-specific policy. <!-- verify: cargo test --workspace, SRS-NFR-01:start:end, proof: ac-2.log-->
