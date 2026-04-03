use std::env;
use std::fs::{self, File};
use std::io::{Read, Write};
use std::path::{Path, PathBuf};

use hf_hub::api::sync::{ApiBuilder, ApiError};
use hf_hub::{Repo, RepoType};
use serde::Deserialize;

use crate::error::{HF_TOKEN_ENV, MetamorphError, Result};
use crate::format::Format;
use crate::source::Source;

pub(crate) const MOCK_ROOT_ENV: &str = "METAMORPH_HF_MOCK_ROOT";
const MOCK_CONFIG_FILE: &str = ".metamorph-hf.json";

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct RemoteArtifactDescriptor {
    pub source: Source,
    pub repo: String,
    pub requested_revision: String,
    pub resolved_revision: String,
    pub remote_path: String,
    pub source_format: Format,
}

impl RemoteArtifactDescriptor {
    pub(crate) fn cached_file_name(&self) -> String {
        Path::new(&self.remote_path)
            .file_name()
            .and_then(|name| name.to_str())
            .unwrap_or("model.gguf")
            .to_owned()
    }
}

pub(crate) trait RemoteSourceProvider {
    fn resolve_gguf(&self, source: &Source) -> Result<RemoteArtifactDescriptor>;
    fn download(&self, artifact: &RemoteArtifactDescriptor, destination: &Path) -> Result<()>;
}

pub(crate) fn provider_from_env() -> Box<dyn RemoteSourceProvider> {
    match env::var(MOCK_ROOT_ENV) {
        Ok(root) => Box::new(MockHuggingFaceProvider::new(PathBuf::from(root))),
        Err(_) => Box::new(LiveHuggingFaceProvider),
    }
}

#[derive(Debug)]
struct LiveHuggingFaceProvider;

impl RemoteSourceProvider for LiveHuggingFaceProvider {
    fn resolve_gguf(&self, source: &Source) -> Result<RemoteArtifactDescriptor> {
        let (repo, revision) = remote_repo_revision(source)?;
        let api = live_api()?;
        let api_repo = api.repo(Repo::with_revision(
            repo.to_owned(),
            RepoType::Model,
            revision.clone(),
        ));
        let info = api_repo
            .info()
            .map_err(|error| map_live_api_error(source, None, error))?;
        let gguf_files = info
            .siblings
            .into_iter()
            .map(|sibling| sibling.rfilename)
            .filter(|filename| filename.to_ascii_lowercase().ends_with(".gguf"))
            .collect::<Vec<_>>();

        let remote_path = match gguf_files.as_slice() {
            [single] => single.clone(),
            [] => {
                return Err(MetamorphError::RemoteLayoutUnsupported {
                    input: source.display_name(),
                    reason: "no `.gguf` artifact was found at the requested revision".to_owned(),
                });
            }
            multiple => {
                return Err(MetamorphError::RemoteLayoutUnsupported {
                    input: source.display_name(),
                    reason: format!(
                        "multiple `.gguf` artifacts were found ({}) and Metamorph cannot choose one implicitly yet",
                        multiple.join(", ")
                    ),
                });
            }
        };

        Ok(RemoteArtifactDescriptor {
            source: source.clone(),
            repo: repo.to_owned(),
            requested_revision: revision,
            resolved_revision: info.sha,
            remote_path,
            source_format: Format::Gguf,
        })
    }

    fn download(&self, artifact: &RemoteArtifactDescriptor, destination: &Path) -> Result<()> {
        let api = live_api()?;
        let api_repo = api.repo(Repo::with_revision(
            artifact.repo.clone(),
            RepoType::Model,
            artifact.requested_revision.clone(),
        ));
        let downloaded = api_repo
            .download(&artifact.remote_path)
            .map_err(|error| map_live_api_error(&artifact.source, Some(destination), error))?;

        if let Some(parent) = destination.parent() {
            fs::create_dir_all(parent)?;
        }
        fs::copy(downloaded, destination)?;
        Ok(())
    }
}

fn live_api() -> Result<hf_hub::api::sync::Api> {
    let mut builder = ApiBuilder::from_env().with_progress(false);
    if let Ok(token) = env::var(HF_TOKEN_ENV) {
        let trimmed = token.trim();
        if !trimmed.is_empty() {
            builder = builder.with_token(Some(trimmed.to_owned()));
        }
    }

    builder
        .build()
        .map_err(|error| MetamorphError::RemoteTransferInterrupted {
            input: "hf://client".to_owned(),
            cache_path: PathBuf::from("."),
            reason: format!("failed to initialize the Hugging Face transport: {error}"),
        })
}

fn map_live_api_error(
    source: &Source,
    destination: Option<&Path>,
    error: ApiError,
) -> MetamorphError {
    let rendered = error.to_string();
    if rendered.contains("401") || rendered.contains("403") {
        return MetamorphError::RemoteCredentialsRequired {
            input: source.display_name(),
            credential_env: HF_TOKEN_ENV,
        };
    }
    if rendered.contains("404") {
        return MetamorphError::RemoteRevisionNotFound {
            input: source.display_name(),
        };
    }

    MetamorphError::RemoteTransferInterrupted {
        input: source.display_name(),
        cache_path: destination.unwrap_or_else(|| Path::new(".")).to_path_buf(),
        reason: rendered,
    }
}

#[derive(Debug, Clone)]
pub(crate) struct MockHuggingFaceProvider {
    root: PathBuf,
}

impl MockHuggingFaceProvider {
    pub(crate) fn new(root: PathBuf) -> Self {
        Self { root }
    }

    fn revision_root(&self, repo: &str, revision: &str) -> PathBuf {
        self.root.join(repo).join(revision)
    }
}

