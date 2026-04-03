# metamorph

`metamorph` is a Rust library and CLI for turning model artifacts from the format they are published in into the format a downstream runtime can actually load.

It is aimed at two audiences:

- CLI operators who need a repeatable inspect -> plan -> convert -> validate -> publish-preview workflow
- Rust integrators who want to embed that workflow instead of rebuilding it in one-off scripts

## What Metamorph Does Today

Metamorph currently ships a real, end-to-end local conversion path and the supporting planning surfaces around it.

| Surface | Works today | Important notes |
| --- | --- | --- |
| `inspect` | Local paths and `hf://repo[@revision]` sources | Hugging Face inspection is heuristic today; it infers format from the repo name rather than remote file listing |
| `compatibility` / `plan` | Local paths and `hf://...` sources | Use `--from` or set `ConvertRequest { from: Some(...), .. }` when the source format cannot be inferred |
| `convert` execution | `gguf -> hf-safetensors`, `gguf -> safetensors` | Local GGUF conversion is fully wired; representative remote `hf://...` GGUF sources fetch on demand into managed cache and support explicit `--refresh` |
| `validate` | Local `safetensors` files and local `hf-safetensors` bundles | Passing outputs are marked reusable |
| `cache` | Deterministic cache identity, local materialization, remote fetch/reuse/refresh reporting | The current remote slice supports representative GGUF repos that expose exactly one GGUF artifact per revision |
| `upload` | Publish preview for local `hf-safetensors` bundles | `--execute` requires `HF_TOKEN` and still stops because remote publish execution is not wired yet |

Executable conversion backends today:

- `gguf -> hf-safetensors`
- `gguf -> safetensors`

Planned-only compatibility paths today:

- same-format relayout
- `safetensors -> hf-safetensors`

## What Metamorph Does Not Do Yet

These gaps matter for both CLI usage and embedding:

- It does not perform real remote publish execution yet.
- It does not treat every compatible path as executable.
- It does not fetch every Hugging Face repository layout; the current remote slice is limited to representative GGUF repos with one GGUF artifact per revision.
- It does not hide lossy conversion behind a silent fallback.

If a path is planned-only, unknown, unsupported, or blocked by missing lossy opt-in, Metamorph is expected to say so explicitly.

## Quick Start

Enter the dev environment:

```bash
direnv allow
nix develop
```

Inside the shell, the CLI is available as `metamorph`.

Top-level help:

```bash
metamorph --help
```

## CLI Workflows

### 1. Inspect a source

Inspect tells you what Metamorph thinks the source is and why.

```bash
metamorph inspect hf://prism-ml/Bonsai-8B-gguf@main
```

Example output:

```text
Source: hf://prism-ml/Bonsai-8B-gguf@main
Detected format: gguf
Notes:
- using pinned revision `main`
```

Use `inspect` first when you are not sure whether a path is a single GGUF file, a plain safetensors artifact, or a Hugging Face-style bundle.

### 2. Plan a conversion before executing it

`convert --plan-only` is the highest-signal CLI entry point. It shows:

- compatibility status
- resolved source format
- target format
- whether the path is lossy
- which backend would run
- blockers
- planned conversion steps

```bash
metamorph convert \
  --input hf://prism-ml/Bonsai-8B-gguf@main \
  --output ./tmp/bonsai-candle \
  --to hf-safetensors \
  --allow-lossy \
  --plan-only
```

Example output:

```text
Compatibility status: executable
Resolved source format: gguf
Requested target format: hf-safetensors
Lossy: true
Compatible backend: gguf-to-hf-safetensors
Compatibility notes:
- using pinned revision `main`
Planned conversion: gguf -> hf-safetensors
Target: ./tmp/bonsai-candle
Execution: executable
Backend: gguf-to-hf-safetensors
Lossy: true
Steps:
- fetch or read GGUF artifacts
- materialize tensors into a Hugging Face-style safetensors layout
- emit tokenizer and config files expected by downstream runtimes
- validate the output bundle
Notes:
- using pinned revision `main`
```

