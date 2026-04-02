use std::fmt;
use std::fs;
use std::path::{Path, PathBuf};
use std::str::FromStr;

use serde::{Deserialize, Serialize};

use crate::error::{MetamorphError, Result};
use crate::format::Format;

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

pub(crate) fn detect_local_format(path: &Path, notes: &mut Vec<String>) -> Result<Option<Format>> {
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

pub(crate) fn detect_remote_format(repo: &str) -> Option<Format> {
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

pub(crate) fn detect_path_format(path: &Path) -> Option<Format> {
    match path.extension().and_then(|extension| extension.to_str()) {
        Some("gguf") => Some(Format::Gguf),
        Some("safetensors") => Some(Format::Safetensors),
        _ => None,
    }
}

pub(crate) fn resolve_local_gguf_path_from_fs(path: &Path) -> Result<PathBuf> {
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
