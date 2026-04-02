# CLAUDE.md

Guidance for **Claude Code** when working with this repository.

## Shared Contract

Before doing work, read:

1. `AGENTS.md`
2. `INSTRUCTIONS.md`
3. `POLICY.md`
4. `ARCHITECTURE.md`

Those files are the repo-wide operating contract. This file should stay thin and only capture Claude-specific harness notes.

## Project-Specific Claude Notes

<!-- BEGIN PROJECT-SPECIFIC -->
- Start work from `nix develop` or a `direnv`-loaded shell so the Rust toolchain and `keel` are present.
- Prefer adding behavior to `crates/metamorph/` and only expose it through `crates/metamorph-cli/` once the library contract is clear.
- When user-visible behavior changes, keep `README.md` and the foundational docs in sync.
- Do not overstate implementation status: conversion planning exists, execution backends are still partial.
<!-- END PROJECT-SPECIFIC -->
