---
# system-managed
id: VFlolixv3
status: done
created_at: 2026-04-03T15:38:03
updated_at: 2026-04-03T15:56:29
# authored
title: Reclassify Same-Format Relayout Into Explicit Capabilities
type: feat
operator-signal:
scope: VFg70aqT7/VFlohn41F
index: 1
started_at: 2026-04-03T15:43:48
completed_at: 2026-04-03T15:56:29
---

# Reclassify Same-Format Relayout Into Explicit Capabilities

## Summary

Replace the generic `same-format-relayout` placeholder with explicit capability entries for the format pairs that now have a real execution or bundle contract, and stop advertising blanket same-format compatibility where no truthful backend story exists yet.

## Acceptance Criteria

- [x] [SRS-01/AC-01] `find_capability()` exposes explicit relayout or bundle-materialization entries for the promoted format pairs and no longer relies on one generic `from == to` placeholder. <!-- verify: cargo test --workspace, SRS-01:start:end, proof: ac-1.log-->
- [x] [SRS-NFR-01/AC-01] Compatibility, planning, and conversion dispatch continue to resolve through shared registry metadata instead of forked per-surface tables. <!-- verify: cargo test --workspace, SRS-NFR-01:start:end, proof: ac-2.log-->
