---
# system-managed
id: VFeq7Ykva
status: backlog
created_at: 2026-04-02T10:59:31
updated_at: 2026-04-02T11:03:30
# authored
title: Dispatch Conversion Execution Through Registered Backends
type: feat
operator-signal:
scope: VFepvp0Xe/VFepxZZwT
index: 2
---

# Dispatch Conversion Execution Through Registered Backends

## Summary

Route execution through the registered backend seam so the existing path and future paths stop depending on open-coded top-level branching.

## Acceptance Criteria

- [ ] [SRS-02/AC-01] Execution dispatch resolves the existing `gguf -> hf-safetensors` path through a registered backend seam rather than an open-coded top-level match. <!-- verify: cargo test --workspace, SRS-02:start:end, proof: ac-1.log-->
- [ ] [SRS-NFR-03/AC-01] The new dispatch layer does not introduce dynamic plugin loading or implicit network behavior. <!-- verify: cargo test --workspace, SRS-NFR-03:start:end, proof: ac-2.log-->
