use clap::{Parser, Subcommand};
use ferris_proof_core::{Layer, VerificationLevel};
use std::path::PathBuf;

pub mod commands;

#[derive(Parser)]
#[command(name = "ferris-proof")]
#[command(about = "Multi-layer correctness pipeline for Rust applications")]
#[command(version)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
    
    #[arg(long, global = true, help = "Path to configuration file")]
    pub config: Option<PathBuf>,
    
    #[arg(long, global = true, help = "Enable verbose output")]
    pub verbose: bool,
    
    #[arg(long, global = true, help = "Output format")]
    pub output_format: Option<OutputFormat>,
}

#[derive(Subcommand, Debug)]
pub enum Commands {
    /// Initialize project with verification configuration
    Init {
        #[arg(long, default_value = "standard", help = "Verification level")]
        level: VerificationLevel,
        #[arg(long, help = "Use interactive mode")]
        interactive: bool,
        #[arg(long, help = "Project template to use")]
        template: Option<String>,
    },
    
    /// Run verification checks
    Check {
        #[arg(long, help = "Specific module to check")]
        module: Option<String>,
        #[arg(long, help = "Specific layer to run")]
        layer: Option<Layer>,
        #[arg(long, help = "Automatically fix violations")]
        fix: bool,
    },
    
    /// Show effective configuration
    Config {
        #[arg(long, help = "Show config for specific file")]
        file: Option<PathBuf>,
        #[arg(long, help = "Validate configuration")]
        validate: bool,
    },
    
    /// Upgrade verification level
    Upgrade {
        #[arg(long, help = "Target verification level")]
        to: VerificationLevel,
        #[arg(long, help = "Show changes without applying")]
        dry_run: bool,
        #[arg(long, help = "Use interactive mode")]
        interactive: bool,
    },
    
    /// Generate verification artifacts
    Generate {
        #[arg(long, help = "Type of artifact to generate")]
        target: GenerateTarget,
        #[arg(long, help = "Output directory")]
        output_dir: Option<PathBuf>,
    },
    
    /// Explain error codes and provide guidance
    Explain {
        #[arg(help = "Error code to explain")]
        error_code: String,
    },
}

#[derive(Clone, Debug)]
pub enum OutputFormat {
    Json,
    Human,
    Compact,
}

impl std::str::FromStr for OutputFormat {
    type Err = String;
    
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "json" => Ok(OutputFormat::Json),
            "human" => Ok(OutputFormat::Human),
            "compact" => Ok(OutputFormat::Compact),
            _ => Err(format!("Invalid output format: {}", s)),
        }
    }
}

#[derive(Clone, Debug)]
pub enum GenerateTarget {
    PropertyTests,
    SessionTypes,
    RefinementTypes,
    FormalSpecs,
}

impl std::str::FromStr for GenerateTarget {
    type Err = String;
    
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "property-tests" => Ok(GenerateTarget::PropertyTests),
            "session-types" => Ok(GenerateTarget::SessionTypes),
            "refinement-types" => Ok(GenerateTarget::RefinementTypes),
            "formal-specs" => Ok(GenerateTarget::FormalSpecs),
            _ => Err(format!("Invalid generate target: {}", s)),
        }
    }
}