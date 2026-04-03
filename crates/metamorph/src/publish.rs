use std::fmt;
use std::fs;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use crate::error::{HF_TOKEN_ENV, MetamorphError, Result};
use crate::format::Format;
use crate::remote_publish::{
    PreparedPublishArtifact, RemotePublishArtifactReport, RemotePublishArtifactStatus,
    RemotePublishStatus, publish_provider_from_env,
};
use crate::source::Target;
use crate::validate::validate;

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

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PublishStatus {
    Preview,
    GuardedRefusal,
    Complete,
    Partial,
    Failed,
}

impl fmt::Display for PublishStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let rendered = match self {
            Self::Preview => "preview",
            Self::GuardedRefusal => "guarded-refusal",
            Self::Complete => "complete",
            Self::Partial => "partial",
            Self::Failed => "failed",
        };
        f.write_str(rendered)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PublishArtifactStatus {
    Pending,
    Transferred,
    Published,
    AlreadyPresent,
    Failed,
}

impl fmt::Display for PublishArtifactStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let rendered = match self {
            Self::Pending => "pending",
            Self::Transferred => "transferred",
            Self::Published => "published",
            Self::AlreadyPresent => "already-present",
            Self::Failed => "failed",
        };
        f.write_str(rendered)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PublishArtifactReport {
    pub local_path: PathBuf,
    pub remote_path: String,
    pub status: PublishArtifactStatus,
    pub detail: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PublishReport {
    pub plan: PublishPlan,
    pub status: PublishStatus,
    pub executed: bool,
    pub artifacts: Vec<PublishArtifactReport>,
    pub notes: Vec<String>,
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
            format!(
                "execute remote writes only after an explicit operator opt-in against the existing repo on `{}`",
                "main"
            ),
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
    let prepared_artifacts = plan
        .artifacts
        .iter()
        .map(|artifact| PreparedPublishArtifact::from_path(artifact))
        .collect::<Result<Vec<_>>>()?;

    if !request.execute {
        return Ok(PublishReport {
            plan,
            status: PublishStatus::Preview,
            executed: false,
            artifacts: planned_artifact_reports(&prepared_artifacts),
            notes: vec![format!(
                "preview: dry run only; rerun with --execute after reviewing licensing, credentials, and destination state for hf://{repo}"
            )],
        });
    }

    if std::env::var_os(HF_TOKEN_ENV).is_none() {
        return Ok(PublishReport {
            plan,
            status: PublishStatus::GuardedRefusal,
            executed: false,
            artifacts: planned_artifact_reports(&prepared_artifacts),
            notes: vec![
                format!(
                    "guarded refusal: publish execution for hf://{repo} requires `{HF_TOKEN_ENV}` before any remote write can begin."
                ),
                "recover by setting `HF_TOKEN` explicitly, confirming the destination repo already exists, and rerunning `upload --execute`.".to_owned(),
            ],
        });
    }

    let provider = publish_provider_from_env();
    let remote_report = provider.publish(&repo, &prepared_artifacts)?;
    Ok(PublishReport {
        plan,
        status: map_publish_status(remote_report.status),
        executed: true,
        artifacts: remote_report
            .artifacts
            .into_iter()
            .map(map_artifact_report)
            .collect(),
        notes: remote_report.notes,
    })
}

fn planned_artifact_reports(artifacts: &[PreparedPublishArtifact]) -> Vec<PublishArtifactReport> {
    artifacts
        .iter()
        .map(|artifact| PublishArtifactReport {
            local_path: artifact.local_path.clone(),
            remote_path: artifact.remote_path.clone(),
            status: PublishArtifactStatus::Pending,
            detail: None,
        })
        .collect()
}

fn map_publish_status(status: RemotePublishStatus) -> PublishStatus {
    match status {
        RemotePublishStatus::GuardedRefusal => PublishStatus::GuardedRefusal,
        RemotePublishStatus::Complete => PublishStatus::Complete,
        RemotePublishStatus::Partial => PublishStatus::Partial,
        RemotePublishStatus::Failed => PublishStatus::Failed,
    }
}

fn map_artifact_report(report: RemotePublishArtifactReport) -> PublishArtifactReport {
    PublishArtifactReport {
        local_path: report.local_path,
        remote_path: report.remote_path,
        status: match report.status {
            RemotePublishArtifactStatus::Pending => PublishArtifactStatus::Pending,
            RemotePublishArtifactStatus::Transferred => PublishArtifactStatus::Transferred,
            RemotePublishArtifactStatus::Published => PublishArtifactStatus::Published,
            RemotePublishArtifactStatus::AlreadyPresent => PublishArtifactStatus::AlreadyPresent,
            RemotePublishArtifactStatus::Failed => PublishArtifactStatus::Failed,
        },
        detail: report.detail,
    }
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
