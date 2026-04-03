use std::collections::HashMap;
use std::fs::{self, File};
use std::io::{BufWriter, Write};
use std::path::Path;
use std::str::FromStr;
use std::sync::Mutex;

use candle_core::quantized::gguf_file;
use candle_core::quantized::{GgmlDType, QTensor};
use candle_core::{Device, Tensor};
use serde_json::Value as JsonValue;
use serde_json::json;
use tempfile::tempdir;

use crate::error::HF_TOKEN_ENV;
use crate::remote::MOCK_ROOT_ENV;
use crate::{
    CompatibilityStatus, ConvertRequest, ExecutionSupport, Format, PublishRequest, Source,
    SourceAcquisitionOptions, SourceAcquisitionOutcome, Target, acquire_source,
    acquire_source_with_options, cache_identity, compatibility, convert, inspect, plan,
    plan_publish, publish, validate,
};

static ENV_LOCK: Mutex<()> = Mutex::new(());

#[test]
fn parses_hugging_face_source_with_revision() {
    let source = Source::from_str("hf://prism-ml/Bonsai-8B-gguf@main").unwrap();

    assert_eq!(
        source,
        Source::HuggingFace {
            repo: "prism-ml/Bonsai-8B-gguf".to_owned(),
            revision: Some("main".to_owned()),
        }
    );
}

#[test]
fn compatibility_reports_lossy_registered_backend() {
    let request = ConvertRequest {
        source: Source::from_str("hf://prism-ml/Bonsai-8B-gguf").unwrap(),
        target: Target::LocalDir("target/out.safetensors".into()),
        from: Some(Format::Gguf),
        to: Format::Safetensors,
        allow_lossy: false,
        refresh_remote: false,
    };

    let report = compatibility(&request).unwrap();

    assert_eq!(report.status, CompatibilityStatus::Executable);
    assert!(report.lossy);
    assert_eq!(report.backend.as_deref(), Some("gguf-to-safetensors"));
    assert!(
        report
            .blockers
            .iter()
            .any(|blocker| blocker.contains("lossy conversion requires explicit opt-in"))
    );
}

#[test]
fn compatibility_reports_unsupported_path() {
    let request = ConvertRequest {
        source: Source::from_str("hf://example/model-safetensors").unwrap(),
        target: Target::LocalDir("target/out".into()),
        from: Some(Format::Safetensors),
        to: Format::Mlx,
        allow_lossy: false,
        refresh_remote: false,
    };

    let report = compatibility(&request).unwrap();

    assert_eq!(report.status, CompatibilityStatus::Unsupported);
    assert!(
        report
            .blockers
            .iter()
            .any(|blocker| blocker.contains("no registered conversion capability"))
    );
}

#[test]
fn requires_lossy_opt_in_for_gguf_to_hf_safetensors() {
    let request = ConvertRequest {
        source: Source::from_str("hf://prism-ml/Bonsai-8B-gguf").unwrap(),
        target: Target::LocalDir("target/out".into()),
        from: Some(Format::Gguf),
        to: Format::HfSafetensors,
        allow_lossy: false,
        refresh_remote: false,
    };

    let error = plan(&request).unwrap_err();
    assert!(
        error
            .to_string()
            .contains("lossy conversion requires explicit opt-in")
    );
}

#[test]
fn rejects_unsupported_conversion_paths() {
    let request = ConvertRequest {
        source: Source::from_str("hf://example/model-safetensors").unwrap(),
        target: Target::LocalDir("target/out".into()),
        from: Some(Format::Safetensors),
        to: Format::Mlx,
        allow_lossy: false,
        refresh_remote: false,
    };

    let error = plan(&request).unwrap_err();
    assert!(error.to_string().contains("unsupported conversion path"));
}

#[test]
fn plans_same_format_relayout_without_loss() {
    let request = ConvertRequest {
        source: Source::from_str("hf://example/model-safetensors").unwrap(),
        target: Target::LocalDir("target/out".into()),
        from: Some(Format::Safetensors),
        to: Format::Safetensors,
        allow_lossy: false,
        refresh_remote: false,
    };

    let plan = plan(&request).unwrap();

    assert!(!plan.lossy);
    assert_eq!(plan.source_format, Format::Safetensors);
    assert_eq!(plan.target_format, Format::Safetensors);
    assert_eq!(plan.execution, ExecutionSupport::PlannedOnly);
    assert_eq!(plan.backend.as_deref(), Some("same-format-relayout"));
}