Important distinction:

- compatibility and planning work on local sources and `hf://...` sources
- execution can fetch a representative remote GGUF source on demand into the managed cache
- use `--from gguf` when the remote repo name is too ambiguous to infer
- use `--refresh` when you want to bypass reusable remote cache state explicitly

### 3. Execute a conversion

For local execution, point `--input` at a local GGUF file or directory and use a local output path.

```bash
metamorph convert \
  --input ./models/bonsai.gguf \
  --from gguf \
  --to hf-safetensors \
  --output ./artifacts/bonsai-candle \
  --allow-lossy
```

For plain safetensors output:

```bash
metamorph convert \
  --input ./models/bonsai.gguf \
  --from gguf \
  --to safetensors \
  --output ./artifacts/bonsai.safetensors \
  --allow-lossy
```

If the safetensors output path is a directory rather than a `.safetensors` file, Metamorph writes `model.safetensors` inside that directory.

Current execution rules:

- `--allow-lossy` is required for both executable GGUF conversion paths
- conversion outputs must be local filesystem targets
- representative remote `hf://...` GGUF sources are fetched on demand before backend execution
- `--refresh` forces remote re-fetch instead of reusing an existing managed cache artifact
- the current remote slice expects exactly one GGUF artifact at the selected repo revision
- `hf://repo` is a publish destination, not a direct conversion target

### 4. Validate an output bundle

Validation tells you whether a local artifact satisfies a reusable contract.

```bash
metamorph validate ./artifacts/bonsai-candle --format hf-safetensors
```

Use `--format` when you want to enforce a specific downstream contract rather than just infer it.

Supported validation contracts today:

- `hf-safetensors`
- `safetensors`

### 5. Inspect cache state and materialize local sources

Show the managed cache root:

```bash
metamorph cache dir
```

Inspect the deterministic cache identity for a source:

```bash
metamorph cache source hf://prism-ml/Bonsai-8B-gguf@main
```

Materialize a managed copy of a local source:

```bash
metamorph cache source ./models/bonsai.gguf --from gguf --materialize
```

Force a remote refresh instead of reuse:

```bash
metamorph cache source hf://prism-ml/Bonsai-8B-gguf@main --refresh
```

`cache source` reports:

- cache key
- cache path
- source format
- status such as `reused-local-path`, `materialized-local-copy`, `reused-managed-local-copy`, `reused-remote-cache`, `fetched-remote`, or `refreshed-remote`
- the resolved path Metamorph would use next

### 6. Preview a publish without mutating anything

`upload` is preview-first.

```bash
metamorph upload \
  --input ./artifacts/bonsai-candle \
  --repo your-org/Bonsai-8B-candle
```

This validates the local bundle, lists the artifacts that would be published, and explains the next step. It does not perform a remote write.

`--execute` is explicit:

```bash
metamorph upload \
  --input ./artifacts/bonsai-candle \
  --repo your-org/Bonsai-8B-candle \
  --execute
```

Current behavior with `--execute`:

- requires `HF_TOKEN`
- still stops with a not-yet-implemented error instead of attempting a partial upload

## CLI Command Reference

Top-level commands:

- `metamorph inspect <INPUT>`
- `metamorph convert --input <INPUT> --output <OUTPUT> --to <FORMAT> [--from <FORMAT>] [--allow-lossy] [--plan-only] [--refresh]`
- `metamorph validate <PATH> [--format <FORMAT>]`
- `metamorph upload --input <PATH> --repo <OWNER/NAME> [--execute]`
- `metamorph cache dir`
- `metamorph cache source <INPUT> [--from <FORMAT>] [--materialize] [--refresh]`

Accepted source forms:

- local file path
- local directory path
- `hf://owner/repo`
- `hf://owner/repo@revision`

