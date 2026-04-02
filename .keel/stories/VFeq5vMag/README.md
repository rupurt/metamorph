---
# system-managed
id: VFeq5vMag
status: backlog
created_at: 2026-04-02T10:59:24
updated_at: 2026-04-02T11:03:27
# authored
title: Extract The Existing Gguf Backend Behind Transform Seams
type: feat
operator-signal:
scope: VFepvp0Xe/VFepwxnmQ
index: 3
---

# Extract The Existing Gguf Backend Behind Transform Seams

## Summary

Separate the current `gguf -> hf-safetensors` execution path from the top-level workflow so the transform layer can host multiple backends without another round of invasive edits.

## Acceptance Criteria

- [ ] [SRS-03/AC-01] The current `gguf -> hf-safetensors` execution path is isolated behind a transform or backend-specific module seam instead of top-level monolith code. <!-- verify: cargo test --workspace, SRS-03:start:end, proof: ac-1.log-->
- [ ] [SRS-NFR-01/AC-03] Existing end-to-end behavior for the first backend remains unchanged after extraction, as shown by conversion and validation tests. <!-- verify: cargo test --workspace, SRS-NFR-01:end, proof: ac-2.log-->
