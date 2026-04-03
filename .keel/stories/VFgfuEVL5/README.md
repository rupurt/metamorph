---
# system-managed
id: VFgfuEVL5
status: done
created_at: 2026-04-02T18:31:29
updated_at: 2026-04-03T15:17:45
# authored
title: Capture Partial Publish And Retry Signals
type: feat
operator-signal:
scope: VFg6zB3Ej/VFgfOlTWF
index: 2
started_at: 2026-04-03T15:11:00
completed_at: 2026-04-03T15:17:45
---

# Capture Partial Publish And Retry Signals

## Summary

Expose partial-publish truth and retry signals so operators can see what succeeded remotely, what remains pending, and what the next explicit safe retry step is.

## Acceptance Criteria

- [x] [SRS-02/AC-01] Publish reports and CLI output surface which artifacts succeeded, which remain pending, and what retry action the operator can take after a partial failure. <!-- verify: cargo test --workspace, SRS-02:start:end, proof: ac-1.log-->
- [x] [SRS-NFR-03/AC-01] Retry surfaces remain explicit and operator-driven rather than turning into hidden automatic repair behavior. <!-- verify: cargo test --workspace, SRS-NFR-03:start:end, proof: ac-2.log-->