#[test]
fn inspects_local_gguf_file() {
    let temp = tempdir().unwrap();
    let path = temp.path().join("model.gguf");
    fs::write(&path, b"gguf").unwrap();

    let report = inspect(&Source::LocalPath(path)).unwrap();

    assert_eq!(report.detected_format, Some(Format::Gguf));
    assert!(report.notes.is_empty());
}

#[test]
fn inspects_hf_style_local_directory() {
    let temp = tempdir().unwrap();
    fs::write(temp.path().join("config.json"), b"{}").unwrap();
    fs::write(temp.path().join("tokenizer.json"), b"{}").unwrap();
    fs::write(temp.path().join("generation_config.json"), b"{}").unwrap();
    fs::write(temp.path().join("model.safetensors"), b"weights").unwrap();

    let report = inspect(&Source::LocalPath(temp.path().to_path_buf())).unwrap();

    assert_eq!(report.detected_format, Some(Format::HfSafetensors));
    assert!(
        report
            .notes
            .contains(&"detected Hugging Face-style model layout".to_owned())
    );
}

#[test]
fn inspects_partial_safetensors_directory_as_plain_safetensors() {
    let temp = tempdir().unwrap();
    fs::write(temp.path().join("weights.safetensors"), b"weights").unwrap();

    let report = inspect(&Source::LocalPath(temp.path().to_path_buf())).unwrap();

    assert_eq!(report.detected_format, Some(Format::Safetensors));
    assert!(
        report
            .notes
            .contains(&"found safetensors files but not a complete Hugging Face layout".to_owned())
    );
}

#[test]
fn inspects_hugging_face_source_and_reports_revision_note() {
    let source = Source::from_str("hf://prism-ml/Bonsai-8B-gguf@main").unwrap();

    let report = inspect(&source).unwrap();

    assert_eq!(report.detected_format, Some(Format::Gguf));
    assert!(
        report
            .notes
            .contains(&"using pinned revision `main`".to_owned())
    );
}

#[test]
fn reports_unknown_when_hugging_face_source_cannot_be_inferred() {
    let source = Source::from_str("hf://example/model").unwrap();

    let report = inspect(&source).unwrap();

    assert_eq!(report.detected_format, None);
    assert!(
        report
            .notes
            .contains(&"format could not be inferred yet".to_owned())
    );
}

#[test]
fn cache_identity_is_deterministic_for_local_source() {
    let _env_guard = ENV_LOCK.lock().unwrap();
    let temp = tempdir().unwrap();
    let cache_root = temp.path().join("cache");
    let source_path = temp.path().join("fixture.gguf");
    write_fixture_gguf(&source_path);

    unsafe { std::env::set_var("METAMORPH_CACHE_DIR", &cache_root) };
    let first = cache_identity(&Source::LocalPath(source_path.clone()), None).unwrap();
    let second = cache_identity(&Source::LocalPath(source_path), None).unwrap();
    unsafe { std::env::remove_var("METAMORPH_CACHE_DIR") };

    assert_eq!(first.key, second.key);
    assert_eq!(first.path, second.path);
    assert_eq!(first.source_format, Some(Format::Gguf));
}

#[test]
fn cache_identity_changes_with_hf_revision() {
    let main = cache_identity(
        &Source::from_str("hf://prism-ml/Bonsai-8B-gguf@main").unwrap(),
        None,
    )
    .unwrap();
    let snapshot = cache_identity(
        &Source::from_str("hf://prism-ml/Bonsai-8B-gguf@snapshot").unwrap(),
        None,
    )
    .unwrap();

    assert_ne!(main.key, snapshot.key);
    assert_ne!(main.path, snapshot.path);
}

