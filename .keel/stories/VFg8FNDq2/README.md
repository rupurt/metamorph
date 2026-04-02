---
# system-managed
id: VFg8FNDq2
status: backlog
created_at: 2026-04-02T16:17:47
updated_at: 2026-04-02T16:19:16
# authored
title: Fetch Remote Sources On Demand Through Source Acquisition
type: feat
operator-signal:
scope: VFg6yYH7e/VFg7nTfTq
index: 1
---

# Fetch Remote Sources On Demand Through Source Acquisition

## Summary

Wire the remote fetch substrate into `acquire_source()` so a representative `hf://` input can be fetched on cache miss and reused later through one shared acquisition contract.

## Acceptance Criteria

- [ ] [SRS-01/AC-01] `acquire_source()` fetches a representative remote source on cache miss and reports a fetched or reused outcome together with the resolved local path. <!-- verify: cargo test --workspace, SRS-01:start:end, proof: ac-1.log-->
- [ ] [SRS-NFR-01/AC-01] Remote acquisition outcomes and resolved-path reporting stay aligned between library-facing acquisition results and later CLI rendering. <!-- verify: cargo test --workspace, SRS-NFR-01:start:end, proof: ac-2.log-->
- [ ] [SRS-NFR-02/AC-01] Existing local acquisition behavior remains intact while remote fetch is integrated. <!-- verify: cargo test --workspace, SRS-NFR-02:start, proof: ac-3.log-->
