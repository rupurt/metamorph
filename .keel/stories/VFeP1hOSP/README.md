---
# system-managed
id: VFeP1hOSP
status: backlog
created_at: 2026-04-02T09:11:53
updated_at: 2026-04-02T09:13:14
# authored
title: Gate Reusable Bundles With Validation
type: feat
operator-signal:
scope: VFeOQzrXV/VFeOTEZi2
index: 3
---

# Gate Reusable Bundles With Validation

## Summary

Make validation the gate for reusable outputs so converted bundles are only treated as cacheable or publishable artifacts after the required Hugging Face-style safetensors layout checks pass.

## Acceptance Criteria

- [ ] [SRS-03/AC-01] Validation rejects malformed or incomplete output bundles before Metamorph reports them as reusable artifacts. <!-- verify: cargo test --workspace, SRS-03:start -->
- [ ] [SRS-03/AC-02] Successful validation produces an explicit reusable-output result for the primary bundle contract. <!-- verify: cargo test --workspace, SRS-03:end -->
- [ ] [SRS-NFR-01/AC-02] Validation outcomes remain aligned between the library and CLI reporting surfaces. <!-- verify: cargo test --workspace, SRS-NFR-01:end -->
