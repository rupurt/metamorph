use std::collections::HashMap;
use std::fs::{self, File};
use std::io::{BufWriter, Write};
use std::path::Path;

use assert_cmd::Command;
use candle_core::quantized::gguf_file;
use candle_core::quantized::{GgmlDType, QTensor};
use candle_core::{Device, Tensor};
use serde_json::Value as JsonValue;
use serde_json::json;
use tempfile::tempdir;

#[test]
fn convert_executes_local_gguf_backend() {
    let temp = tempdir().unwrap();
    let source_path = temp.path().join("fixture.gguf");
    let output_path = temp.path().join("bundle");

    write_fixture_gguf(&source_path);

    let output = Command::cargo_bin("metamorph")
        .unwrap()
        .args([
            "convert",
            "--input",
            source_path.to_str().unwrap(),
            "--output",
            output_path.to_str().unwrap(),
            "--to",
            "hf-safetensors",
            "--allow-lossy",
        ])
        .output()
        .unwrap();

    assert!(output.status.success());

    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains("Compatibility status: executable"));
    assert!(stdout.contains("Compatible backend: gguf-to-hf-safetensors"));
    assert!(stdout.contains("Planned conversion: gguf -> hf-safetensors"));
    assert!(stdout.contains("Execution: executable"));
    assert!(stdout.contains("Backend: gguf-to-hf-safetensors"));
    assert!(stdout.contains("Lossy: true"));
    assert!(stdout.contains(&format!("Converted bundle: {}", output_path.display())));

    for required in [
        "config.json",
        "tokenizer.json",
        "generation_config.json",
        "model.safetensors",
    ] {
        assert!(output_path.join(required).is_file());
    }

    let config: JsonValue =
        serde_json::from_slice(&fs::read(output_path.join("config.json")).unwrap()).unwrap();
    assert_eq!(config["model_type"], "llama");
}

#[test]
fn convert_requires_allow_lossy_for_cli() {
    let temp = tempdir().unwrap();
    let source_path = temp.path().join("fixture.gguf");
    let output_path = temp.path().join("bundle");

    write_fixture_gguf(&source_path);

    let output = Command::cargo_bin("metamorph")
        .unwrap()
        .args([
            "convert",
            "--input",
            source_path.to_str().unwrap(),
            "--output",
            output_path.to_str().unwrap(),
            "--to",
            "hf-safetensors",
        ])
        .output()
        .unwrap();

    assert!(!output.status.success());

    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains("Compatibility status: executable"));
    assert!(stdout.contains("Blockers:"));
    assert!(stdout.contains("lossy conversion requires explicit opt-in"));

    let stderr = String::from_utf8(output.stderr).unwrap();
    assert!(stderr.contains("lossy conversion requires explicit opt-in: gguf -> hf-safetensors"));
}

#[test]
fn convert_executes_local_gguf_to_safetensors_backend() {
    let temp = tempdir().unwrap();
    let source_path = temp.path().join("fixture.gguf");
    let output_path = temp.path().join("weights.safetensors");

    write_fixture_gguf(&source_path);

    let output = Command::cargo_bin("metamorph")
        .unwrap()
        .args([
            "convert",
            "--input",
            source_path.to_str().unwrap(),
            "--output",
            output_path.to_str().unwrap(),
            "--to",
            "safetensors",
            "--allow-lossy",
        ])
        .output()
        .unwrap();

    assert!(output.status.success());

    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains("Compatibility status: executable"));
    assert!(stdout.contains("Compatible backend: gguf-to-safetensors"));
    assert!(stdout.contains("Planned conversion: gguf -> safetensors"));
    assert!(stdout.contains("Execution: executable"));
    assert!(stdout.contains("Backend: gguf-to-safetensors"));
    assert!(stdout.contains(&format!("Converted bundle: {}", output_path.display())));
    assert!(output_path.is_file());
}

#[test]
fn convert_executes_local_safetensors_relayout() {
    let temp = tempdir().unwrap();
    let source_dir = temp.path().join("source");
    let output_dir = temp.path().join("out");
    fs::create_dir_all(&source_dir).unwrap();
    write_valid_safetensors(source_dir.join("weights.safetensors").as_path());

    let output = Command::cargo_bin("metamorph")
        .unwrap()
        .args([
            "convert",
            "--input",
            source_dir.to_str().unwrap(),
            "--output",
            output_dir.to_str().unwrap(),
            "--to",
            "safetensors",
        ])
        .output()
        .unwrap();

    assert!(output.status.success());

    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains("Compatibility status: executable"));
    assert!(stdout.contains("Compatible backend: safetensors-relayout"));
    assert!(stdout.contains("Execution: executable"));
    assert!(stdout.contains(&format!("Converted bundle: {}", output_dir.display())));
    assert!(output_dir.join("weights.safetensors").is_file());
}

