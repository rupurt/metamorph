use std::fmt;
use std::fs;
use std::io::BufReader;
use std::path::{Path, PathBuf};

use candle_core::quantized::gguf_file;
use serde::{Deserialize, Serialize};

use crate::error::{MetamorphError, Result};
use crate::format::Format;
use crate::remote::provider_from_env;
use crate::source::{Source, inspect, resolve_local_gguf_path_from_fs};
use crate::validate::{validate_hf_safetensors_bundle, validate_safetensors_artifacts};

const REMOTE_MANIFEST_FILE: &str = ".metamorph-remote-source.json";

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CacheIdentity {
    pub key: String,
    pub path: PathBuf,
    pub source_format: Option<Format>,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct SourceAcquisitionOptions {
    pub materialize_local_copy: bool,
    pub refresh_remote: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum SourceAcquisitionOutcome {
    ReusedLocalPath,
    MaterializedLocalCopy,
    ReusedManagedLocalCopy,
    ReusedRemoteCache,
    FetchedRemoteArtifact,
    RefreshedRemoteArtifact,
}

impl fmt::Display for SourceAcquisitionOutcome {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let label = match self {
            Self::ReusedLocalPath => "reused-local-path",
            Self::MaterializedLocalCopy => "materialized-local-copy",
            Self::ReusedManagedLocalCopy => "reused-managed-local-copy",
            Self::ReusedRemoteCache => "reused-remote-cache",
            Self::FetchedRemoteArtifact => "fetched-remote",
            Self::RefreshedRemoteArtifact => "refreshed-remote",
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
struct RemoteSourceManifest {
    provider: String,
    repo: String,
    requested_revision: String,
    resolved_revision: String,
    artifact_path: String,
    source_format: Format,
}

struct MaterializedLocalSource {
    path: PathBuf,
    preexisting: bool,
}

struct RemoteCacheState {
    path: PathBuf,
    manifest: Option<RemoteSourceManifest>,
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
    acquire_source_with_options(
        source,
        from,
        SourceAcquisitionOptions {
            materialize_local_copy,
            refresh_remote: false,
        },
    )
}

pub fn acquire_source_with_options(
    source: &Source,
    from: Option<Format>,
    options: SourceAcquisitionOptions,
) -> Result<SourceAcquisitionReport> {
    let inspect_report = inspect(source)?;
    let cache_identity = cache_identity(source, from)?;
    let detected_format = cache_identity
        .source_format
        .or(inspect_report.detected_format);
    let notes = inspect_report.notes;

    match source {
        Source::LocalPath(path) => acquire_local_source(
            source,
            path,
            detected_format,
            cache_identity,
            notes,
            options,
        ),
        Source::HuggingFace { .. } => {
            acquire_remote_source(source, detected_format, cache_identity, notes, options)
        }
    }
}

fn acquire_local_source(
    source: &Source,
    path: &Path,
    detected_format: Option<Format>,
    cache_identity: CacheIdentity,
    mut notes: Vec<String>,
    options: SourceAcquisitionOptions,
) -> Result<SourceAcquisitionReport> {
    let resolved_source_path = fs::canonicalize(path).unwrap_or_else(|_| path.to_path_buf());
    if options.materialize_local_copy {
        let materialized =
            materialize_local_source(&resolved_source_path, &cache_identity.path, detected_format)?;
        let outcome = if materialized.preexisting {
            SourceAcquisitionOutcome::ReusedManagedLocalCopy
        } else {
            SourceAcquisitionOutcome::MaterializedLocalCopy
        };
        if materialized.preexisting {
            notes.push(format!(
                "reused an existing managed cache copy at `{}`",
                materialized.path.display()
            ));
        } else {
            notes.push(format!(
                "materialized a managed cache copy at `{}`",
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

    notes.push("reused existing local source without copying it into managed storage".to_owned());
    if options.refresh_remote {
        notes.push(
            "ignored `refresh_remote`; refresh only applies to remote `hf://...` sources"
                .to_owned(),
        );
    }

    Ok(SourceAcquisitionReport {
        source: source.clone(),
        detected_format,
        cache_identity,
        outcome: SourceAcquisitionOutcome::ReusedLocalPath,
        resolved_path: resolved_source_path,
        notes,
    })
}

fn acquire_remote_source(
    source: &Source,
    detected_format: Option<Format>,
    cache_identity: CacheIdentity,
    mut notes: Vec<String>,
    options: SourceAcquisitionOptions,
) -> Result<SourceAcquisitionReport> {
    if !options.refresh_remote
        && let Some(reused_state) =
            resolve_remote_cache_state(source, &cache_identity.path, detected_format)?
    {
        match reused_state.manifest {
            Some(manifest) => notes.push(format!(
                "reused cached remote artifact `{}` from resolved revision `{}`",
                manifest.artifact_path, manifest.resolved_revision
            )),
            None => notes.push(
                "reused a legacy cached remote artifact without revision metadata; rerun with `--refresh` to replace it with managed remote metadata".to_owned(),
            ),
        }

        return Ok(SourceAcquisitionReport {
            source: source.clone(),
            detected_format,
            cache_identity,
            outcome: SourceAcquisitionOutcome::ReusedRemoteCache,
            resolved_path: reused_state.path,
            notes,
        });
    }

    if detected_format != Some(Format::Gguf) {
        return Err(MetamorphError::RemoteFetchUnsupported {
            input: source.display_name(),
            reason: match detected_format {
                Some(format) => format!(
                    "remote on-demand fetch is currently limited to representative GGUF sources, not `{format}`"
                ),
                None => "Metamorph could not infer the remote format yet".to_owned(),
            },
        });
    }

    let provider = provider_from_env();
    let descriptor = provider.resolve_gguf(source)?;
    let had_existing_cache = cache_identity.path.exists();
    let manifest = RemoteSourceManifest {
        provider: "hugging-face".to_owned(),
        repo: descriptor.repo.clone(),
        requested_revision: descriptor.requested_revision.clone(),
        resolved_revision: descriptor.resolved_revision.clone(),
        artifact_path: descriptor.remote_path.clone(),
        source_format: descriptor.source_format,
    };

    let stage_root = staging_cache_root(&cache_identity.path);
    if stage_root.exists() {
        fs::remove_dir_all(&stage_root)?;
    }
    fs::create_dir_all(&stage_root)?;

    let stage_result = (|| -> Result<PathBuf> {
        let stage_artifact_path = stage_root.join(descriptor.cached_file_name());
        provider.download(&descriptor, &stage_artifact_path)?;
        validate_remote_artifact(&stage_artifact_path, descriptor.source_format).map_err(
            |error| MetamorphError::RemoteTransferInterrupted {
                input: source.display_name(),
                cache_path: stage_artifact_path.clone(),
                reason: error.to_string(),
            },
        )?;
        write_remote_manifest(&stage_root, &manifest)?;
        replace_cache_root(&stage_root, &cache_identity.path)?;
        Ok(cache_identity.path.join(descriptor.cached_file_name()))
    })();

    if stage_result.is_err() {
        let _ = fs::remove_dir_all(&stage_root);
    }

    let resolved_path = stage_result?;
    let outcome = if options.refresh_remote && had_existing_cache {
        SourceAcquisitionOutcome::RefreshedRemoteArtifact
    } else {
        SourceAcquisitionOutcome::FetchedRemoteArtifact
    };
    notes.push(format!(
        "{} remote artifact `{}` from resolved revision `{}` into managed cache",
        match outcome {
            SourceAcquisitionOutcome::RefreshedRemoteArtifact => "refreshed",
            _ => "fetched",
        },
        manifest.artifact_path,
        manifest.resolved_revision
    ));

    Ok(SourceAcquisitionReport {
        source: source.clone(),
        detected_format,
        cache_identity,
        outcome,
        resolved_path,
        notes,
    })
}

fn build_cache_key(source: &Source, source_format: Option<Format>) -> String {
    let mut key_input = match source {
        Source::LocalPath(path) => format!(
            "local:{}",
            fs::canonicalize(path)
                .unwrap_or_else(|_| path.to_path_buf())
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
        Some(Format::Gguf) => {
            let gguf_path = resolve_local_gguf_path_from_fs(path)?;
            validate_remote_artifact(&gguf_path, Format::Gguf)?;
            Ok(Some(gguf_path))
        }
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

fn resolve_remote_cache_state(
    source: &Source,
    cache_root: &Path,
    detected_format: Option<Format>,
) -> Result<Option<RemoteCacheState>> {
    if !cache_root.exists() {
        return Ok(None);
    }

    if let Some(manifest) = read_remote_manifest(cache_root, source)? {
        validate_remote_manifest(source, cache_root, &manifest)?;
        let cached_path = cache_root.join(
            Path::new(&manifest.artifact_path)
                .file_name()
                .and_then(|name| name.to_str())
                .unwrap_or("model.gguf"),
        );
        validate_remote_artifact(&cached_path, manifest.source_format).map_err(|error| {
            MetamorphError::RemoteCacheStateInvalid {
                input: source.display_name(),
                cache_path: cache_root.to_path_buf(),
                reason: error.to_string(),
            }
        })?;

        return Ok(Some(RemoteCacheState {
            path: cached_path,
            manifest: Some(manifest),
        }));
    }

    match resolve_cached_source_path(cache_root, detected_format) {
        Ok(Some(path)) => Ok(Some(RemoteCacheState {
            path,
            manifest: None,
        })),
        Ok(None) => Ok(None),
        Err(error) => Err(MetamorphError::RemoteCacheStateInvalid {
            input: source.display_name(),
            cache_path: cache_root.to_path_buf(),
            reason: error.to_string(),
        }),
    }
}

fn validate_remote_manifest(
    source: &Source,
    cache_root: &Path,
    manifest: &RemoteSourceManifest,
) -> Result<()> {
    let Source::HuggingFace { repo, revision } = source else {
        return Ok(());
    };
    let requested_revision = revision.clone().unwrap_or_else(|| "main".to_owned());
    if manifest.repo != *repo {
        return Err(MetamorphError::RemoteCacheStateInvalid {
            input: source.display_name(),
            cache_path: cache_root.to_path_buf(),
            reason: format!(
                "manifest repo `{}` does not match requested repo `{repo}`",
                manifest.repo
            ),
        });
    }
    if manifest.requested_revision != requested_revision {
        return Err(MetamorphError::RemoteCacheStateInvalid {
            input: source.display_name(),
            cache_path: cache_root.to_path_buf(),
            reason: format!(
                "manifest revision `{}` does not match requested revision `{requested_revision}`",
                manifest.requested_revision
            ),
        });
    }
    Ok(())
}

fn read_remote_manifest(
    cache_root: &Path,
    source: &Source,
) -> Result<Option<RemoteSourceManifest>> {
    let path = cache_root.join(REMOTE_MANIFEST_FILE);
    if !path.exists() {
        return Ok(None);
    }

    let content = fs::read_to_string(&path)?;
    let manifest = serde_json::from_str(&content).map_err(|error| {
        MetamorphError::RemoteCacheStateInvalid {
            input: source.display_name(),
            cache_path: cache_root.to_path_buf(),
            reason: format!("failed to parse `{REMOTE_MANIFEST_FILE}`: {error}"),
        }
    })?;
    Ok(Some(manifest))
}

fn write_remote_manifest(cache_root: &Path, manifest: &RemoteSourceManifest) -> Result<()> {
    let path = cache_root.join(REMOTE_MANIFEST_FILE);
    let content = serde_json::to_vec_pretty(manifest)?;
    fs::write(path, content)?;
    Ok(())
}

fn validate_remote_artifact(path: &Path, format: Format) -> Result<()> {
    match format {
        Format::Gguf => validate_gguf_artifact(path),
        Format::HfSafetensors => {
            validate_hf_safetensors_bundle(path)?;
            Ok(())
        }
        Format::Safetensors => {
            validate_safetensors_artifacts(path)?;
            Ok(())
        }
        _ => Ok(()),
    }
}

fn validate_gguf_artifact(path: &Path) -> Result<()> {
    let gguf_path = resolve_local_gguf_path_from_fs(path)?;
    let mut reader = BufReader::new(fs::File::open(&gguf_path)?);
    gguf_file::Content::read(&mut reader).map_err(MetamorphError::from)?;
    Ok(())
}

fn staging_cache_root(cache_root: &Path) -> PathBuf {
    let file_name = cache_root
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or("source");
    cache_root
        .parent()
        .unwrap_or_else(|| Path::new("."))
        .join(format!(".{file_name}.staging"))
}

fn backup_cache_root(cache_root: &Path) -> PathBuf {
    let file_name = cache_root
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or("source");
    cache_root
        .parent()
        .unwrap_or_else(|| Path::new("."))
        .join(format!(".{file_name}.backup"))
}

fn replace_cache_root(stage_root: &Path, cache_root: &Path) -> Result<()> {
    let backup_root = backup_cache_root(cache_root);
    if backup_root.exists() {
        fs::remove_dir_all(&backup_root)?;
    }

    if cache_root.exists() {
        fs::rename(cache_root, &backup_root)?;
        if let Err(error) = fs::rename(stage_root, cache_root) {
            let _ = fs::rename(&backup_root, cache_root);
            return Err(error.into());
        }
        fs::remove_dir_all(&backup_root)?;
    } else {
        fs::rename(stage_root, cache_root)?;
    }

    Ok(())
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
