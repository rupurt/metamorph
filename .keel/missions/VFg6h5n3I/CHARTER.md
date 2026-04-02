# Turn Compatible Paths Into Executable Backends - Charter

Archetype: Strategic

## Goals

| ID | Description | Verification |
|----|-------------|--------------|
| MG-01 | Paths that Metamorph already recognizes as compatible can be promoted into real execution backends without breaking the registry-driven contract. | board: VFg70aqT7 |
| MG-02 | Planned-only paths such as same-format relayout and `safetensors -> hf-safetensors` gain explicit execution coverage or are reclassified with evidence-backed reasoning. | board: VFg70aqT7 |
| MG-03 | Additional execution backends preserve structured compatibility reporting, explicit blocker surfaces, and reusable-output validation. | board: VFg70aqT7 |
| MG-04 | The docs continue to state clearly which paths are executable, planned-only, unsupported, or lossy-gated. | board: VFg70aqT7 |

## Constraints

- Preserve the shared registry as the single source of truth for compatibility, planning, and execution dispatch.
- Do not broaden execution by introducing silent lossy fallbacks; any lossy path still requires explicit opt-in.
- Keep validation wired into newly executable outputs so added backends produce reusable artifacts rather than best-effort files.
- Favor incremental backend delivery over speculative format-matrix expansion.

## Halting Rules

- DO NOT halt while any MG-* goal lacks planned board work for backend promotion, validation, and compatibility-report truthfulness.
- DO NOT halt while the registry advertises planned-only paths that have no clear execution or de-scoping plan.
- YIELD to human before widening scope into dynamic plugin loading or large new runtime families that exceed the written proof surface.
- HALT when the next tranche of compatible paths has become executable or has been explicitly reclassified, with matching validation and documentation.
