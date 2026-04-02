# Operationalize Validation Cache And Publishing Flows - Charter

Archetype: Strategic

## Goals

| ID | Description | Verification |
|----|-------------|--------------|
| MG-01 | Metamorph can acquire and cache source artifacts deterministically enough for repeatable local reuse. | board: VFeOQzrXV |
| MG-02 | Metamorph validates converted artifacts and associated metadata before they are treated as reusable or publishable outputs. | board: VFeOQzrXV |
| MG-03 | Metamorph can optionally publish or mirror a converted artifact set to a destination such as Hugging Face without making network side effects implicit. | board: VFeOQzrXV |
| MG-04 | Operators have a clear recovery path when cache, validation, or publish steps fail. | board: VFeOQzrXV |

## Constraints

- Uploads and mirrors must remain explicit user actions.
- License, redistribution, and attribution questions require human review before public publishing flows are considered complete.
- Cache semantics should favor deterministic paths and stable metadata over convenience.
- Validation is part of the happy path, not an optional afterthought.

## Halting Rules

- DO NOT halt while epic `VFeOQzrXV` lacks planned voyages for both deterministic cache plus validation reuse and explicit publish plus recovery behavior.
- DO NOT halt while voyage `VFeOTEZi2` is missing executable stories that cover cache identity, source acquisition or reuse, validation gating, and recovery messaging with proof-bearing acceptance criteria.
- DO NOT halt while voyage `VFeOTDZi1` is missing executable stories that cover publish planning, dry-run or preview behavior, guarded execution, and policy-aware recovery guidance.
- YIELD to human before declaring public publish flows complete when licensing, redistribution, or attribution questions require product or legal judgment.
- HALT when epic `VFeOQzrXV` has delivered deterministic local reuse, explicit validation gates, guarded publish surfaces, and aligned operator guidance.