#[test]
fn acquire_source_reuses_existing_local_path() {
    let temp = tempdir().unwrap();
    let source_path = temp.path().join("fixture.gguf");
    write_fixture_gguf(&source_path);

    let report = acquire_source(&Source::LocalPath(source_path.clone()), None, false).unwrap();

    assert_eq!(report.outcome, SourceAcquisitionOutcome::ReusedLocalPath);
    assert_eq!(report.detected_format, Some(Format::Gguf));
    assert_eq!(report.resolved_path, fs::canonicalize(source_path).unwrap());
}

#[test]
fn acquire_source_can_materialize_local_copy() {
    let _env_guard = ENV_LOCK.lock().unwrap();
    let temp = tempdir().unwrap();
    let cache_root = temp.path().join("cache");
    let source_path = temp.path().join("fixture.gguf");
    write_fixture_gguf(&source_path);

    unsafe { std::env::set_var("METAMORPH_CACHE_DIR", &cache_root) };
    let report = acquire_source(&Source::LocalPath(source_path), None, true).unwrap();
    unsafe { std::env::remove_var("METAMORPH_CACHE_DIR") };

    assert_eq!(
        report.outcome,
        SourceAcquisitionOutcome::MaterializedLocalCopy
    );
    assert!(report.resolved_path.is_file());
    assert!(report.cache_identity.path.is_dir());
}

#[test]
fn acquire_source_fetches_remote_source_on_miss() {
    let _env_guard = ENV_LOCK.lock().unwrap();
    let temp = tempdir().unwrap();
    let cache_root = temp.path().join("cache");
    let mock_root = temp.path().join("mock");
    let source = Source::from_str("hf://prism-ml/Bonsai-8B-gguf@main").unwrap();

    write_mock_remote_gguf_repo(
        &mock_root,
        "prism-ml/Bonsai-8B-gguf",
        "main",
        "Bonsai-8B-Q4.gguf",
        Some("sha-main-001"),
    );

    unsafe {
        std::env::set_var("METAMORPH_CACHE_DIR", &cache_root);
        std::env::set_var(MOCK_ROOT_ENV, &mock_root);
    }
    let report = acquire_source(&source, None, false).unwrap();
    unsafe {
        std::env::remove_var("METAMORPH_CACHE_DIR");
        std::env::remove_var(MOCK_ROOT_ENV);
    }

    assert_eq!(
        report.outcome,
        SourceAcquisitionOutcome::FetchedRemoteArtifact
    );
    assert!(report.resolved_path.is_file());
    assert_eq!(
        report
            .resolved_path
            .file_name()
            .and_then(|name| name.to_str()),
        Some("Bonsai-8B-Q4.gguf")
    );
    let manifest_path = report
        .cache_identity
        .path
        .join(".metamorph-remote-source.json");
    let manifest = fs::read_to_string(manifest_path).unwrap();
    assert!(manifest.contains("\"resolved_revision\": \"sha-main-001\""));
}

#[test]
fn acquire_source_reuses_fetched_remote_artifact_when_present() {
    let _env_guard = ENV_LOCK.lock().unwrap();
    let temp = tempdir().unwrap();
    let cache_root = temp.path().join("cache");
    let mock_root = temp.path().join("mock");
    let source = Source::from_str("hf://prism-ml/Bonsai-8B-gguf@main").unwrap();

    write_mock_remote_gguf_repo(
        &mock_root,
        "prism-ml/Bonsai-8B-gguf",
        "main",
        "Bonsai-8B-Q4.gguf",
        Some("sha-main-001"),
    );

    unsafe {
        std::env::set_var("METAMORPH_CACHE_DIR", &cache_root);
        std::env::set_var(MOCK_ROOT_ENV, &mock_root);
    }
    let first = acquire_source(&source, None, false).unwrap();
    let second = acquire_source(&source, None, false).unwrap();
    unsafe {
        std::env::remove_var("METAMORPH_CACHE_DIR");
        std::env::remove_var(MOCK_ROOT_ENV);
    }

    assert_eq!(
        first.outcome,
        SourceAcquisitionOutcome::FetchedRemoteArtifact
    );
    assert_eq!(second.outcome, SourceAcquisitionOutcome::ReusedRemoteCache);
    assert_eq!(second.resolved_path, first.resolved_path);
    assert!(
        second
            .notes
            .iter()
            .any(|note| note.contains("reused cached remote artifact"))
    );
}

