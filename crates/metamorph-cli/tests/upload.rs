use std::collections::HashMap;
use std::fs;

use assert_cmd::Command;
use candle_core::{Device, Tensor};
use serde_json::json;
use tempfile::tempdir;

#[test]
fn upload_dry_run_previews_publish_plan() {
    let temp = tempdir().unwrap();
    write_valid_hf_bundle(temp.path());

    let output = Command::cargo_bin("metamorph")
        .unwrap()
        .args([
            "upload",
            "--input",
            temp.path().to_str().unwrap(),
            "--repo",
            "your-org/Bonsai-8B-candle",
        ])
        .output()
        .unwrap();

    assert!(output.status.success());

    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains("Planned publish:"));
    assert!(stdout.contains("Validated format: hf-safetensors"));
    assert!(stdout.contains("Publish status: preview"));
    assert!(stdout.contains("Executed: false"));
    assert!(stdout.contains("preview: dry run only"));
}

#[test]
fn upload_rejects_incomplete_bundle() {
    let temp = tempdir().unwrap();
    fs::write(temp.path().join("config.json"), b"{}").unwrap();
    fs::write(temp.path().join("tokenizer.json"), b"{}").unwrap();
    write_valid_safetensors(temp.path().join("model.safetensors").as_path());

    let output = Command::cargo_bin("metamorph")
        .unwrap()
        .args([
            "upload",
            "--input",
            temp.path().to_str().unwrap(),
            "--repo",
            "your-org/Bonsai-8B-candle",
        ])
        .output()
        .unwrap();

    assert!(!output.status.success());

    let stderr = String::from_utf8(output.stderr).unwrap();
    assert!(stderr.contains("missing required file `generation_config.json`"));
}

#[test]
fn upload_execute_requires_hf_token() {
    let temp = tempdir().unwrap();
    write_valid_hf_bundle(temp.path());

    let output = Command::cargo_bin("metamorph")
        .unwrap()
        .env_remove("HF_TOKEN")
        .args([
            "upload",
            "--input",
            temp.path().to_str().unwrap(),
            "--repo",
            "your-org/Bonsai-8B-candle",
            "--execute",
        ])
        .output()
        .unwrap();

    assert!(!output.status.success());

    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains("Publish status: guarded-refusal"));
    assert!(stdout.contains("requires `HF_TOKEN`"));

    let stderr = String::from_utf8(output.stderr).unwrap();
    assert!(stderr.contains("guarded refusal"));
}

#[test]
fn upload_rejects_invalid_publish_destination() {
    let temp = tempdir().unwrap();
    write_valid_hf_bundle(temp.path());

    let output = Command::cargo_bin("metamorph")
        .unwrap()
        .args([
            "upload",
            "--input",
            temp.path().to_str().unwrap(),
            "--repo",
            "invalid-destination",
        ])
        .output()
        .unwrap();

    assert!(!output.status.success());

    let stderr = String::from_utf8(output.stderr).unwrap();
    assert!(stderr.contains("expected `owner/name`"));
}

#[test]
fn upload_execute_publishes_bundle_through_mock_provider() {
    let temp = tempdir().unwrap();
    let mock_root = temp.path().join("mock");
    let repo_root = mock_root.join("your-org").join("Bonsai-8B-candle");
    write_valid_hf_bundle(temp.path());
    fs::create_dir_all(&repo_root).unwrap();

    let output = Command::cargo_bin("metamorph")
        .unwrap()
        .env("METAMORPH_HF_MOCK_ROOT", &mock_root)
        .env("HF_TOKEN", "hf_test_token")
        .args([
            "upload",
            "--input",
            temp.path().to_str().unwrap(),
            "--repo",
            "your-org/Bonsai-8B-candle",
            "--execute",
        ])
        .output()
        .unwrap();

    assert!(output.status.success());

    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains("Publish status: complete"));
    assert!(stdout.contains("[published]"));
    assert!(repo_root.join("_published").join("config.json").is_file());
    assert!(
        repo_root
            .join("_published")
            .join("model.safetensors")
            .is_file()
    );
}

