---
# system-managed
id: VFloljYuK
status: done
created_at: 2026-04-03T15:38:03
updated_at: 2026-04-03T15:56:30
# authored
title: Surface Local-Only And Metadata Blockers In Compatibility
type: feat
operator-signal:
scope: VFg70aqT7/VFlohn41F
index: 2
started_at: 2026-04-03T15:43:48
completed_at: 2026-04-03T15:56:30
---

# Surface Local-Only And Metadata Blockers In Compatibility

## Summary

Add request-specific blockers so the compatibility surface stays truthful when a backend exists but the current request still cannot run because it is remote, targets a remote destination, or lacks required metadata sidecars.

## Acceptance Criteria

- [x] [SRS-02/AC-01] Compatibility reports for promoted local backends surface blockers for remote sources or unsupported targets instead of implying those requests are runnable. <!-- verify: cargo test --workspace, SRS-02:start:end, proof: ac-1.log-->
- [x] [SRS-03/AC-01] Compatibility reports for `safetensors -> hf-safetensors` surface missing metadata sidecars or unsupported source shapes before conversion executes. <!-- verify: cargo test --workspace, SRS-03:start:end, proof: ac-2.log-->
