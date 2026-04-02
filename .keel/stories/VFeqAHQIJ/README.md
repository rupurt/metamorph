---
# system-managed
id: VFeqAHQIJ
status: done
created_at: 2026-04-02T10:59:41
updated_at: 2026-04-02T11:19:02
# authored
title: Align Docs And Board Contracts With The Extension Surface
type: feat
operator-signal:
scope: VFepvp0Xe/VFepyCJ2Q
index: 3
started_at: 2026-04-02T11:19:00
completed_at: 2026-04-02T11:19:02
---

# Align Docs And Board Contracts With The Extension Surface

## Summary

Update the repo docs and planning artifacts so they describe the modular architecture, capability registry, and currently shipped paths without overstating what is still only planned.

## Acceptance Criteria

- [x] [SRS-03/AC-01] `README.md`, `ARCHITECTURE.md`, `USER_GUIDE.md`, and `CODE_WALKTHROUGH.md` describe the modular architecture, backend registry, and currently supported paths truthfully. <!-- verify: cargo test --workspace, SRS-03:start:end, proof: ac-1.log-->
- [x] [SRS-04/AC-01] Unsupported or blocked requests explain actionable next steps or recovery guidance for operators and downstream integrators. <!-- verify: cargo test --workspace, SRS-04:start:end, proof: ac-2.log-->
- [x] [SRS-NFR-02/AC-01] Mission, epic, and voyage artifacts stop overstating support levels and track the delivered extension contract precisely. <!-- verify: cargo test --workspace, SRS-NFR-02:start:end, proof: ac-3.log-->
