use std::fs;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use crate::error::{HF_TOKEN_ENV, MetamorphError, Result};
use crate::format::Format;
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

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PublishReport {
    pub plan: PublishPlan,
    pub executed: bool,
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