#[derive(Debug, Default, Deserialize)]
struct MockRepoConfig {
    resolved_revision: Option<String>,
    interrupt_after_bytes: Option<usize>,
    require_token: Option<bool>,
}

impl RemoteSourceProvider for MockHuggingFaceProvider {
    fn resolve_gguf(&self, source: &Source) -> Result<RemoteArtifactDescriptor> {
        let (repo, revision) = remote_repo_revision(source)?;
        let root = self.revision_root(repo, &revision);
        if !root.exists() {
            return Err(MetamorphError::RemoteRevisionNotFound {
                input: source.display_name(),
            });
        }

        let config = read_mock_repo_config(&root)?;
        if config.require_token.unwrap_or(false) {
            let token = env::var(HF_TOKEN_ENV).unwrap_or_default();
            if token.trim().is_empty() {
                return Err(MetamorphError::RemoteCredentialsRequired {
                    input: source.display_name(),
                    credential_env: HF_TOKEN_ENV,
                });
            }
        }

        let mut gguf_files = list_relative_files(&root)?
            .into_iter()
            .filter(|path| path.to_ascii_lowercase().ends_with(".gguf"))
            .collect::<Vec<_>>();
        gguf_files.sort();

        let remote_path = match gguf_files.as_slice() {
            [single] => single.clone(),
            [] => {
                return Err(MetamorphError::RemoteLayoutUnsupported {
                    input: source.display_name(),
                    reason: "the mock repo does not expose a `.gguf` artifact".to_owned(),
                });
            }
            multiple => {
                return Err(MetamorphError::RemoteLayoutUnsupported {
                    input: source.display_name(),
                    reason: format!(
                        "the mock repo exposes multiple `.gguf` artifacts ({})",
                        multiple.join(", ")
                    ),
                });
            }
        };

        Ok(RemoteArtifactDescriptor {
            source: source.clone(),
            repo: repo.to_owned(),
            requested_revision: revision.clone(),
            resolved_revision: config
                .resolved_revision
                .unwrap_or_else(|| format!("mock-{revision}")),
            remote_path,
            source_format: Format::Gguf,
        })
    }

    fn download(&self, artifact: &RemoteArtifactDescriptor, destination: &Path) -> Result<()> {
        let root = self.revision_root(&artifact.repo, &artifact.requested_revision);
        let config = read_mock_repo_config(&root)?;
        let source_path = root.join(&artifact.remote_path);
        if !source_path.exists() {
            return Err(MetamorphError::RemoteLayoutUnsupported {
                input: artifact.source.display_name(),
                reason: format!(
                    "the configured remote artifact `{}` is missing from the mock provider",
                    artifact.remote_path
                ),
            });
        }

        if let Some(parent) = destination.parent() {
            fs::create_dir_all(parent)?;
        }

        let mut input = File::open(&source_path)?;
        let mut output = File::create(destination)?;
        if let Some(limit) = config.interrupt_after_bytes {
            let mut remaining = limit;
            let mut buffer = [0u8; 8192];
            while remaining > 0 {
                let chunk_len = buffer.len().min(remaining);
                let read = input.read(&mut buffer[..chunk_len])?;
                if read == 0 {
                    break;
                }
                output.write_all(&buffer[..read])?;
                remaining = remaining.saturating_sub(read);
            }
            output.flush()?;
            return Err(MetamorphError::RemoteTransferInterrupted {
                input: artifact.source.display_name(),
                cache_path: destination.to_path_buf(),
                reason: format!("mock provider interrupted after {limit} bytes"),
            });
        }

        std::io::copy(&mut input, &mut output)?;
        output.flush()?;
        Ok(())
    }
}

fn remote_repo_revision(source: &Source) -> Result<(&str, String)> {
    match source {
        Source::HuggingFace { repo, revision } => Ok((
            repo.as_str(),
            revision.clone().unwrap_or_else(|| "main".to_owned()),
        )),
        _ => Err(MetamorphError::RemoteFetchUnsupported {
            input: source.display_name(),
            reason: "only `hf://...` sources can be fetched remotely".to_owned(),
        }),
    }
}

fn read_mock_repo_config(root: &Path) -> Result<MockRepoConfig> {
    let path = root.join(MOCK_CONFIG_FILE);
    if !path.exists() {
        return Ok(MockRepoConfig::default());
    }

    let content = fs::read_to_string(path)?;
    let parsed = serde_json::from_str(&content).map_err(|error| {
        MetamorphError::RemoteLayoutUnsupported {
            input: "hf://mock".to_owned(),
            reason: format!("invalid mock provider config: {error}"),
        }
    })?;
    Ok(parsed)
}

fn list_relative_files(root: &Path) -> Result<Vec<String>> {
    let mut files = Vec::new();
    collect_relative_files(root, root, &mut files)?;
    Ok(files)
}

fn collect_relative_files(root: &Path, current: &Path, output: &mut Vec<String>) -> Result<()> {
    let mut entries = fs::read_dir(current)?
        .filter_map(std::result::Result::ok)
        .collect::<Vec<_>>();
    entries.sort_by_key(|entry| entry.path());

    for entry in entries {
        let path = entry.path();
        let relative = path
            .strip_prefix(root)
            .map(|candidate| candidate.to_path_buf())
            .unwrap_or(path.clone());
        if path.is_dir() {
            collect_relative_files(root, &path, output)?;
            continue;
        }
        if relative.as_os_str() == MOCK_CONFIG_FILE {
            continue;
        }
        output.push(relative.to_string_lossy().replace('\\', "/"));
    }

    Ok(())
}
