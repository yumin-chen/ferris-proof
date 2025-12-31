use clap::{Parser, Subcommand, ValueEnum};
use ferris_proof_core::{Layer, VerificationLevel};
use std::path::PathBuf;

pub mod commands;

#[derive(Parser)]
#[command(name = "ferris-proof")]
#[command(about = "Multi-layer correctness pipeline for Rust applications")]
#[command(version)]
#[command(long_about = "FerrisProof is a multi-layer correctness pipeline for Rust applications that combines formal modeling (TLA+, Alloy), Rust's type system, and property-based testing to ensure systems are memory-safe, structurally sound, and functionally correct.")]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
    
    /// Path to configuration file (overrides default discovery)
    #[arg(long, global = true, value_name = "FILE")]
    pub config: Option<PathBuf>,
    
    /// Enable verbose output (can be repeated for more verbosity)
    #[arg(short, long, global = true, action = clap::ArgAction::Count)]
    pub verbose: u8,
    
    /// Output format for results and reports
    #[arg(long, global = true, value_enum)]
    pub output_format: Option<OutputFormat>,
    
    /// Disable colored output (respects NO_COLOR environment variable)
    #[arg(long, global = true)]
    pub no_color: bool,
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
    
    /// Manage verification cache
    Cache {
        #[command(subcommand)]
        action: CacheAction,
    },
}

#[derive(Subcommand, Debug)]
pub enum CacheAction {
    /// Show cache information and statistics
    Info,
    
    /// Clean up expired cache entries
    Cleanup,
    
    /// Clear all cache entries
    Clear,
    
    /// Compact cache by removing expired entries and optimizing storage
    Compact,
    
    /// Check cache health and integrity
    Health,
    
    /// Repair corrupted cache entries
    Repair,
}

#[derive(Clone, Debug, ValueEnum)]
pub enum OutputFormat {
    /// Human-readable output with colors and formatting
    Human,
    /// JSON output for machine parsing
    Json,
    /// Compact single-line format for CI environments
    Compact,
}

impl Default for OutputFormat {
    fn default() -> Self {
        OutputFormat::Human
    }
}

#[derive(Clone, Debug, ValueEnum)]
pub enum GenerateTarget {
    /// Generate property-based tests
    #[value(name = "property-tests")]
    PropertyTests,
    /// Generate session type definitions
    #[value(name = "session-types")]
    SessionTypes,
    /// Generate refinement type definitions
    #[value(name = "refinement-types")]
    RefinementTypes,
    /// Generate formal specification templates
    #[value(name = "formal-specs")]
    FormalSpecs,
}