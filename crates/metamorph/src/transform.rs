use std::collections::HashMap;
use std::fmt;
use std::fs::{self, File};
use std::io::{BufReader, BufWriter, Write};
use std::path::{Path, PathBuf};

use candle_core::quantized::gguf_file;
use candle_core::quantized::tokenizer::TokenizerFromGguf;
use candle_core::{Device, Tensor};
use serde::{Deserialize, Serialize};
use serde_json::{Map as JsonMap, Value as JsonValue, json};
use tokenizers::Tokenizer;

use crate::cache::{
    SourceAcquisitionOptions, SourceAcquisitionReport, acquire_source_with_options,
};
use crate::error::{MetamorphError, Result};
use crate::format::Format;
use crate::plan::{ConvertRequest, plan};
use crate::source::{Source, Target, detect_path_format, resolve_local_gguf_path_from_fs};
use crate::validate::validate;

const OPTIONAL_HF_SIDECAR_FILES: &[&str] = &[
    "tokenizer_config.json",
    "special_tokens_map.json",
    "added_tokens.json",
];

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum ExecutionSupport {
    Executable,
    PlannedOnly,
}

impl fmt::Display for ExecutionSupport {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let label = match self {
            Self::Executable => "executable",
            Self::PlannedOnly => "planned-only",
        };