#[test]
fn acquire_source_refreshes_remote_source_when_requested() {
    let _env_guard = ENV_LOCK.lock().unwrap();
    let temp = tempdir().unwrap();
    let cache_root = temp.path().join("cache");
    let mock_root = temp.path().join("mock");
    let source = Source::from_str("hf://prism-ml/Bonsai-8B-gguf@main").unwrap();

    write_mock_remote_gguf_repo(
        &mock_root,
        "prism-ml/Bonsai-8B-gguf",
        "main",
        "Bonsai-8B-Q4.gguf",
        Some("sha-main-001"),
    );

    unsafe {
        std::env::set_var("METAMORPH_CACHE_DIR", &cache_root);
        std::env::set_var(MOCK_ROOT_ENV, &mock_root);
    }
    let initial = acquire_source(&source, None, false).unwrap();
    write_mock_remote_gguf_repo(
        &mock_root,
        "prism-ml/Bonsai-8B-gguf",
        "main",
        "Bonsai-8B-Q4.gguf",
        Some("sha-main-002"),
    );
    let refreshed = acquire_source_with_options(
        &source,
        None,
        SourceAcquisitionOptions {
            materialize_local_copy: false,
            refresh_remote: true,
        },
    )
    .unwrap();
    unsafe {
        std::env::remove_var("METAMORPH_CACHE_DIR");
        std::env::remove_var(MOCK_ROOT_ENV);
    }

    assert_eq!(
        initial.outcome,
        SourceAcquisitionOutcome::FetchedRemoteArtifact
    );
    assert_eq!(
        refreshed.outcome,
        SourceAcquisitionOutcome::RefreshedRemoteArtifact
    );
    let manifest = fs::read_to_string(
        refreshed
            .cache_identity
            .path
            .join(".metamorph-remote-source.json"),
    )
    .unwrap();
    assert!(manifest.contains("\"resolved_revision\": \"sha-main-002\""));
}

#[test]
fn remote_acquisition_reports_auth_failure() {
    let _env_guard = ENV_LOCK.lock().unwrap();
    let temp = tempdir().unwrap();
    let cache_root = temp.path().join("cache");
    let mock_root = temp.path().join("mock");
    let source = Source::from_str("hf://private-org/Secret-gguf@main").unwrap();

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

    unsafe {
        std::env::set_var("METAMORPH_CACHE_DIR", &cache_root);
        std::env::set_var(MOCK_ROOT_ENV, &mock_root);
        std::env::remove_var(HF_TOKEN_ENV);
    }
    let error = acquire_source(&source, None, false).unwrap_err();
    unsafe {
        std::env::remove_var("METAMORPH_CACHE_DIR");
        std::env::remove_var(MOCK_ROOT_ENV);
    }

    assert!(
        error
            .to_string()
            .contains("requires credentials in `HF_TOKEN`")
    );
}

#[test]
fn remote_acquisition_reports_missing_revision() {
    let _env_guard = ENV_LOCK.lock().unwrap();
    let temp = tempdir().unwrap();
    let cache_root = temp.path().join("cache");
    let mock_root = temp.path().join("mock");
    let source = Source::from_str("hf://prism-ml/Bonsai-8B-gguf@missing").unwrap();

    unsafe {
        std::env::set_var("METAMORPH_CACHE_DIR", &cache_root);
        std::env::set_var(MOCK_ROOT_ENV, &mock_root);
    }
    let error = acquire_source(&source, None, false).unwrap_err();
    unsafe {
        std::env::remove_var("METAMORPH_CACHE_DIR");
        std::env::remove_var(MOCK_ROOT_ENV);
    }

    assert!(
        error
            .to_string()
            .contains("could not be resolved on Hugging Face")
    );
}

