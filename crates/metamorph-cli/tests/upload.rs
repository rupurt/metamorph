use std::collections::HashMap;
use std::fs;

use assert_cmd::Command;
use candle_core::{Device, Tensor};
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
    assert!(stdout.contains("Executed: false"));
    assert!(stdout.contains("dry run only"));
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

    let stderr = String::from_utf8(output.stderr).unwrap();
    assert!(stderr.contains("requires credentials in `HF_TOKEN`"));
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
