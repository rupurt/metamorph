# metamorph

`metamorph` is a Rust library and CLI for converting local AI model artifacts between runtime-specific formats.

It exists for the annoying gap between how models are published and how local runtimes actually consume them.
One project ships `gguf`, another expects Hugging Face-style `config.json + tokenizer.json + safetensors`,
and a third wants a completely different layout. `metamorph` is the glue.

## Why this exists

Local-first AI stacks keep running into the same problem:

- model publishers optimize for one runtime
- application runtimes support a different format
- the missing conversion step gets rebuilt ad hoc in every project

`metamorph` is meant to make that conversion step explicit, reusable, and automatable.

The initial motivating case is straightforward:

- a model is published in a runtime-oriented format such as `gguf`
- a Rust application using Candle or another safetensors-based loader needs a different layout
- the application should be able to download, convert, cache, and use the result without bespoke one-off scripts

## Project goals

- Provide a solid Rust library for model format conversion and artifact normalization.
- Ship a CLI that can download source artifacts, convert them, validate the result, and optionally upload the output.
- Make conversions composable so apps can embed `metamorph` instead of shelling out.
- Preserve as much metadata as possible across conversions.
- Standardize model layouts that local runtimes can actually consume.
- Make caching, resumability, and deterministic outputs first-class.

## Non-goals

- Training or fine-tuning models.
- Serving inference directly.
- Re-implementing every runtime loader.
- Pretending all conversions are lossless.

Some transformations are format changes. Others are material changes in representation, quantization, or precision.
`metamorph` should make that distinction obvious.

## Target users

- Rust applications that need to ingest models dynamically
- local AI runtimes that want a single conversion layer
- developers migrating models between ecosystems
- infrastructure teams publishing internal model mirrors

## What `metamorph` should do

### Library

The library should expose a structured conversion pipeline:

- fetch or read source artifacts
- inspect and identify the source format
- construct a conversion plan
- transform weights, config, tokenizer, and metadata
- validate the output layout
- write to disk, cache, or another destination

At a high level, the library looks like this today:

```rust
use metamorph::{ConvertRequest, Format, Source, Target};

let result = metamorph::convert(ConvertRequest {
    source: "hf://prism-ml/Bonsai-8B-gguf@main".parse::<Source>()?,
    target: Target::LocalDir("./cache/bonsai-candle".into()),
    from: Some(Format::Gguf),
    to: Format::HfSafetensors,
    allow_lossy: true,
})?;
```

### CLI

The CLI is a thin, scriptable layer on top of the library.

Current commands:

- `metamorph inspect`
- `metamorph convert`
- `metamorph validate`
- `metamorph upload`
- `metamorph cache`

Current examples:

```bash
metamorph inspect hf://prism-ml/Bonsai-8B-gguf

metamorph cache source hf://prism-ml/Bonsai-8B-gguf@main

metamorph cache source ./fixtures/bonsai.gguf --materialize

metamorph convert \
  --from gguf \
  --to hf-safetensors \
  --input ./fixtures/bonsai.gguf \
  --output ./artifacts/bonsai-candle \
  --allow-lossy

metamorph validate ./artifacts/bonsai-candle --format hf-safetensors

metamorph upload \
  --input ./artifacts/bonsai-candle \
  --repo your-org/Bonsai-8B-candle

metamorph upload \
  --input ./artifacts/bonsai-candle \
  --repo your-org/Bonsai-8B-candle \
  --execute
```

`upload` is preview-first: without `--execute` it validates the bundle and renders the publish plan without mutating anything. With `--execute`, the CLI currently requires `HF_TOKEN` and then stops with an explicit not-yet-implemented message rather than attempting a hidden or partial remote write.

## Core concepts

### Source

Where model artifacts come from:

- local directories
- local files
- Hugging Face repositories
- internal registries or object stores

### Format

How model data is represented:

- `gguf`
- `safetensors`
- `mlx`
- runtime-specific layouts built on top of those formats

### Layout

How artifacts are organized for a consumer:

- single-file model packages
- Hugging Face repository structure
- local runtime cache directories
- application-specific bundles

### Conversion plan

A concrete, inspectable description of what will happen:

- direct repack
- metadata rewrite
- tensor rename or reshape
- dequantization or requantization
- tokenizer/config normalization

## Guiding principles

- Be explicit about lossy vs lossless conversions.
- Separate artifact transport from tensor transformation.
- Treat validation as part of conversion, not an afterthought.
- Make the library composable before making the CLI fancy.
- Prefer deterministic outputs so caches and uploads are stable.
- Keep runtime-specific adapters at the edges.

## Initial format priorities

The first useful version does not need to support everything.
It should solve one real workflow end to end.

Suggested first targets:

1. Inspect Hugging Face and local artifacts.
2. Convert local `gguf` models into a Candle-friendly Hugging Face-style safetensors layout.
3. Validate that the output directory contains the files a downstream runtime expects and treat passing bundles as reusable outputs.
4. Expose explicit cache identity and publish preview surfaces before wiring remote fetch and write backends.

That gives downstream applications one dependable path:

- download
- convert
- cache
- run

## Motivating example

An application may support only Candle-style local models with:

- `config.json`
- `tokenizer.json`
- `generation_config.json`
- `model.safetensors` or sharded safetensors

But an upstream publisher may ship only:

- `gguf`
- runtime-specific quantization
- metadata tuned for another loader

`metamorph` should let the app say:

```text
I know how to run format X.
The model exists in format Y.
Fetch it, convert it, cache it, validate it, and hand me a path I can load.
```

## Planned library surface

The library should likely grow around a few stable building blocks:

- `source`: local and remote artifact acquisition
- `format`: format detection and descriptors
- `plan`: conversion planning and compatibility checks
- `transform`: tensor and metadata transforms
- `validate`: output verification
- `cache`: reproducible local artifact storage
- `publish`: upload or mirror operations

## Error model

`metamorph` should fail clearly and early.

Examples:

- unsupported source format
- unsupported conversion path
- required metadata missing
- conversion would be lossy without explicit opt-in
- output layout invalid for the requested target

## What success looks like

Success is not "supports every model format."

Success is:

- another Rust project can depend on `metamorph` as a library
- the CLI can inspect, cache-plan, convert, validate, and preview publish behavior without hiding lossy or network-sensitive steps
- downstream runtimes stop carrying bespoke conversion code
- adding a new conversion path feels incremental instead of invasive

## Roadmap

### Phase 1

- crate structure
- format inspection
- local and Hugging Face sources
- basic conversion planning
- Candle-oriented safetensors layout emission

### Phase 2

- remote fetch backends for cache misses on `hf://` sources
- remote upload execution once credentials, policy gates, and licensing review flow are explicit
- richer metadata preservation

### Phase 3

- plugin-style conversion backends
- more runtime targets
- smarter compatibility reporting

## Status

This repository now ships:

- source inspection for local paths and `hf://` references
- a working local `gguf -> hf-safetensors` conversion path
- validation that marks complete Hugging Face-style bundles as reusable outputs
- deterministic cache identity and source-acquisition reporting through `metamorph cache source`
- preview-first upload planning with explicit `--execute` gating

Remote fetch and remote publish execution are still planned. Current CLI behavior reports cache misses and publish prerequisites explicitly instead of performing hidden network side effects.

If you are building a runtime that needs to bridge model ecosystems, that is the exact problem `metamorph` is for.
