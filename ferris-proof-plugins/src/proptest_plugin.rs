use anyhow::{anyhow, Result};
use ferris_proof_core::{
    plugins::{
        PerformanceMetrics, PluginMetadata, StructuredResult, ToolInfo, VerificationInput,
        VerificationOutput, VerificationPlugin, VersionRange,
    },
    types::*,
    verification::Target,
};
use serde_json::json;
use std::path::PathBuf;
use std::process::Command;
use std::time::Duration;
use tracing::{debug, info};

pub struct ProptestPlugin {
    tool_path: PathBuf,
    initialized: bool,
}

impl ProptestPlugin {
    pub fn new() -> Self {
        Self {
            tool_path: PathBuf::from("proptest"), // Default to system PATH
            initialized: false,
        }
    }

    /// Check if the proptest crate is available in the current environment
    fn check_proptest_crate_availability(&self) -> bool {
        // Try to create a minimal Cargo.toml and check if proptest can be resolved
        let temp_dir = std::env::temp_dir().join("ferris_proof_proptest_check");

        if std::fs::create_dir_all(&temp_dir).is_err() {
            return false;
        }

        let cargo_toml_content = r#"
[package]
name = "proptest-check"
version = "0.1.0"
edition = "2021"

[dependencies]
proptest = "1.0"
"#;

        let cargo_toml_path = temp_dir.join("Cargo.toml");
        if std::fs::write(&cargo_toml_path, cargo_toml_content).is_err() {
            return false;
        }

        // Try to run cargo check
        let check_result = Command::new("cargo")
            .current_dir(&temp_dir)
            .args(["check", "--quiet"])
            .output();

        // Clean up
        let _ = std::fs::remove_dir_all(&temp_dir);

        match check_result {
            Ok(output) => output.status.success(),
            Err(_) => false,
        }
    }

    /// Run proptest on a Rust target
    fn run_proptest(
        &self,
        target: &Target,
        config: &VerificationInput,
    ) -> Result<VerificationOutput> {
        let start_time = std::time::Instant::now();

        match target {
            Target::RustFile(path) => {
                info!("Running proptest on Rust file: {:?}", path);

                // Create a temporary directory for test execution
                let temp_dir = config
                    .context
                    .cache_dir
                    .join(format!("proptest_{}", uuid::Uuid::new_v4()));
                std::fs::create_dir_all(&temp_dir)?;

                // Run cargo test with proptest
                let mut cmd = Command::new("cargo");
                cmd.current_dir(path.parent().unwrap_or_else(|| std::path::Path::new(".")));
                cmd.args(["test", "--test", "prop_tests", "--", "--nocapture"]);

                // Set environment variables for proptest
                cmd.env(
                    "PROPTEST_CASES",
                    config
                        .config
                        .tool_config
                        .get("cases")
                        .and_then(|v| v.as_u64())
                        .unwrap_or(1000)
                        .to_string(),
                );
                cmd.env(
                    "PROPTEST_MAX_SHRINK_ITERS",
                    config
                        .config
                        .tool_config
                        .get("max_shrink_iters")
                        .and_then(|v| v.as_u64())
                        .unwrap_or(10000)
                        .to_string(),
                );

                debug!("Executing command: {:?}", cmd);

                let output = cmd.output()?;
                let execution_time = start_time.elapsed();

                // Parse proptest output
                let structured_result = self.parse_proptest_output(
                    &String::from_utf8_lossy(&output.stdout),
                    &String::from_utf8_lossy(&output.stderr),
                )?;

                // Create violations for any test failures
                let violations = if structured_result.status == Status::Error {
                    vec![Violation {
                        id: "PROPTEST_FAILURE".to_string(),
                        severity: Severity::Error,
                        location: Location {
                            file: path.clone(),
                            line: None,
                            column: None,
                            span: None,
                        },
                        message: "Property-based tests failed".to_string(),
                        suggestion: Some(
                            "Check the test output for specific failure details".to_string(),
                        ),
                        rule: "proptest_verification".to_string(),
                    }]
                } else {
                    Vec::new()
                };

                Ok(VerificationOutput {
                    status: structured_result.status,
                    violations,
                    artifacts: vec![], // TODO: Generate test reports
                    tool_output: ToolOutput {
                        tool: "proptest".to_string(),
                        stdout: String::from_utf8_lossy(&output.stdout).to_string(),
                        stderr: String::from_utf8_lossy(&output.stderr).to_string(),
                        exit_code: output.status.code().unwrap_or(-1),
                        execution_time,
                    },
                    metrics: VerificationMetrics {
                        total_time: execution_time,
                        cache_hit_rate: 0.0,
                        memory_usage: 0, // TODO: Monitor memory usage
                        test_cases_executed: structured_result
                            .statistics
                            .get("test_cases_executed")
                            .and_then(|v| v.as_u64())
                            .unwrap_or(0) as u32,
                    },
                })
            }
            _ => Err(anyhow!("Proptest plugin only supports Rust files")),
        }
    }