#[test]
fn upload_reports_partial_mock_publish() {
    let temp = tempdir().unwrap();
    let mock_root = temp.path().join("mock");
    let repo_root = mock_root.join("your-org").join("Bonsai-8B-candle");
    write_valid_hf_bundle(temp.path());
    fs::create_dir_all(&repo_root).unwrap();
    write_mock_publish_config(
        &repo_root,
        json!({
            "interrupt_after_uploads": 2
        }),
    );

    let output = Command::cargo_bin("metamorph")
        .unwrap()
        .env("METAMORPH_HF_MOCK_ROOT", &mock_root)
        .env("HF_TOKEN", "hf_test_token")
        .args([
            "upload",
            "--input",
            temp.path().to_str().unwrap(),
            "--repo",
            "your-org/Bonsai-8B-candle",
            "--execute",
        ])
        .output()
        .unwrap();

    assert!(!output.status.success());

    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains("Publish status: partial"));
    assert!(stdout.contains("[published]"));
    assert!(stdout.contains("[pending]"));

    let stderr = String::from_utf8(output.stderr).unwrap();
    assert!(stderr.contains("partial publish"));
}

#[test]
fn upload_reports_missing_mock_destination() {
    let temp = tempdir().unwrap();
    let mock_root = temp.path().join("mock");
    write_valid_hf_bundle(temp.path());

    let output = Command::cargo_bin("metamorph")
        .unwrap()
        .env("METAMORPH_HF_MOCK_ROOT", &mock_root)
        .env("HF_TOKEN", "hf_test_token")
        .args([
            "upload",
            "--input",
            temp.path().to_str().unwrap(),
            "--repo",
            "your-org/Bonsai-8B-candle",
            "--execute",
        ])
        .output()
        .unwrap();

    assert!(!output.status.success());

    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains("Publish status: failed"));
    assert!(stdout.contains("does not exist in the mock provider"));
}

#[test]
fn upload_reports_guarded_refusal_for_mock_permission_failure() {
    let temp = tempdir().unwrap();
    let mock_root = temp.path().join("mock");
    let repo_root = mock_root.join("your-org").join("Bonsai-8B-candle");
    write_valid_hf_bundle(temp.path());
    fs::create_dir_all(&repo_root).unwrap();
    write_mock_publish_config(
        &repo_root,
        json!({
            "deny_write": true
        }),
    );

    let output = Command::cargo_bin("metamorph")
        .unwrap()
        .env("METAMORPH_HF_MOCK_ROOT", &mock_root)
        .env("HF_TOKEN", "hf_test_token")
        .args([
            "upload",
            "--input",
            temp.path().to_str().unwrap(),
            "--repo",
            "your-org/Bonsai-8B-candle",
            "--execute",
        ])
        .output()
        .unwrap();

    assert!(!output.status.success());

    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains("Publish status: guarded-refusal"));
    assert!(stdout.contains("denied write access"));
}

fn write_valid_hf_bundle(path: &std::path::Path) {
    fs::write(path.join("config.json"), b"{}").unwrap();
    fs::write(path.join("tokenizer.json"), b"{}").unwrap();
    fs::write(path.join("generation_config.json"), b"{}").unwrap();
    write_valid_safetensors(path.join("model.safetensors").as_path());
}

fn write_mock_publish_config(path: &std::path::Path, value: serde_json::Value) {
    fs::write(
        path.join(".metamorph-hf-publish.json"),
        serde_json::to_vec_pretty(&value).unwrap(),
    )
    .unwrap();
}

fn write_valid_safetensors(path: &std::path::Path) {
    let device = Device::Cpu;
    let tensor = Tensor::from_vec(vec![0f32, 1.0, 2.0, 3.0], (2, 2), &device).unwrap();
    let tensors = HashMap::from([("weight".to_owned(), tensor)]);

    candle_core::safetensors::save(&tensors, path).unwrap();
}
