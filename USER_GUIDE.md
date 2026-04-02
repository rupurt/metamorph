# Metamorph User Guide

This guide describes the operator-visible purpose and workflows of **Metamorph**.

## Product Story

Metamorph is a Rust library and CLI for bridging the gap between how local AI models are published and how downstream runtimes actually need to consume them. It is for runtime integrators, infrastructure teams, and application developers who need a repeatable way to inspect, convert, validate, cache, and optionally republish model artifacts without rebuilding that glue logic inside every project.

## Core User Flows

1. First-run orientation
   - Enter the environment with `nix develop`.
   - Inspect a local path or Hugging Face source with `metamorph inspect`.
   - Confirm the detected source format before planning a conversion.
   - Run `metamorph convert --plan-only ...` to see the compatibility status, lossy flag, registered backend, and blockers before execution.
   - Use `metamorph cache source` to inspect the deterministic cache identity, local reuse path, or remote cache-miss location for that source.
2. Main repeatable loop
   - Plan or run a conversion from one format to another. Today the executable local paths are `gguf -> hf-safetensors` and `gguf -> safetensors`.
   - Validate that the output layout matches the downstream runtime expectation and only treat passing bundles as reusable outputs.
   - Preview an upload plan with `metamorph upload` before any remote write.
   - Materialize a managed local cache copy only when you ask for it.
3. Error and recovery path
   - Metamorph should fail with clear messages when a format is unsupported, a path is only planned and not executable yet, a path is lossy without opt-in, a remote source is missing from cache, or the output layout is invalid.
   - Operators should be able to recover by adjusting inputs, enabling lossy behavior explicitly, populating a cache entry, or choosing a supported target.
   - Publish execution remains preview-first; `--execute` is explicit and currently stops on missing `HF_TOKEN` or the unwired remote backend instead of performing a hidden upload.

## Personas

- Runtime integrator: embeds the library in another Rust system and needs deterministic conversion behavior.
- Model infrastructure engineer: mirrors or republishes converted artifacts for a team or organization.
- Application developer: wants a CLI that can turn an upstream model into a format their local runtime can actually load.

## Acceptance Lens

Good product behavior in Metamorph looks like this:

- The tool is explicit about what it detected and what it plans to do.
- The tool is explicit about whether a requested path is executable, planned-only, or unsupported.
- Lossy and lossless paths are clearly separated.
- The resulting artifact layout is useful to the downstream consumer, not just technically written to disk.
- Operators are not surprised by hidden cache, network, or publish behavior.
- The library API and CLI tell the same story.