#[test]
fn remote_acquisition_reports_malformed_layout() {
    let _env_guard = ENV_LOCK.lock().unwrap();
    let temp = tempdir().unwrap();
    let cache_root = temp.path().join("cache");
    let mock_root = temp.path().join("mock");
    let repo_root = mock_root
        .join("prism-ml")
        .join("Bonsai-8B-gguf")
        .join("main");
    let source = Source::from_str("hf://prism-ml/Bonsai-8B-gguf@main").unwrap();

    fs::create_dir_all(&repo_root).unwrap();
    fs::write(repo_root.join("README.md"), b"not a gguf repo").unwrap();

    unsafe {
        std::env::set_var("METAMORPH_CACHE_DIR", &cache_root);
        std::env::set_var(MOCK_ROOT_ENV, &mock_root);
    }
    let error = acquire_source(&source, None, false).unwrap_err();
    unsafe {
        std::env::remove_var("METAMORPH_CACHE_DIR");
        std::env::remove_var(MOCK_ROOT_ENV);
    }

    assert!(
        error
            .to_string()
            .contains("does not expose a supported fetch layout")
    );
}

#[test]
fn remote_acquisition_reports_stale_cache_state() {
    let _env_guard = ENV_LOCK.lock().unwrap();
    let temp = tempdir().unwrap();
    let cache_root = temp.path().join("cache");
    let mock_root = temp.path().join("mock");
    let source = Source::from_str("hf://prism-ml/Bonsai-8B-gguf@main").unwrap();

    write_mock_remote_gguf_repo(
        &mock_root,
        "prism-ml/Bonsai-8B-gguf",
        "main",
        "Bonsai-8B-Q4.gguf",
        Some("sha-main-001"),
    );

    unsafe {
        std::env::set_var("METAMORPH_CACHE_DIR", &cache_root);
        std::env::set_var(MOCK_ROOT_ENV, &mock_root);
    }
    let report = acquire_source(&source, None, false).unwrap();
    fs::write(&report.resolved_path, b"broken gguf").unwrap();
    let error = acquire_source(&source, None, false).unwrap_err();
    unsafe {
        std::env::remove_var("METAMORPH_CACHE_DIR");
        std::env::remove_var(MOCK_ROOT_ENV);
    }

    assert!(error.to_string().contains("stale or invalid"));
    assert!(error.to_string().contains("--refresh"));
}

#[test]
fn converts_local_gguf_into_hf_safetensors_bundle() {
    let temp = tempdir().unwrap();
    let source_path = temp.path().join("fixture.gguf");
    let output_path = temp.path().join("bundle");

    write_fixture_gguf(&source_path);

    let request = ConvertRequest {
        source: Source::LocalPath(source_path),
        target: Target::LocalDir(output_path.clone()),
        from: None,
        to: Format::HfSafetensors,
        allow_lossy: true,
        refresh_remote: false,
    };

    convert(&request).unwrap();

    for required in [
        "config.json",
        "tokenizer.json",
        "generation_config.json",
        "model.safetensors",
    ] {
        assert!(output_path.join(required).is_file());
    }

    let report = inspect(&Source::LocalPath(output_path.clone())).unwrap();
    assert_eq!(report.detected_format, Some(Format::HfSafetensors));

    let config: JsonValue =
        serde_json::from_slice(&fs::read(output_path.join("config.json")).unwrap()).unwrap();
    assert_eq!(config["model_type"], "llama");
    assert_eq!(config["hidden_size"], 32);
    assert_eq!(config["num_hidden_layers"], 1);

    let tensors =
        candle_core::safetensors::load(output_path.join("model.safetensors"), &Device::Cpu)
            .unwrap();
    assert!(tensors.contains_key("tok_embeddings.weight"));
}

#[test]
fn converts_local_gguf_into_safetensors_artifact() {
    let temp = tempdir().unwrap();
    let source_path = temp.path().join("fixture.gguf");
    let output_path = temp.path().join("weights.safetensors");

    write_fixture_gguf(&source_path);

    let request = ConvertRequest {
        source: Source::LocalPath(source_path),
        target: Target::LocalDir(output_path.clone()),
        from: None,
        to: Format::Safetensors,
        allow_lossy: true,
        refresh_remote: false,
    };

    convert(&request).unwrap();

    assert!(output_path.is_file());

    let report = validate(&output_path, Some(Format::Safetensors)).unwrap();
    assert_eq!(report.format, Format::Safetensors);

    let tensors = candle_core::safetensors::load(&output_path, &Device::Cpu).unwrap();
    assert!(tensors.contains_key("tok_embeddings.weight"));
}

