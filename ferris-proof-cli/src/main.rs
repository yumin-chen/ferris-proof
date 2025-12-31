use clap::Parser;
use ferris_proof_cli::{Cli, Commands, CacheAction};
use std::process;
use tracing::{error, info};

#[tokio::main]
async fn main() {
    // Initialize tracing
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .init();

    let cli = Cli::parse();
    
    info!("FerrisProof starting with command: {:?}", cli.command);

    let result = match cli.command {
        Commands::Init { level, interactive, template } => {
            ferris_proof_cli::commands::init::run(level, interactive, template).await
        }
        Commands::Check { module, layer, fix } => {
            ferris_proof_cli::commands::check::run(module, layer, fix).await
        }
        Commands::Config { file, validate } => {
            ferris_proof_cli::commands::config::run(file, validate).await
        }
        Commands::Upgrade { to, dry_run, interactive } => {
            ferris_proof_cli::commands::upgrade::run(to, dry_run, interactive).await
        }
        Commands::Generate { target, output_dir } => {
            ferris_proof_cli::commands::generate::run(target, output_dir).await
        }
        Commands::Explain { error_code } => {
            ferris_proof_cli::commands::explain::run(error_code).await
        }
        Commands::Cache { action } => {
            ferris_proof_cli::commands::cache::run(action).await
        }
    };

    match result {
        Ok(exit_code) => process::exit(exit_code),
        Err(e) => {
            error!("Error: {}", e);
            process::exit(1);
        }
    }
}