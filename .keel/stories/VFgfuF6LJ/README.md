---
# system-managed
id: VFgfuF6LJ
status: backlog
created_at: 2026-04-02T18:31:29
updated_at: 2026-04-02T18:32:22
# authored
title: Refresh README And Foundational Docs For Executable Upload
type: feat
operator-signal:
scope: VFg6zB3Ej/VFgfOlTWF
index: 3
---

# Refresh README And Foundational Docs For Executable Upload

## Summary

Bring the README and foundational docs up to date with the executable upload contract so operators and integrators can understand preview, execute, partial failure, and existing-repo preconditions without reverse-engineering the code.

## Acceptance Criteria

- [ ] [SRS-04/AC-01] `README.md` and `USER_GUIDE.md` describe the executable upload contract truthfully, including existing-repo preconditions, explicit execute semantics, and human-sensitive seams. <!-- verify: cargo test --workspace, SRS-04:start:end, proof: ac-1.log-->
- [ ] [SRS-NFR-01/AC-02] `ARCHITECTURE.md` and `CODE_WALKTHROUGH.md` describe preview, complete publish, partial publish, guarded refusal, and retry surfaces consistently with the CLI story. <!-- verify: cargo test --workspace, SRS-NFR-01:end, proof: ac-2.log-->
