---
# system-managed
id: VFbpqDTMe
status: backlog
created_at: 2026-04-01T22:39:35
updated_at: 2026-04-01T22:40:53
# authored
title: Implement GGUF To HF Safetensors Backend
type: feat
operator-signal:
scope: VFbp961HM/VFbpfEuEU
index: 2
---

# Implement GGUF To HF Safetensors Backend

## Summary

Implement the first executable `gguf -> hf-safetensors` path for a Candle-oriented bundle. This story covers execution behavior, explicit lossy gating, and the minimum file layout needed for downstream use.

## Acceptance Criteria

- [ ] [SRS-02/AC-01] The planner rejects unsupported requests and requires explicit opt-in for lossy `gguf -> hf-safetensors` conversions. <!-- verify: cargo test --workspace, SRS-02:start -->
- [ ] [SRS-02/AC-02] `convert()` and the CLI can execute the first supported path and materialize the expected bundle shape. <!-- verify: cargo test --workspace, SRS-02:continues -->
- [ ] [SRS-NFR-01/AC-02] The CLI and library present the same lossy-conversion truth for the first backend. <!-- verify: cargo test --workspace, SRS-NFR-01:continues -->
