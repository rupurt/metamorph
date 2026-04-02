use std::fmt;
use std::str::FromStr;

use serde::{Deserialize, Serialize};

use crate::error::{MetamorphError, Result};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum Format {
    Gguf,
    HfSafetensors,
    Safetensors,
    Mlx,
}

impl Format {
    pub fn is_lossy_to(self, other: Self) -> bool {
        matches!(
            (self, other),
            (Self::Gguf, Self::HfSafetensors) | (Self::Gguf, Self::Safetensors)
        )
    }
}

impl fmt::Display for Format {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let label = match self {
            Self::Gguf => "gguf",
            Self::HfSafetensors => "hf-safetensors",
            Self::Safetensors => "safetensors",
            Self::Mlx => "mlx",
        };

        f.write_str(label)
    }
}

impl FromStr for Format {
    type Err = MetamorphError;

    fn from_str(value: &str) -> Result<Self> {
        let normalized = value.trim().to_ascii_lowercase().replace('_', "-");

        match normalized.as_str() {
            "gguf" => Ok(Self::Gguf),
            "hf-safetensors" | "huggingface-safetensors" | "hf" => Ok(Self::HfSafetensors),
            "safetensors" => Ok(Self::Safetensors),
            "mlx" => Ok(Self::Mlx),
            _ => Err(MetamorphError::UnsupportedFormat(value.to_owned())),
        }
    }
}