#[test]
fn convert_executes_local_hf_bundle_relayout() {
    let temp = tempdir().unwrap();
    let source_dir = temp.path().join("source");
    let output_dir = temp.path().join("out");
    fs::create_dir_all(&source_dir).unwrap();
    write_valid_hf_bundle(&source_dir);
    fs::write(source_dir.join("tokenizer_config.json"), b"{}").unwrap();

    let output = Command::cargo_bin("metamorph")
        .unwrap()
        .args([
            "convert",
            "--input",
            source_dir.to_str().unwrap(),
            "--output",
            output_dir.to_str().unwrap(),
            "--to",
            "hf-safetensors",
        ])
        .output()
        .unwrap();

    assert!(output.status.success());

    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains("Compatibility status: executable"));
    assert!(stdout.contains("Compatible backend: hf-safetensors-relayout"));
    assert!(stdout.contains("Execution: executable"));
    assert!(output_dir.join("model.safetensors").is_file());
    assert!(output_dir.join("tokenizer_config.json").is_file());
}

#[test]
fn convert_executes_metadata_backed_bundle_materialization() {
    let temp = tempdir().unwrap();
    let source_dir = temp.path().join("source");
    let output_dir = temp.path().join("out");
    fs::create_dir_all(&source_dir).unwrap();
    write_valid_safetensors(source_dir.join("weights.safetensors").as_path());
    fs::write(source_dir.join("config.json"), b"{}").unwrap();
    fs::write(source_dir.join("tokenizer.json"), b"{}").unwrap();

    let output = Command::cargo_bin("metamorph")
        .unwrap()
        .args([
            "convert",
            "--input",
            source_dir.to_str().unwrap(),
            "--output",
            output_dir.to_str().unwrap(),
            "--to",
            "hf-safetensors",
        ])
        .output()
        .unwrap();

    assert!(output.status.success());

    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains("Compatibility status: executable"));
    assert!(stdout.contains("Compatible backend: safetensors-to-hf-safetensors"));
    assert!(stdout.contains("Execution: executable"));
    assert!(stdout.contains("empty generation config"));
    assert!(output_dir.join("model.safetensors").is_file());
    assert!(output_dir.join("generation_config.json").is_file());
}

#[test]
fn convert_reports_blocked_bundle_materialization() {
    let temp = tempdir().unwrap();
    let source_dir = temp.path().join("source");
    let output_dir = temp.path().join("out");
    fs::create_dir_all(&source_dir).unwrap();
    write_valid_safetensors(source_dir.join("weights.safetensors").as_path());

    let output = Command::cargo_bin("metamorph")
        .unwrap()
        .args([
            "convert",
            "--input",
            source_dir.to_str().unwrap(),
            "--output",
            output_dir.to_str().unwrap(),
            "--to",
            "hf-safetensors",
        ])
        .output()
        .unwrap();

    assert!(!output.status.success());

    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains("Compatibility status: executable"));
    assert!(stdout.contains("Compatible backend: safetensors-to-hf-safetensors"));
    assert!(stdout.contains("Blockers:"));
    assert!(stdout.contains("missing `config.json`"));
    assert!(stdout.contains("missing `tokenizer.json`"));

    let stderr = String::from_utf8(output.stderr).unwrap();
    assert!(stderr.contains("missing `config.json`"));
}

#[test]
fn convert_reports_local_only_blocker_for_remote_relayout() {
    let temp = tempdir().unwrap();
    let output_path = temp.path().join("bundle");

    let output = Command::cargo_bin("metamorph")
        .unwrap()
        .args([
            "convert",
            "--input",
            "hf://example/model-safetensors",
            "--output",
            output_path.to_str().unwrap(),
            "--from",
            "safetensors",
            "--to",
            "safetensors",
            "--plan-only",
        ])
        .output()
        .unwrap();

    assert!(output.status.success());

    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains("Compatibility status: executable"));
    assert!(stdout.contains("Compatible backend: safetensors-relayout"));
    assert!(stdout.contains("requires a local source path"));
}

#[test]
fn convert_plan_only_reports_unsupported_path_reasoning() {
    let temp = tempdir().unwrap();
    let output_path = temp.path().join("bundle");

    let output = Command::cargo_bin("metamorph")
        .unwrap()
        .args([
            "convert",
            "--input",
            "hf://example/model-safetensors",
            "--output",
            output_path.to_str().unwrap(),
            "--from",
            "safetensors",
            "--to",
            "mlx",
            "--plan-only",
        ])
        .output()
        .unwrap();

    assert!(!output.status.success());

    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains("Compatibility status: unsupported"));
    assert!(stdout.contains("Blockers:"));
    assert!(stdout.contains("no registered conversion capability exists for `safetensors -> mlx`"));

    let stderr = String::from_utf8(output.stderr).unwrap();
    assert!(stderr.contains("unsupported conversion path: safetensors -> mlx"));
}

