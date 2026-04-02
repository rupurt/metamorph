---
# system-managed
id: VFeOyo75T
status: backlog
created_at: 2026-04-02T09:11:42
updated_at: 2026-04-02T09:13:14
# authored
title: Define Deterministic Cache Identity
type: feat
operator-signal:
scope: VFeOQzrXV/VFeOTEZi2
index: 1
---

# Define Deterministic Cache Identity

## Summary

Define the first deterministic cache identity contract for representative local and `hf://` sources so later acquisition, reuse, and publish work can build on a stable local naming scheme.

## Acceptance Criteria

- [ ] [SRS-01/AC-01] The library defines which source attributes participate in cache identity, including source kind, detected format, and revision-equivalent metadata when available. <!-- verify: cargo test --workspace, SRS-01:start -->
- [ ] [SRS-NFR-02/AC-01] Repeated planning or acquisition requests for the same representative source resolve to the same cache identity in tests or command proof. <!-- verify: cargo test --workspace, SRS-NFR-02:start:end -->
