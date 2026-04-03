---
# system-managed
id: VFgfuAvFN
status: done
created_at: 2026-04-02T18:31:29
updated_at: 2026-04-03T15:17:22
# authored
title: Introduce A Hugging Face Publish Provider Seam
type: feat
operator-signal:
scope: VFg6zB3Ej/VFgfOkNWv
index: 1
started_at: 2026-04-03T15:10:35
completed_at: 2026-04-03T15:17:22
---

# Introduce A Hugging Face Publish Provider Seam

## Summary

Define the library-owned provider seam that can target an explicitly named existing Hugging Face repository and upload the publish-plan artifact set without pushing remote write policy into the CLI.

## Acceptance Criteria

- [x] [SRS-01/AC-01] `crates/metamorph` defines a publish provider or executor seam that can target an existing Hugging Face repository and upload the planned artifact set for a validated bundle without embedding remote write policy in CLI code. <!-- verify: cargo test --workspace, SRS-01:start:end, proof: ac-1.log-->
- [x] [SRS-NFR-01/AC-01] The new publish seam is library-owned and reusable from publish execution code rather than being hidden behind CLI-specific upload handlers. <!-- verify: cargo test --workspace, SRS-NFR-01:start:end, proof: ac-2.log-->
