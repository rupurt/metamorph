---
# system-managed
id: VFeP62U17
status: done
created_at: 2026-04-02T09:12:10
updated_at: 2026-04-02T10:38:57
# authored
title: Guard Publish Execution With Validation And Intent
type: feat
operator-signal:
scope: VFeOQzrXV/VFeOTDZi1
index: 2
started_at: 2026-04-02T10:33:11
completed_at: 2026-04-02T10:38:57
---

# Guard Publish Execution With Validation And Intent

## Summary

Guard publish execution so Metamorph refuses unsafe uploads, requires validated inputs, and only mutates the destination when the operator makes an explicit execution choice.

## Acceptance Criteria

- [x] [SRS-03/AC-01] Publish execution refuses unvalidated bundles or destinations that do not satisfy the required preflight checks. <!-- verify: cargo test --workspace, SRS-03:start, proof: ac-1.log-->
- [x] [SRS-03/AC-02] The CLI requires explicit operator intent before remote mutation occurs. <!-- verify: cargo test --workspace, SRS-03:end, proof: ac-2.log-->
- [x] [SRS-NFR-01/AC-02] Execution behavior does not hide network-side effects behind implicit defaults. <!-- verify: cargo test --workspace, SRS-NFR-01:end, proof: ac-3.log-->
