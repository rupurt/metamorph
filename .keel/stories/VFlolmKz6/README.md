---
# system-managed
id: VFlolmKz6
status: done
created_at: 2026-04-03T15:38:03
updated_at: 2026-04-03T15:56:34
# authored
title: Materialize Metadata-Backed Safetensors Bundles
type: feat
operator-signal:
scope: VFg70aqT7/VFlohna1u
index: 3
started_at: 2026-04-03T15:43:48
completed_at: 2026-04-03T15:56:34
---

# Materialize Metadata-Backed Safetensors Bundles

## Summary

Materialize a reusable local `hf-safetensors` bundle from a plain safetensors source when the source also provides the required Hugging Face metadata sidecars.

## Acceptance Criteria

- [x] [SRS-03/AC-01] `convert()` executes local `safetensors -> hf-safetensors` when one supported safetensors artifact plus the required metadata sidecars are present. <!-- verify: cargo test --workspace, SRS-03:start:end, proof: ac-1.log-->
- [x] [SRS-04/AC-01] The materialization path rejects missing metadata sidecars or unsupported source shapes before reporting a reusable output. <!-- verify: cargo test --workspace, SRS-04:start:end, proof: ac-2.log-->
