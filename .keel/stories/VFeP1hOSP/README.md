---
# system-managed
id: VFeP1hOSP
status: done
created_at: 2026-04-02T09:11:53
updated_at: 2026-04-02T10:38:44
# authored
title: Gate Reusable Bundles With Validation
type: feat
operator-signal:
scope: VFeOQzrXV/VFeOTEZi2
index: 3
started_at: 2026-04-02T10:33:11
completed_at: 2026-04-02T10:38:44
---

# Gate Reusable Bundles With Validation

## Summary

Make validation the gate for reusable outputs so converted bundles are only treated as cacheable or publishable artifacts after the required Hugging Face-style safetensors layout checks pass.

## Acceptance Criteria

- [x] [SRS-03/AC-01] Validation rejects malformed or incomplete output bundles before Metamorph reports them as reusable artifacts. <!-- verify: cargo test --workspace, SRS-03:start, proof: ac-1.log-->
- [x] [SRS-03/AC-02] Successful validation produces an explicit reusable-output result for the primary bundle contract. <!-- verify: cargo test --workspace, SRS-03:end, proof: ac-2.log-->
- [x] [SRS-NFR-01/AC-02] Validation outcomes remain aligned between the library and CLI reporting surfaces. <!-- verify: cargo test --workspace, SRS-NFR-01:start:end, proof: ac-3.log-->
