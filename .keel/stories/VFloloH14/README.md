---
# system-managed
id: VFloloH14
status: done
created_at: 2026-04-03T15:38:03
updated_at: 2026-04-03T15:56:36
# authored
title: Add End-To-End Proof For Relayout And Blocked Cases
type: feat
operator-signal:
scope: VFg70aqT7/VFloho8ze
index: 2
started_at: 2026-04-03T15:43:48
completed_at: 2026-04-03T15:56:36
---

# Add End-To-End Proof For Relayout And Blocked Cases

## Summary

Add the end-to-end proof that makes the promoted matrix credible: successful relayouts, successful metadata-backed bundle promotion, and representative blocked or reclassified requests.

## Acceptance Criteria

- [x] [SRS-02/AC-01] Automated proof covers successful local relayout and successful metadata-backed `safetensors -> hf-safetensors`. <!-- verify: cargo test --workspace, SRS-02:start:end, proof: ac-1.log-->
- [x] [SRS-03/AC-01] Automated proof covers representative blocked or reclassified requests such as missing metadata or unsupported same-format pairs. <!-- verify: cargo test --workspace, SRS-03:start:end, proof: ac-2.log-->
