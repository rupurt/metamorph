---
# system-managed
id: VFeq9l6DC
status: backlog
created_at: 2026-04-02T10:59:39
updated_at: 2026-04-02T11:03:30
# authored
title: Render Compatibility Reasoning In The CLI
type: feat
operator-signal:
scope: VFepvp0Xe/VFepyCJ2Q
index: 2
---

# Render Compatibility Reasoning In The CLI

## Summary

Render compatibility reasoning through the CLI so operators can understand supported, lossy, and blocked requests without reverse-engineering planner failures.

## Acceptance Criteria

- [ ] [SRS-02/AC-01] CLI planning surfaces render compatibility reasoning for supported, lossy, and unsupported requests without collapsing everything into raw error text. <!-- verify: cargo test --workspace, SRS-02:start, proof: ac-1.log-->
