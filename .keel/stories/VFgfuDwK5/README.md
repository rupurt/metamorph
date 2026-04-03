---
# system-managed
id: VFgfuDwK5
status: backlog
created_at: 2026-04-02T18:31:29
updated_at: 2026-04-02T18:32:22
# authored
title: Surface Recovery Guidance For Remote Publish Failures
type: feat
operator-signal:
scope: VFg6zB3Ej/VFgfOlTWF
index: 1
---

# Surface Recovery Guidance For Remote Publish Failures

## Summary

Replace generic remote publish failures with recovery guidance that distinguishes the main guarded-refusal and remote-failure classes operators will hit during executable upload.

## Acceptance Criteria

- [ ] [SRS-01/AC-01] Remote publish failures and guarded refusals distinguish missing credentials, missing destination, permission failure, interrupted transfer, and partial publish state in the operator-facing recovery path. <!-- verify: cargo test --workspace, SRS-01:start:end, proof: ac-1.log-->
- [ ] [SRS-NFR-01/AC-01] Library and CLI output use consistent terminology for guarded refusal, publish failure, and partial publish recovery classes. <!-- verify: cargo test --workspace, SRS-NFR-01:start, proof: ac-2.log-->
