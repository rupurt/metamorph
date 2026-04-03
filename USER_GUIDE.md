# Metamorph User Guide

This guide is for operators using the `metamorph` CLI and for teams wiring it into an operational workflow.

## Product Story

Metamorph exists for the gap between how model artifacts are published and how downstream runtimes actually load them.

Today the most complete path is:

1. inspect a source
2. plan a conversion
3. fetch or reuse a representative remote GGUF source when needed
4. convert GGUF into a reusable safetensors-based output
5. validate that output
6. preview or execute an explicit publish to an existing destination

The CLI is designed to make each step explicit instead of hiding transport, lossy conversion, or publish behavior behind convenience flags.

## Who This Is For

- Runtime integrators who need a dependable local conversion path
- Application developers who want a command-line workflow before embedding the library
- Infrastructure teams that need to inspect cache state and execute or preview controlled republishing

## Operator Loop

### 1. Inspect first

Start with `inspect` when the source format is not obvious:

```bash
metamorph inspect ./models/bonsai.gguf
metamorph inspect hf://prism-ml/Bonsai-8B-gguf@main
```

Use the result to decide whether you trust the inferred format or want to force it with `--from`.

### 2. Plan before executing

Use `convert --plan-only` before any real conversion:

```bash
metamorph convert \
  --input ./models/bonsai.gguf \
  --output ./artifacts/bonsai-candle \
  --to hf-safetensors \
  --from gguf \
  --allow-lossy \
  --plan-only
```

Read the output in this order:

- `Compatibility status`
- `Resolved source format`
- `Compatible backend`
- `Blockers`
- `Steps`

Current statuses mean:

- `executable`: an execution backend exists for this path
- `planned-only`: reserved for a compatible path the registry knows but does not execute yet; the current shipped matrix does not rely on this placeholder
- `unsupported`: no capability is registered for the requested path
- `unknown-source-format`: Metamorph could not infer the source format

### 3. Distinguish planning from execution

This distinction is important:

- planning can work with local paths and `hf://...` references
- execution is still local-targeted today
- representative remote GGUF sources now fetch on demand into managed cache instead of requiring manual cache seeding
- `--refresh` keeps remote re-fetch explicit rather than implicit
- broader remote repo layouts are still bounded and return recovery guidance instead of silent guesses

### 4. Run a conversion

Current executable conversions:

- `gguf -> hf-safetensors`
- `gguf -> safetensors`
- `safetensors -> safetensors`
- `hf-safetensors -> hf-safetensors`
- `safetensors -> hf-safetensors`

Additional rules for the new local paths:

- the relayout paths are local-only
- `safetensors -> hf-safetensors` currently expects exactly one local `.safetensors` artifact plus `config.json` and `tokenizer.json`
- if `generation_config.json` is missing, Metamorph writes an empty one into the target bundle and says so in the plan notes

Example:

```bash
metamorph convert \
  --input ./models/bonsai.gguf \
  --output ./artifacts/bonsai-candle \
  --to hf-safetensors \
  --from gguf \
  --allow-lossy
```

Lossy behavior is explicit. If you omit `--allow-lossy` for a GGUF conversion, Metamorph stops instead of silently changing representation.

### 5. Validate before reusing an output

Use validation as the gate for cache reuse, application loading, or publish preview.

```bash
metamorph validate ./artifacts/bonsai-candle --format hf-safetensors
metamorph validate ./artifacts/bonsai.safetensors --format safetensors
```

A passing validation result means the artifact satisfies the reusable contract for that format.

### 6. Inspect cache behavior

Use `cache dir` and `cache source` to understand what Metamorph would reuse or where it expects data to live.

```bash
metamorph cache dir
metamorph cache source ./models/bonsai.gguf --from gguf --materialize
metamorph cache source hf://prism-ml/Bonsai-8B-gguf@main
```

Cache outcomes currently mean:

- `reused-local-path`: Metamorph kept using the original local path
- `materialized-local-copy`: Metamorph copied a local source into managed storage
- `reused-managed-local-copy`: Metamorph reused an existing managed copy of a local source
- `reused-remote-cache`: the managed cache already contains the fetched remote artifact
- `fetched-remote`: Metamorph fetched a representative remote artifact into managed storage
- `refreshed-remote`: Metamorph replaced cached remote state because refresh was requested

### 7. Preview or execute a publish

`upload` is intentionally safe by default:

```bash
metamorph upload \
  --input ./artifacts/bonsai-candle \
  --repo your-org/Bonsai-8B-candle
```

This validates the bundle and prints the publish plan without mutating anything.

`--execute` is explicit:

- it requires `HF_TOKEN`
- it targets an existing Hugging Face repo on `main`
- it reports `complete`, `partial`, `guarded-refusal`, or `failed`
- it prints per-artifact status so retry work is legible

If the repo already matches the validated local bundle, Metamorph reports `already-present` for those artifacts instead of pretending it republished them.

## Recovery Guide

If a command fails, the recovery path should be legible from the output.

Common cases:

- `unknown-source-format`
  - Add `--from <FORMAT>` or point at a more explicit local layout.
- `planned-only`
  - Reserved for future compatible-but-unwired paths. In the current matrix, blocked requests usually show `executable` plus blockers, while truly unsupported pairs show `unsupported`.
- `unsupported`
  - No conversion capability exists for that source/target pair yet.
- lossy opt-in failure
  - Re-run with `--allow-lossy` only if that representation change is acceptable for your workflow.
- remote credential failure
  - Set `HF_TOKEN` for private or gated repos, or switch to a public/local source.
- remote revision failure
  - Verify the repo and revision, then retry.
- remote layout failure
  - The current fetch slice expects one representative GGUF artifact at the selected revision. Choose a repo revision that satisfies that shape or use a local source path.
- stale remote cache state
  - Re-run with `--refresh` to replace the broken managed cache entry.
- validation failure
  - Treat the output as non-reusable until the missing file, wrong layout, or invalid safetensors artifact is fixed.
- publish credential error
  - Set `HF_TOKEN`, confirm it has write access, and rerun with `--execute`.
- publish guarded refusal
  - The execute path was intentionally stopped before or during remote preflight. Review the notes, fix credentials or destination permissions, and retry explicitly.
- partial publish
  - Some remote work finished and some did not. Review the per-artifact status, keep the validated local bundle unchanged, and rerun `upload --execute`.
- publish failure
  - No complete publish was recorded. Confirm the destination repo exists, review the recovery note, and retry explicitly.

## Practical Expectations

Use Metamorph today when you need:

- a real local GGUF conversion path
- real local safetensors relayout and bundle normalization
- on-demand fetch for representative remote GGUF sources
- explicit compatibility reporting before execution
- deterministic cache identity
- validation-backed reusable outputs
- an explicit publish surface for validated bundles into existing Hugging Face repos

Do not treat it yet as:

- a generic Hugging Face downloader for every repo layout
- a promise that every same-format path is meaningful or supported
- a generic model registry sync engine
- a repo-creation or branch-management client

For library-facing details, use [README.md](README.md). For the implementation boundaries behind these workflows, use [ARCHITECTURE.md](ARCHITECTURE.md).
