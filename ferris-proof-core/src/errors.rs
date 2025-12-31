use crate::types::{Location, Violation};
use std::path::PathBuf;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum FerrisProofError {
    #[error("Configuration error: {message}")]
    Configuration {
        message: String,
        location: Option<Location>,
        suggestions: Vec<String>,
    },

    #[error("Tool error: {tool} - {message}")]
    Tool {
        tool: String,
        message: String,
        exit_code: Option<i32>,
        stderr: Option<String>,
    },

    #[error("Verification failed: {violations_count} violation(s)")]
    Verification {
        violations: Vec<Violation>,
        violations_count: usize,
    },

    #[error("IO error: {message}")]
    Io {
        message: String,
        path: Option<PathBuf>,
        #[source]
        source: std::io::Error,
    },

    #[error("Parse error: {message}")]
    Parse {
        message: String,
        location: Location,
        expected: Option<String>,
    },
}

impl FerrisProofError {
    pub fn code(&self) -> &str {
        match self {
            Self::Configuration { .. } => "FP-CF-001",
            Self::Tool { .. } => "FP-TL-001",
            Self::Verification { .. } => "FP-VR-001",
            Self::Io { .. } => "FP-IO-001",
            Self::Parse { .. } => "FP-PS-001",
        }
    }

    pub fn explanation(&self) -> String {
        match self.code() {
            "FP-CF-001" => "Invalid configuration detected. Check your ferrisproof.toml file for syntax errors or invalid values.".to_string(),
            "FP-TL-001" => "External verification tool error. Ensure all required tools are installed and accessible.".to_string(),
            "FP-VR-001" => "Verification violations found. Review the reported issues and fix them before proceeding.".to_string(),
            "FP-IO-001" => "File system operation failed. Check file permissions and disk space.".to_string(),
            "FP-PS-001" => "Parse error in input file. Check syntax and format.".to_string(),
            _ => format!("No detailed explanation available for error code {}", self.code()),
        }
    }
}
