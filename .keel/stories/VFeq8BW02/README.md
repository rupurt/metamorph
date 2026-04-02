---
# system-managed
id: VFeq8BW02
status: done
created_at: 2026-04-02T10:59:33
updated_at: 2026-04-02T11:18:50
# authored
title: Add The Gguf To Safetensors Backend
type: feat
operator-signal:
scope: VFepvp0Xe/VFepxZZwT
index: 3
started_at: 2026-04-02T11:18:47
completed_at: 2026-04-02T11:18:50
---

# Add The Gguf To Safetensors Backend

## Summary

Prove the new extension seam with a second backend that turns GGUF input into validated safetensors output without bypassing the existing lossy and proof contracts.

## Acceptance Criteria

- [x] [SRS-03/AC-01] Metamorph can plan and execute `gguf -> safetensors` through the registered backend seam with explicit lossy opt-in. <!-- verify: cargo test --workspace, SRS-03:start, proof: ac-1.log-->
- [x] [SRS-03/AC-02] The resulting `.safetensors` artifact or bundle validates through the existing validation surface and CLI proof. <!-- verify: cargo test --workspace, SRS-03:end, proof: ac-2.log-->
- [x] [SRS-NFR-02/AC-01] Repeated runs for the same representative input produce deterministic enough output naming and validation results for repeatable proof capture. <!-- verify: cargo test --workspace, SRS-NFR-02:start:end, proof: ac-3.log-->
