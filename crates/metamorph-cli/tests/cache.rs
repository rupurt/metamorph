use std::fs::{self, File};
use std::io::{BufWriter, Write};
use std::path::Path;

use assert_cmd::Command;
use candle_core::quantized::gguf_file;
use candle_core::quantized::{GgmlDType, QTensor};
use candle_core::{Device, Tensor};
use serde_json::json;
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
fn cache_source_fetches_and_reuses_remote_artifact() {
    let temp = tempdir().unwrap();
    let cache_dir = temp.path().join("cache");
    let mock_root = temp.path().join("mock");

    write_mock_remote_gguf_repo(
        &mock_root,
        "prism-ml/Bonsai-8B-gguf",
        "main",
        "Bonsai-8B-Q4.gguf",
        Some("sha-main-001"),
    );

    let first = Command::cargo_bin("metamorph")
        .unwrap()
        .env("METAMORPH_CACHE_DIR", &cache_dir)
        .env("METAMORPH_HF_MOCK_ROOT", &mock_root)
        .args(["cache", "source", "hf://prism-ml/Bonsai-8B-gguf@main"])
        .output()
        .unwrap();

    assert!(first.status.success());

    let first_stdout = String::from_utf8(first.stdout).unwrap();
    assert!(first_stdout.contains("Detected format: gguf"));
    assert!(first_stdout.contains("Status: fetched-remote"));
    assert!(first_stdout.contains("Resolved path:"));
    assert!(first_stdout.contains("fetched remote artifact"));

    let second = Command::cargo_bin("metamorph")
        .unwrap()
        .env("METAMORPH_CACHE_DIR", &cache_dir)
        .env("METAMORPH_HF_MOCK_ROOT", &mock_root)
        .args(["cache", "source", "hf://prism-ml/Bonsai-8B-gguf@main"])
        .output()
        .unwrap();

    assert!(second.status.success());

    let second_stdout = String::from_utf8(second.stdout).unwrap();
    assert!(second_stdout.contains("Status: reused-remote-cache"));
    assert!(second_stdout.contains("reused cached remote artifact"));
}

#[test]
fn cache_source_refreshes_remote_artifact_explicitly() {
    let temp = tempdir().unwrap();
    let cache_dir = temp.path().join("cache");
    let mock_root = temp.path().join("mock");

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
        .args(["cache", "source", "hf://prism-ml/Bonsai-8B-gguf@main"])
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
            "cache",
            "source",
            "hf://prism-ml/Bonsai-8B-gguf@main",
            "--refresh",
        ])
        .output()
        .unwrap();

    assert!(refreshed.status.success());

    let stdout = String::from_utf8(refreshed.stdout).unwrap();
    assert!(stdout.contains("Status: refreshed-remote"));
    assert!(stdout.contains("resolved revision `sha-main-002`"));
}

#[test]
fn cache_source_reports_remote_auth_failure() {
    let temp = tempdir().unwrap();
    let cache_dir = temp.path().join("cache");
    let mock_root = temp.path().join("mock");

    write_mock_remote_gguf_repo(
        &mock_root,
        "private-org/Secret-gguf",
        "main",
        "Secret.gguf",
        Some("sha-private-001"),
    );
    write_mock_remote_config(
        &mock_root,
        "private-org/Secret-gguf",
        "main",
        json!({ "require_token": true }),
    );

    let output = Command::cargo_bin("metamorph")
        .unwrap()
        .env("METAMORPH_CACHE_DIR", &cache_dir)
        .env("METAMORPH_HF_MOCK_ROOT", &mock_root)
        .env_remove("HF_TOKEN")
        .args(["cache", "source", "hf://private-org/Secret-gguf@main"])
        .output()
        .unwrap();

    assert!(!output.status.success());

    let stderr = String::from_utf8(output.stderr).unwrap();
    assert!(stderr.contains("requires credentials in `HF_TOKEN`"));
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
        write_mock_remote_config(
            root,
            repo,
            revision,
            json!({ "resolved_revision": resolved_revision }),
        );
    }
}

fn write_mock_remote_config(root: &Path, repo: &str, revision: &str, config: serde_json::Value) {
    let repo_root = root.join(repo).join(revision);
    fs::create_dir_all(&repo_root).unwrap();
    fs::write(
        repo_root.join(".metamorph-hf.json"),
        serde_json::to_vec_pretty(&config).unwrap(),
    )
    .unwrap();
}
