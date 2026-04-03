---
# system-managed
id: VFg8FMeq0
status: done
created_at: 2026-04-02T16:17:47
updated_at: 2026-04-02T17:01:31
# authored
title: Introduce A Hugging Face Fetch Provider Seam
type: feat
operator-signal:
scope: VFg6yYH7e/VFg7mjSPU
index: 1
started_at: 2026-04-02T16:59:12
completed_at: 2026-04-02T17:01:31
---

# Introduce A Hugging Face Fetch Provider Seam

## Summary

Define the library-owned provider seam that can resolve representative `hf://repo[@revision]` inputs into fetchable remote artifacts without pushing transport policy into the CLI.

## Acceptance Criteria

- [x] [SRS-01/AC-01] `crates/metamorph` defines a provider seam that can resolve representative Hugging Face sources into remote artifact listings or download handles without embedding the fetch policy in CLI code. <!-- verify: cargo test --workspace, SRS-01:start:end, proof: ac-1.log-->
- [x] [SRS-NFR-03/AC-01] The new fetch seam is library-owned and reusable from acquisition code rather than being hidden behind command-specific logic. <!-- verify: cargo test --workspace, SRS-NFR-03:start:end, proof: ac-2.log-->
