use std::fmt;
use std::fs;
use std::path::{Path, PathBuf};
use std::str::FromStr;

use serde::{Deserialize, Serialize};
use thiserror::Error;

pub type Result<T> = std::result::Result<T, MetamorphError>;

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
    #[error("feature not implemented yet: {0}")]
    NotImplemented(&'static str),
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

pub fn convert(_request: &ConvertRequest) -> Result<()> {
    Err(MetamorphError::NotImplemented(
        "conversion execution backends are not wired yet",
    ))
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
    use super::{ConvertRequest, Format, Source, Target, plan};
    use std::str::FromStr;

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
}
