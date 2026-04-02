use std::fs::{self, File};
use std::io::{BufWriter, Write};

use assert_cmd::Command;
use candle_core::quantized::gguf_file;
use candle_core::quantized::{GgmlDType, QTensor};
use candle_core::{Device, Tensor};
use tempfile::tempdir;

#[test]
fn cache_source_reports_local_reuse() {
    let temp = tempdir().unwrap();
    let source_path = temp.path().join("fixture.gguf");
    write_fixture_gguf(&source_path);

    let output = Command::cargo_bin("metamorph")
        .unwrap()
        .env("METAMORPH_CACHE_DIR", temp.path().join("cache"))
        .args(["cache", "source", source_path.to_str().unwrap()])
        .output()
        .unwrap();

    assert!(output.status.success());

    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains("Detected format: gguf"));
    assert!(stdout.contains("Status: reused-local-path"));
    assert!(stdout.contains("Cache path:"));
    assert!(stdout.contains("Resolved path:"));
}

#[test]
fn cache_source_can_materialize_local_copy() {
    let temp = tempdir().unwrap();
    let cache_dir = temp.path().join("cache");
    let source_path = temp.path().join("fixture.gguf");
    write_fixture_gguf(&source_path);

    let output = Command::cargo_bin("metamorph")
        .unwrap()
        .env("METAMORPH_CACHE_DIR", &cache_dir)
        .args([
            "cache",
            "source",
            source_path.to_str().unwrap(),
            "--materialize",
        ])
        .output()
        .unwrap();

    assert!(output.status.success());

    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains("Status: materialized-local-copy"));
    assert!(stdout.contains("materialized a managed cache copy"));

    let cached_files = fs::read_dir(&cache_dir)
        .unwrap()
        .filter_map(Result::ok)
        .collect::<Vec<_>>();
    assert!(!cached_files.is_empty());
}

#[test]
fn cache_source_reports_remote_cache_miss() {
    let temp = tempdir().unwrap();

    let output = Command::cargo_bin("metamorph")
        .unwrap()
        .env("METAMORPH_CACHE_DIR", temp.path().join("cache"))
        .args(["cache", "source", "hf://prism-ml/Bonsai-8B-gguf@main"])
        .output()
        .unwrap();

    assert!(output.status.success());

    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains("Detected format: gguf"));
    assert!(stdout.contains("Status: cache-miss"));
    assert!(stdout.contains("remote fetch is not implemented yet"));
}

fn write_fixture_gguf(path: &std::path::Path) {
    let device = Device::Cpu;
    let tensor = Tensor::from_vec(vec![0f32, 1.0, 2.0, 3.0], (2, 2), &device).unwrap();
    let qtensor = QTensor::quantize(&tensor, GgmlDType::F32).unwrap();

    let metadata = vec![
        (
            "general.architecture",
            gguf_file::Value::String("llama".to_owned()),
        ),
        ("llama.context_length", gguf_file::Value::U32(64)),
        ("llama.embedding_length", gguf_file::Value::U32(32)),
        ("llama.block_count", gguf_file::Value::U32(1)),
        ("llama.feed_forward_length", gguf_file::Value::U32(64)),
        ("llama.attention.head_count", gguf_file::Value::U32(2)),
        ("llama.attention.head_count_kv", gguf_file::Value::U32(2)),
        ("llama.rope.freq_base", gguf_file::Value::F32(10000.0)),
        (
            "llama.attention.layer_norm_rms_epsilon",
            gguf_file::Value::F32(0.00001),
        ),
        (
            "tokenizer.ggml.model",
            gguf_file::Value::String("gpt2".to_owned()),
        ),
        (
            "tokenizer.ggml.pre",
            gguf_file::Value::String("gpt2".to_owned()),
        ),
        (
            "tokenizer.ggml.tokens",
            gguf_file::Value::Array(vec![
                gguf_file::Value::String("<unk>".to_owned()),
                gguf_file::Value::String("a".to_owned()),
                gguf_file::Value::String("b".to_owned()),
                gguf_file::Value::String("ab".to_owned()),
            ]),
        ),
        (
            "tokenizer.ggml.merges",
            gguf_file::Value::Array(vec![gguf_file::Value::String("a b".to_owned())]),
        ),
        ("tokenizer.ggml.unk_token_id", gguf_file::Value::U32(0)),
        ("tokenizer.ggml.bos_token_id", gguf_file::Value::U32(1)),
        ("tokenizer.ggml.eos_token_id", gguf_file::Value::U32(2)),
        (
            "tokenizer.ggml.add_bos_token",
            gguf_file::Value::Bool(false),
        ),
        (
            "tokenizer.ggml.add_eos_token",
            gguf_file::Value::Bool(false),
        ),
    ];
    let metadata_refs = metadata
        .iter()
        .map(|(name, value)| (*name, value))
        .collect::<Vec<_>>();

    let tensors = [("tok_embeddings.weight", qtensor)];
    let tensor_refs = tensors
        .iter()
        .map(|(name, tensor)| (*name, tensor))
        .collect::<Vec<_>>();

    let mut writer = BufWriter::new(File::create(path).unwrap());
    gguf_file::write(&mut writer, &metadata_refs, &tensor_refs).unwrap();
    writer.flush().unwrap();
}
