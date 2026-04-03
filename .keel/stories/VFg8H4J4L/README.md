---
# system-managed
id: VFg8H4J4L
status: done
created_at: 2026-04-02T16:17:53
updated_at: 2026-04-02T17:01:41
# authored
title: Allow Remote GGUF Conversion To Fetch On Cache Miss
type: feat
operator-signal:
scope: VFg6yYH7e/VFg7nTfTq
index: 3
started_at: 2026-04-02T16:59:16
completed_at: 2026-04-02T17:01:41
---

# Allow Remote GGUF Conversion To Fetch On Cache Miss

## Summary

Let supported remote GGUF conversion execute after fetching on demand so operators no longer have to seed the cache manually before converting a representative `hf://` source.

## Acceptance Criteria

- [x] [SRS-03/AC-01] A supported remote GGUF conversion path fetches its source on cache miss and then continues through the existing backend execution flow without manual cache prepopulation. <!-- verify: cargo test --workspace, SRS-03:start:end, proof: ac-1.log-->
- [x] [SRS-04/AC-01] Remote conversion and its CLI entry points continue to consume the library-owned acquisition flow instead of introducing CLI-specific fetch or cache policy. <!-- verify: cargo test --workspace, SRS-04:start:end, proof: ac-2.log-->
- [x] [SRS-NFR-02/AC-02] Local conversion behavior remains intact while remote fetch-on-convert is added. <!-- verify: cargo test --workspace, SRS-NFR-02:end, proof: ac-3.log-->
- [x] [SRS-NFR-03/AC-02] Remote conversion output keeps fetch versus reuse legible enough for operators to understand when a network side effect occurred. <!-- verify: cargo test --workspace, SRS-NFR-03:end, proof: ac-4.log-->
