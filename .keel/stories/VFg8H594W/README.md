---
# system-managed
id: VFg8H594W
status: backlog
created_at: 2026-04-02T16:17:53
updated_at: 2026-04-02T16:19:16
# authored
title: Refresh README And Foundational Docs For Remote Fetch
type: feat
operator-signal:
scope: VFg6yYH7e/VFg7oHwbx
index: 3
---

# Refresh README And Foundational Docs For Remote Fetch

## Summary

Bring the README and foundational docs up to date with the shipped remote acquisition contract so operators and integrators can understand fetch, reuse, refresh, and recovery behavior without reverse-engineering the code.

## Acceptance Criteria

- [ ] [SRS-04/AC-01] `README.md` and `USER_GUIDE.md` describe the remote fetch and refresh contract truthfully, including what is automatic versus explicit. <!-- verify: cargo test --workspace, SRS-04:start:end, proof: ac-1.log-->
- [ ] [SRS-NFR-03/AC-02] `ARCHITECTURE.md` and `CODE_WALKTHROUGH.md` describe the library-owned remote acquisition policy and proof surfaces consistently with the CLI story. <!-- verify: cargo test --workspace, SRS-NFR-03:end, proof: ac-2.log-->
