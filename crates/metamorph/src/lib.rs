use std::collections::HashMap;
use std::fmt;
use std::fs::{self, File};
use std::io::{BufReader, BufWriter, Write};
use std::path::{Path, PathBuf};
use std::str::FromStr;

use candle_core::quantized::gguf_file;
use candle_core::quantized::tokenizer::TokenizerFromGguf;
use candle_core::{Device, Tensor};
use serde::{Deserialize, Serialize};
use serde_json::{Map as JsonMap, Value as JsonValue, json};
use thiserror::Error;
use tokenizers::Tokenizer;

pub type Result<T> = std::result::Result<T, MetamorphError>;
const HF_TOKEN_ENV: &str = "HF_TOKEN";

#[derive(Debug, Error)]
pub enum MetamorphError {
    #[error("unsupported format `{0}`")]
    UnsupportedFormat(String),
    #[error("unsupported conversion path: {from} -> {to}")]
    UnsupportedConversionPath { from: Format, to: Format },
    #[error("lossy conversion requires explicit opt-in: {from} -> {to}")]
    LossyConversionRequiresOptIn { from: Format, to: Format },
    #[error("could not infer a source format from `{0}`")]
    UnknownFormatForSource(String),
    #[error("invalid Hugging Face source `{0}`")]
    InvalidHuggingFaceSource(String),
    #[error("path does not exist: {0}")]
    MissingPath(PathBuf),
    #[error("conversion execution does not support source `{0}` yet")]
    UnsupportedExecutionSource(String),
    #[error("conversion execution does not support target `{0}` yet")]
    UnsupportedExecutionTarget(String),
    #[error("expected a local GGUF file or a directory containing exactly one GGUF file: {0}")]
    InvalidLocalGgufSource(String),
    #[error("output bundle at `{path}` is invalid: {reason}")]
    InvalidOutputBundle { path: PathBuf, reason: String },
    #[error(
        "source `{input}` is not cached locally yet; expected a managed artifact under `{cache_path}`. Recover by populating that cache entry or using a local source path."
    )]
    SourceNotCached { input: String, cache_path: PathBuf },
    #[error("invalid publish destination `{0}`; expected `owner/name`")]
    InvalidPublishDestination(String),
    #[error(
        "publish execution for `{destination}` requires credentials in `{credential_env}`. Set that environment variable or rerun without `--execute` to keep this as a dry run."
    )]
    PublishCredentialsRequired {
        destination: String,
        credential_env: &'static str,
    },
    #[error(
        "remote publish execution is not implemented yet for `{0}`. Use the dry run to review the plan, keep the validated local bundle, and revisit execution once a backend and policy approval path exist."
    )]
    PublishExecutionNotImplemented(String),
    #[error("feature not implemented yet: {0}")]
    NotImplemented(&'static str),
    #[error(transparent)]
    Candle(#[from] candle_core::Error),
    #[error(transparent)]
    Json(#[from] serde_json::Error),
    #[error(transparent)]
    Tokenizer(#[from] tokenizers::Error),
    #[error(transparent)]
    Io(#[from] std::io::Error),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum Format {
    Gguf,
    HfSafetensors,
    Safetensors,
    Mlx,
}

impl Format {
    pub fn is_lossy_to(self, other: Self) -> bool {
        matches!(
            (self, other),
            (Self::Gguf, Self::HfSafetensors) | (Self::Gguf, Self::Safetensors)
        )
    }
}

impl fmt::Display for Format {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let label = match self {
            Self::Gguf => "gguf",
            Self::HfSafetensors => "hf-safetensors",
            Self::Safetensors => "safetensors",
            Self::Mlx => "mlx",
        };

        f.write_str(label)
    }
}

impl FromStr for Format {
    type Err = MetamorphError;

    fn from_str(value: &str) -> Result<Self> {
        let normalized = value.trim().to_ascii_lowercase().replace('_', "-");

        match normalized.as_str() {
            "gguf" => Ok(Self::Gguf),
            "hf-safetensors" | "huggingface-safetensors" | "hf" => Ok(Self::HfSafetensors),
            "safetensors" => Ok(Self::Safetensors),
            "mlx" => Ok(Self::Mlx),
            _ => Err(MetamorphError::UnsupportedFormat(value.to_owned())),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum Source {
    LocalPath(PathBuf),
    HuggingFace {
        repo: String,
        revision: Option<String>,
    },
}

impl Source {
    pub fn display_name(&self) -> String {
        match self {
            Self::LocalPath(path) => path.display().to_string(),
            Self::HuggingFace { repo, revision } => match revision {
                Some(revision) => format!("hf://{repo}@{revision}"),
                None => format!("hf://{repo}"),
            },
        }
    }
}

impl fmt::Display for Source {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.display_name())
    }
}

impl FromStr for Source {
    type Err = MetamorphError;

    fn from_str(value: &str) -> Result<Self> {
        if let Some(rest) = value.strip_prefix("hf://") {
            if rest.is_empty() {
                return Err(MetamorphError::InvalidHuggingFaceSource(value.to_owned()));
            }

            let (repo, revision) = match rest.split_once('@') {
                Some((repo, revision)) if !repo.is_empty() && !revision.is_empty() => {
                    (repo.to_owned(), Some(revision.to_owned()))
                }
                Some(_) => {
                    return Err(MetamorphError::InvalidHuggingFaceSource(value.to_owned()));
                }
                None => (rest.to_owned(), None),
            };

            return Ok(Self::HuggingFace { repo, revision });
        }

        Ok(Self::LocalPath(PathBuf::from(value)))
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum Target {
    LocalDir(PathBuf),
    HuggingFaceRepo(String),
}

impl fmt::Display for Target {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::LocalDir(path) => write!(f, "{}", path.display()),
            Self::HuggingFaceRepo(repo) => write!(f, "hf://{repo}"),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct InspectReport {
    pub source: Source,
    pub detected_format: Option<Format>,
    pub notes: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ConvertRequest {
    pub source: Source,
    pub target: Target,
    pub from: Option<Format>,
    pub to: Format,
    pub allow_lossy: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ConversionPlan {
    pub source_format: Format,
    pub target_format: Format,
    pub target: Target,
    pub lossy: bool,
    pub steps: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ValidationReport {
    pub path: PathBuf,
    pub format: Format,
    pub reusable: bool,
    pub checked_paths: Vec<PathBuf>,
    pub notes: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CacheIdentity {
    pub key: String,
    pub path: PathBuf,
    pub source_format: Option<Format>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum SourceAcquisitionOutcome {
    ReusedLocalPath,
    MaterializedLocalCopy,
    CacheHit,
    CacheMiss,
}

impl fmt::Display for SourceAcquisitionOutcome {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let label = match self {
            Self::ReusedLocalPath => "reused-local-path",
            Self::MaterializedLocalCopy => "materialized-local-copy",
            Self::CacheHit => "cache-hit",
            Self::CacheMiss => "cache-miss",
        };

        f.write_str(label)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SourceAcquisitionReport {
    pub source: Source,
    pub detected_format: Option<Format>,
    pub cache_identity: CacheIdentity,
    pub outcome: SourceAcquisitionOutcome,
    pub resolved_path: PathBuf,
    pub notes: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PublishPlan {
    pub input: PathBuf,
    pub validated_format: Format,
    pub destination: Target,
    pub artifacts: Vec<PathBuf>,
    pub steps: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PublishRequest {
    pub input: PathBuf,
    pub target: Target,
    pub execute: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PublishReport {
    pub plan: PublishPlan,
    pub executed: bool,
    pub notes: Vec<String>,
}

pub fn inspect(source: &Source) -> Result<InspectReport> {
    let mut notes = Vec::new();
    let detected_format = match source {
        Source::LocalPath(path) => detect_local_format(path, &mut notes)?,
        Source::HuggingFace { repo, revision } => {
            if let Some(revision) = revision {
                notes.push(format!("using pinned revision `{revision}`"));
            }
            detect_remote_format(repo)
        }
    };

    if detected_format.is_none() {
        notes.push("format could not be inferred yet".to_owned());
    }

    Ok(InspectReport {
        source: source.clone(),
        detected_format,
        notes,
    })
}

pub fn plan(request: &ConvertRequest) -> Result<ConversionPlan> {
    let inferred = inspect(&request.source)?;
    let source_format = request
        .from
        .or(inferred.detected_format)
        .ok_or_else(|| MetamorphError::UnknownFormatForSource(request.source.display_name()))?;

    if !supports_conversion(source_format, request.to) {
        return Err(MetamorphError::UnsupportedConversionPath {
            from: source_format,
            to: request.to,
        });
    }

    let lossy = source_format.is_lossy_to(request.to);
    if lossy && !request.allow_lossy {
        return Err(MetamorphError::LossyConversionRequiresOptIn {
            from: source_format,
            to: request.to,
        });
    }

    let steps = match (source_format, request.to) {
        (from, to) if from == to => vec![
            "inspect source artifacts".to_owned(),
            "normalize metadata and layout".to_owned(),
            "write target bundle".to_owned(),
        ],
        (Format::Gguf, Format::HfSafetensors) => vec![
            "fetch or read GGUF artifacts".to_owned(),
            "materialize tensors into a Hugging Face-style safetensors layout".to_owned(),
            "emit tokenizer and config files expected by downstream runtimes".to_owned(),
            "validate the output bundle".to_owned(),
        ],
        (Format::Gguf, Format::Safetensors) => vec![
            "fetch or read GGUF artifacts".to_owned(),
            "materialize tensors into safetensors".to_owned(),
            "validate converted weights".to_owned(),
        ],
        (Format::Safetensors, Format::HfSafetensors) => vec![
            "inspect safetensors artifacts".to_owned(),
            "normalize files into a Hugging Face-style repository layout".to_owned(),
            "validate the output bundle".to_owned(),
        ],
        _ => {
            return Err(MetamorphError::UnsupportedConversionPath {
                from: source_format,
                to: request.to,
            });
        }
    };

    Ok(ConversionPlan {
        source_format,
        target_format: request.to,
        target: request.target.clone(),
        lossy,
        steps,
    })
}

pub fn cache_dir() -> PathBuf {
    if let Ok(explicit) = std::env::var("METAMORPH_CACHE_DIR") {
        return PathBuf::from(explicit);
    }

    if let Ok(xdg_cache_home) = std::env::var("XDG_CACHE_HOME") {
        return PathBuf::from(xdg_cache_home).join("metamorph");
    }

    if let Ok(home) = std::env::var("HOME") {
        return PathBuf::from(home).join(".cache").join("metamorph");
    }

    PathBuf::from(".cache").join("metamorph")
}

pub fn cache_identity(source: &Source, from: Option<Format>) -> Result<CacheIdentity> {
    let inspect_report = inspect(source)?;
    let source_format = from.or(inspect_report.detected_format);
    let cache_key = build_cache_key(source, source_format);
    let source_kind = match source {
        Source::LocalPath(_) => "local",
        Source::HuggingFace { .. } => "hf",
    };
    let format_segment = source_format
        .map(|format| format.to_string())
        .unwrap_or_else(|| "unknown".to_owned());
    let locator_segment = cache_locator_segment(source);

    Ok(CacheIdentity {
        key: cache_key.clone(),
        path: cache_dir()
            .join("sources")
            .join(source_kind)
            .join(format_segment)
            .join(format!("{locator_segment}-{cache_key}")),
        source_format,
    })
}

pub fn acquire_source(
    source: &Source,
    from: Option<Format>,
    materialize_local_copy: bool,
) -> Result<SourceAcquisitionReport> {
    let inspect_report = inspect(source)?;
    let cache_identity = cache_identity(source, from)?;
    let detected_format = cache_identity
        .source_format
        .or(inspect_report.detected_format);
    let mut notes = inspect_report.notes;

    match source {
        Source::LocalPath(path) => {
            let resolved_source_path = fs::canonicalize(path).unwrap_or_else(|_| path.clone());
            if materialize_local_copy {
                let materialized = materialize_local_source(
                    &resolved_source_path,
                    &cache_identity.path,
                    detected_format,
                )?;
                let outcome = if materialized.preexisting {
                    SourceAcquisitionOutcome::CacheHit
                } else {
                    SourceAcquisitionOutcome::MaterializedLocalCopy
                };
                if outcome == SourceAcquisitionOutcome::MaterializedLocalCopy {
                    notes.push(format!(
                        "materialized a managed cache copy at `{}`",
                        materialized.path.display()
                    ));
                } else {
                    notes.push(format!(
                        "reused an existing managed cache copy at `{}`",
                        materialized.path.display()
                    ));
                }

                return Ok(SourceAcquisitionReport {
                    source: source.clone(),
                    detected_format,
                    cache_identity,
                    outcome,
                    resolved_path: materialized.path,
                    notes,
                });
            }

            notes.push(
                "reused existing local source without copying it into managed storage".to_owned(),
            );

            Ok(SourceAcquisitionReport {
                source: source.clone(),
                detected_format,
                cache_identity,
                outcome: SourceAcquisitionOutcome::ReusedLocalPath,
                resolved_path: resolved_source_path,
                notes,
            })
        }
        Source::HuggingFace { .. } => {
            if let Some(cached_path) =
                resolve_cached_source_path(&cache_identity.path, detected_format)?
            {
                notes.push(format!(
                    "using cached source artifact at `{}`",
                    cached_path.display()
                ));
                return Ok(SourceAcquisitionReport {
                    source: source.clone(),
                    detected_format,
                    cache_identity,
                    outcome: SourceAcquisitionOutcome::CacheHit,
                    resolved_path: cached_path,
                    notes,
                });
            }

            notes.push(format!(
                "remote fetch is not implemented yet; populate `{}` or use a local source path",
                cache_identity.path.display()
            ));

            Ok(SourceAcquisitionReport {
                source: source.clone(),
                detected_format,
                cache_identity: cache_identity.clone(),
                outcome: SourceAcquisitionOutcome::CacheMiss,
                resolved_path: expected_cached_source_path(
                    &cache_identity.path,
                    detected_format,
                    source,
                ),
                notes,
            })
        }
    }
}

pub fn convert(request: &ConvertRequest) -> Result<()> {
    let conversion_plan = plan(request)?;

    match (conversion_plan.source_format, conversion_plan.target_format) {
        (Format::Gguf, Format::HfSafetensors) => convert_local_gguf_to_hf_safetensors(request),
        _ => Err(MetamorphError::NotImplemented(
            "execution backend for this supported plan is not wired yet",
        )),
    }
}

pub fn validate(path: &Path, expected: Option<Format>) -> Result<ValidationReport> {
    let source = Source::LocalPath(path.to_path_buf());
    let inspect_report = inspect(&source)?;

    if let Some(expected) = expected {
        let checked_paths = validate_for_format(path, expected)?;

        if let Some(detected) = inspect_report.detected_format
            && detected != expected
        {
            return Err(MetamorphError::InvalidOutputBundle {
                path: path.to_path_buf(),
                reason: format!("expected `{expected}`, but detected `{detected}`"),
            });
        }

        return Ok(ValidationReport {
            path: path.to_path_buf(),
            format: expected,
            reusable: true,
            checked_paths,
            notes: vec![format!(
                "bundle satisfies the reusable `{expected}` contract"
            )],
        });
    }

    let detected = inspect_report
        .detected_format
        .ok_or_else(|| MetamorphError::UnknownFormatForSource(path.display().to_string()))?;
    let checked_paths = validate_for_format(path, detected)?;

    Ok(ValidationReport {
        path: path.to_path_buf(),
        format: detected,
        reusable: true,
        checked_paths,
        notes: vec![format!(
            "bundle satisfies the reusable `{detected}` contract"
        )],
    })
}

pub fn plan_publish(input: &Path, repo: &str) -> Result<PublishPlan> {
    validate_publish_destination(repo)?;
    let validation = validate(input, Some(Format::HfSafetensors))?;
    let artifacts = collect_publish_artifacts(input)?;

    Ok(PublishPlan {
        input: input.to_path_buf(),
        validated_format: validation.format,
        destination: Target::HuggingFaceRepo(repo.to_owned()),
        artifacts,
        steps: vec![
            "validate the local bundle before any remote activity".to_owned(),
            format!("preview the artifact set for hf://{repo}"),
            "execute remote writes only after an explicit operator opt-in".to_owned(),
        ],
    })
}

pub fn publish(request: &PublishRequest) -> Result<PublishReport> {
    let repo = match &request.target {
        Target::HuggingFaceRepo(repo) => repo.clone(),
        Target::LocalDir(path) => {
            return Err(MetamorphError::UnsupportedExecutionTarget(
                path.display().to_string(),
            ));
        }
    };
    let plan = plan_publish(&request.input, &repo)?;

    if !request.execute {
        return Ok(PublishReport {
            plan,
            executed: false,
            notes: vec![format!(
                "dry run only; rerun with --execute after reviewing licensing, credentials, and destination state for hf://{repo}"
            )],
        });
    }

    if std::env::var_os(HF_TOKEN_ENV).is_none() {
        return Err(MetamorphError::PublishCredentialsRequired {
            destination: format!("hf://{repo}"),
            credential_env: HF_TOKEN_ENV,
        });
    }

    Err(MetamorphError::PublishExecutionNotImplemented(format!(
        "hf://{repo}"
    )))
}

fn convert_local_gguf_to_hf_safetensors(request: &ConvertRequest) -> Result<()> {
    let source_path = resolve_local_gguf_path(&request.source)?;
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

    Ok(())
}

fn resolve_local_gguf_path(source: &Source) -> Result<PathBuf> {
    let acquisition = acquire_source(source, Some(Format::Gguf), false)?;

    match acquisition.outcome {
        SourceAcquisitionOutcome::ReusedLocalPath
        | SourceAcquisitionOutcome::MaterializedLocalCopy
        | SourceAcquisitionOutcome::CacheHit => {
            resolve_local_gguf_path_from_fs(&acquisition.resolved_path)
        }
        SourceAcquisitionOutcome::CacheMiss => Err(MetamorphError::SourceNotCached {
            input: source.display_name(),
            cache_path: acquisition.cache_identity.path,
        }),
    }
}

fn resolve_local_gguf_path_from_fs(path: &Path) -> Result<PathBuf> {
    if !path.exists() {
        return Err(MetamorphError::MissingPath(path.to_path_buf()));
    }

    if path.is_file() {
        return match detect_path_format(path) {
            Some(Format::Gguf) => Ok(path.to_path_buf()),
            _ => Err(MetamorphError::InvalidLocalGgufSource(
                path.display().to_string(),
            )),
        };
    }

    let mut gguf_files = fs::read_dir(path)?
        .filter_map(std::result::Result::ok)
        .map(|entry| entry.path())
        .filter(|entry| matches!(detect_path_format(entry), Some(Format::Gguf)))
        .collect::<Vec<_>>();
    gguf_files.sort();

    match gguf_files.as_slice() {
        [single] => Ok(single.clone()),
        [] => Err(MetamorphError::InvalidLocalGgufSource(
            path.display().to_string(),
        )),
        _ => Err(MetamorphError::InvalidLocalGgufSource(format!(
            "{} (multiple gguf files found)",
            path.display()
        ))),
    }
}

fn resolve_local_target_dir(target: &Target) -> Result<PathBuf> {
    match target {
        Target::LocalDir(path) => Ok(path.clone()),
        Target::HuggingFaceRepo(repo) => Err(MetamorphError::UnsupportedExecutionTarget(format!(
            "hf://{repo}"
        ))),
    }
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

fn validate_for_format(path: &Path, format: Format) -> Result<Vec<PathBuf>> {
    match format {
        Format::HfSafetensors => validate_hf_safetensors_bundle(path),
        Format::Safetensors => validate_safetensors_artifacts(path),
        _ => Err(MetamorphError::NotImplemented(
            "validation is not wired yet for this format",
        )),
    }
}

fn validate_hf_safetensors_bundle(path: &Path) -> Result<Vec<PathBuf>> {
    let mut checked_paths = Vec::new();
    for required in [
        "config.json",
        "tokenizer.json",
        "generation_config.json",
        "model.safetensors",
    ] {
        let required_path = path.join(required);
        if !required_path.is_file() {
            return Err(MetamorphError::InvalidOutputBundle {
                path: path.to_path_buf(),
                reason: format!("missing required file `{required}`"),
            });
        }
        checked_paths.push(required_path);
    }

    let report = inspect(&Source::LocalPath(path.to_path_buf()))?;
    if report.detected_format != Some(Format::HfSafetensors) {
        return Err(MetamorphError::InvalidOutputBundle {
            path: path.to_path_buf(),
            reason: "output does not inspect as `hf-safetensors`".to_owned(),
        });
    }

    Ok(checked_paths)
}

fn validate_safetensors_artifacts(path: &Path) -> Result<Vec<PathBuf>> {
    if !path.exists() {
        return Err(MetamorphError::MissingPath(path.to_path_buf()));
    }

    if path.is_file() {
        if detect_path_format(path) != Some(Format::Safetensors) {
            return Err(MetamorphError::InvalidOutputBundle {
                path: path.to_path_buf(),
                reason: "expected a `.safetensors` file".to_owned(),
            });
        }

        candle_core::safetensors::load(path, &Device::Cpu)?;
        return Ok(vec![path.to_path_buf()]);
    }

    let safetensors_files = fs::read_dir(path)?
        .filter_map(std::result::Result::ok)
        .map(|entry| entry.path())
        .filter(|entry| detect_path_format(entry) == Some(Format::Safetensors))
        .collect::<Vec<_>>();

    if safetensors_files.is_empty() {
        return Err(MetamorphError::InvalidOutputBundle {
            path: path.to_path_buf(),
            reason: "missing required safetensors artifacts".to_owned(),
        });
    }

    for safetensors_file in safetensors_files {
        candle_core::safetensors::load(&safetensors_file, &Device::Cpu)?;
    }

    Ok(fs::read_dir(path)?
        .filter_map(std::result::Result::ok)
        .map(|entry| entry.path())
        .filter(|entry| detect_path_format(entry) == Some(Format::Safetensors))
        .collect())
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

fn supports_conversion(from: Format, to: Format) -> bool {
    from == to
        || matches!(
            (from, to),
            (Format::Gguf, Format::HfSafetensors)
                | (Format::Gguf, Format::Safetensors)
                | (Format::Safetensors, Format::HfSafetensors)
        )
}

fn build_cache_key(source: &Source, source_format: Option<Format>) -> String {
    let mut key_input = match source {
        Source::LocalPath(path) => format!(
            "local:{}",
            fs::canonicalize(path)
                .unwrap_or_else(|_| path.clone())
                .display()
        ),
        Source::HuggingFace { repo, revision } => {
            format!("hf:{repo}@{}", revision.as_deref().unwrap_or("default"))
        }
    };

    key_input.push(':');
    key_input.push_str(
        &source_format
            .map(|format| format.to_string())
            .unwrap_or_else(|| "unknown".to_owned()),
    );

    format!("{:016x}", fnv1a_hash(&key_input))
}

fn cache_locator_segment(source: &Source) -> String {
    match source {
        Source::LocalPath(path) => {
            let label = path
                .file_name()
                .and_then(|name| name.to_str())
                .unwrap_or("local-source");
            sanitize_cache_segment(label)
        }
        Source::HuggingFace { repo, revision } => {
            let base = match revision {
                Some(revision) => format!("{repo}-{revision}"),
                None => repo.clone(),
            };
            sanitize_cache_segment(&base)
        }
    }
}

fn sanitize_cache_segment(value: &str) -> String {
    let sanitized = value
        .chars()
        .map(|character| {
            if character.is_ascii_alphanumeric() {
                character.to_ascii_lowercase()
            } else {
                '-'
            }
        })
        .collect::<String>();
    let collapsed = sanitized
        .split('-')
        .filter(|segment| !segment.is_empty())
        .collect::<Vec<_>>()
        .join("-");

    if collapsed.is_empty() {
        "source".to_owned()
    } else {
        collapsed
    }
}

fn fnv1a_hash(value: &str) -> u64 {
    const OFFSET_BASIS: u64 = 0xcbf29ce484222325;
    const PRIME: u64 = 0x0000_0100_0000_01b3;

    let mut hash = OFFSET_BASIS;
    for byte in value.as_bytes() {
        hash ^= u64::from(*byte);
        hash = hash.wrapping_mul(PRIME);
    }

    hash
}

fn expected_cached_source_path(
    cache_root: &Path,
    detected_format: Option<Format>,
    source: &Source,
) -> PathBuf {
    match detected_format {
        Some(Format::Gguf) => cache_root.join(cached_file_name(source, "gguf")),
        Some(Format::Safetensors) => cache_root.join(cached_file_name(source, "safetensors")),
        _ => cache_root.to_path_buf(),
    }
}

fn cached_file_name(source: &Source, extension: &str) -> String {
    let base = match source {
        Source::LocalPath(path) => path
            .file_stem()
            .and_then(|stem| stem.to_str())
            .map(sanitize_cache_segment)
            .unwrap_or_else(|| "source".to_owned()),
        Source::HuggingFace { repo, .. } => sanitize_cache_segment(repo),
    };
    format!("{base}.{extension}")
}

fn resolve_cached_source_path(path: &Path, format: Option<Format>) -> Result<Option<PathBuf>> {
    if !path.exists() {
        return Ok(None);
    }

    match format {
        Some(Format::Gguf) => resolve_local_gguf_path_from_fs(path).map(Some),
        Some(Format::HfSafetensors) => {
            validate_hf_safetensors_bundle(path)?;
            Ok(Some(path.to_path_buf()))
        }
        Some(Format::Safetensors) => {
            validate_safetensors_artifacts(path)?;
            Ok(Some(path.to_path_buf()))
        }
        _ => Ok(Some(path.to_path_buf())),
    }
}

struct MaterializedLocalSource {
    path: PathBuf,
    preexisting: bool,
}

fn materialize_local_source(
    source_path: &Path,
    cache_root: &Path,
    detected_format: Option<Format>,
) -> Result<MaterializedLocalSource> {
    let destination = expected_cached_source_path(
        cache_root,
        detected_format,
        &Source::LocalPath(source_path.to_path_buf()),
    );

    if destination.exists() || cache_root.exists() {
        let resolved_path =
            resolve_cached_source_path(cache_root, detected_format)?.unwrap_or(destination);
        return Ok(MaterializedLocalSource {
            path: resolved_path,
            preexisting: true,
        });
    }

    if source_path.is_file() {
        fs::create_dir_all(cache_root)?;
        fs::copy(source_path, &destination)?;
        return Ok(MaterializedLocalSource {
            path: destination,
            preexisting: false,
        });
    }

    copy_dir_all(source_path, cache_root)?;
    Ok(MaterializedLocalSource {
        path: cache_root.to_path_buf(),
        preexisting: false,
    })
}

fn copy_dir_all(source: &Path, destination: &Path) -> Result<()> {
    fs::create_dir_all(destination)?;
    for entry in fs::read_dir(source)? {
        let entry = entry?;
        let source_path = entry.path();
        let destination_path = destination.join(entry.file_name());
        if source_path.is_dir() {
            copy_dir_all(&source_path, &destination_path)?;
        } else {
            if let Some(parent) = destination_path.parent() {
                fs::create_dir_all(parent)?;
            }
            fs::copy(&source_path, &destination_path)?;
        }
    }

    Ok(())
}

fn validate_publish_destination(repo: &str) -> Result<()> {
    let Some((owner, name)) = repo.split_once('/') else {
        return Err(MetamorphError::InvalidPublishDestination(repo.to_owned()));
    };

    if owner.trim().is_empty()
        || name.trim().is_empty()
        || owner.contains(char::is_whitespace)
        || name.contains(char::is_whitespace)
    {
        return Err(MetamorphError::InvalidPublishDestination(repo.to_owned()));
    }

    Ok(())
}

fn collect_publish_artifacts(path: &Path) -> Result<Vec<PathBuf>> {
    if path.is_file() {
        return Ok(vec![path.to_path_buf()]);
    }

    let mut artifacts = fs::read_dir(path)?
        .filter_map(std::result::Result::ok)
        .map(|entry| entry.path())
        .filter(|entry| entry.is_file())
        .collect::<Vec<_>>();
    artifacts.sort();

    Ok(artifacts)
}

fn detect_local_format(path: &Path, notes: &mut Vec<String>) -> Result<Option<Format>> {
    if !path.exists() {
        return Err(MetamorphError::MissingPath(path.to_path_buf()));
    }

    if path.is_file() {
        return Ok(detect_path_format(path));
    }

    let entries = fs::read_dir(path)?
        .filter_map(std::result::Result::ok)
        .map(|entry| entry.path())
        .collect::<Vec<_>>();

    if entries
        .iter()
        .any(|entry| matches!(detect_path_format(entry), Some(Format::Gguf)))
    {
        return Ok(Some(Format::Gguf));
    }

    let has_config = entries
        .iter()
        .any(|entry| entry.file_name().is_some_and(|name| name == "config.json"));
    let has_tokenizer = entries.iter().any(|entry| {
        entry
            .file_name()
            .is_some_and(|name| name == "tokenizer.json")
    });
    let has_safetensors = entries
        .iter()
        .any(|entry| matches!(detect_path_format(entry), Some(Format::Safetensors)));

    if has_config && has_tokenizer && has_safetensors {
        notes.push("detected Hugging Face-style model layout".to_owned());
        return Ok(Some(Format::HfSafetensors));
    }

    if has_safetensors {
        notes.push("found safetensors files but not a complete Hugging Face layout".to_owned());
        return Ok(Some(Format::Safetensors));
    }

    Ok(None)
}

fn detect_remote_format(repo: &str) -> Option<Format> {
    let normalized = repo.to_ascii_lowercase();

    if normalized.contains("gguf") {
        return Some(Format::Gguf);
    }

    if normalized.contains("mlx") {
        return Some(Format::Mlx);
    }

    if normalized.contains("safetensors") {
        return Some(Format::HfSafetensors);
    }

    None
}

fn detect_path_format(path: &Path) -> Option<Format> {
    match path.extension().and_then(|extension| extension.to_str()) {
        Some("gguf") => Some(Format::Gguf),
        Some("safetensors") => Some(Format::Safetensors),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::{
        ConvertRequest, Format, PublishRequest, Source, SourceAcquisitionOutcome, Target,
        acquire_source, cache_identity, convert, inspect, plan, plan_publish, publish, validate,
    };
    use candle_core::quantized::gguf_file;
    use candle_core::quantized::{GgmlDType, QTensor};
    use candle_core::{Device, Tensor};
    use serde_json::Value as JsonValue;
    use std::collections::HashMap;
    use std::fs::{self, File};
    use std::io::{BufWriter, Write};
    use std::str::FromStr;
    use std::sync::Mutex;
    use tempfile::tempdir;

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
    fn requires_lossy_opt_in_for_gguf_to_hf_safetensors() {
        let request = ConvertRequest {
            source: Source::from_str("hf://prism-ml/Bonsai-8B-gguf").unwrap(),
            target: Target::LocalDir("target/out".into()),
            from: Some(Format::Gguf),
            to: Format::HfSafetensors,
            allow_lossy: false,
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
        };

        let plan = plan(&request).unwrap();

        assert!(!plan.lossy);
        assert_eq!(plan.source_format, Format::Safetensors);
        assert_eq!(plan.target_format, Format::Safetensors);
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
        assert!(report.notes.contains(
            &"found safetensors files but not a complete Hugging Face layout".to_owned()
        ));
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
        let temp = tempdir().unwrap();
        let source_path = temp.path().join("fixture.gguf");
        write_fixture_gguf(&source_path);

        let first = cache_identity(&Source::LocalPath(source_path.clone()), None).unwrap();
        let second = cache_identity(&Source::LocalPath(source_path), None).unwrap();

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
    fn acquire_source_reports_cache_miss_for_remote_source() {
        let _env_guard = ENV_LOCK.lock().unwrap();
        let temp = tempdir().unwrap();
        let cache_root = temp.path().join("cache");
        let source = Source::from_str("hf://prism-ml/Bonsai-8B-gguf@main").unwrap();

        unsafe { std::env::set_var("METAMORPH_CACHE_DIR", &cache_root) };
        let report = acquire_source(&source, None, false).unwrap();
        unsafe { std::env::remove_var("METAMORPH_CACHE_DIR") };

        assert_eq!(report.outcome, SourceAcquisitionOutcome::CacheMiss);
        assert!(
            report
                .resolved_path
                .ends_with("prism-ml-bonsai-8b-gguf.gguf")
        );
    }

    #[test]
    fn acquire_source_uses_cached_remote_artifact_when_present() {
        let _env_guard = ENV_LOCK.lock().unwrap();
        let temp = tempdir().unwrap();
        let cache_root = temp.path().join("cache");
        let source = Source::from_str("hf://prism-ml/Bonsai-8B-gguf@main").unwrap();

        unsafe { std::env::set_var("METAMORPH_CACHE_DIR", &cache_root) };
        let identity = cache_identity(&source, None).unwrap();
        fs::create_dir_all(&identity.path).unwrap();
        let cached_source_path = identity.path.join("prism-ml-bonsai-8b-gguf.gguf");
        write_fixture_gguf(&cached_source_path);
        let report = acquire_source(&source, None, false).unwrap();
        unsafe { std::env::remove_var("METAMORPH_CACHE_DIR") };

        assert_eq!(report.outcome, SourceAcquisitionOutcome::CacheHit);
        assert_eq!(report.resolved_path, cached_source_path);
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
}
