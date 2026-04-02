---
# system-managed
id: VFeq5I2VV
status: backlog
created_at: 2026-04-02T10:59:22
updated_at: 2026-04-02T11:03:27
# authored
title: Move Operational Concerns Into Dedicated Library Modules
type: feat
operator-signal:
scope: VFepvp0Xe/VFepwxnmQ
index: 2
---

# Move Operational Concerns Into Dedicated Library Modules

## Summary

Move the current inspection, planning, cache, validation, and publish logic into the modules that own those concerns so later extension work no longer depends on a monolithic file.

## Acceptance Criteria

- [ ] [SRS-02/AC-01] Inspection, planning, cache or acquisition, validation, and publish logic move out of the monolithic `lib.rs` into the corresponding modules without changing the shipped workflow results. <!-- verify: cargo test --workspace, SRS-02:start, proof: ac-1.log-->
- [ ] [SRS-NFR-01/AC-02] Existing inspect, convert, cache, validate, and upload tests stay green through the module move. <!-- verify: cargo test --workspace, SRS-NFR-01:mid, proof: ac-2.log-->