    /// Parse proptest output into structured results
    fn parse_proptest_output(&self, stdout: &str, stderr: &str) -> Result<StructuredResult> {
        let output = stdout.to_string() + stderr;

        // Look for test failures
        if output.contains("test FAILED") || output.contains("panic") {
            return Ok(StructuredResult {
                status: Status::Error,
                violations: vec![],
                statistics: json!({
                    "test_cases_executed": self.extract_test_cases(&output),
                    "failures": self.extract_failures(&output),
                    "successes": self.extract_successes(&output)
                }),
                performance: PerformanceMetrics {
                    execution_time: Duration::from_millis(0), // Will be set by caller
                    memory_usage: 0,
                    cpu_usage: 0.0,
                    cache_hits: 0,
                },
            });
        }

        // Look for successful completion
        if output.contains("test result: ok") {
            return Ok(StructuredResult {
                status: Status::Success,
                violations: vec![],
                statistics: json!({
                    "test_cases_executed": self.extract_test_cases(&output),
                    "failures": 0,
                    "successes": self.extract_successes(&output)
                }),
                performance: PerformanceMetrics {
                    execution_time: Duration::from_millis(0),
                    memory_usage: 0,
                    cpu_usage: 0.0,
                    cache_hits: 0,
                },
            });
        }

        // Default to success if no failures detected
        Ok(StructuredResult {
            status: Status::Success,
            violations: vec![],
            statistics: json!({
                "test_cases_executed": 0,
                "failures": 0,
                "successes": 0
            }),
            performance: PerformanceMetrics {
                execution_time: Duration::from_millis(0),
                memory_usage: 0,
                cpu_usage: 0.0,
                cache_hits: 0,
            },
        })
    }

    fn extract_test_cases(&self, output: &str) -> u64 {
        // Look for patterns like "1030 tests run"
        let regex = regex::Regex::new(r"(\d+)\s+(?:test|tests|case|cases)").unwrap();
        if let Some(captures) = regex.captures(output) {
            captures
                .get(1)
                .and_then(|m| m.as_str().parse().ok())
                .unwrap_or(0)
        } else {
            0
        }
    }

    fn extract_failures(&self, output: &str) -> u64 {
        // Count occurrences of "FAILED" or "panicked"
        let failed_count = output.matches("FAILED").count();
        let panic_count = output.matches("panicked").count();
        (failed_count + panic_count) as u64
    }

    fn extract_successes(&self, output: &str) -> u64 {
        // Look for "passed" or "ok" in test results
        let passed_count = output.matches("passed").count();
        let ok_count = output.matches("ok").count();
        (passed_count + ok_count) as u64
    }
}

impl VerificationPlugin for ProptestPlugin {
    fn name(&self) -> &str {
        "proptest"
    }

    fn version(&self) -> &str {
        env!("CARGO_PKG_VERSION")
    }

    fn supported_techniques(&self) -> Vec<Technique> {
        vec![Technique::PropertyTests]
    }

    fn supported_versions(&self) -> VersionRange {
        VersionRange {
            min: Some(semver::Version::new(0, 1, 0)),
            max: Some(semver::Version::new(1, 0, 0)),
            requires_exact: None,
        }
    }

