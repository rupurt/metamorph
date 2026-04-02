---
# system-managed
id: VFbpfFREX
status: done
created_at: 2026-04-01T22:38:53
updated_at: 2026-04-01T23:10:52
# authored
title: Implement Source Inspection Contract
type: feat
operator-signal:
scope: VFbp961HM/VFbpfEuEU
index: 1
started_at: 2026-04-01T22:41:39
completed_at: 2026-04-01T23:10:52
---

# Implement Source Inspection Contract

## Summary

Author the first real inspection slice so both the library and CLI can report local and `hf://` source formats truthfully. This story covers detection behavior, CLI presentation, and tests for representative inputs.

## Acceptance Criteria

- [x] [SRS-01/AC-01] The library inspects representative local paths and `hf://` references and returns explicit detected-format or unknown results. <!-- verify: cargo test --workspace, SRS-01:start, proof: ac-1.log-->
- [x] [SRS-01/AC-02] The CLI renders the inspection result clearly for operators without hiding unknown-format cases. <!-- verify: cargo test --workspace, SRS-01:end, proof: ac-2.log-->
- [x] [SRS-NFR-01/AC-01] Library and CLI inspection behavior stay aligned through tests or command-level proof. <!-- verify: cargo test --workspace, SRS-NFR-01:start:end, proof: ac-3.log-->
