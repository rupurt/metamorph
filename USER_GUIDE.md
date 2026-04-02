# Metamorph User Guide

This guide is for operators using the `metamorph` CLI and for teams wiring it into an operational workflow.

## Product Story

Metamorph exists for the gap between how model artifacts are published and how downstream runtimes actually load them.

Today the most complete path is:

1. inspect a source
2. plan a conversion
3. convert local GGUF into a reusable safetensors-based output
4. validate that output
5. preview how it would be cached or republished

The CLI is designed to make each step explicit instead of hiding transport, lossy conversion, or publish behavior behind convenience flags.

## Who This Is For

- Runtime integrators who need a dependable local conversion path
- Application developers who want a command-line workflow before embedding the library
- Infrastructure teams that need to inspect cache state and preview republishing

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
- `planned-only`: the path is known but execution is not wired yet
- `unsupported`: no capability is registered for the requested path
- `unknown-source-format`: Metamorph could not infer the source format

### 3. Distinguish planning from execution

This distinction is important:

- planning can work with local paths and `hf://...` references
- execution is local-first today

If you ask Metamorph to execute a remote source today, it only succeeds when the corresponding source artifact already exists in the managed cache. Otherwise you get an explicit cache-miss path instead of a hidden download attempt.

### 4. Run a conversion

Current executable conversions:

- `gguf -> hf-safetensors`
- `gguf -> safetensors`

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
- `cache-hit`: the managed cache already contains what the next step needs
- `cache-miss`: the cache key and expected path are known, but the data is not there yet

### 7. Preview a publish

`upload` is intentionally safe by default:

```bash
metamorph upload \
  --input ./artifacts/bonsai-candle \
  --repo your-org/Bonsai-8B-candle
```

This validates the bundle and prints the publish plan without mutating anything.

`--execute` is reserved for the future remote-write path. Today it requires `HF_TOKEN` and then stops with a not-yet-implemented error rather than performing a hidden partial upload.

## Recovery Guide

If a command fails, the recovery path should be legible from the output.

Common cases:

- `unknown-source-format`
  - Add `--from <FORMAT>` or point at a more explicit local layout.
- `planned-only`
  - The path is recognized, but execution is not implemented yet. Choose an executable target or stay in planning mode.
- `unsupported`
  - No conversion capability exists for that source/target pair yet.
- lossy opt-in failure
  - Re-run with `--allow-lossy` only if that representation change is acceptable for your workflow.
- remote `cache-miss`
  - Use a local source path or pre-populate the managed cache path Metamorph reported.
- validation failure
  - Treat the output as non-reusable until the missing file, wrong layout, or invalid safetensors artifact is fixed.
- publish credential error
  - Set `HF_TOKEN` if you are testing the execute path.
- publish not implemented
  - Stay on the preview flow; remote writes are still a future seam.

## Practical Expectations

Use Metamorph today when you need:

- a real local GGUF conversion path
- explicit compatibility reporting before execution
- deterministic cache identity
- validation-backed reusable outputs
- a publish preview surface that does not mutate remote state

Do not treat it yet as:

- an automatic Hugging Face downloader
- a generic model registry sync engine
- a fully wired publish client

For library-facing details, use [README.md](README.md). For the implementation boundaries behind these workflows, use [ARCHITECTURE.md](ARCHITECTURE.md).
