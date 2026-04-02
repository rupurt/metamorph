# Execute Guarded Remote Publish Flows - Charter

Archetype: Strategic

## Goals

| ID | Description | Verification |
|----|-------------|--------------|
| MG-01 | Metamorph can execute a real remote publish flow for validated local bundles instead of stopping at preview. | board: VFg6zB3Ej |
| MG-02 | Remote publish execution remains explicit, credential-aware, and policy-gated rather than becoming an accidental side effect. | board: VFg6zB3Ej |
| MG-03 | Operators can inspect remote publish outcomes, partial-failure states, and retry guidance through both the library and CLI. | board: VFg6zB3Ej |
| MG-04 | The documented upload contract stays truthful about what is preview-only, what is executable, and which preconditions are enforced. | board: VFg6zB3Ej |

## Constraints

- Uploads must remain explicit user actions; no background or implicit remote writes.
- License, redistribution, and destination-policy checks remain human-sensitive seams even after remote execution is implemented.
- Validation of the local bundle stays part of the publish happy path.
- Credential handling and remote errors must be surfaced directly rather than hidden behind partial success language.

## Halting Rules

- DO NOT halt while any MG-* goal lacks planned board work for remote execution, guarded credentials, and publish recovery behavior.
- DO NOT halt while `upload --execute` still cannot carry a validated bundle through a real remote write path.
- YIELD to human before declaring public publish flows complete when licensing, redistribution, or destination-governance questions require product or legal judgment.
- HALT when Metamorph can execute a guarded remote publish flow with explicit preflight, credential checks, outcome reporting, and aligned operator guidance.
