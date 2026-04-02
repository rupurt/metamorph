# Metamorph Constitution

This document is downstream from Keel and defines how **Metamorph** wants humans and agents to work together.

## Why This Exists

- Keep the repo aligned around one job: converting model artifacts between runtime-specific formats.
- Make technical integrity win when convenience would hide a lossy or misleading conversion.
- Give planning and delivery work a stable product frame while the codebase is still early.

## Decision Hierarchy

Use this descending order when decisions conflict:

1. ADRs
2. Constitution
3. Policy
4. Architecture
5. README and approved planning artifacts
6. Live board state in `.keel/`
7. Temporary chat intent or operator preference

If the README and implementation diverge, do not hand-wave it away. Update the code, the README, or both.

## Project Values

- Truth over convenience. Distinguish lossless format normalization from lossy dequantization or requantization.
- Library first. The reusable Rust library is the product core; the CLI is a thin operational layer on top.
- Validation is part of delivery. A conversion is not done when bytes are written; it is done when the output can be inspected and validated honestly.
- Determinism matters. Prefer stable outputs, stable naming, and stable metadata so caching and mirroring stay reliable.
- Small end-to-end slices beat broad speculative scaffolding. Land one real source-to-target path before generalizing.

## Collaboration Rules

- Require a human decision for licensing or redistribution assumptions, destructive artifact operations, or material changes to the public product promise.
- Prefer the smallest vertical slice that proves a real workflow such as inspect -> plan -> convert -> validate.
- Treat “good enough to ship” as: behavior, docs, and proof all agree.
- Do not represent planned capabilities as implemented capabilities. When a backend is stubbed, say so plainly.

## Revision Notes

- Keep this document short and high authority.
- Put workflow details in `INSTRUCTIONS.md`.
- Put structural and technical specifics in `POLICY.md` and `ARCHITECTURE.md`.
