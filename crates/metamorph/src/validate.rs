use std::fs;
use std::path::{Path, PathBuf};

use candle_core::Device;
use serde::{Deserialize, Serialize};

use crate::error::{MetamorphError, Result};
use crate::format::Format;
use crate::source::{Source, detect_path_format, inspect};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ValidationReport {
    pub path: PathBuf,
    pub format: Format,
    pub reusable: bool,
    pub checked_paths: Vec<PathBuf>,
    pub notes: Vec<String>,
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

pub(crate) fn validate_for_format(path: &Path, format: Format) -> Result<Vec<PathBuf>> {
    match format {
        Format::HfSafetensors => validate_hf_safetensors_bundle(path),
        Format::Safetensors => validate_safetensors_artifacts(path),
        _ => Err(MetamorphError::NotImplemented(
            "validation is not wired yet for this format",
        )),
    }
}

pub(crate) fn validate_hf_safetensors_bundle(path: &Path) -> Result<Vec<PathBuf>> {
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

    validate_safetensors_artifacts(&path.join("model.safetensors"))?;

    let report = inspect(&Source::LocalPath(path.to_path_buf()))?;
    if report.detected_format != Some(Format::HfSafetensors) {
        return Err(MetamorphError::InvalidOutputBundle {
            path: path.to_path_buf(),
            reason: "output does not inspect as `hf-safetensors`".to_owned(),
        });
    }

    Ok(checked_paths)
}

pub(crate) fn validate_safetensors_artifacts(path: &Path) -> Result<Vec<PathBuf>> {
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

    let mut safetensors_files = fs::read_dir(path)?
        .filter_map(std::result::Result::ok)
        .map(|entry| entry.path())
        .filter(|entry| detect_path_format(entry) == Some(Format::Safetensors))
        .collect::<Vec<_>>();
    safetensors_files.sort();

    if safetensors_files.is_empty() {
        return Err(MetamorphError::InvalidOutputBundle {
            path: path.to_path_buf(),
            reason: "missing required safetensors artifacts".to_owned(),
        });
    }

    for safetensors_file in &safetensors_files {
        candle_core::safetensors::load(safetensors_file, &Device::Cpu)?;
    }

    Ok(safetensors_files)
}
