use std::fmt;

use serde::{Deserialize, Serialize};

use crate::error::{MetamorphError, Result};
use crate::format::Format;
use crate::source::{Source, Target, inspect};
use crate::transform::{ExecutionSupport, find_capability};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ConvertRequest {
    pub source: Source,
    pub target: Target,
    pub from: Option<Format>,
    pub to: Format,
    pub allow_lossy: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum CompatibilityStatus {
    Executable,
    PlannedOnly,
    Unsupported,
    UnknownSourceFormat,
}

impl fmt::Display for CompatibilityStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let label = match self {
            Self::Executable => "executable",
            Self::PlannedOnly => "planned-only",
            Self::Unsupported => "unsupported",
            Self::UnknownSourceFormat => "unknown-source-format",
        };

        f.write_str(label)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CompatibilityReport {
    pub source: Source,
    pub requested_source_format: Option<Format>,
    pub inferred_source_format: Option<Format>,
    pub source_format: Option<Format>,
    pub target_format: Format,
    pub status: CompatibilityStatus,
    pub lossy: bool,
    pub backend: Option<String>,
    pub blockers: Vec<String>,
    pub notes: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ConversionPlan {
    pub source_format: Format,
    pub target_format: Format,
    pub target: Target,
    pub lossy: bool,
    pub execution: ExecutionSupport,
    pub backend: Option<String>,
    pub steps: Vec<String>,
    pub notes: Vec<String>,
}

pub fn compatibility(request: &ConvertRequest) -> Result<CompatibilityReport> {
    let inspect_report = inspect(&request.source)?;
    let inferred_source_format = inspect_report.detected_format;
    let source_format = request.from.or(inferred_source_format);
    let mut notes = inspect_report.notes;

    if let Some(requested_source_format) = request.from {
        match inferred_source_format {
            Some(inferred_source_format) if inferred_source_format != requested_source_format => {
                notes.push(format!(
                    "using explicit source format `{requested_source_format}` instead of inferred `{inferred_source_format}`"
                ));
            }
            None => notes.push(format!(
                "using explicit source format `{requested_source_format}` because the input could not be inferred"
            )),
            _ => {}
        }
    }

    let Some(source_format) = source_format else {
        return Ok(CompatibilityReport {
            source: request.source.clone(),
            requested_source_format: request.from,
            inferred_source_format,
            source_format: None,
            target_format: request.to,
            status: CompatibilityStatus::UnknownSourceFormat,
            lossy: false,
            backend: None,
            blockers: vec![
                "Metamorph could not infer the source format yet. Supply `--from` or use a more explicit input layout.".to_owned(),
            ],
            notes,
        });
    };

    let Some(capability) = find_capability(source_format, request.to) else {
        return Ok(CompatibilityReport {
            source: request.source.clone(),
            requested_source_format: request.from,
            inferred_source_format,
            source_format: Some(source_format),
            target_format: request.to,
            status: CompatibilityStatus::Unsupported,
            lossy: false,
            backend: None,
            blockers: vec![format!(
                "no registered conversion capability exists for `{source_format} -> {}`",
                request.to
            )],
            notes,
        });
    };

    let mut blockers = Vec::new();
    if capability.lossy && !request.allow_lossy {
        blockers.push(format!(
            "lossy conversion requires explicit opt-in: {source_format} -> {}",
            request.to
        ));
    }
    if capability.execution_support == ExecutionSupport::PlannedOnly {
        blockers.push(format!(
            "execution backend `{}` is not wired yet for this compatible path",
            capability.backend_label().unwrap_or("planned-only")
        ));
    }

    Ok(CompatibilityReport {
        source: request.source.clone(),
        requested_source_format: request.from,
        inferred_source_format,
        source_format: Some(source_format),
        target_format: request.to,
        status: match capability.execution_support {
            ExecutionSupport::Executable => CompatibilityStatus::Executable,
            ExecutionSupport::PlannedOnly => CompatibilityStatus::PlannedOnly,
        },
        lossy: capability.lossy,
        backend: capability.backend_label().map(str::to_owned),
        blockers,
        notes,
    })
}

pub fn plan(request: &ConvertRequest) -> Result<ConversionPlan> {
    let compatibility_report = compatibility(request)?;
    let source_format = compatibility_report
        .source_format
        .ok_or_else(|| MetamorphError::UnknownFormatForSource(request.source.display_name()))?;
    let capability = find_capability(source_format, request.to).ok_or_else(|| {
        MetamorphError::UnsupportedConversionPath {
            from: source_format,
            to: request.to,
        }
    })?;

    if capability.lossy && !request.allow_lossy {
        return Err(MetamorphError::LossyConversionRequiresOptIn {
            from: source_format,
            to: request.to,
        });
    }

    Ok(ConversionPlan {
        source_format,
        target_format: request.to,
        target: request.target.clone(),
        lossy: capability.lossy,
        execution: capability.execution_support,
        backend: capability.backend_label().map(str::to_owned),
        steps: capability
            .steps
            .iter()
            .map(|step| (*step).to_owned())
            .collect(),
        notes: compatibility_report.notes,
    })
}
