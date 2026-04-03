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
use crate::source::{Target, resolve_local_gguf_path_from_fs};
use crate::validate::validate;

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
pub struct ConversionCapability {
    pub from: Format,
    pub to: Format,
    pub lossy: bool,
    pub execution_support: ExecutionSupport,
    backend: Option<BackendKind>,
    backend_label: Option<&'static str>,
    pub steps: &'static [&'static str],
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ConversionReport {
    pub acquisition: SourceAcquisitionReport,
    pub output: PathBuf,
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
}

const SAME_FORMAT_STEPS: &[&str] = &[
    "inspect source artifacts",
    "normalize metadata and layout within the existing format",
    "write the target bundle",
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
    "inspect safetensors artifacts",
    "normalize files into a Hugging Face-style repository layout",
    "validate the output bundle",
];

pub fn find_capability(from: Format, to: Format) -> Option<ConversionCapability> {
    if from == to {
        return Some(ConversionCapability {
            from,
            to,
            lossy: false,
            execution_support: ExecutionSupport::PlannedOnly,
            backend: None,
            backend_label: Some("same-format-relayout"),
            steps: SAME_FORMAT_STEPS,
        });
    }

    match (from, to) {
        (Format::Gguf, Format::HfSafetensors) => Some(ConversionCapability {
            from,
            to,
            lossy: true,
            execution_support: ExecutionSupport::Executable,
            backend: Some(BackendKind::GgufToHfSafetensors),
            backend_label: Some("gguf-to-hf-safetensors"),
            steps: GGUF_TO_HF_SAFETENSORS_STEPS,
        }),
        (Format::Gguf, Format::Safetensors) => Some(ConversionCapability {
            from,
            to,
            lossy: true,
            execution_support: ExecutionSupport::Executable,
            backend: Some(BackendKind::GgufToSafetensors),
            backend_label: Some("gguf-to-safetensors"),
            steps: GGUF_TO_SAFETENSORS_STEPS,
        }),
        (Format::Safetensors, Format::HfSafetensors) => Some(ConversionCapability {
            from,
            to,
            lossy: false,
            execution_support: ExecutionSupport::PlannedOnly,
            backend: None,
            backend_label: Some("safetensors-to-hf-safetensors"),
            steps: SAFETENSORS_TO_HF_SAFETENSORS_STEPS,
        }),
        _ => None,
    }
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
