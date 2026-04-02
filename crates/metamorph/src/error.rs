use std::path::PathBuf;

use thiserror::Error;

use crate::format::Format;

pub type Result<T> = std::result::Result<T, MetamorphError>;
pub(crate) const HF_TOKEN_ENV: &str = "HF_TOKEN";

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
    #[error("conversion execution does not support source `{0}` yet")]
    UnsupportedExecutionSource(String),
    #[error("conversion execution does not support target `{0}` yet")]
    UnsupportedExecutionTarget(String),
    #[error("expected a local GGUF file or a directory containing exactly one GGUF file: {0}")]
    InvalidLocalGgufSource(String),
    #[error("output bundle at `{path}` is invalid: {reason}")]
    InvalidOutputBundle { path: PathBuf, reason: String },
    #[error(
        "source `{input}` is not cached locally yet; expected a managed artifact under `{cache_path}`. Recover by populating that cache entry or using a local source path."
    )]
    SourceNotCached { input: String, cache_path: PathBuf },
    #[error("invalid publish destination `{0}`; expected `owner/name`")]
    InvalidPublishDestination(String),
    #[error(
        "publish execution for `{destination}` requires credentials in `{credential_env}`. Set that environment variable or rerun without `--execute` to keep this as a dry run."
    )]
    PublishCredentialsRequired {
        destination: String,
        credential_env: &'static str,
    },
    #[error(
        "remote publish execution is not implemented yet for `{0}`. Use the dry run to review the plan, keep the validated local bundle, and revisit execution once a backend and policy approval path exist."
    )]
    PublishExecutionNotImplemented(String),
    #[error("feature not implemented yet: {0}")]
    NotImplemented(&'static str),
    #[error(transparent)]
    Candle(#[from] candle_core::Error),
    #[error(transparent)]
    Json(#[from] serde_json::Error),
    #[error(transparent)]
    Tokenizer(#[from] tokenizers::Error),
    #[error(transparent)]
    Io(#[from] std::io::Error),
}
