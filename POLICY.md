# Metamorph Policy

This document is downstream from Keel and defines the operational invariants that must remain true in **Metamorph**.

## Engine Contract

Metamorph uses Keel as its planning and lifecycle engine.

- The board lives in `.keel/`.
- The canonical tactical rhythm is `keel turn`.
- Lifecycle moves, proof, and closure should happen through Keel rather than ad hoc board file edits.

## The Core Objective: Zero Drift

Metamorph should not drift between product intent, docs, and code.

Progress is blocked when any of these appear:

- Product drift: `README.md`, the CLI, and the library describe different capabilities.
- Conversion drift: a source-to-target path exists in docs or code but does not state whether it is lossy.
- Proof drift: behavior changes land without tests, validation, or command-level evidence.
- Scaffold drift: placeholder text or generic Keel defaults remain in authoritative repo docs.

## Entity Invariants

- Missions should describe a real operator outcome such as enabling a concrete conversion workflow.
- Epics should group coherent format or runtime capabilities, not arbitrary implementation chores.
- Voyages should name the exact source format, target format, and proof bar they intend to satisfy.
- Stories that change behavior should identify the exact files touched and the command or test evidence that proves the acceptance criteria.

## Repo Invariants

- Public behavior changes must update the relevant documentation in the same change. At minimum this includes `README.md` and any affected foundational docs.
- Lossy conversions must require explicit opt-in in both the API and the CLI surface.
- Download, upload, or publish behavior must be explicit. Metamorph must not perform network-side effects silently.
- New conversion logic belongs in the library crate. The CLI should stay thin and orchestration-focused.
- Behavior-changing work should pass:
  - `nix develop -c cargo test --workspace`
  - `nix develop -c cargo clippy --workspace --all-targets --all-features -- -D warnings`
  - `nix build .#metamorph`

## Safety Rails

- Stop for human review when model licensing, redistribution rights, or attribution expectations are unclear.
- Never commit tokens, credentials, or private model access material.
- Destructive operations such as deleting caches, replacing mirrored artifacts, or uploading converted models to a public destination require explicit user intent.
- Do not market stubs as working features. `NotImplemented` is an acceptable state; false confidence is not.

## Local Exceptions

- Until formal ADRs and deeper planning artifacts exist, `README.md` is the authoritative product brief.
- Docs-first work is acceptable in this repository when it sharpens the contract for future implementation, but the docs must be explicit about what is still planned.
