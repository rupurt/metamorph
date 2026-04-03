---
# system-managed
id: VFlolosTq
status: done
created_at: 2026-04-03T15:38:03
updated_at: 2026-04-03T15:56:37
# authored
title: Refresh README And Foundational Docs For New Backend Truth
type: feat
operator-signal:
scope: VFg70aqT7/VFloho8ze
index: 3
started_at: 2026-04-03T15:43:48
completed_at: 2026-04-03T15:56:37
---

# Refresh README And Foundational Docs For New Backend Truth

## Summary

Refresh the README and foundational docs so the shipped backend matrix is described in integration and CLI terms instead of leaving the old planned-only language in place.

## Acceptance Criteria

- [x] [SRS-04/AC-01] README and foundational docs list the newly executable relayout paths and the local metadata contract for `safetensors -> hf-safetensors`. <!-- verify: cargo test --workspace, SRS-04:start:end, proof: ac-1.log-->
- [x] [SRS-NFR-03/AC-01] The docs remain explicit about local-only execution and blocked or unsupported cases rather than implying broader backend coverage. <!-- verify: cargo test --workspace, SRS-NFR-03:start:end, proof: ac-2.log-->
