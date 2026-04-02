---
# system-managed
id: VFeq4jwRJ
status: backlog
created_at: 2026-04-02T10:59:20
updated_at: 2026-04-02T11:03:27
# authored
title: Define Module Facade And Domain Reexports
type: feat
operator-signal:
scope: VFepvp0Xe/VFepwxnmQ
index: 1
---

# Define Module Facade And Domain Reexports

## Summary

Create the first stable module tree and top-level facade so the library can stop growing as one file without forcing immediate churn onto existing callers.

## Acceptance Criteria

- [ ] [SRS-01/AC-01] `crates/metamorph/src` defines dedicated modules for `source`, `format`, `plan`, `transform`, `validate`, `cache`, and `publish`, with `lib.rs` reduced to a facade or equivalent entry point. <!-- verify: cargo test --workspace, SRS-01:start:end, proof: ac-1.log-->
- [ ] [SRS-NFR-01/AC-01] Existing public workflow entry points remain available, or any migration is explicit and proven by compiling the current tests and examples. <!-- verify: cargo test --workspace, SRS-NFR-01:start, proof: ac-2.log-->
