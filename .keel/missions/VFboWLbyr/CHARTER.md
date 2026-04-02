# Ship The First End-To-End Candle Conversion Path - Charter

Archetype: Strategic

## Goals

| ID | Description | Verification |
|----|-------------|--------------|
| MG-01 | Metamorph can inspect local and Hugging Face model sources and surface a truthful source-format report through both the library and CLI. | board: VFbp961HM |
| MG-02 | Metamorph can plan and execute a first useful `gguf -> hf-safetensors` path for Candle-oriented local runtimes with explicit lossy opt-in. | board: VFbp961HM |
| MG-03 | Metamorph can validate that the converted output matches the downstream Candle-style bundle contract. | board: VFbp961HM |

## Constraints

- Keep the library crate as the source of truth for conversion behavior and keep the CLI thin.
- Do not silently dequantize or otherwise hide lossy behavior.
- Prefer one real vertical slice over broad multi-format scaffolding.
- Keep `README.md` and the foundational docs aligned with whatever this mission actually ships.

## Halting Rules

- DO NOT halt while epic `VFbp961HM` lacks a planned voyage that covers inspection, conversion, and validation for the first Candle-oriented path.
- DO NOT halt while the first voyage under `VFbp961HM` is missing executable stories with proof-bearing acceptance criteria.
- YIELD to human before widening the mission into additional model families or runtime targets beyond the first Candle-oriented slice.
- HALT when epic `VFbp961HM` delivers a working inspect -> convert -> validate path and only prioritization of follow-on expansion remains.