#[test]
fn convert_executes_remote_gguf_backend_after_fetch() {
    let temp = tempdir().unwrap();
    let cache_dir = temp.path().join("cache");
    let mock_root = temp.path().join("mock");
    let output_path = temp.path().join("bundle");

    write_mock_remote_gguf_repo(
        &mock_root,
        "prism-ml/Bonsai-8B-gguf",
        "main",
        "Bonsai-8B-Q4.gguf",
        Some("sha-main-001"),
    );

    let output = Command::cargo_bin("metamorph")
        .unwrap()
        .env("METAMORPH_CACHE_DIR", &cache_dir)
        .env("METAMORPH_HF_MOCK_ROOT", &mock_root)
        .args([
            "convert",
            "--input",
            "hf://prism-ml/Bonsai-8B-gguf@main",
            "--output",
            output_path.to_str().unwrap(),
            "--from",
            "gguf",
            "--to",
            "hf-safetensors",
            "--allow-lossy",
        ])
        .output()
        .unwrap();

    assert!(output.status.success());

    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains("Compatibility status: executable"));
    assert!(stdout.contains("Acquisition status: fetched-remote"));
    assert!(stdout.contains("Acquisition path:"));
    assert!(stdout.contains("fetched remote artifact"));
    assert!(stdout.contains(&format!("Converted bundle: {}", output_path.display())));

    for required in [
        "config.json",
        "tokenizer.json",
        "generation_config.json",
        "model.safetensors",
    ] {
        assert!(output_path.join(required).is_file());
    }
}

#[test]
fn convert_refreshes_remote_source_explicitly() {
    let temp = tempdir().unwrap();
    let cache_dir = temp.path().join("cache");
    let mock_root = temp.path().join("mock");
    let output_path = temp.path().join("bundle");

    write_mock_remote_gguf_repo(
        &mock_root,
        "prism-ml/Bonsai-8B-gguf",
        "main",
        "Bonsai-8B-Q4.gguf",
        Some("sha-main-001"),
    );

    let initial = Command::cargo_bin("metamorph")
        .unwrap()
        .env("METAMORPH_CACHE_DIR", &cache_dir)
        .env("METAMORPH_HF_MOCK_ROOT", &mock_root)
        .args([
            "convert",
            "--input",
            "hf://prism-ml/Bonsai-8B-gguf@main",
            "--output",
            output_path.to_str().unwrap(),
            "--from",
            "gguf",
            "--to",
            "hf-safetensors",
            "--allow-lossy",
        ])
        .output()
        .unwrap();
    assert!(initial.status.success());

    write_mock_remote_gguf_repo(
        &mock_root,
        "prism-ml/Bonsai-8B-gguf",
        "main",
        "Bonsai-8B-Q4.gguf",
        Some("sha-main-002"),
    );
    let refreshed = Command::cargo_bin("metamorph")
        .unwrap()
        .env("METAMORPH_CACHE_DIR", &cache_dir)
        .env("METAMORPH_HF_MOCK_ROOT", &mock_root)
        .args([
            "convert",
            "--input",
            "hf://prism-ml/Bonsai-8B-gguf@main",
            "--output",
            output_path.to_str().unwrap(),
            "--from",
            "gguf",
            "--to",
            "hf-safetensors",
            "--allow-lossy",
            "--refresh",
        ])
        .output()
        .unwrap();

    assert!(refreshed.status.success());

    let stdout = String::from_utf8(refreshed.stdout).unwrap();
    assert!(stdout.contains("Acquisition status: refreshed-remote"));
    assert!(stdout.contains("resolved revision `sha-main-002`"));
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

fn write_valid_hf_bundle(path: &std::path::Path) {
    fs::write(path.join("config.json"), b"{}").unwrap();
    fs::write(path.join("tokenizer.json"), b"{}").unwrap();
    fs::write(path.join("generation_config.json"), b"{}").unwrap();
    write_valid_safetensors(path.join("model.safetensors").as_path());
}

fn write_valid_safetensors(path: &std::path::Path) {
    let device = Device::Cpu;
    let tensor = Tensor::from_vec(vec![0f32, 1.0, 2.0, 3.0], (2, 2), &device).unwrap();
    let tensors = HashMap::from([("weight".to_owned(), tensor)]);

    candle_core::safetensors::save(&tensors, path).unwrap();
}

fn write_mock_remote_gguf_repo(
    root: &Path,
    repo: &str,
    revision: &str,
    artifact_name: &str,
    resolved_revision: Option<&str>,
) {
    let repo_root = root.join(repo).join(revision);
    fs::create_dir_all(&repo_root).unwrap();
    write_fixture_gguf(&repo_root.join(artifact_name));

    if let Some(resolved_revision) = resolved_revision {
        fs::write(
            repo_root.join(".metamorph-hf.json"),
            serde_json::to_vec_pretty(&json!({ "resolved_revision": resolved_revision })).unwrap(),
        )
        .unwrap();
    }
}
