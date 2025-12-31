use anyhow::{Context, Result};
use ferris_proof_config::{Config, ConfigManager};
use std::path::PathBuf;
use colored::Colorize;

pub async fn run(file: Option<PathBuf>, validate: bool) -> Result<i32> {
    if validate {
        return validate_configuration().await;
    }
    
    if let Some(file_path) = file {
        show_file_configuration(&file_path).await
    } else {
        show_project_configuration().await
    }
}

async fn validate_configuration() -> Result<i32> {
    println!("{}", "Validating configuration...".cyan());
    
    let current_dir = std::env::current_dir()
        .context("Failed to get current directory")?;
    
    match ConfigManager::from_project_root(&current_dir) {
        Ok(config_manager) => {
            match config_manager.validate() {
                Ok(()) => {
                    println!("✓ {}", "Configuration is valid".green());
                    Ok(0)
                }
                Err(error) => {
                    println!("✗ {}", "Configuration validation failed:".red());
                    println!("  • {}", format!("{}", error).red());
                    Ok(1)
                }
            }
        }
        Err(e) => {
            println!("✗ {}", format!("Failed to load configuration: {}", e).red());
            Ok(1)
        }
    }
}

async fn show_file_configuration(file_path: &PathBuf) -> Result<i32> {
    println!("{}", format!("Configuration for file: {}", file_path.display()).cyan());
    
    let current_dir = std::env::current_dir()
        .context("Failed to get current directory")?;
    
    let _config_manager = ConfigManager::from_project_root(&current_dir)
        .context("Failed to load configuration manager")?;
    
    let effective_config = _config_manager.for_file(file_path);
    
    // Display the effective configuration
    display_effective_config(&effective_config);
    
    Ok(0)
}

async fn show_project_configuration() -> Result<i32> {
    println!("{}", "Project Configuration".cyan());
    
    let current_dir = std::env::current_dir()
        .context("Failed to get current directory")?;
    
    // Check if ferrisproof.toml exists
    let config_file = current_dir.join("ferrisproof.toml");
    if !config_file.exists() {
        println!("✗ {}", "No ferrisproof.toml found in current directory".red());
        println!("  Run {} to initialize a project", "ferris-proof init".cyan());
        return Ok(1);
    }
    
    let config_manager = ConfigManager::from_project_root(&current_dir)
        .context("Failed to load configuration manager")?;
    
    // Load and display the root configuration
    let config_content = std::fs::read_to_string(&config_file)
        .context("Failed to read ferrisproof.toml")?;
    
    let root_config: Config = toml::from_str(&config_content)
        .context("Failed to parse ferrisproof.toml")?;
    
    display_config(&root_config);
    
    // Show discovered configuration files
    println!("\n{}", "Configuration Files:".yellow());
    println!("  • {}", config_file.display().to_string().green());
    
    Ok(0)
}

fn display_config(config: &Config) {
    println!("\n{}", "Profile Configuration:".yellow());
    println!("  Level: {}", format!("{:?}", config.profile.level).green());
    println!("  Enforcement: {}", format!("{:?}", config.profile.enforcement).green());
    println!("  Enabled Techniques:");
    for technique in &config.profile.enabled_techniques {
        println!("    • {}", format!("{:?}", technique).green());
    }
    
    println!("\n{}", "Tool Configuration:".yellow());
    if let Some(ref proptest_config) = config.tools.proptest {
        println!("  Proptest:");
        if let Some(cases) = proptest_config.cases {
            println!("    Cases: {}", cases.to_string().green());
        }
        if let Some(max_shrink_iters) = proptest_config.max_shrink_iters {
            println!("    Max Shrink Iterations: {}", max_shrink_iters.to_string().green());
        }
    }
    
    if let Some(ref tla_config) = config.tools.tla_plus {
        println!("  TLA+:");
        if let Some(ref path) = tla_config.tlc_path {
            println!("    TLC Path: {}", path.display().to_string().green());
        }
        if let Some(timeout) = tla_config.timeout {
            println!("    Timeout: {}s", timeout.to_string().green());
        }
        if let Some(workers) = tla_config.workers {
            println!("    Workers: {}", workers.to_string().green());
        }
    }
    
    if let Some(ref alloy_config) = config.tools.alloy {
        println!("  Alloy:");
        if let Some(ref path) = alloy_config.analyzer_path {
            println!("    Analyzer Path: {}", path.display().to_string().green());
        }
        if let Some(scope) = alloy_config.scope {
            println!("    Scope: {}", scope.to_string().green());
        }
    }
    
    if let Some(ref kani_config) = config.tools.kani {
        println!("  Kani:");
        if let Some(ref path) = kani_config.cbmc_path {
            println!("    CBMC Path: {}", path.display().to_string().green());
        }
        if let Some(unwind) = kani_config.unwind {
            println!("    Unwind: {}", unwind.to_string().green());
        }
    }
    
    println!("\n{}", "Features:".yellow());
    println!("  Cache Enabled: {}", config.features.cache_enabled.to_string().green());
    println!("  Parallel Execution: {}", config.features.parallel_execution.to_string().green());
    println!("  Generate Reports: {}", config.features.generate_reports.to_string().green());
    
    println!("\n{}", "Thresholds:".yellow());
    println!("  Max Verification Time: {}s", config.thresholds.max_verification_time.to_string().green());
    println!("  Max Memory Usage: {} bytes", config.thresholds.max_memory_usage.to_string().green());
    println!("  Cache TTL: {}s", config.thresholds.cache_ttl.to_string().green());
    
    println!("\n{}", "CI Configuration:".yellow());
    println!("  Fail on Violations: {}", config.ci.fail_on_violations.to_string().green());
    println!("  Generate Artifacts: {}", config.ci.generate_artifacts.to_string().green());
    println!("  Upload Reports: {}", config.ci.upload_reports.to_string().green());
}

fn display_effective_config(config: &ferris_proof_config::manager::EffectiveConfig) {
    println!("\n{}", "Effective Configuration:".yellow());
    println!("  Level: {}", format!("{:?}", config.level).green());
    println!("  Enforcement: {}", format!("{:?}", config.enforcement).green());
    println!("  Enabled Techniques:");
    for technique in &config.enabled_techniques {
        println!("    • {}", format!("{:?}", technique).green());
    }
}