Formats understood today:

- `gguf`
- `safetensors`
- `hf-safetensors`
- `mlx`

## Library Integration Guide

The library is the source of truth. The CLI is intentionally thin and mostly renders the reports returned by the library.

### Public workflow types

The main public workflow is built around these types:

- `Source`
- `Target`
- `Format`
- `ConvertRequest`
- `CompatibilityReport`
- `ConversionPlan`
- `ValidationReport`
- `CacheIdentity`
- `SourceAcquisitionReport`
- `PublishPlan`
- `PublishReport`

### Plan before executing

For integrations, the safest default is:

1. parse a `Source`
2. call `compatibility()`
3. inspect `status`, `backend`, and `blockers`
4. only call `convert()` when the path is executable and unblocked

```rust
use std::path::Path;
use std::str::FromStr;

use metamorph::{
    compatibility, convert, validate, CompatibilityStatus, ConvertRequest, Format, Source, Target,
};

fn convert_local_model() -> metamorph::Result<()> {
    let request = ConvertRequest {
        source: Source::from_str("./models/bonsai.gguf")?,
        target: Target::LocalDir("./artifacts/bonsai-candle".into()),
        from: Some(Format::Gguf),
        to: Format::HfSafetensors,
        allow_lossy: true,
        refresh_remote: false,
    };

    let report = compatibility(&request)?;
    if report.status != CompatibilityStatus::Executable || !report.blockers.is_empty() {
        eprintln!("conversion is not ready: {report:#?}");
        return Ok(());
    }

    convert(&request)?;
    let validation = validate(
        Path::new("./artifacts/bonsai-candle"),
        Some(Format::HfSafetensors),
    )?;
    assert!(validation.reusable);

    Ok(())
}
```

Why check both `status` and `blockers`:

- `status` tells you whether a compatible backend class exists
- `blockers` tells you whether the specific request is still gated by something like missing lossy opt-in or an unwired execution backend

### Treat transport and conversion as separate concerns

`compatibility()` and `plan()` can reason about remote `hf://...` sources without downloading them.

`convert()` still resolves the input through source acquisition. That means:

- local sources execute directly
- representative remote GGUF sources fetch on demand into deterministic managed cache paths
- `ConvertRequest { refresh_remote: true, .. }` forces a re-fetch instead of remote cache reuse
- broader remote repo layouts are still intentionally bounded and return explicit recovery guidance

Relevant helpers:

- `inspect()` for source detection
- `cache_identity()` for deterministic cache location
- `acquire_source()` for default reuse or fetch behavior
- `acquire_source_with_options()` when you need explicit refresh control

### Use validation as the reusable-output gate

If you intend to cache, reuse, or publish an output, validate it first.

```rust
use std::path::Path;

use metamorph::{validate, Format};

fn validate_bundle() -> metamorph::Result<()> {
    let report = validate(Path::new("./artifacts/bonsai-candle"), Some(Format::HfSafetensors))?;
    assert!(report.reusable);
    Ok(())
}
```

### Use publish planning as a safe integration surface

`plan_publish()` is useful even before remote upload execution exists:

- it validates the input bundle
- it lists the exact files that would be published
- it keeps repo naming validation in one place

## Behavioral Guarantees

The current repo contract is:

- lossy conversions require explicit opt-in
- compatibility reporting and execution dispatch come from the same registry-driven truth
- validation is part of the conversion workflow, not an optional afterthought
- caching is deterministic and inspectable
- upload is preview-first
- the library surface stays ahead of the CLI surface

## Repository Map

- `crates/metamorph/` is the reusable library
- `crates/metamorph-cli/` is the thin CLI
- [USER_GUIDE.md](USER_GUIDE.md) is the operator playbook
- [ARCHITECTURE.md](ARCHITECTURE.md) describes system boundaries
- [CODE_WALKTHROUGH.md](CODE_WALKTHROUGH.md) maps commands and features to source files
