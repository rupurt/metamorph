---
# system-managed
id: VFeq70Ojz
status: done
created_at: 2026-04-02T10:59:29
updated_at: 2026-04-02T11:18:46
# authored
title: Define Conversion Capability Registry
type: feat
operator-signal:
scope: VFepvp0Xe/VFepxZZwT
index: 1
started_at: 2026-04-02T11:18:44
completed_at: 2026-04-02T11:18:46
---

# Define Conversion Capability Registry

## Summary

Define the shared capability registry that planning, execution, and later compatibility reporting will use as the single source of truth for supported paths and lossy semantics.

## Acceptance Criteria

- [x] [SRS-01/AC-01] Supported source-to-target paths, lossy status, and required execution metadata are defined in one capability registry the planner can query. <!-- verify: cargo test --workspace, SRS-01:start:end, proof: ac-1.log-->
- [x] [SRS-NFR-01/AC-01] The registry remains the shared source of truth for library and CLI compatibility decisions. <!-- verify: cargo test --workspace, SRS-NFR-01:start, proof: ac-2.log-->
