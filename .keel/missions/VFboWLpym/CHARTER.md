# Turn Metamorph Into An Extensible Conversion Platform - Charter

Archetype: Strategic

## Goals

| ID | Description | Verification |
|----|-------------|--------------|
| MG-01 | The library surface evolves from an early single-file scaffold into stable modules for source, format, plan, transform, validate, cache, and publish concerns. | board: stories land for module extraction or interface stabilization without bloating the CLI, with tests and docs updated alongside the refactor |
| MG-02 | Adding a new source format, target format, or runtime-oriented layout becomes incremental rather than invasive. | board: voyages close that demonstrate at least one additional path or backend added through the stabilized seams |
| MG-03 | Metamorph can report compatibility and conversion constraints clearly enough that downstream integrators can embed it without reverse-engineering internal assumptions. | board: compatibility-reporting or planning stories land with library and CLI proof that unsupported or lossy paths are explained clearly |
| MG-04 | The repo's governance docs, README, and board structure continue to describe the implemented architecture truthfully as the system expands. | board: planning and docs stories close whenever architecture shifts, preventing product and architecture drift |

## Constraints

- Keep runtime-specific adapters at the edges and avoid baking one consumer's loader assumptions into the core.
- Preserve the distinction between transport, format, layout, and transformation.
- Favor explicit seams over premature abstraction.
- Do not broaden scope faster than proof can keep up.

## Halting Rules

- DO NOT halt while any MG-* goal has unfinished board work
- HALT when all MG-* goals with `board:` verification are satisfied
- YIELD to human when only `metric:` or `manual:` goals remain
