# Turn Metamorph Into An Extensible Conversion Platform - Charter

Archetype: Strategic

## Goals

| ID | Description | Verification |
|----|-------------|--------------|
| MG-01 | The library surface evolves from an early single-file scaffold into stable modules for source, format, plan, transform, validate, cache, and publish concerns. | board: VFepvp0Xe |
| MG-02 | Adding a new source format, target format, or runtime-oriented layout becomes incremental rather than invasive. | board: VFepvp0Xe |
| MG-03 | Metamorph can report compatibility and conversion constraints clearly enough that downstream integrators can embed it without reverse-engineering internal assumptions. | board: VFepvp0Xe |
| MG-04 | The repo's governance docs, README, and board structure continue to describe the implemented architecture truthfully as the system expands. | board: VFepvp0Xe |

## Constraints

- Keep runtime-specific adapters at the edges and avoid baking one consumer's loader assumptions into the core.
- Preserve the distinction between transport, format, layout, and transformation.
- Favor explicit seams over premature abstraction.
- Do not broaden scope faster than proof can keep up.

## Halting Rules

- DO NOT halt while epic `VFepvp0Xe` lacks planned voyages for module extraction, backend extension, and compatibility-reporting or docs alignment.
- DO NOT halt while voyage `VFepwxnmQ` is missing executable stories that cover module facade extraction, operational module moves, backend isolation, and CLI or architecture alignment.
- DO NOT halt while voyage `VFepxZZwT` is missing executable stories that cover capability registry definition, backend dispatch, `gguf -> safetensors` delivery, and extension guardrails.
- DO NOT halt while voyage `VFepyCJ2Q` is missing executable stories that cover structured compatibility reports, CLI reasoning, and doc or board truthfulness.
- YIELD to human before broadening the mission into dynamic plugin loading, additional remote side effects, or new runtime contracts that exceed the written proof surface.
- HALT when epic `VFepvp0Xe` has delivered stable module seams, one additional backend through the registry, clear compatibility reporting, and aligned docs.
