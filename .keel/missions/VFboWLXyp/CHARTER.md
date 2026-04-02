# Operationalize Validation Cache And Publishing Flows - Charter

Archetype: Strategic

## Goals

| ID | Description | Verification |
|----|-------------|--------------|
| MG-01 | Metamorph can acquire and cache source artifacts deterministically enough for repeatable local reuse. | board: stories land for cache layout, source acquisition, and resumable or repeatable fetch behavior with proof in tests or command evidence |
| MG-02 | Metamorph validates converted artifacts and associated metadata before they are treated as reusable or publishable outputs. | board: validation stories close with proof that bad layouts or missing metadata fail clearly and valid bundles pass cleanly |
| MG-03 | Metamorph can optionally publish or mirror a converted artifact set to a destination such as Hugging Face without making network side effects implicit. | board: upload or mirror stories land with explicit CLI and library surfaces, permission gates, and proof of dry-run or safe publish behavior |
| MG-04 | Operators have a clear recovery path when cache, validation, or publish steps fail. | board: docs and command behavior land together so failure modes and recovery steps are discoverable from the CLI and repo docs |

## Constraints

- Uploads and mirrors must remain explicit user actions.
- License, redistribution, and attribution questions require human review before public publishing flows are considered complete.
- Cache semantics should favor deterministic paths and stable metadata over convenience.
- Validation is part of the happy path, not an optional afterthought.

## Halting Rules

- DO NOT halt while any MG-* goal has unfinished board work
- HALT when all MG-* goals with `board:` verification are satisfied
- YIELD to human when only `metric:` or `manual:` goals remain
