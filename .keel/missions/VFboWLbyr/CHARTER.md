# Ship The First End-To-End Candle Conversion Path - Charter

Archetype: Strategic

## Goals

| ID | Description | Verification |
|----|-------------|--------------|
| MG-01 | Metamorph can inspect local and Hugging Face model sources and surface a truthful source-format report through both the library and CLI. | board: stories land for local and HF inspection, tests prove format inference behavior, and CLI proof shows `metamorph inspect` output for a representative source |
| MG-02 | Metamorph can plan and execute a first useful `gguf -> hf-safetensors` path for Candle-oriented local runtimes with explicit lossy opt-in. | board: a voyage lands the planner and first execution backend, tests cover lossy gating, and the CLI can show and run the path without hidden behavior |
| MG-03 | Metamorph can validate that the converted output matches the downstream Candle-style bundle contract. | board: stories land for validation of `config.json`, `tokenizer.json`, `generation_config.json`, and safetensors presence, with proof captured in tests and CLI validation runs |

## Constraints

- Keep the library crate as the source of truth for conversion behavior and keep the CLI thin.
- Do not silently dequantize or otherwise hide lossy behavior.
- Prefer one real vertical slice over broad multi-format scaffolding.
- Keep `README.md` and the foundational docs aligned with whatever this mission actually ships.

## Halting Rules

- DO NOT halt while any MG-* goal has unfinished board work
- HALT when all MG-* goals with `board:` verification are satisfied
- YIELD to human when only `metric:` or `manual:` goals remain
