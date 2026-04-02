use std::fs;

use assert_cmd::Command;
use tempfile::tempdir;

#[test]
fn inspect_reports_detected_remote_format() {
    let mut command = Command::cargo_bin("metamorph").unwrap();
    let output = command
        .args(["inspect", "hf://prism-ml/Bonsai-8B-gguf@main"])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let stdout = String::from_utf8(output).unwrap();

    assert!(stdout.contains("Source: hf://prism-ml/Bonsai-8B-gguf@main"));
    assert!(stdout.contains("Detected format: gguf"));
    assert!(stdout.contains("using pinned revision `main`"));
}

#[test]
fn inspect_reports_detected_local_hf_layout() {
    let temp = tempdir().unwrap();
    fs::write(temp.path().join("config.json"), b"{}").unwrap();
    fs::write(temp.path().join("tokenizer.json"), b"{}").unwrap();
    fs::write(temp.path().join("generation_config.json"), b"{}").unwrap();
    fs::write(temp.path().join("model.safetensors"), b"weights").unwrap();

    let mut command = Command::cargo_bin("metamorph").unwrap();
    let output = command
        .args(["inspect", temp.path().to_str().unwrap()])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let stdout = String::from_utf8(output).unwrap();

    assert!(stdout.contains("Detected format: hf-safetensors"));
    assert!(stdout.contains("detected Hugging Face-style model layout"));
}

#[test]
fn inspect_reports_unknown_local_layout() {
    let temp = tempdir().unwrap();
    fs::write(temp.path().join("notes.txt"), b"unknown").unwrap();

    let mut command = Command::cargo_bin("metamorph").unwrap();
    let output = command
        .args(["inspect", temp.path().to_str().unwrap()])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let stdout = String::from_utf8(output).unwrap();

    assert!(stdout.contains("Detected format: unknown"));
    assert!(stdout.contains("format could not be inferred yet"));
}
