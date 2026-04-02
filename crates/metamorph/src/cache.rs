use std::fmt;
use std::fs;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use crate::error::Result;
use crate::format::Format;
use crate::source::{Source, inspect, resolve_local_gguf_path_from_fs};
use crate::validate::{validate_hf_safetensors_bundle, validate_safetensors_artifacts};

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

    if destination.exists() {
        let resolved_path =
            resolve_cached_source_path(cache_root, detected_format)?.unwrap_or(destination);
        return Ok(MaterializedLocalSource {
            path: resolved_path,
            preexisting: true,
        });
    }

    if cache_root.exists()
        && let Some(resolved_path) = resolve_cached_source_path(cache_root, detected_format)?
    {
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
