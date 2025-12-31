use clap::Parser;
use ferris_proof_cli::{Cli, Commands};
use std::process;
use tracing::{error, info, Level};

#[tokio::main]
async fn main() {
    let cli = Cli::parse();

    // Initialize tracing based on verbosity level
    let log_level = match cli.verbose {
        0 => Level::WARN,
        1 => Level::INFO,
        2 => Level::DEBUG,
        _ => Level::TRACE,
    };

    tracing_subscriber::fmt()
        .with_max_level(log_level)
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .init();

    info!("FerrisProof starting with command: {:?}", cli.command);

    let result = match cli.command {
        Commands::Init {
            level,
            interactive,
            template,
        } => ferris_proof_cli::commands::init::run(level, interactive, template).await,
        Commands::Check { module, layer, fix } => {
            ferris_proof_cli::commands::check::run(module, layer, fix).await
        }
        Commands::Config { file, validate } => {
            ferris_proof_cli::commands::config::run(file, validate).await
        }
        Commands::Upgrade {
            to,
            dry_run,
            interactive,
        } => ferris_proof_cli::commands::upgrade::run(to, dry_run, interactive).await,
        Commands::Generate { target, output_dir } => {
            ferris_proof_cli::commands::generate::run(target, output_dir).await
        }
        Commands::Explain { error_code } => {
            ferris_proof_cli::commands::explain::run(error_code).await
        }
        Commands::Cache { action } => ferris_proof_cli::commands::cache::run(action).await,
    };

    match result {
        Ok(exit_code) => process::exit(exit_code),
        Err(e) => {
            error!("Error: {}", e);
            process::exit(1);
        }
    }
}
