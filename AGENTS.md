# AGENTS.md

Shared guidance for AI agents working with Metamorph.

## Downstream Contract

This repository uses Keel as its project-management engine. This file is downstream from Keel and should remain recognizable when upstream engine guidance changes.

`AGENTS.md` and `INSTRUCTIONS.md` are the sync-sensitive files in this scaffold. When you absorb a newer Keel version, preserve the `PROJECT-SPECIFIC` blocks instead of rewriting the whole file from memory.

## Read This First

1. `INSTRUCTIONS.md` for the repo's procedural turn loop.
2. `POLICY.md` for local operational invariants.
3. `ARCHITECTURE.md` and `USER_GUIDE.md` for product and system context.
4. `CODE_WALKTHROUGH.md` for source layout and key abstractions.
5. `keel turn`, `keel mission next --status`, and `keel doctor --status` for the live board state.

## Core Principles

- Use Keel as the canonical planning and lifecycle surface.
- Prefer explicit proof over chat-only claims.
- Close loop debt with sealing commits instead of leaving dirty work behind.
- Escalate only when the work requires human product, design, legal, or operational judgment.

## Decision Resolution Hierarchy

When faced with ambiguity, resolve decisions in this descending order:
1.  **ADRs**: Binding architectural constraints.
2.  **CONSTITUTION**: The philosophy of collaboration.
3.  **POLICY**: The engine's operational invariants.
4.  **ARCHITECTURE**: Source layout and technical boundaries.
5.  **PLANNING**: PRD/SRS/SDD authored for the current mission.

## Foundational Documents

These define the constraints and workflow of the Metamorph environment:

- `INSTRUCTIONS.md` — Step-by-step procedural loops and checklists.
- `POLICY.md` — Operational invariants and engine constraints.
- `CONSTITUTION.md` — Collaboration philosophy and decision hierarchy.
- `ARCHITECTURE.md` — Implementation architecture and technical boundaries.
- `CODE_WALKTHROUGH.md` — Source layout, key abstractions, and data-flow orientation.
- `USER_GUIDE.md` — Operator-visible product story and workflow guidance.
- `.keel/adrs/` — Binding architecture decisions.

Use this order when interpreting constraints: ADRs → Constitution → Policy → Architecture → Planning artifacts.

## Project-Specific Conventions

<!-- BEGIN PROJECT-SPECIFIC -->
- Enter the repo through `nix develop` or `direnv`; do not assume `cargo` is available on the plain shell PATH.
- Treat `README.md` as the current product contract unless a newer ADR or approved planning artifact supersedes it.
- Keep the Rust library in `crates/metamorph/` as the source of truth for conversion behavior. The CLI in `crates/metamorph-cli/` should stay thin.
- When changing user-visible behavior, update the README and the affected foundational docs in the same change.
- Preserve explicit lossy-conversion semantics. Do not add a conversion path that silently dequantizes, requantizes, or changes layout without surfacing that fact.
- Useful local commands:
  - `nix develop -c cargo test --workspace`
  - `nix develop -c cargo clippy --workspace --all-targets --all-features -- -D warnings`
  - `nix build .#metamorph`
  - `nix develop -c cargo run -p metamorph-cli -- --help`
  - `nix develop -c keel turn`
<!-- END PROJECT-SPECIFIC -->

## Sync Notes

- Upstream source: Keel's `AGENTS.md`
- Preserve the project-specific block above during syncs.
- Push detailed workflow rules into `INSTRUCTIONS.md`, not this file.
