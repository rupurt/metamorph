use std::collections::HashMap;
use std::env;
use std::fmt;
use std::fs::{self, File};
use std::io::{Read, Seek, SeekFrom};
use std::path::{Path, PathBuf};

use base64::Engine as _;
use base64::engine::general_purpose::STANDARD as BASE64;
use serde::{Deserialize, Serialize};
use serde_json::json;
use sha1::{Digest as Sha1Digest, Sha1};
use sha2::Sha256;
use ureq::{Agent, SendBody};

use crate::error::{HF_TOKEN_ENV, MetamorphError, Result};
use crate::remote::MOCK_ROOT_ENV;

const DEFAULT_ENDPOINT: &str = "https://huggingface.co";
const DEFAULT_REVISION: &str = "main";
const HF_ENDPOINT_ENV: &str = "HF_ENDPOINT";
const SAMPLE_BYTES: usize = 512;
const LFS_CONTENT_TYPE: &str = "application/vnd.git-lfs+json";
const NDJSON_CONTENT_TYPE: &str = "application/x-ndjson";
const USER_AGENT: &str = concat!("metamorph/", env!("CARGO_PKG_VERSION"));
const MOCK_PUBLISH_CONFIG_FILE: &str = ".metamorph-hf-publish.json";
const MOCK_PUBLISHED_ROOT: &str = "_published";

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct PreparedPublishArtifact {
    pub(crate) local_path: PathBuf,
    pub(crate) remote_path: String,
    size: u64,
    sample_base64: String,
    sha256_hex: String,
    git_blob_sha1_hex: String,
}

impl PreparedPublishArtifact {
    pub(crate) fn from_path(path: &Path) -> Result<Self> {
        let remote_path = path
            .file_name()
            .and_then(|value| value.to_str())
            .unwrap_or("artifact")
            .to_owned();

        let mut file = File::open(path)?;
        let mut sample = vec![0u8; SAMPLE_BYTES];
        let sample_len = file.read(&mut sample)?;
        sample.truncate(sample_len);
        file.seek(SeekFrom::Start(0))?;

        let mut sha256 = Sha256::new();
        let mut git_blob = Sha1::new();
        git_blob.update(format!("blob {}\0", file.metadata()?.len()).as_bytes());

        let mut buffer = [0u8; 16 * 1024];
        loop {
            let read = file.read(&mut buffer)?;
            if read == 0 {
                break;
            }
            sha256.update(&buffer[..read]);
            git_blob.update(&buffer[..read]);
        }

        Ok(Self {
            local_path: path.to_path_buf(),
            remote_path,
            size: file.metadata()?.len(),
            sample_base64: BASE64.encode(sample),
            sha256_hex: format!("{:x}", sha256.finalize()),
            git_blob_sha1_hex: format!("{:x}", git_blob.finalize()),
        })
    }

