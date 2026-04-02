use std::collections::HashMap;
use std::fs;

use assert_cmd::Command;
use candle_core::{Device, Tensor};
use tempfile::tempdir;

#[test]
fn validate_accepts_complete_hf_bundle() {
    let temp = tempdir().unwrap();
    write_valid_hf_bundle(temp.path());

    let output = Command::cargo_bin("metamorph")
        .unwrap()
        .args([
            "validate",
            temp.path().to_str().unwrap(),
            "--format",
            "hf-safetensors",
        ])
        .output()
        .unwrap();

    assert!(output.status.success());

    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains("Validated"));
    assert!(stdout.contains("hf-safetensors"));
}

#[test]
fn validate_rejects_incomplete_hf_bundle() {
    let temp = tempdir().unwrap();
    fs::write(temp.path().join("config.json"), b"{}").unwrap();
    fs::write(temp.path().join("tokenizer.json"), b"{}").unwrap();
    write_valid_safetensors(temp.path().join("model.safetensors").as_path());

    let output = Command::cargo_bin("metamorph")
        .unwrap()
        .args([
            "validate",
            temp.path().to_str().unwrap(),
            "--format",
            "hf-safetensors",
        ])
        .output()
        .unwrap();

    assert!(!output.status.success());

    let stderr = String::from_utf8(output.stderr).unwrap();
    assert!(stderr.contains("missing required file `generation_config.json`"));
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
