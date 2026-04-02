# Fetch Hugging Face Sources On Demand - Charter

Archetype: Strategic

## Goals

| ID | Description | Verification |
|----|-------------|--------------|
| MG-01 | Metamorph can fetch a Hugging Face source on demand instead of treating remote execution as cache-hit-only. | board: VFg6yYH7e |
| MG-02 | Remote acquisition preserves deterministic cache identity and tells operators exactly what was fetched, reused, or refreshed. | board: VFg6yYH7e |
| MG-03 | The CLI and library expose truthful recovery guidance for auth failures, missing revisions, interrupted downloads, and stale cache state. | board: VFg6yYH7e |
| MG-04 | README and operator guidance describe remote fetch behavior precisely, including what remains explicit versus automatic. | board: VFg6yYH7e |

## Constraints

- Preserve deterministic cache semantics; on-demand fetch must not turn cache location or reuse rules into hidden state.
- Keep the library crate as the source of truth for remote acquisition behavior and keep the CLI as a rendering layer.
- Do not mask remote failures behind generic cache-miss messaging once real fetch exists.
- Preserve explicit lossy-conversion semantics; remote fetch is transport work, not permission to soften conversion policy.

## Halting Rules

- DO NOT halt while any MG-* goal lacks planned board work for remote fetch, cache integration, and recovery behavior.
- DO NOT halt while on-demand `hf://repo[@revision]` execution still requires manual cache prepopulation.
- YIELD to human before broadening fetch scope into provider-specific sync policy, background refresh daemons, or destructive cache eviction semantics.
- HALT when Metamorph can fetch Hugging Face sources on demand with deterministic cache outcomes, operator-visible recovery guidance, and aligned docs.
