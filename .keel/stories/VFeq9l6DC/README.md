---
# system-managed
id: VFeq9l6DC
status: done
created_at: 2026-04-02T10:59:39
updated_at: 2026-04-02T11:19:00
# authored
title: Render Compatibility Reasoning In The CLI
type: feat
operator-signal:
scope: VFepvp0Xe/VFepyCJ2Q
index: 2
started_at: 2026-04-02T11:18:59
completed_at: 2026-04-02T11:19:00
---

# Render Compatibility Reasoning In The CLI

## Summary

Render compatibility reasoning through the CLI so operators can understand supported, lossy, and blocked requests without reverse-engineering planner failures.

## Acceptance Criteria

- [x] [SRS-02/AC-01] CLI planning surfaces render compatibility reasoning for supported, lossy, and unsupported requests without collapsing everything into raw error text. <!-- verify: cargo test --workspace, SRS-02:start:end, proof: ac-1.log-->