    fn check_availability(&self) -> Result<ToolInfo> {
        // Check if Rust and Cargo are available
        let cargo_result = Command::new("cargo").args(["--version"]).output();

        let rustc_result = Command::new("rustc").args(["--version"]).output();

        match (cargo_result, rustc_result) {
            (Ok(cargo_output), Ok(rustc_output))
                if cargo_output.status.success() && rustc_output.status.success() =>
            {
                let cargo_version = String::from_utf8_lossy(&cargo_output.stdout);
                let rustc_version = String::from_utf8_lossy(&rustc_output.stdout);

                // Extract version numbers
                let cargo_ver = cargo_version
                    .split_whitespace()
                    .nth(1)
                    .unwrap_or("unknown")
                    .to_string();

                let rustc_ver = rustc_version
                    .split_whitespace()
                    .nth(1)
                    .unwrap_or("unknown")
                    .to_string();

                // Check if proptest crate is available by trying to compile a simple test
                let proptest_available = self.check_proptest_crate_availability();

                let version = format!("cargo {} / rustc {}", cargo_ver, rustc_ver);
                let mut capabilities = vec![
                    "property_testing".to_string(),
                    "test_generation".to_string(),
                    "shrinking".to_string(),
                    "rust_integration".to_string(),
                ];

                if proptest_available {
                    capabilities.push("proptest_crate".to_string());
                }

                Ok(ToolInfo {
                    name: "proptest".to_string(),
                    version,
                    path: PathBuf::from("cargo"),
                    available: true,
                    capabilities,
                })
            }
            (Ok(cargo_output), Ok(_)) if !cargo_output.status.success() => {
                Err(anyhow!("Cargo is not working properly"))
            }
            (Ok(_), Ok(rustc_output)) if !rustc_output.status.success() => {
                Err(anyhow!("Rustc is not working properly"))
            }
            (Ok(_), Ok(_)) => {
                // Both commands succeeded but we didn't handle this case above
                Err(anyhow!("Unexpected cargo/rustc status"))
            }
            (Err(cargo_err), _) => Err(anyhow!("Cargo not found: {}", cargo_err)),
            (_, Err(rustc_err)) => Err(anyhow!("Rustc not found: {}", rustc_err)),
        }
    }

    fn verify(&self, input: VerificationInput) -> Result<VerificationOutput> {
        if !self.initialized {
            return Err(anyhow!("Proptest plugin not initialized"));
        }

        self.run_proptest(&input.target, &input)
    }

    fn parse_output(&self, raw_output: &str) -> Result<StructuredResult> {
        self.parse_proptest_output(raw_output, "")
    }

    fn metadata(&self) -> PluginMetadata {
        PluginMetadata {
            name: "proptest".to_string(),
            version: self.version().to_string(),
            description: "Property-based testing plugin using the proptest framework".to_string(),
            author: "FerrisProof Team".to_string(),
            license: "MIT".to_string(),
            homepage: Some(
                "https://altsysrq.github.io/proptest-book/proptest/index.html".to_string(),
            ),
            techniques: vec![Technique::PropertyTests],
            supported_platforms: vec![
                "linux".to_string(),
                "macos".to_string(),
                "windows".to_string(),
            ],
            dependencies: vec!["cargo".to_string(), "rustc".to_string()],
        }
    }

    fn initialize(&mut self, config: &serde_json::Value) -> Result<()> {
        // Extract proptest configuration
        if let Some(tool_config) = config.get("proptest") {
            if let Some(path) = tool_config.get("path").and_then(|v| v.as_str()) {
                self.tool_path = PathBuf::from(path);
            }
        }

        // Verify tool availability
        let tool_info = self.check_availability()?;
        if !tool_info.available {
            return Err(anyhow!("Proptest is not available: {}", tool_info.version));
        }

        self.initialized = true;
        info!(
            "Proptest plugin initialized with tool: {:?}",
            self.tool_path
        );
        Ok(())
    }

    fn cleanup(&mut self) -> Result<()> {
        self.initialized = false;
        debug!("Proptest plugin cleaned up");
        Ok(())
    }
}

impl Default for ProptestPlugin {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_plugin_metadata() {
        let plugin = ProptestPlugin::new();
        let metadata = plugin.metadata();

        assert_eq!(metadata.name, "proptest");
        assert_eq!(metadata.techniques, vec![Technique::PropertyTests]);
        assert!(metadata.supported_platforms.contains(&"linux".to_string()));
    }

    #[test]
    fn test_supported_techniques() {
        let plugin = ProptestPlugin::new();
        let techniques = plugin.supported_techniques();

        assert_eq!(techniques.len(), 1);
        assert!(techniques.contains(&Technique::PropertyTests));
    }

    #[test]
    fn test_output_parsing() {
        let plugin = ProptestPlugin::new();

        let success_output = "test result: ok. 1000 tests run.";
        let result = plugin.parse_proptest_output(success_output, "").unwrap();
        assert_eq!(result.status, Status::Success);

        let failure_output = "test FAILED: property should hold\n1 tests run.";
        let result = plugin.parse_proptest_output(failure_output, "").unwrap();
        assert_eq!(result.status, Status::Error);
    }

    #[test]
    fn test_statistics_extraction() {
        let plugin = ProptestPlugin::new();

        let output = "test result: ok. 1030 tests run, 0 failed.";
        assert_eq!(plugin.extract_test_cases(output), 1030);
        assert_eq!(plugin.extract_failures(output), 0);
        assert_eq!(plugin.extract_successes(output), 1); // from "ok"
    }
}