    fn matches_remote_oid(&self, upload_mode: UploadMode, remote_oid: Option<&str>) -> bool {
        match (upload_mode, remote_oid) {
            (_, None) => false,
            (UploadMode::Regular, Some(remote_oid)) => self.git_blob_sha1_hex == remote_oid,
            (UploadMode::Lfs, Some(remote_oid)) => self.sha256_hex == remote_oid,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) enum RemotePublishStatus {
    GuardedRefusal,
    Complete,
    Partial,
    Failed,
}

impl fmt::Display for RemotePublishStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let rendered = match self {
            Self::GuardedRefusal => "guarded-refusal",
            Self::Complete => "complete",
            Self::Partial => "partial",
            Self::Failed => "failed",
        };
        f.write_str(rendered)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) enum RemotePublishArtifactStatus {
    Pending,
    Transferred,
    Published,
    AlreadyPresent,
    Failed,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) struct RemotePublishArtifactReport {
    pub(crate) local_path: PathBuf,
    pub(crate) remote_path: String,
    pub(crate) status: RemotePublishArtifactStatus,
    pub(crate) detail: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) struct RemotePublishReport {
    pub(crate) status: RemotePublishStatus,
    pub(crate) artifacts: Vec<RemotePublishArtifactReport>,
    pub(crate) notes: Vec<String>,
}

impl RemotePublishReport {
    fn new_pending(artifacts: &[PreparedPublishArtifact]) -> Self {
        Self {
            status: RemotePublishStatus::Failed,
            artifacts: artifacts
                .iter()
                .map(|artifact| RemotePublishArtifactReport {
                    local_path: artifact.local_path.clone(),
                    remote_path: artifact.remote_path.clone(),
                    status: RemotePublishArtifactStatus::Pending,
                    detail: None,
                })
                .collect(),
            notes: Vec::new(),
        }
    }
}

pub(crate) trait RemotePublishProvider {
    fn publish(
        &self,
        repo: &str,
        artifacts: &[PreparedPublishArtifact],
    ) -> Result<RemotePublishReport>;
}

pub(crate) fn publish_provider_from_env() -> Box<dyn RemotePublishProvider> {
    match env::var(MOCK_ROOT_ENV) {
        Ok(root) => Box::new(MockHuggingFacePublishProvider::new(PathBuf::from(root))),
        Err(_) => Box::new(LiveHuggingFacePublishProvider::from_env()),
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum UploadMode {
    Regular,
    Lfs,
}

impl UploadMode {
    fn parse(value: &str) -> Option<Self> {
        match value {
            "regular" => Some(Self::Regular),
            "lfs" => Some(Self::Lfs),
            _ => None,
        }
    }
}

#[derive(Debug, Clone)]
struct PublishTransfer {
    artifact: PreparedPublishArtifact,
    upload_mode: UploadMode,
}

#[derive(Debug, Default, Clone)]
struct StatusSummary {
    published: usize,
    transferred: usize,
    already_present: usize,
}

impl StatusSummary {
    fn from_artifacts(artifacts: &[RemotePublishArtifactReport]) -> Self {
        let mut summary = Self::default();
        for artifact in artifacts {
            match artifact.status {
                RemotePublishArtifactStatus::Published => summary.published += 1,
                RemotePublishArtifactStatus::Transferred => summary.transferred += 1,
                RemotePublishArtifactStatus::AlreadyPresent => summary.already_present += 1,
                RemotePublishArtifactStatus::Failed => {}
                RemotePublishArtifactStatus::Pending => {}
            }
        }
        summary
    }
}

#[derive(Debug, Clone)]
struct LiveHuggingFacePublishProvider {
    agent: Agent,
    endpoint: String,
    token: Option<String>,
}

impl LiveHuggingFacePublishProvider {
    fn from_env() -> Self {
        let endpoint = env::var(HF_ENDPOINT_ENV).unwrap_or_else(|_| DEFAULT_ENDPOINT.to_owned());
        let token = env::var(HF_TOKEN_ENV)
            .ok()
            .map(|value| value.trim().to_owned())
            .filter(|value| !value.is_empty());
        let agent: Agent = Agent::config_builder()
            .http_status_as_error(false)
            .user_agent(USER_AGENT)
            .build()
            .into();

        Self {
            agent,
            endpoint,
            token,
        }
    }

    fn auth_header(&self) -> Option<String> {
        self.token.as_ref().map(|token| format!("Bearer {token}"))
    }

    fn post_json<T: serde::Serialize>(
        &self,
        url: &str,
        body: &T,
    ) -> Result<ureq::http::Response<ureq::Body>> {
        let mut request = self.agent.post(url);
        if let Some(header) = self.auth_header() {
            request = request.header("Authorization", &header);
        }
        request.send_json(body).map_err(map_ureq_transport_error)
    }

    fn post_bytes(
        &self,
        url: &str,
        body: Vec<u8>,
        content_type: &str,
    ) -> Result<ureq::http::Response<ureq::Body>> {
        let mut request = self.agent.post(url);
        if let Some(header) = self.auth_header() {
            request = request.header("Authorization", &header);
        }
        request = request.header("Content-Type", content_type);
        request.send(body).map_err(map_ureq_transport_error)
    }

    fn put_file(&self, url: &str, path: &Path) -> Result<ureq::http::Response<ureq::Body>> {
        let file = File::open(path)?;
        self.agent
            .put(url)
            .send(file)
            .map_err(map_ureq_transport_error)
    }

    fn put_file_part(
        &self,
        url: &str,
        path: &Path,
        offset: u64,
        len: u64,
    ) -> Result<ureq::http::Response<ureq::Body>> {
        let mut file = File::open(path)?;
        file.seek(SeekFrom::Start(offset))?;
        let mut reader = file.take(len);
        self.agent
            .put(url)
            .header("Content-Length", &len.to_string())
            .send(SendBody::from_reader(&mut reader))
            .map_err(map_ureq_transport_error)
    }

    fn preupload(
        &self,
        repo: &str,
        artifacts: &[PreparedPublishArtifact],
    ) -> Result<PreuploadResponse> {
        let payload = json!({
            "files": artifacts
                .iter()
                .map(|artifact| json!({
                    "path": artifact.remote_path,
                    "sample": artifact.sample_base64,
                    "size": artifact.size,
                }))
                .collect::<Vec<_>>(),
        });
        let url = format!(
            "{}/api/models/{repo}/preupload/{DEFAULT_REVISION}",
            self.endpoint
        );
        let mut response = self.post_json(&url, &payload)?;
        read_json_response(&mut response)
    }

    fn lfs_batch(&self, repo: &str, artifacts: &[&PublishTransfer]) -> Result<LfsBatchResponse> {
        let payload = json!({
            "operation": "upload",
            "transfers": ["basic", "multipart"],
            "objects": artifacts
                .iter()
                .map(|artifact| json!({
                    "oid": artifact.artifact.sha256_hex,
                    "size": artifact.artifact.size,
                }))
                .collect::<Vec<_>>(),
            "hash_algo": "sha256",
            "ref": { "name": DEFAULT_REVISION },
        });
        let url = format!("{}/{}.git/info/lfs/objects/batch", self.endpoint, repo);
        let mut response =
            self.post_bytes(&url, serde_json::to_vec(&payload)?, LFS_CONTENT_TYPE)?;
        read_json_response(&mut response)
    }

    fn upload_lfs(
        &self,
        action: &LfsBatchObject,
        artifact: &PreparedPublishArtifact,
    ) -> Result<()> {
        let Some(actions) = &action.actions else {
            return Ok(());
        };

        let upload =
            actions
                .upload
                .as_ref()
                .ok_or_else(|| MetamorphError::RemoteLayoutUnsupported {
                    input: "hf://publish".to_owned(),
                    reason: "missing LFS upload action in remote response".to_owned(),
                })?;
        let upload_url = self.fix_endpoint(upload.href.as_str());

        if let Some(chunk_size) = upload
            .header
            .as_ref()
            .and_then(|header| header.get("chunk_size"))
            .and_then(|value| value.as_str())
        {
            let chunk_size =
                chunk_size
                    .parse::<u64>()
                    .map_err(|_| MetamorphError::RemoteLayoutUnsupported {
                        input: "hf://publish".to_owned(),
                        reason: "invalid LFS multipart chunk size in remote response".to_owned(),
                    })?;
            self.upload_lfs_multipart(&upload_url, upload.header.as_ref(), artifact, chunk_size)?;
        } else {
            let response = self.put_file(&upload_url, &artifact.local_path)?;
            if !response.status().is_success() {
                return Err(MetamorphError::RemoteLayoutUnsupported {
                    input: "hf://publish".to_owned(),
                    reason: "single-part LFS upload failed".to_owned(),
                });
            }
        }

        if let Some(verify) = &actions.verify {
            let payload = json!({
                "oid": artifact.sha256_hex,
                "size": artifact.size,
            });
            let verify_url = self.fix_endpoint(verify.href.as_str());
            let response =
                self.post_bytes(&verify_url, serde_json::to_vec(&payload)?, LFS_CONTENT_TYPE)?;
            if !response.status().is_success() {
                return Err(MetamorphError::RemoteLayoutUnsupported {
                    input: "hf://publish".to_owned(),
                    reason: "LFS verify request failed".to_owned(),
                });
            }
        }

        Ok(())
    }

    fn upload_lfs_multipart(
        &self,
        completion_url: &str,
        header: Option<&serde_json::Value>,
        artifact: &PreparedPublishArtifact,
        chunk_size: u64,
    ) -> Result<()> {
        let header = header.and_then(|value| value.as_object()).ok_or_else(|| {
            MetamorphError::RemoteLayoutUnsupported {
                input: "hf://publish".to_owned(),
                reason: "missing LFS multipart headers in remote response".to_owned(),
            }
        })?;
        let mut part_urls = header
            .iter()
            .filter_map(|(key, value)| key.parse::<usize>().ok().zip(value.as_str()))
            .collect::<Vec<_>>();
        part_urls.sort_by_key(|(part, _)| *part);

        let expected_parts =
            usize::try_from(artifact.size.div_ceil(chunk_size)).unwrap_or(part_urls.len());
        if part_urls.len() != expected_parts {
            return Err(MetamorphError::RemoteLayoutUnsupported {
                input: "hf://publish".to_owned(),
                reason: "invalid LFS multipart response from remote provider".to_owned(),
            });
        }

        let mut parts = Vec::with_capacity(part_urls.len());
        for (index, part_url) in part_urls {
            let part_number = u64::try_from(index).unwrap_or(1);
            let offset = (part_number.saturating_sub(1)).saturating_mul(chunk_size);
            let remaining = artifact.size.saturating_sub(offset);
            let len = remaining.min(chunk_size);

            let response = self.put_file_part(
                &self.fix_endpoint(part_url),
                &artifact.local_path,
                offset,
                len,
            )?;
            if !response.status().is_success() {
                return Err(MetamorphError::RemoteLayoutUnsupported {
                    input: "hf://publish".to_owned(),
                    reason: "multipart LFS upload failed".to_owned(),
                });
            }
            let etag = response
                .headers()
                .get("etag")
                .and_then(|value| value.to_str().ok())
                .filter(|value| !value.is_empty())
                .ok_or_else(|| MetamorphError::RemoteLayoutUnsupported {
                    input: "hf://publish".to_owned(),
                    reason: "missing multipart upload etag".to_owned(),
                })?;
            parts.push(json!({
                "partNumber": index,
                "etag": etag,
            }));
        }

        let payload = json!({
            "oid": artifact.sha256_hex,
            "parts": parts,
        });
        let response = self.post_bytes(
            completion_url,
            serde_json::to_vec(&payload)?,
            LFS_CONTENT_TYPE,
        )?;
        if !response.status().is_success() {
            return Err(MetamorphError::RemoteLayoutUnsupported {
                input: "hf://publish".to_owned(),
                reason: "multipart LFS completion request failed".to_owned(),
            });
        }
        Ok(())
    }

    fn commit(
        &self,
        repo: &str,
        transfers: &[PublishTransfer],
    ) -> Result<ureq::http::Response<ureq::Body>> {
        let mut lines = Vec::new();
        lines.push(serde_json::to_vec(&json!({
            "key": "header",
            "value": {
                "summary": "Publish validated bundle via Metamorph",
                "description": format!("Publish validated hf-safetensors bundle to hf://{repo}"),
            }
        }))?);

        for transfer in transfers {
            match transfer.upload_mode {
                UploadMode::Regular => {
                    let content = fs::read(&transfer.artifact.local_path)?;
                    lines.push(serde_json::to_vec(&json!({
                        "key": "file",
                        "value": {
                            "content": BASE64.encode(content),
                            "path": transfer.artifact.remote_path,
                            "encoding": "base64",
                        }
                    }))?);
                }
                UploadMode::Lfs => {
                    lines.push(serde_json::to_vec(&json!({
                        "key": "lfsFile",
                        "value": {
                            "path": transfer.artifact.remote_path,
                            "algo": "sha256",
                            "oid": transfer.artifact.sha256_hex,
                            "size": transfer.artifact.size,
                        }
                    }))?);
                }
            }
        }

        let mut payload = Vec::new();
        for line in lines {
            payload.extend_from_slice(&line);
            payload.push(b'\n');
        }

        let url = format!(
            "{}/api/models/{repo}/commit/{DEFAULT_REVISION}",
            self.endpoint
        );
        self.post_bytes(&url, payload, NDJSON_CONTENT_TYPE)
    }

    fn fix_endpoint(&self, value: &str) -> String {
        if self.endpoint == DEFAULT_ENDPOINT {
            return value.to_owned();
        }
        value.replacen(DEFAULT_ENDPOINT, &self.endpoint, 1)
    }
}

impl RemotePublishProvider for LiveHuggingFacePublishProvider {
    fn publish(
        &self,
        repo: &str,
        artifacts: &[PreparedPublishArtifact],
    ) -> Result<RemotePublishReport> {
        let mut report = RemotePublishReport::new_pending(artifacts);
        report.notes.push(format!(
            "executing explicit upload to hf://{repo} on revision `{DEFAULT_REVISION}`"
        ));

        let preupload = match self.preupload(repo, artifacts) {
            Ok(response) => response,
            Err(error) => {
                report.status = RemotePublishStatus::Failed;
                report.notes.push(format!(
                    "publish failure: could not negotiate the remote upload plan for hf://{repo}: {error}"
                ));
                report.notes.push(
                    "recover by confirming the destination repo exists, the token has write access, and rerunning `upload --execute`.".to_owned(),
                );
                return Ok(report);
            }
        };

        let mut transfers = Vec::new();
        for artifact in artifacts {
            let Some(remote) = preupload
                .files
                .iter()
                .find(|candidate| candidate.path == artifact.remote_path)
            else {
                report.status = RemotePublishStatus::Failed;
                report.notes.push(format!(
                    "publish failure: the remote provider did not return upload metadata for `{}`.",
                    artifact.remote_path
                ));
                return Ok(report);
            };

            let Some(upload_mode) = UploadMode::parse(&remote.upload_mode) else {
                report.status = RemotePublishStatus::Failed;
                report.notes.push(format!(
                    "publish failure: the remote provider returned an unsupported upload mode `{}` for `{}`.",
                    remote.upload_mode, artifact.remote_path
                ));
                return Ok(report);
            };

            let artifact_report = report
                .artifacts
                .iter_mut()
                .find(|candidate| candidate.remote_path == artifact.remote_path)
                .expect("artifact reports are initialized from the plan");

            if remote.should_ignore {
                artifact_report.status = RemotePublishArtifactStatus::Failed;
                artifact_report.detail =
                    Some("the remote destination policy refused this artifact".to_owned());
                continue;
            }

            if artifact.matches_remote_oid(upload_mode, remote.oid.as_deref()) {
                artifact_report.status = RemotePublishArtifactStatus::AlreadyPresent;
                artifact_report.detail =
                    Some("remote destination already matches this artifact".to_owned());
                continue;
            }

            transfers.push(PublishTransfer {
                artifact: artifact.clone(),
                upload_mode,
            });
        }

        let ignored_failures = report
            .artifacts
            .iter()
            .any(|artifact| artifact.status == RemotePublishArtifactStatus::Failed);
        if ignored_failures {
            report.status = RemotePublishStatus::Failed;
            report.notes.push(
                "publish failure: one or more artifacts were refused by the remote destination policy.".to_owned(),
            );
            report.notes.push(
                "recover by renaming or removing the refused artifact, then rerunning `upload --execute`.".to_owned(),
            );
            return Ok(report);
        }

        let lfs_transfers = transfers
            .iter()
            .filter(|transfer| transfer.upload_mode == UploadMode::Lfs)
            .collect::<Vec<_>>();
        if !lfs_transfers.is_empty() {
            let batch = match self.lfs_batch(repo, &lfs_transfers) {
                Ok(batch) => batch,
                Err(error) => {
                    report.status = RemotePublishStatus::Failed;
                    report.notes.push(format!(
                        "publish failure: the remote provider rejected LFS staging for hf://{repo}: {error}"
                    ));
                    report.notes.push(
                        "recover by confirming the destination repo exists and the token has write access, then rerunning `upload --execute`.".to_owned(),
                    );
                    return Ok(report);
                }
            };
            let actions = batch
                .objects
                .into_iter()
                .map(|action| (action.oid.clone(), action))
                .collect::<HashMap<_, _>>();

            for transfer in &lfs_transfers {
                let Some(action) = actions.get(&transfer.artifact.sha256_hex) else {
                    report.status = RemotePublishStatus::Failed;
                    report.notes.push(format!(
                        "publish failure: the remote provider omitted LFS instructions for `{}`.",
                        transfer.artifact.remote_path
                    ));
                    return Ok(report);
                };

                if let Some(error) = &action.error {
                    let artifact = report
                        .artifacts
                        .iter_mut()
                        .find(|candidate| candidate.remote_path == transfer.artifact.remote_path)
                        .expect("artifact report exists");
                    artifact.status = RemotePublishArtifactStatus::Failed;
                    artifact.detail = Some(format!(
                        "remote LFS staging refused this artifact: {} ({})",
                        error.message, error.code
                    ));
                    report.status = RemotePublishStatus::Failed;
                    report.notes.push(format!(
                        "publish failure: remote LFS staging rejected `{}` with code {}.",
                        transfer.artifact.remote_path, error.code
                    ));
                    report.notes.push(
                        "recover by confirming write permissions and rerunning `upload --execute`."
                            .to_owned(),
                    );
                    return Ok(report);
                }

                match self.upload_lfs(action, &transfer.artifact) {
                    Ok(()) => {
                        let artifact = report
                            .artifacts
                            .iter_mut()
                            .find(|candidate| {
                                candidate.remote_path == transfer.artifact.remote_path
                            })
                            .expect("artifact report exists");
                        artifact.status = RemotePublishArtifactStatus::Transferred;
                        artifact.detail =
                            Some("remote LFS storage accepted this artifact".to_owned());
                    }
                    Err(error) => {
                        let artifact = report
                            .artifacts
                            .iter_mut()
                            .find(|candidate| {
                                candidate.remote_path == transfer.artifact.remote_path
                            })
                            .expect("artifact report exists");
                        artifact.status = RemotePublishArtifactStatus::Failed;
                        artifact.detail = Some(format!(
                            "remote transfer interrupted before final commit: {error}"
                        ));
                        let summary = StatusSummary::from_artifacts(&report.artifacts);
                        report.status = if summary.transferred + summary.already_present > 0 {
                            RemotePublishStatus::Partial
                        } else {
                            RemotePublishStatus::Failed
                        };
                        report.notes.push(format!(
                            "partial publish: remote transfer stopped before the repo commit was finalized for hf://{repo}."
                        ));
                        report.notes.push(
                            "recover by rerunning `upload --execute`; transferred LFS blobs can be reused on retry.".to_owned(),
                        );
                        return Ok(report);
                    }
                }
            }
        }

        if transfers.is_empty() {
            report.status = RemotePublishStatus::Complete;
            report.notes.push(format!(
                "complete publish: hf://{repo} already matches the validated local bundle."
            ));
            return Ok(report);
        }

        match self.commit(repo, &transfers) {
            Ok(mut response) if response.status().is_success() => {
                let commit: CommitResponse = read_json_response(&mut response)?;
                for transfer in &transfers {
                    let artifact = report
                        .artifacts
                        .iter_mut()
                        .find(|candidate| candidate.remote_path == transfer.artifact.remote_path)
                        .expect("artifact report exists");
                    artifact.status = RemotePublishArtifactStatus::Published;
                    artifact.detail = Some(format!(
                        "published on `{DEFAULT_REVISION}` via {}",
                        match transfer.upload_mode {
                            UploadMode::Regular => "regular commit payload",
                            UploadMode::Lfs => "LFS-backed commit",
                        }
                    ));
                }
                report.status = RemotePublishStatus::Complete;
                report.notes.push(format!(
                    "complete publish: created remote commit `{}` for hf://{repo}.",
                    commit.commit_oid
                ));
                report
                    .notes
                    .push(format!("commit url: {}", commit.commit_url));
                Ok(report)
            }
            Ok(mut response) => {
                let status = response.status().as_u16();
                let body = response
                    .body_mut()
                    .read_to_string()
                    .unwrap_or_else(|_| String::new());
                let summary = StatusSummary::from_artifacts(&report.artifacts);
                report.status = if status == 401 || status == 403 {
                    RemotePublishStatus::GuardedRefusal
                } else if summary.transferred + summary.already_present > 0 {
                    RemotePublishStatus::Partial
                } else {
                    RemotePublishStatus::Failed
                };
                report.notes.push(commit_failure_note(repo, status, &body));
                report.notes.push(commit_recovery_note(status).to_owned());
                Ok(report)
            }
            Err(error) => {
                let summary = StatusSummary::from_artifacts(&report.artifacts);
                report.status = if summary.transferred + summary.already_present > 0 {
                    RemotePublishStatus::Partial
                } else {
                    RemotePublishStatus::Failed
                };
                report.notes.push(format!(
                    "publish failure: the remote commit request did not complete cleanly for hf://{repo}: {error}"
                ));
                report.notes.push(
                    "recover by rerunning `upload --execute`; transferred LFS blobs can be reused on retry."
                        .to_owned(),
                );
                Ok(report)
            }
        }
    }
}

#[derive(Debug, Clone)]
struct MockHuggingFacePublishProvider {
    root: PathBuf,
}

impl MockHuggingFacePublishProvider {
    fn new(root: PathBuf) -> Self {
        Self { root }
    }

    fn repo_root(&self, repo: &str) -> PathBuf {
        self.root.join(repo)
    }

    fn published_root(&self, repo: &str) -> PathBuf {
        self.repo_root(repo).join(MOCK_PUBLISHED_ROOT)
    }
}

#[derive(Debug, Default, Deserialize)]
struct MockPublishConfig {
    require_token: Option<bool>,
    deny_write: Option<bool>,
    interrupt_after_uploads: Option<usize>,
}

impl RemotePublishProvider for MockHuggingFacePublishProvider {
    fn publish(
        &self,
        repo: &str,
        artifacts: &[PreparedPublishArtifact],
    ) -> Result<RemotePublishReport> {
        let mut report = RemotePublishReport::new_pending(artifacts);
        let repo_root = self.repo_root(repo);
        if !repo_root.exists() {
            report.status = RemotePublishStatus::Failed;
            report.notes.push(format!(
                "publish failure: hf://{repo} does not exist in the mock provider."
            ));
            report.notes.push(
                "recover by creating the destination repo first, then rerunning `upload --execute`."
                    .to_owned(),
            );
            return Ok(report);
        }

        let config = read_mock_publish_config(&repo_root)?;
        if config.require_token.unwrap_or(false)
            && env::var(HF_TOKEN_ENV).unwrap_or_default().trim().is_empty()
        {
            report.status = RemotePublishStatus::GuardedRefusal;
            report.notes.push(format!(
                "guarded refusal: publish execution for hf://{repo} requires `{HF_TOKEN_ENV}` in this mock provider."
            ));
            report.notes.push(
                "recover by setting `HF_TOKEN` explicitly before rerunning `upload --execute`."
                    .to_owned(),
            );
            return Ok(report);
        }
        if config.deny_write.unwrap_or(false) {
            report.status = RemotePublishStatus::GuardedRefusal;
            report.notes.push(format!(
                "guarded refusal: the mock provider denied write access to hf://{repo}."
            ));
            report.notes.push(
                "recover by using a token with write permission or choosing a repo you control."
                    .to_owned(),
            );
            return Ok(report);
        }

        let published_root = self.published_root(repo);
        fs::create_dir_all(&published_root)?;
        let mut published_count = 0usize;
        for artifact in artifacts {
            let report_artifact = report
                .artifacts
                .iter_mut()
                .find(|candidate| candidate.remote_path == artifact.remote_path)
                .expect("artifact report exists");

            let destination = published_root.join(&artifact.remote_path);
            if destination.exists()
                && PreparedPublishArtifact::from_path(&destination)?
                    .matches_remote_oid(UploadMode::Lfs, Some(&artifact.sha256_hex))
            {
                report_artifact.status = RemotePublishArtifactStatus::AlreadyPresent;
                report_artifact.detail =
                    Some("mock provider already contains this artifact".to_owned());
                continue;
            }

            if let Some(parent) = destination.parent() {
                fs::create_dir_all(parent)?;
            }
            fs::copy(&artifact.local_path, &destination)?;
            published_count += 1;
            report_artifact.status = RemotePublishArtifactStatus::Published;
            report_artifact.detail = Some("mock provider published this artifact".to_owned());

            if let Some(limit) = config.interrupt_after_uploads
                && published_count >= limit
            {
                report.status = RemotePublishStatus::Partial;
                report.notes.push(format!(
                    "partial publish: the mock provider interrupted execution after {limit} artifact(s)."
                ));
                report.notes.push(
                    "recover by rerunning `upload --execute`; already published artifacts will be reported as `already-present`.".to_owned(),
                );
                for pending in report
                    .artifacts
                    .iter_mut()
                    .filter(|candidate| candidate.status == RemotePublishArtifactStatus::Pending)
                {
                    pending.detail = Some("still pending after partial mock publish".to_owned());
                }
                return Ok(report);
            }
        }

        report.status = RemotePublishStatus::Complete;
        report.notes.push(format!(
            "complete publish: mock provider accepted the validated bundle for hf://{repo}."
        ));
        Ok(report)
    }
}

#[derive(Debug, Deserialize)]
struct PreuploadResponse {
    files: Vec<PreuploadFile>,
}

#[derive(Debug, Deserialize)]
struct PreuploadFile {
    path: String,
    #[serde(rename = "uploadMode")]
    upload_mode: String,
    #[serde(rename = "shouldIgnore")]
    should_ignore: bool,
    oid: Option<String>,
}

#[derive(Debug, Deserialize)]
struct LfsBatchResponse {
    objects: Vec<LfsBatchObject>,
}

#[derive(Debug, Deserialize)]
struct LfsBatchObject {
    oid: String,
    #[serde(default)]
    actions: Option<LfsActions>,
    #[serde(default)]
    error: Option<LfsError>,
}

#[derive(Debug, Deserialize)]
struct LfsActions {
    upload: Option<LfsAction>,
    verify: Option<LfsAction>,
}

#[derive(Debug, Deserialize)]
struct LfsAction {
    href: String,
    #[serde(default)]
    header: Option<serde_json::Value>,
}

#[derive(Debug, Deserialize)]
struct LfsError {
    code: u16,
    message: String,
}

#[derive(Debug, Deserialize)]
struct CommitResponse {
    #[serde(rename = "commitUrl")]
    commit_url: String,
    #[serde(rename = "commitOid")]
    commit_oid: String,
}

fn map_ureq_transport_error(error: ureq::Error) -> MetamorphError {
    MetamorphError::RemoteLayoutUnsupported {
        input: "hf://publish".to_owned(),
        reason: format!("remote publish transport error: {error}"),
    }
}

fn read_json_response<T: serde::de::DeserializeOwned>(
    response: &mut ureq::http::Response<ureq::Body>,
) -> Result<T> {
    response
        .body_mut()
        .read_json()
        .map_err(|error| MetamorphError::RemoteLayoutUnsupported {
            input: "hf://publish".to_owned(),
            reason: format!("unexpected remote publish response payload: {error}"),
        })
}

fn commit_failure_note(repo: &str, status: u16, body: &str) -> String {
    match status {
        401 => format!(
            "guarded refusal: the destination rejected the configured token for hf://{repo}."
        ),
        403 => {
            format!("guarded refusal: write access to hf://{repo} was denied by the destination.")
        }
        404 => format!("publish failure: hf://{repo} was not found during the final commit step."),
        _ => format!(
            "publish failure: the final remote commit for hf://{repo} returned HTTP {status}{}",
            if body.trim().is_empty() {
                ".".to_owned()
            } else {
                format!(" with body `{}`.", body.trim())
            }
        ),
    }
}

fn commit_recovery_note(status: u16) -> &'static str {
    match status {
        401 => {
            "recover by setting a valid `HF_TOKEN` with write access, then rerunning `upload --execute`."
        }
        403 => "recover by using credentials with write permission or choosing a repo you control.",
        404 => "recover by creating the destination repo first, then rerunning `upload --execute`.",
        _ => {
            "recover by reviewing the destination state and rerunning `upload --execute` explicitly."
        }
    }
}

fn read_mock_publish_config(root: &Path) -> Result<MockPublishConfig> {
    let path = root.join(MOCK_PUBLISH_CONFIG_FILE);
    if !path.exists() {
        return Ok(MockPublishConfig::default());
    }
    let content = fs::read_to_string(path)?;
    serde_json::from_str(&content).map_err(|error| MetamorphError::RemoteLayoutUnsupported {
        input: "hf://mock-publish".to_owned(),
        reason: format!("invalid mock publish config: {error}"),
    })
}
