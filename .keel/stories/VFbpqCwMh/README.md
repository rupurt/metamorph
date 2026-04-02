---
# system-managed
id: VFbpqCwMh
status: done
created_at: 2026-04-01T22:39:35
updated_at: 2026-04-01T23:24:31
# authored
title: Implement Candle Bundle Validation
type: feat
operator-signal:
scope: VFbp961HM/VFbpfEuEU
index: 1
started_at: 2026-04-01T23:22:21
completed_at: 2026-04-01T23:24:31
---

# Implement Candle Bundle Validation

## Summary

Implement validation for the first Candle-friendly bundle contract so operators can tell the difference between a merely written directory and a loadable output.

## Acceptance Criteria

- [x] [SRS-03/AC-01] Validation rejects output bundles missing required files such as `config.json`, `tokenizer.json`, `generation_config.json`, or safetensors artifacts. <!-- verify: cargo test --workspace, SRS-03:start, proof: ac-1.log-->
- [x] [SRS-03/AC-02] Validation accepts a bundle that satisfies the expected Candle-oriented layout contract. <!-- verify: cargo test --workspace, SRS-03:end, proof: ac-2.log-->
- [x] [SRS-NFR-01/AC-03] Validation outcomes are surfaced consistently through the library and CLI. <!-- verify: cargo test --workspace, SRS-NFR-01:end, proof: ac-3.log-->
