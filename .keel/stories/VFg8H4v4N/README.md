---
# system-managed
id: VFg8H4v4N
status: backlog
created_at: 2026-04-02T16:17:53
updated_at: 2026-04-02T16:19:16
# authored
title: Surface Recovery Guidance For Remote Acquisition Failures
type: feat
operator-signal:
scope: VFg6yYH7e/VFg7oHwbx
index: 2
---

# Surface Recovery Guidance For Remote Acquisition Failures

## Summary

Replace generic remote acquisition failures with recovery guidance that distinguishes credentials, revisions, transfer problems, malformed remote layouts, and stale cached state.

## Acceptance Criteria

- [ ] [SRS-02/AC-01] Remote acquisition failures distinguish credentials, missing revision, interrupted transfer, malformed remote layout, and stale cached state in the operator-facing recovery path. <!-- verify: cargo test --workspace, SRS-02:start:end, proof: ac-1.log-->
- [ ] [SRS-NFR-03/AC-01] The library and CLI use consistent terminology for fetched, reused, refreshed, and failed remote acquisition states. <!-- verify: cargo test --workspace, SRS-NFR-03:start, proof: ac-2.log-->
