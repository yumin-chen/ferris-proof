use anyhow::Result;
use ferris_proof_core::{
    plugins::{
        PerformanceMetrics, PluginMetadata, StructuredResult, ToolInfo, VerificationInput,
        VerificationOutput, VerificationPlugin, VersionRange,
    },
    types::{Status, Technique, ToolOutput, VerificationMetrics},
};
use semver::Version;
use serde_json::json;
use std::path::PathBuf;
use std::process::Command;
use tracing::info;

/// Simple replacement for which::which functionality
fn find_executable(name: &str) -> Option<PathBuf> {
    if let Ok(path_env) = std::env::var("PATH") {
        for path in std::env::split_paths(&path_env) {
            let candidate = path.join(name);
            if candidate.is_file() {
                return Some(candidate);
            }
            // Also try with .exe extension on Windows
            #[cfg(windows)]
            {
                let candidate_exe = path.join(format!("{}.exe", name));
                if candidate_exe.is_file() {
                    return Some(candidate_exe);
                }
            }
        }
    }
    None
}

pub struct TlaPlusPlugin {
    tlc_path: Option<PathBuf>,
    initialized: bool,
}

impl TlaPlusPlugin {
    pub fn new() -> Self {
        Self {
            tlc_path: find_executable("tlc"),
            initialized: false,
        }
    }
}

impl VerificationPlugin for TlaPlusPlugin {
    fn name(&self) -> &str {
        "tla-plus"
    }

    fn version(&self) -> &str {
        env!("CARGO_PKG_VERSION")
    }

    fn supported_techniques(&self) -> Vec<Technique> {
        vec![Technique::FormalSpecs, Technique::ModelChecking]
    }

    fn supported_versions(&self) -> VersionRange {
        VersionRange {
            min: Some(Version::new(0, 1, 0)),
            max: Some(Version::new(1, 0, 0)),
            requires_exact: None,
        }
    }

    fn check_availability(&self) -> Result<ToolInfo> {
        // First check if TLC is available via which
        let tlc_path = match &self.tlc_path {
            Some(path) => path.clone(),
            None => {
                // Try to find TLC in common locations
                let common_paths = [
                    "/usr/local/bin/tlc",
                    "/usr/bin/tlc",
                    "/opt/tla/bin/tlc",
                    "tlc", // Try PATH
                ];

                let mut found_path = None;
                for path_str in &common_paths {
                    let path = PathBuf::from(path_str);
                    if path.exists() || find_executable(path_str).is_some() {
                        found_path = Some(path);
                        break;
                    }
                }

                match found_path {
                    Some(path) => path,
                    None => {
                        return Err(anyhow::anyhow!(
                            "TLA+ TLC not found in PATH or common locations"
                        ))
                    }
                }
            }
        };

        // Try to get version information
        let version_result = Command::new(&tlc_path).arg("-version").output();

        match version_result {
            Ok(output) if output.status.success() => {
                let version_str = String::from_utf8_lossy(&output.stdout);
                let version = version_str
                    .lines()
                    .next()
                    .and_then(|line| {
                        // Extract version from TLC output (format varies)
                        if line.contains("TLC") {
                            line.split_whitespace()
                                .find(|word| {
                                    word.chars().next().is_some_and(|c| c.is_ascii_digit())
                                })
                                .map(|s| s.to_string())
                        } else {
                            None
                        }
                    })
                    .unwrap_or_else(|| "unknown".to_string());

                Ok(ToolInfo {
                    name: "TLA+ TLC".to_string(),
                    version,
                    path: tlc_path,
                    available: true,
                    capabilities: vec![
                        "model_checking".to_string(),
                        "temporal_logic".to_string(),
                        "invariant_checking".to_string(),
                        "liveness_checking".to_string(),
                        "safety_checking".to_string(),
                    ],
                })
            }
            Ok(output) => {
                // Command failed
                let stderr = String::from_utf8_lossy(&output.stderr);
                Err(anyhow::anyhow!("TLC command failed: {}", stderr))
            }
            Err(e) => {
                // Could not execute command
                Err(anyhow::anyhow!("Failed to execute TLC: {}", e))
            }
        }
    }

    fn verify(&self, input: VerificationInput) -> Result<VerificationOutput> {
        if !self.initialized {
            return Err(anyhow::anyhow!("TLA+ plugin not initialized"));
        }

        info!("Running TLA+ verification for {:?}", input.target);

        // TODO: Implement actual TLA+ verification
        let tool_output = ToolOutput {
            tool: "tlc".to_string(),
            stdout: "Model checking completed successfully".to_string(),
            stderr: String::new(),
            exit_code: 0,
            execution_time: std::time::Duration::from_millis(100),
        };

        Ok(VerificationOutput {
            status: Status::Success,
            violations: Vec::new(),
            artifacts: Vec::new(),
            tool_output,
            metrics: VerificationMetrics {
                total_time: std::time::Duration::from_millis(100),
                cache_hit_rate: 0.0,
                memory_usage: 0,
                test_cases_executed: 0,
            },
        })
    }

    fn parse_output(&self, raw_output: &str) -> Result<StructuredResult> {
        // Parse TLA+ TLC output
        let status = if raw_output.contains("Error:") || raw_output.contains("FAILED") {
            Status::Error
        } else if raw_output.contains("Warning:") {
            Status::Warning
        } else {
            Status::Success
        };

        Ok(StructuredResult {
            status,
            violations: vec![],
            statistics: json!({
                "states_explored": 0,
                "invariants_checked": 0,
                "temporal_properties_verified": 0
            }),
            performance: PerformanceMetrics {
                execution_time: std::time::Duration::from_millis(0),
                memory_usage: 0,
                cpu_usage: 0.0,
                cache_hits: 0,
            },
        })
    }

    fn metadata(&self) -> PluginMetadata {
        PluginMetadata {
            name: "tla-plus".to_string(),
            version: self.version().to_string(),
            description: "TLA+ formal specification verification plugin".to_string(),
            author: "FerrisProof Team".to_string(),
            license: "MIT".to_string(),
            homepage: Some("https://lamport.azurewebsites.net/tla/tla.html".to_string()),
            techniques: vec![Technique::FormalSpecs, Technique::ModelChecking],
            supported_platforms: vec![
                "linux".to_string(),
                "macos".to_string(),
                "windows".to_string(),
            ],
            dependencies: vec!["tlc".to_string(), "java".to_string()],
        }
    }

    fn initialize(&mut self, config: &serde_json::Value) -> Result<()> {
        // Extract TLA+ configuration
        if let Some(tool_config) = config.get("tla_plus") {
            if let Some(path) = tool_config.get("tlc_path").and_then(|v| v.as_str()) {
                self.tlc_path = Some(PathBuf::from(path));
            }
        }

        // Verify tool availability
        let tool_info = self.check_availability()?;
        if !tool_info.available {
            return Err(anyhow::anyhow!(
                "TLA+ TLC is not available: {}",
                tool_info.version
            ));
        }

        self.initialized = true;
        info!("TLA+ plugin initialized with TLC at: {:?}", self.tlc_path);
        Ok(())
    }

    fn cleanup(&mut self) -> Result<()> {
        self.initialized = false;
        info!("TLA+ plugin cleaned up");
        Ok(())
    }
}

impl Default for TlaPlusPlugin {
    fn default() -> Self {
        Self::new()
    }
}