        f.write_str(label)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ExecutionSourceSupport {
    LocalOnly,
    LocalOrRemoteGguf,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ExecutionTargetSupport {
    LocalOnly,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ConversionCapability {
    pub from: Format,
    pub to: Format,
    pub lossy: bool,
    pub execution_support: ExecutionSupport,
    backend: Option<BackendKind>,
    backend_label: Option<&'static str>,
    source_support: ExecutionSourceSupport,
    target_support: ExecutionTargetSupport,
    pub steps: &'static [&'static str],
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ConversionReport {
    pub acquisition: SourceAcquisitionReport,
    pub output: PathBuf,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub(crate) struct RequestAssessment {
    pub blockers: Vec<String>,
    pub notes: Vec<String>,
}

impl ConversionCapability {
    pub fn backend_label(&self) -> Option<&'static str> {
        self.backend_label
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum BackendKind {
    GgufToHfSafetensors,
    GgufToSafetensors,
    SafetensorsRelayout,
    HfSafetensorsRelayout,
    SafetensorsToHfSafetensors,
}

#[derive(Debug, Clone)]
struct BundleMaterializationSource {
    tensor_path: Option<PathBuf>,
    config_path: Option<PathBuf>,
    tokenizer_path: Option<PathBuf>,
    generation_config_path: Option<PathBuf>,
    optional_sidecars: Vec<PathBuf>,
    blockers: Vec<String>,
    notes: Vec<String>,
}

const SAFETENSORS_RELAYOUT_STEPS: &[&str] = &[
    "inspect local safetensors artifacts",
    "copy safetensors artifacts into the requested local layout",
    "validate the output artifacts",
];

const HF_SAFETENSORS_RELAYOUT_STEPS: &[&str] = &[
    "inspect the local hf-safetensors bundle",
    "copy bundle files into the requested local layout",
    "validate the output bundle",
];

const GGUF_TO_HF_SAFETENSORS_STEPS: &[&str] = &[
    "fetch or read GGUF artifacts",
    "materialize tensors into a Hugging Face-style safetensors layout",
    "emit tokenizer and config files expected by downstream runtimes",
    "validate the output bundle",
];

const GGUF_TO_SAFETENSORS_STEPS: &[&str] = &[
    "fetch or read GGUF artifacts",
    "materialize tensors into safetensors",
    "validate converted weights",
];

const SAFETENSORS_TO_HF_SAFETENSORS_STEPS: &[&str] = &[
    "inspect local safetensors artifacts and metadata sidecars",
    "materialize files into a Hugging Face-style repository layout",
    "validate the output bundle",
];

pub fn find_capability(from: Format, to: Format) -> Option<ConversionCapability> {
    match (from, to) {
        (Format::Gguf, Format::HfSafetensors) => Some(ConversionCapability {
            from,
            to,
            lossy: true,
            execution_support: ExecutionSupport::Executable,
            backend: Some(BackendKind::GgufToHfSafetensors),
            backend_label: Some("gguf-to-hf-safetensors"),
            source_support: ExecutionSourceSupport::LocalOrRemoteGguf,
            target_support: ExecutionTargetSupport::LocalOnly,
            steps: GGUF_TO_HF_SAFETENSORS_STEPS,
        }),
        (Format::Gguf, Format::Safetensors) => Some(ConversionCapability {
            from,
            to,
            lossy: true,
            execution_support: ExecutionSupport::Executable,
            backend: Some(BackendKind::GgufToSafetensors),
            backend_label: Some("gguf-to-safetensors"),
            source_support: ExecutionSourceSupport::LocalOrRemoteGguf,
            target_support: ExecutionTargetSupport::LocalOnly,
            steps: GGUF_TO_SAFETENSORS_STEPS,
        }),
        (Format::Safetensors, Format::Safetensors) => Some(ConversionCapability {
            from,
            to,
            lossy: false,
            execution_support: ExecutionSupport::Executable,
            backend: Some(BackendKind::SafetensorsRelayout),
            backend_label: Some("safetensors-relayout"),
            source_support: ExecutionSourceSupport::LocalOnly,
            target_support: ExecutionTargetSupport::LocalOnly,
            steps: SAFETENSORS_RELAYOUT_STEPS,
        }),
        (Format::HfSafetensors, Format::HfSafetensors) => Some(ConversionCapability {
            from,
            to,
            lossy: false,
            execution_support: ExecutionSupport::Executable,
            backend: Some(BackendKind::HfSafetensorsRelayout),
            backend_label: Some("hf-safetensors-relayout"),
            source_support: ExecutionSourceSupport::LocalOnly,
            target_support: ExecutionTargetSupport::LocalOnly,
            steps: HF_SAFETENSORS_RELAYOUT_STEPS,
        }),
        (Format::Safetensors, Format::HfSafetensors) => Some(ConversionCapability {
            from,
            to,
            lossy: false,
            execution_support: ExecutionSupport::Executable,
            backend: Some(BackendKind::SafetensorsToHfSafetensors),
            backend_label: Some("safetensors-to-hf-safetensors"),
            source_support: ExecutionSourceSupport::LocalOnly,
            target_support: ExecutionTargetSupport::LocalOnly,
            steps: SAFETENSORS_TO_HF_SAFETENSORS_STEPS,
        }),
        _ => None,
    }
}

pub(crate) fn assess_request(
    request: &ConvertRequest,
    capability: &ConversionCapability,
) -> Result<RequestAssessment> {
    let mut assessment = RequestAssessment::default();

    if capability.source_support == ExecutionSourceSupport::LocalOnly
        && !matches!(request.source, Source::LocalPath(_))
    {
        assessment.blockers.push(format!(
            "execution backend `{}` currently requires a local source path",
            capability.backend_label().unwrap_or("local-only")
        ));
    }

    if capability.target_support == ExecutionTargetSupport::LocalOnly
        && !matches!(request.target, Target::LocalDir(_))
    {
        assessment.blockers.push(format!(
            "execution backend `{}` currently requires a local output path",
            capability.backend_label().unwrap_or("local-only")
        ));
    }

    if matches!(
        (capability.from, capability.to),
        (Format::Safetensors, Format::HfSafetensors)
    ) && assessment.blockers.is_empty()
    {
        let source_assessment = inspect_bundle_materialization_source_for_path(&request.source)?;
        assessment.blockers.extend(source_assessment.blockers);
        assessment.notes.extend(source_assessment.notes);
    }

    Ok(assessment)
}

pub fn convert(request: &ConvertRequest) -> Result<ConversionReport> {
    let conversion_plan = plan(request)?;
    let capability = find_capability(conversion_plan.source_format, conversion_plan.target_format)
        .ok_or_else(|| MetamorphError::UnsupportedConversionPath {
            from: conversion_plan.source_format,
            to: conversion_plan.target_format,
        })?;

    if capability.execution_support == ExecutionSupport::PlannedOnly {
        return Err(MetamorphError::NotImplemented(
            "execution backend for this compatible path is not wired yet",
        ));
    }

    match capability.backend {
        Some(BackendKind::GgufToHfSafetensors) => convert_local_gguf_to_hf_safetensors(request),
        Some(BackendKind::GgufToSafetensors) => convert_local_gguf_to_safetensors(request),
        Some(BackendKind::SafetensorsRelayout) => convert_local_safetensors_relayout(request),
        Some(BackendKind::HfSafetensorsRelayout) => convert_local_hf_safetensors_relayout(request),
        Some(BackendKind::SafetensorsToHfSafetensors) => {
            convert_local_safetensors_to_hf_safetensors(request)
        }
        None => Err(MetamorphError::NotImplemented(
            "execution backend for this compatible path is not wired yet",
        )),
    }
}

fn convert_local_gguf_to_hf_safetensors(request: &ConvertRequest) -> Result<ConversionReport> {
    let acquisition = acquire_gguf_source(request)?;
    let source_path = resolve_local_gguf_path_from_fs(&acquisition.resolved_path)?;
    let output_dir = resolve_local_target_dir(&request.target)?;

    fs::create_dir_all(&output_dir)?;

    let mut reader = BufReader::new(File::open(&source_path)?);
    let content = gguf_file::Content::read(&mut reader)?;

    let tensors = dequantize_gguf_tensors(&content, &mut reader)?;
    write_json_file(
        &output_dir.join("config.json"),
        &build_config_json(&content, &source_path)?,
    )?;
    write_json_file(
        &output_dir.join("generation_config.json"),
        &build_generation_config_json(&content),
    )?;
    write_tokenizer_file(&content, &output_dir.join("tokenizer.json"))?;
    candle_core::safetensors::save(&tensors, output_dir.join("model.safetensors"))?;
    validate(&output_dir, Some(Format::HfSafetensors))?;

    Ok(ConversionReport {
        acquisition,
        output: output_dir,
    })
}

fn convert_local_gguf_to_safetensors(request: &ConvertRequest) -> Result<ConversionReport> {
    let acquisition = acquire_gguf_source(request)?;
    let source_path = resolve_local_gguf_path_from_fs(&acquisition.resolved_path)?;
    let output_path = resolve_local_safetensors_target(&request.target)?;

    if let Some(parent) = output_path.parent() {
        fs::create_dir_all(parent)?;
    }

    let mut reader = BufReader::new(File::open(&source_path)?);
    let content = gguf_file::Content::read(&mut reader)?;
    let tensors = dequantize_gguf_tensors(&content, &mut reader)?;

    candle_core::safetensors::save(&tensors, &output_path)?;
    validate(&output_path, Some(Format::Safetensors))?;

    Ok(ConversionReport {
        acquisition,
        output: output_path,
    })
}

fn convert_local_safetensors_relayout(request: &ConvertRequest) -> Result<ConversionReport> {
    let backend_label = "safetensors-relayout";
    let acquisition = acquire_local_source(request, Some(Format::Safetensors), backend_label)?;
    let source_artifacts = collect_local_safetensors_artifacts(&acquisition.resolved_path)?;
    let target = resolve_local_target_dir(&request.target)?;

    let output = if source_artifacts.len() == 1
        && target.extension().and_then(|extension| extension.to_str()) == Some("safetensors")
    {
        copy_file_if_needed(&source_artifacts[0], &target)?;
        validate(&target, Some(Format::Safetensors))?;
        target
    } else {
        if source_artifacts.len() > 1
            && target.extension().and_then(|extension| extension.to_str()) == Some("safetensors")
        {
            return Err(MetamorphError::InvalidLocalConversionSource {
                path: acquisition.resolved_path.clone(),
                reason: "multiple safetensors artifacts require a directory target".to_owned(),
            });
        }

        fs::create_dir_all(&target)?;
        for artifact in &source_artifacts {
            let destination = target.join(
                artifact
                    .file_name()
                    .expect("safetensors artifact should have a file name"),
            );
            copy_file_if_needed(artifact, &destination)?;
        }
        validate(&target, Some(Format::Safetensors))?;
        target
    };

    Ok(ConversionReport {
        acquisition,
        output,
    })
}

fn convert_local_hf_safetensors_relayout(request: &ConvertRequest) -> Result<ConversionReport> {
    let backend_label = "hf-safetensors-relayout";
    let acquisition = acquire_local_source(request, Some(Format::HfSafetensors), backend_label)?;
    let source_dir = resolve_local_hf_safetensors_source_dir(&acquisition.resolved_path)?;
    let target = resolve_local_target_dir(&request.target)?;

    if source_dir == target {
        validate(&target, Some(Format::HfSafetensors))?;
        return Ok(ConversionReport {
            acquisition,
            output: target,
        });
    }

    copy_directory_contents(&source_dir, &target)?;
    validate(&target, Some(Format::HfSafetensors))?;

    Ok(ConversionReport {
        acquisition,
        output: target,
    })
}

fn convert_local_safetensors_to_hf_safetensors(
    request: &ConvertRequest,
) -> Result<ConversionReport> {
    let backend_label = "safetensors-to-hf-safetensors";
    let acquisition = acquire_local_source(request, Some(Format::Safetensors), backend_label)?;
    let source = inspect_bundle_materialization_source(&acquisition.resolved_path)?;
    let output_dir = resolve_local_target_dir(&request.target)?;

    fs::create_dir_all(&output_dir)?;

    copy_file_if_needed(
        source
            .tensor_path
            .as_ref()
            .expect("bundle materialization source should have a tensor path"),
        &output_dir.join("model.safetensors"),
    )?;
    copy_file_if_needed(
        source
            .config_path
            .as_ref()
            .expect("bundle materialization source should have config.json"),
        &output_dir.join("config.json"),
    )?;
    copy_file_if_needed(
        source
            .tokenizer_path
            .as_ref()
            .expect("bundle materialization source should have tokenizer.json"),
        &output_dir.join("tokenizer.json"),
    )?;

    match &source.generation_config_path {
        Some(path) => {
            copy_file_if_needed(path, &output_dir.join("generation_config.json"))?;
        }
        None => {
            write_json_file(
                &output_dir.join("generation_config.json"),
                &JsonValue::Object(JsonMap::new()),
            )?;
        }
    }

    for sidecar in &source.optional_sidecars {
        let destination = output_dir.join(
            sidecar
                .file_name()
                .expect("optional sidecar should have a file name"),
        );
        copy_file_if_needed(sidecar, &destination)?;
    }

    validate(&output_dir, Some(Format::HfSafetensors))?;

    Ok(ConversionReport {
        acquisition,
        output: output_dir,
    })
}

fn acquire_gguf_source(request: &ConvertRequest) -> Result<SourceAcquisitionReport> {
    acquire_source_with_options(
        &request.source,
        Some(Format::Gguf),
        SourceAcquisitionOptions {
            materialize_local_copy: false,
            refresh_remote: request.refresh_remote,
        },
    )
}

fn acquire_local_source(
    request: &ConvertRequest,
    expected_format: Option<Format>,
    backend_label: &'static str,
) -> Result<SourceAcquisitionReport> {
    if !matches!(request.source, Source::LocalPath(_)) {
        return Err(MetamorphError::UnsupportedExecutionSource(format!(
            "{} (backend `{backend_label}` currently requires a local source path)",
            request.source.display_name()
        )));
    }

    acquire_source_with_options(
        &request.source,
        expected_format,
        SourceAcquisitionOptions {
            materialize_local_copy: false,
            refresh_remote: false,
        },
    )
}

fn resolve_local_target_dir(target: &Target) -> Result<PathBuf> {
    match target {
        Target::LocalDir(path) => Ok(path.clone()),
        Target::HuggingFaceRepo(repo) => Err(MetamorphError::UnsupportedExecutionTarget(format!(
            "hf://{repo}"
        ))),
    }
}

fn resolve_local_safetensors_target(target: &Target) -> Result<PathBuf> {
    let path = resolve_local_target_dir(target)?;
    if path.extension().and_then(|extension| extension.to_str()) == Some("safetensors") {
        return Ok(path);
    }

    Ok(path.join("model.safetensors"))
}

fn collect_local_safetensors_artifacts(path: &Path) -> Result<Vec<PathBuf>> {
    if !path.exists() {
        return Err(MetamorphError::MissingPath(path.to_path_buf()));
    }

    if path.is_file() {
        return match detect_path_format(path) {
            Some(Format::Safetensors) => Ok(vec![path.to_path_buf()]),
            _ => Err(MetamorphError::InvalidLocalConversionSource {
                path: path.to_path_buf(),
                reason: "expected a local `.safetensors` file".to_owned(),
            }),
        };
    }

    let mut artifacts = fs::read_dir(path)?
        .filter_map(std::result::Result::ok)
        .map(|entry| entry.path())
        .filter(|entry| detect_path_format(entry) == Some(Format::Safetensors))
        .collect::<Vec<_>>();
    artifacts.sort();

    if artifacts.is_empty() {
        return Err(MetamorphError::InvalidLocalConversionSource {
            path: path.to_path_buf(),
            reason: "expected a local `.safetensors` file or a directory containing `.safetensors` artifacts".to_owned(),
        });
    }

    Ok(artifacts)
}

fn resolve_local_hf_safetensors_source_dir(path: &Path) -> Result<PathBuf> {
    if !path.is_dir() {
        return Err(MetamorphError::InvalidLocalConversionSource {
            path: path.to_path_buf(),
            reason: "expected a local directory for `hf-safetensors` relayout".to_owned(),
        });
    }

    validate(path, Some(Format::HfSafetensors))?;
    Ok(path.to_path_buf())
}

fn inspect_bundle_materialization_source_for_path(
    source: &Source,
) -> Result<BundleMaterializationSource> {
    match source {
        Source::LocalPath(path) => inspect_bundle_materialization_source_from_local_path(path),
        Source::HuggingFace { repo, revision } => {
            let display = match revision {
                Some(revision) => format!("hf://{repo}@{revision}"),
                None => format!("hf://{repo}"),
            };

            Ok(BundleMaterializationSource {
                tensor_path: None,
                config_path: None,
                tokenizer_path: None,
                generation_config_path: None,
                optional_sidecars: Vec::new(),
                blockers: vec![format!(
                    "`safetensors -> hf-safetensors` currently requires a local source path, not `{display}`"
                )],
                notes: Vec::new(),
            })
        }
    }
}

fn inspect_bundle_materialization_source(path: &Path) -> Result<BundleMaterializationSource> {
    let source = inspect_bundle_materialization_source_from_local_path(path)?;
    if source.blockers.is_empty() {
        Ok(source)
    } else {
        Err(MetamorphError::InvalidLocalConversionSource {
            path: path.to_path_buf(),
            reason: source.blockers.join(" "),
        })
    }
}

fn inspect_bundle_materialization_source_from_local_path(
    path: &Path,
) -> Result<BundleMaterializationSource> {
    let artifacts = collect_local_safetensors_artifacts(path)?;
    let sidecar_root = if path.is_file() {
        path.parent()
            .map(Path::to_path_buf)
            .unwrap_or_else(|| PathBuf::from("."))
    } else {
        path.to_path_buf()
    };

    let config_path = sidecar_root.join("config.json");
    let tokenizer_path = sidecar_root.join("tokenizer.json");
    let generation_config_path = sidecar_root.join("generation_config.json");

    let mut blockers = Vec::new();
    let mut notes = Vec::new();

    let tensor_path = match artifacts.as_slice() {
        [single] => Some(single.clone()),
        _ => {
            blockers.push(
                "`safetensors -> hf-safetensors` currently expects exactly one local `.safetensors` artifact"
                    .to_owned(),
            );
            None
        }
    };

    let config = if config_path.is_file() {
        Some(config_path)
    } else {
        blockers.push(
            "source path is missing `config.json` required for `safetensors -> hf-safetensors`"
                .to_owned(),
        );
        None
    };

    let tokenizer = if tokenizer_path.is_file() {
        Some(tokenizer_path)
    } else {
        blockers.push(
            "source path is missing `tokenizer.json` required for `safetensors -> hf-safetensors`"
                .to_owned(),
        );
        None
    };

    let generation_config = if generation_config_path.is_file() {
        Some(generation_config_path)
    } else {
        notes.push(
            "source path is missing `generation_config.json`; the output bundle will emit an empty generation config"
                .to_owned(),
        );
        None
    };

    let mut optional_sidecars = OPTIONAL_HF_SIDECAR_FILES
        .iter()
        .map(|name| sidecar_root.join(name))
        .filter(|path| path.is_file())
        .collect::<Vec<_>>();
    optional_sidecars.sort();

    Ok(BundleMaterializationSource {
        tensor_path,
        config_path: config,
        tokenizer_path: tokenizer,
        generation_config_path: generation_config,
        optional_sidecars,
        blockers,
        notes,
    })
}

fn copy_directory_contents(source: &Path, target: &Path) -> Result<()> {
    let entries = fs::read_dir(source)?
        .filter_map(std::result::Result::ok)
        .map(|entry| entry.path())
        .collect::<Vec<_>>();

    fs::create_dir_all(target)?;

    for source_path in entries {
        let destination = target.join(
            source_path
                .file_name()
                .expect("directory entry should have a file name"),
        );
        if source_path.is_dir() {
            copy_directory_contents(&source_path, &destination)?;
        } else {
            copy_file_if_needed(&source_path, &destination)?;
        }
    }

    Ok(())
}

fn copy_file_if_needed(source: &Path, target: &Path) -> Result<()> {
    if same_location(source, target) {
        return Ok(());
    }

    if let Some(parent) = target.parent() {
        fs::create_dir_all(parent)?;
    }
    fs::copy(source, target)?;
    Ok(())
}

fn same_location(source: &Path, target: &Path) -> bool {
    if source == target {
        return true;
    }

    if source.exists() && target.exists() {
        return fs::canonicalize(source).ok() == fs::canonicalize(target).ok();
    }

    false
}

fn dequantize_gguf_tensors<R: std::io::Read + std::io::Seek>(
    content: &gguf_file::Content,
    reader: &mut R,
) -> Result<HashMap<String, Tensor>> {
    let mut tensor_names = content.tensor_infos.keys().cloned().collect::<Vec<_>>();
    tensor_names.sort();

    let device = Device::Cpu;
    let mut tensors = HashMap::with_capacity(tensor_names.len());

    for tensor_name in tensor_names {
        let tensor = content
            .tensor(reader, &tensor_name, &device)?
            .dequantize_f16(&device)?;
        tensors.insert(tensor_name, tensor);
    }

    Ok(tensors)
}

fn write_tokenizer_file(content: &gguf_file::Content, path: &Path) -> Result<()> {
    let tokenizer = Tokenizer::from_gguf(content)?;
    tokenizer.save(path, true)?;
    Ok(())
}

fn build_config_json(content: &gguf_file::Content, source_path: &Path) -> Result<JsonValue> {
    let architecture =
        metadata_string(content, "general.architecture").unwrap_or_else(|| "unknown".to_owned());
    let architecture_prefix = architecture.clone();

    let mut config = JsonMap::new();
    config.insert(
        "architectures".to_owned(),
        json!([architecture_class_name(&architecture)]),
    );
    config.insert("model_type".to_owned(), json!(architecture));
    config.insert("torch_dtype".to_owned(), json!("float16"));

    if let Some(source_name) = source_path.file_stem().and_then(|stem| stem.to_str()) {
        config.insert("_name_or_path".to_owned(), json!(source_name));
    }

    insert_u64(
        &mut config,
        "hidden_size",
        metadata_u64(content, &format!("{architecture_prefix}.embedding_length"))?,
    );
    insert_u64(
        &mut config,
        "intermediate_size",
        metadata_u64(
            content,
            &format!("{architecture_prefix}.feed_forward_length"),
        )?,
    );
    insert_u64(
        &mut config,
        "num_hidden_layers",
        metadata_u64(content, &format!("{architecture_prefix}.block_count"))?,
    );
    insert_u64(
        &mut config,
        "num_attention_heads",
        metadata_u64(
            content,
            &format!("{architecture_prefix}.attention.head_count"),
        )?,
    );
    insert_u64(
        &mut config,
        "num_key_value_heads",
        metadata_u64(
            content,
            &format!("{architecture_prefix}.attention.head_count_kv"),
        )?,
    );
    insert_u64(
        &mut config,
        "max_position_embeddings",
        metadata_u64(content, &format!("{architecture_prefix}.context_length"))?,
    );
    insert_f64(
        &mut config,
        "rope_theta",
        metadata_f64(content, &format!("{architecture_prefix}.rope.freq_base"))?,
    );
    insert_f64(
        &mut config,
        "rms_norm_eps",
        metadata_f64(
            content,
            &format!("{architecture_prefix}.attention.layer_norm_rms_epsilon"),
        )?,
    );
    insert_u64(
        &mut config,
        "vocab_size",
        metadata_array_len(content, "tokenizer.ggml.tokens"),
    );
    insert_u64(
        &mut config,
        "bos_token_id",
        metadata_u64(content, "tokenizer.ggml.bos_token_id")?,
    );
    insert_u64(
        &mut config,
        "eos_token_id",
        metadata_u64(content, "tokenizer.ggml.eos_token_id")?,
    );
    insert_u64(
        &mut config,
        "pad_token_id",
        metadata_u64(content, "tokenizer.ggml.pad_token_id")?,
    );
    insert_u64(
        &mut config,
        "unk_token_id",
        metadata_u64(content, "tokenizer.ggml.unk_token_id")?,
    );

    Ok(JsonValue::Object(config))
}

fn build_generation_config_json(content: &gguf_file::Content) -> JsonValue {
    let mut generation_config = JsonMap::new();
    let architecture = metadata_string(content, "general.architecture");

    insert_u64(
        &mut generation_config,
        "bos_token_id",
        metadata_u64(content, "tokenizer.ggml.bos_token_id")
            .ok()
            .flatten(),
    );
    insert_u64(
        &mut generation_config,
        "eos_token_id",
        metadata_u64(content, "tokenizer.ggml.eos_token_id")
            .ok()
            .flatten(),
    );
    insert_u64(
        &mut generation_config,
        "pad_token_id",
        metadata_u64(content, "tokenizer.ggml.pad_token_id")
            .ok()
            .flatten(),
    );
    insert_u64(
        &mut generation_config,
        "max_length",
        architecture.and_then(|architecture| {
            metadata_u64(content, &format!("{architecture}.context_length"))
                .ok()
                .flatten()
        }),
    );

    JsonValue::Object(generation_config)
}

fn write_json_file(path: &Path, value: &JsonValue) -> Result<()> {
    let mut writer = BufWriter::new(File::create(path)?);
    serde_json::to_writer_pretty(&mut writer, value)?;
    writer.write_all(b"\n")?;
    writer.flush()?;
    Ok(())
}

fn architecture_class_name(architecture: &str) -> String {
    format!("{}ForCausalLM", pascal_case(architecture))
}

fn pascal_case(value: &str) -> String {
    let mut output = String::new();

    for part in value
        .split(|character: char| !character.is_ascii_alphanumeric())
        .filter(|part| !part.is_empty())
    {
        let mut characters = part.chars();
        if let Some(first) = characters.next() {
            output.push(first.to_ascii_uppercase());
            output.extend(characters.map(|character| character.to_ascii_lowercase()));
        }
    }

    if output.is_empty() {
        "Unknown".to_owned()
    } else {
        output
    }
}

fn insert_u64(object: &mut JsonMap<String, JsonValue>, key: &str, value: Option<u64>) {
    if let Some(value) = value {
        object.insert(key.to_owned(), json!(value));
    }
}

fn insert_f64(object: &mut JsonMap<String, JsonValue>, key: &str, value: Option<f64>) {
    if let Some(value) = value {
        object.insert(key.to_owned(), json!(value));
    }
}

fn metadata_string(content: &gguf_file::Content, key: &str) -> Option<String> {
    content
        .metadata
        .get(key)
        .and_then(|value| value.to_string().ok().cloned())
}

fn metadata_array_len(content: &gguf_file::Content, key: &str) -> Option<u64> {
    content
        .metadata
        .get(key)
        .and_then(|value| value.to_vec().ok())
        .map(|values| values.len() as u64)
}

fn metadata_u64(content: &gguf_file::Content, key: &str) -> Result<Option<u64>> {
    match content.metadata.get(key) {
        Some(value) => Ok(Some(value_as_u64(value)?)),
        None => Ok(None),
    }
}

fn metadata_f64(content: &gguf_file::Content, key: &str) -> Result<Option<f64>> {
    match content.metadata.get(key) {
        Some(value) => Ok(Some(value_as_f64(value)?)),
        None => Ok(None),
    }
}

fn value_as_u64(value: &gguf_file::Value) -> Result<u64> {
    let value = match value {
        gguf_file::Value::U8(value) => *value as u64,
        gguf_file::Value::I8(value) if *value >= 0 => *value as u64,
        gguf_file::Value::U16(value) => *value as u64,
        gguf_file::Value::I16(value) if *value >= 0 => *value as u64,
        gguf_file::Value::U32(value) => *value as u64,
        gguf_file::Value::I32(value) if *value >= 0 => *value as u64,
        gguf_file::Value::U64(value) => *value,
        gguf_file::Value::I64(value) if *value >= 0 => *value as u64,
        gguf_file::Value::Bool(value) => u64::from(*value),
        _ => {
            return Err(MetamorphError::InvalidOutputBundle {
                path: PathBuf::from("<gguf-metadata>"),
                reason: format!("metadata value `{value:?}` is not an unsigned integer"),
            });
        }
    };

    Ok(value)
}

fn value_as_f64(value: &gguf_file::Value) -> Result<f64> {
    let value = match value {
        gguf_file::Value::F32(value) => *value as f64,
        gguf_file::Value::F64(value) => *value,
        gguf_file::Value::U8(value) => *value as f64,
        gguf_file::Value::I8(value) => *value as f64,
        gguf_file::Value::U16(value) => *value as f64,
        gguf_file::Value::I16(value) => *value as f64,
        gguf_file::Value::U32(value) => *value as f64,
        gguf_file::Value::I32(value) => *value as f64,
        gguf_file::Value::U64(value) => *value as f64,
        gguf_file::Value::I64(value) => *value as f64,
        _ => {
            return Err(MetamorphError::InvalidOutputBundle {
                path: PathBuf::from("<gguf-metadata>"),
                reason: format!("metadata value `{value:?}` is not numeric"),
            });
        }
    };

    Ok(value)
}
