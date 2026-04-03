# Prove And Document Compatible Path Promotion - Software Design Description

> Prove the new executable matrix through CLI evidence and documentation.

**SRS:** [SRS.md](SRS.md)

## Overview

This voyage keeps the promoted backend matrix legible. It extends CLI rendering and automated proof directly from the library-owned compatibility and conversion surfaces, then updates the docs to match what shipped.

## Context & Boundaries

In scope: CLI rendering, tests, and documentation.

Out of scope: additional backend logic beyond the behavior already defined by voyages 1 and 2.

```
┌─────────────────────────────────────────┐
│              This Voyage                │
│                                         │
│  ┌──────────────┐  ┌─────────────────┐ │
│  │ CLI output   │  │ Proof and docs  │ │
│  └──────────────┘  └─────────────────┘ │
└─────────────────────────────────────────┘
         ↑                    ↑
    Library reports       Shipped matrix
```

## Dependencies

| Dependency | Type | Purpose | Version/API |
|------------|------|---------|-------------|
| `crates/metamorph-cli/src/main.rs` | Internal module | Convert-command rendering | current workspace |
| `crates/metamorph-cli/tests/convert.rs` | Internal module | CLI proof | current workspace |
| `crates/metamorph/src/tests.rs` | Internal module | Library proof | current workspace |
| Foundational docs | Project docs | Product contract alignment | current workspace |

## Key Decisions

| Decision | Choice | Rationale |
|----------|--------|-----------|
| CLI policy source | Render library reports instead of inventing CLI-only rules | Keeps operator output aligned with integration behavior |
| Proof strategy | Cover both success and blocked cases | The matrix is only trustworthy if it proves where execution stops as well as where it succeeds |
| Docs scope | Update README plus the three foundational context docs | User-visible backend truth spans both CLI and library audiences |

## Architecture

The CLI continues to call `compatibility()`, `plan()`, and `convert()` from the library. Tests assert on the rendered output, while docs reflect the same backend labels and blocker semantics.

## Components

- `convert_command()`
  Purpose: print compatibility status, backend, blockers, and execution results for promoted paths.
- CLI and library tests
  Purpose: lock in the success and blocked matrix.
- README and foundational docs
  Purpose: state the executable matrix and source-contract requirements plainly.

## Interfaces

- `metamorph convert --plan-only`
- `metamorph convert`
- `compatibility()`, `plan()`, and `convert()` from the library API

## Data Flow

1. Compatibility and plan data are rendered by the CLI.
2. Conversion is executed for runnable requests.
3. Tests assert on backend labels, blockers, and output files.
4. Docs are updated from the implemented matrix rather than aspirational claims.

## Error Handling

| Error Condition | Detection | Response | Recovery |
|-----------------|-----------|----------|----------|
| CLI output drifts from library truth | Tests fail on rendered backend or blocker text | Adjust CLI rendering to consume library data directly | Re-run CLI tests |
| Docs overclaim backend coverage | Review against the implemented matrix | Update docs in the same change set | Keep planned or unsupported paths labeled honestly |