#[test]
fn convert_fetches_remote_gguf_on_demand() {
    let _env_guard = ENV_LOCK.lock().unwrap();
    let temp = tempdir().unwrap();
    let cache_root = temp.path().join("cache");
    let mock_root = temp.path().join("mock");
    let output_path = temp.path().join("bundle");

    write_mock_remote_gguf_repo(
        &mock_root,
        "prism-ml/Bonsai-8B-gguf",
        "main",
        "Bonsai-8B-Q4.gguf",
        Some("sha-main-001"),
    );

    let request = ConvertRequest {
        source: Source::from_str("hf://prism-ml/Bonsai-8B-gguf@main").unwrap(),
        target: Target::LocalDir(output_path.clone()),
        from: Some(Format::Gguf),
        to: Format::HfSafetensors,
        allow_lossy: true,
        refresh_remote: false,
    };

    unsafe {
        std::env::set_var("METAMORPH_CACHE_DIR", &cache_root);
        std::env::set_var(MOCK_ROOT_ENV, &mock_root);
    }
    let report = convert(&request).unwrap();
    unsafe {
        std::env::remove_var("METAMORPH_CACHE_DIR");
        std::env::remove_var(MOCK_ROOT_ENV);
    }

    assert_eq!(
        report.acquisition.outcome,
        SourceAcquisitionOutcome::FetchedRemoteArtifact
    );
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
fn validation_rejects_missing_required_hf_bundle_files() {
    let temp = tempdir().unwrap();
    fs::write(temp.path().join("config.json"), b"{}").unwrap();
    fs::write(temp.path().join("tokenizer.json"), b"{}").unwrap();
    write_valid_safetensors(temp.path().join("model.safetensors").as_path());

    let error = validate(temp.path(), Some(Format::HfSafetensors)).unwrap_err();

    assert!(
        error
            .to_string()
            .contains("missing required file `generation_config.json`")
    );
}

#[test]
fn validation_accepts_complete_hf_bundle() {
    let temp = tempdir().unwrap();
    write_valid_hf_bundle(temp.path());

    let report = validate(temp.path(), Some(Format::HfSafetensors)).unwrap();

    assert_eq!(report.format, Format::HfSafetensors);
    assert!(report.reusable);
    assert_eq!(report.checked_paths.len(), 4);
}

#[test]
fn publish_plan_validates_bundle_and_lists_artifacts() {
    let temp = tempdir().unwrap();
    write_valid_hf_bundle(temp.path());

    let plan = plan_publish(temp.path(), "your-org/Bonsai-8B-candle").unwrap();

    assert_eq!(plan.validated_format, Format::HfSafetensors);
    assert_eq!(
        plan.destination,
        Target::HuggingFaceRepo("your-org/Bonsai-8B-candle".into())
    );
    assert_eq!(plan.artifacts.len(), 4);
}

#[test]
fn publish_requires_credentials_for_execute_mode() {
    let _env_guard = ENV_LOCK.lock().unwrap();
    let temp = tempdir().unwrap();
    write_valid_hf_bundle(temp.path());
    unsafe { std::env::remove_var("HF_TOKEN") };

    let error = publish(&PublishRequest {
        input: temp.path().to_path_buf(),
        target: Target::HuggingFaceRepo("your-org/Bonsai-8B-candle".into()),
        execute: true,
    })
    .unwrap_err();

    assert!(
        error
            .to_string()
            .contains("requires credentials in `HF_TOKEN`")
    );
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
        write_mock_remote_config(
            root,
            repo,
            revision,
            json!({ "resolved_revision": resolved_revision }),
        );
    }
}

fn write_mock_remote_config(root: &Path, repo: &str, revision: &str, config: JsonValue) {
    let repo_root = root.join(repo).join(revision);
    fs::create_dir_all(&repo_root).unwrap();
    fs::write(
        repo_root.join(".metamorph-hf.json"),
        serde_json::to_vec_pretty(&config).unwrap(),
    )
    .unwrap();
}
