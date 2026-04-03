---
# system-managed
id: VFgfuCtIb
status: backlog
created_at: 2026-04-02T18:31:29
updated_at: 2026-04-02T18:32:22
# authored
title: Render Real Publish Outcomes In Upload
type: feat
operator-signal:
scope: VFg6zB3Ej/VFgfOkuYG
index: 2
---

# Render Real Publish Outcomes In Upload

## Summary

Render the new publish execution truth through `metamorph upload` so operators can see the same plan, completion state, and per-artifact outcome details that the library reports.

## Acceptance Criteria

- [ ] [SRS-02/AC-01] `metamorph upload` preserves preview-only behavior by default and renders the same execution truth as the library when `--execute` is supplied. <!-- verify: cargo test --workspace, SRS-02:start:end, proof: ac-1.log-->
- [ ] [SRS-NFR-01/AC-01] Command output distinguishes preview, complete publish, partial publish, and guarded refusal clearly enough for operators to understand when remote mutation occurred. <!-- verify: cargo test --workspace, SRS-NFR-01:start:end, proof: ac-2.log-->
