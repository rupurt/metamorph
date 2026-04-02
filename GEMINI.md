# GEMINI.md

Guidance for **Gemini CLI** and **Google AI Studio** when working with this repository.

## Shared Contract

Before doing work, read:

1. `AGENTS.md`
2. `INSTRUCTIONS.md`
3. `POLICY.md`
4. `ARCHITECTURE.md`

Those files are the repo-wide operating contract. This file should stay thin and only capture Gemini-specific harness notes.

## Project-Specific Gemini Notes

<!-- BEGIN PROJECT-SPECIFIC -->
- Start work from `nix develop` or a `direnv`-loaded shell so the Rust toolchain and `keel` are present.
- Prefer library-first changes in `crates/metamorph/`; keep the CLI in `crates/metamorph-cli/` thin.
- Keep `README.md`, the foundational docs, and the CLI behavior aligned when you change public behavior.
- Treat lossy conversion semantics and publish behavior as high-sensitivity surfaces that must stay explicit.
<!-- END PROJECT-SPECIFIC -->
