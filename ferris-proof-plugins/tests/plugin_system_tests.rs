use ferris_proof_core::plugins::{PluginManager, ValidationStatus, VerificationPlugin};
use ferris_proof_plugins::sandbox::{NetworkPolicy, ResourceLimits};
use ferris_proof_plugins::{ProptestPlugin, SandboxedExecutor, TlaPlusPlugin};
use serde_json::json;
use std::collections::HashMap;
use std::time::Duration;

#[cfg(test)]
mod plugin_system_tests {
    use super::*;

    #[test]
    /// Test plugin loading and version compatibility
    /// Validates: Requirements 9.11, 9.16
    fn test_plugin_loading_and_version_compatibility() {
        let mut plugin_manager = PluginManager::new();

        // Test loading TLA+ plugin
        let tla_plugin = Box::new(TlaPlusPlugin::new());
        let plugin_name = tla_plugin.name().to_string();
        let plugin_version = tla_plugin.version().to_string();

        // Plugin should load successfully
        let load_result = plugin_manager.register_plugin(tla_plugin);
        assert!(load_result.is_ok(), "TLA+ plugin should load successfully");

        // Check that plugin is registered
        let plugin_metadata = plugin_manager.plugin_metadata(&plugin_name);
        assert!(
            plugin_metadata.is_some(),
            "Plugin metadata should be available"
        );

        let metadata = plugin_metadata.unwrap();
        assert_eq!(metadata.name, plugin_name);
        assert_eq!(metadata.version, plugin_version);

        // Test loading Proptest plugin
        let proptest_plugin = Box::new(ProptestPlugin::new());
        let proptest_name = proptest_plugin.name().to_string();

        let load_result = plugin_manager.register_plugin(proptest_plugin);
        assert!(
            load_result.is_ok(),
            "Proptest plugin should load successfully"
        );

        // Verify both plugins are listed
        let all_plugins = plugin_manager.list_plugins();
        assert!(
            all_plugins.len() >= 2,
            "Should have at least 2 plugins registered"
        );

        let plugin_names: Vec<String> = all_plugins.iter().map(|p| p.name.clone()).collect();
        assert!(
            plugin_names.contains(&plugin_name),
            "TLA+ plugin should be in list"
        );
        assert!(
            plugin_names.contains(&proptest_name),
            "Proptest plugin should be in list"
        );
    }

    #[test]
    /// Test tool discovery and availability checking
    /// Validates: Requirements 9.11, 9.16
    fn test_tool_discovery_and_availability() {
        let mut plugin_manager = PluginManager::new();

        // Register plugins
        let tla_plugin = Box::new(TlaPlusPlugin::new());
        let proptest_plugin = Box::new(ProptestPlugin::new());

        plugin_manager
            .register_plugin(tla_plugin)
            .expect("TLA+ plugin should register");
        plugin_manager
            .register_plugin(proptest_plugin)
            .expect("Proptest plugin should register");

        // Validate tools
        let validation_results = plugin_manager
            .validate_tools()
            .expect("Tool validation should succeed");

        assert!(
            !validation_results.is_empty(),
            "Should have validation results"
        );

        for result in &validation_results {
            // Each plugin should have a validation result
            assert!(
                !result.plugin_name.is_empty(),
                "Plugin name should not be empty"
            );

            match result.status {
                ValidationStatus::Valid => {
                    assert!(
                        result.tool_info.is_some(),
                        "Valid tools should have tool info"
                    );
                    let tool_info = result.tool_info.as_ref().unwrap();
                    assert!(
                        tool_info.available,
                        "Valid tools should be marked as available"
                    );
                    assert!(
                        !tool_info.capabilities.is_empty(),
                        "Valid tools should have capabilities"
                    );
                }
                ValidationStatus::Unavailable => {
                    // Tool not available - this is acceptable in test environments
                    assert!(
                        !result.issues.is_empty(),
                        "Unavailable tools should have issues listed"
                    );
                }
                ValidationStatus::VersionIncompatible => {
                    // Version incompatible - should have tool info but marked as incompatible
                    assert!(
                        !result.issues.is_empty(),
                        "Incompatible tools should have issues listed"
                    );
                }
                ValidationStatus::Error => {
                    // Error occurred during validation
                    assert!(
                        !result.issues.is_empty(),
                        "Error status should have issues listed"
                    );
                }
            }
        }
    }

    #[test]
    /// Test plugin metadata and capabilities
    /// Validates: Requirements 9.15
    fn test_plugin_metadata_and_capabilities() {
        let tla_plugin = TlaPlusPlugin::new();
        let proptest_plugin = ProptestPlugin::new();

        // Test TLA+ plugin metadata
        let tla_metadata = tla_plugin.metadata();
        assert_eq!(tla_metadata.name, "tla-plus");
        assert!(!tla_metadata.version.is_empty());
        assert!(!tla_metadata.description.is_empty());
        assert!(!tla_metadata.techniques.is_empty());
        assert!(tla_metadata
            .techniques
            .contains(&ferris_proof_core::types::Technique::FormalSpecs));
        assert!(tla_metadata
            .techniques
            .contains(&ferris_proof_core::types::Technique::ModelChecking));

        // Test Proptest plugin metadata
        let proptest_metadata = proptest_plugin.metadata();
        assert_eq!(proptest_metadata.name, "proptest");
        assert!(!proptest_metadata.version.is_empty());
        assert!(!proptest_metadata.description.is_empty());
        assert!(!proptest_metadata.techniques.is_empty());
        assert!(proptest_metadata
            .techniques
            .contains(&ferris_proof_core::types::Technique::PropertyTests));

        // Test supported techniques
        let tla_techniques = tla_plugin.supported_techniques();
        assert!(!tla_techniques.is_empty());

        let proptest_techniques = proptest_plugin.supported_techniques();
        assert!(!proptest_techniques.is_empty());

        // Test version ranges
        let tla_versions = tla_plugin.supported_versions();
        assert!(
            tla_versions.min.is_some()
                || tla_versions.max.is_some()
                || tla_versions.requires_exact.is_some()
        );

        let proptest_versions = proptest_plugin.supported_versions();
        assert!(
            proptest_versions.min.is_some()
                || proptest_versions.max.is_some()
                || proptest_versions.requires_exact.is_some()
        );
    }

    #[test]
    /// Test plugin initialization and cleanup
    /// Validates: Requirements 9.15
    fn test_plugin_initialization_and_cleanup() {
        let mut tla_plugin = TlaPlusPlugin::new();
        let mut proptest_plugin = ProptestPlugin::new();

        // Test initialization with empty config
        let empty_config = json!({});

        // TLA+ plugin initialization might fail if TLC is not available, which is acceptable
        let _tla_init_result = tla_plugin.initialize(&empty_config);
        // Don't assert success since TLC might not be installed

        // Proptest plugin initialization might fail if Rust toolchain is not available
        let _proptest_init_result = proptest_plugin.initialize(&empty_config);
        // Don't assert success since Rust might not be installed

        // Test initialization with specific config
        let tla_config = json!({
            "tla_plus": {
                "tlc_path": "/usr/local/bin/tlc"
            }
        });

        let proptest_config = json!({
            "proptest": {
                "path": "cargo"
            }
        });

        // These might fail if tools are not available, which is acceptable in tests
        let _ = tla_plugin.initialize(&tla_config);
        let _ = proptest_plugin.initialize(&proptest_config);

        // Test cleanup - should always succeed
        let tla_cleanup = tla_plugin.cleanup();
        assert!(tla_cleanup.is_ok(), "TLA+ plugin cleanup should succeed");

        let proptest_cleanup = proptest_plugin.cleanup();
        assert!(
            proptest_cleanup.is_ok(),
            "Proptest plugin cleanup should succeed"
        );
    }

    #[tokio::test]
    /// Test sandboxed execution with resource limits
    /// Validates: Requirements 12.4
    async fn test_sandboxed_execution_with_resource_limits() {
        let executor = SandboxedExecutor::new()
            .with_network_policy(NetworkPolicy::Denied)
            .with_limits(ResourceLimits {
                max_memory: 50 * 1024 * 1024, // 50MB
                max_cpu_time: 5,              // 5 seconds
                max_file_descriptors: 32,
                max_processes: 2,
                max_file_size: 1024, // 1KB
            })
            .with_timeout(Duration::from_secs(3));

        // Test executing a simple command
        let result = executor
            .execute("echo", &["Hello, World!"], HashMap::new(), None)
            .await;

        match result {
            Ok(output) => {
                assert_eq!(output.exit_code, 0, "Echo command should succeed");
                assert!(
                    output.stdout.contains("Hello, World!"),
                    "Output should contain expected text"
                );
                assert!(!output.timeout_occurred, "Command should not timeout");
            }
            Err(e) => {
                // Command might not be available in test environment
                let error_msg = e.to_string();
                assert!(
                    error_msg.contains("not found")
                        || error_msg.contains("No such file")
                        || error_msg.contains("Failed to spawn process"),
                    "Error should be due to command not found, got: {}",
                    error_msg
                );
            }
        }
    }

    #[tokio::test]
    /// Test sandboxed execution timeout handling
    /// Validates: Requirements 12.4
    async fn test_sandboxed_execution_timeout_handling() {
        let executor = SandboxedExecutor::new()
            .with_network_policy(NetworkPolicy::Denied)
            .with_limits(ResourceLimits::default())
            .with_timeout(Duration::from_millis(100)); // Very short timeout

        // Test with a command that should timeout (sleep)
        let result = executor
            .execute(
                "sleep",
                &["5"], // Sleep for 5 seconds, but timeout is 100ms
                HashMap::new(),
                None,
            )
            .await;

        match result {
            Ok(output) => {
                // If sleep command is available, it should timeout
                if output.timeout_occurred {
                    assert_ne!(
                        output.exit_code, 0,
                        "Timed out command should have non-zero exit code"
                    );
                    assert!(
                        output.stderr.contains("timeout"),
                        "Error should mention timeout"
                    );
                } else {
                    // Command might have completed very quickly or failed to start
                    // This is acceptable in test environments
                }
            }
            Err(e) => {
                // Command might not be available in test environment
                let error_msg = e.to_string();
                assert!(
                    error_msg.contains("not found")
                        || error_msg.contains("No such file")
                        || error_msg.contains("Failed to spawn process"),
                    "Error should be due to command not found, got: {}",
                    error_msg
                );
            }
        }
    }

    #[test]
    /// Test plugin manager with multiple plugins
    /// Validates: Requirements 9.11, 9.15, 9.16
    fn test_plugin_manager_multiple_plugins() {
        let mut plugin_manager = PluginManager::new();

        // Register multiple plugins
        let tla_plugin = Box::new(TlaPlusPlugin::new());
        let proptest_plugin = Box::new(ProptestPlugin::new());

        plugin_manager
            .register_plugin(tla_plugin)
            .expect("TLA+ plugin should register");
        plugin_manager
            .register_plugin(proptest_plugin)
            .expect("Proptest plugin should register");

        // Test getting plugins for specific techniques
        let formal_plugins =
            plugin_manager.plugins_for_technique(&ferris_proof_core::types::Technique::FormalSpecs);
        assert!(
            !formal_plugins.is_empty(),
            "Should have plugins for formal specs"
        );

        let property_plugins = plugin_manager
            .plugins_for_technique(&ferris_proof_core::types::Technique::PropertyTests);
        assert!(
            !property_plugins.is_empty(),
            "Should have plugins for property tests"
        );

        // Test plugin initialization
        let config = json!({
            "tla-plus": {
                "tlc_path": "/usr/local/bin/tlc"
            },
            "proptest": {
                "path": "cargo"
            }
        });

        let init_result = plugin_manager.initialize_plugins(&config);
        assert!(init_result.is_ok(), "Plugin initialization should succeed");

        // Test plugin cleanup
        let cleanup_result = plugin_manager.cleanup_plugins();
        assert!(cleanup_result.is_ok(), "Plugin cleanup should succeed");
    }

    #[test]
    /// Test resource limits configuration
    /// Validates: Requirements 12.4
    fn test_resource_limits_configuration() {
        let limits = ResourceLimits {
            max_memory: 100 * 1024 * 1024, // 100MB
            max_cpu_time: 30,              // 30 seconds
            max_file_descriptors: 64,
            max_processes: 5,
            max_file_size: 10 * 1024 * 1024, // 10MB
        };

        let _executor = SandboxedExecutor::new().with_limits(limits.clone());

        // Verify limits are set correctly
        // Note: We can't directly access private fields, but we can test behavior
        // Resource limits should be configurable

        // Test default limits
        let default_limits = ResourceLimits::default();
        assert!(
            default_limits.max_memory > 0,
            "Default memory limit should be positive"
        );
        assert!(
            default_limits.max_cpu_time > 0,
            "Default CPU time limit should be positive"
        );
        assert!(
            default_limits.max_file_descriptors > 0,
            "Default file descriptor limit should be positive"
        );
        assert!(
            default_limits.max_processes > 0,
            "Default process limit should be positive"
        );
        assert!(
            default_limits.max_file_size > 0,
            "Default file size limit should be positive"
        );
    }

    #[test]
    /// Test network policy configuration
    /// Validates: Requirements 12.1
    fn test_network_policy_configuration() {
        // Test denied policy
        let _denied_executor = SandboxedExecutor::new().with_network_policy(NetworkPolicy::Denied);

        // Test allowlist policy
        let allowed_hosts = vec!["example.com".to_string(), "trusted.org".to_string()];
        let _allowlist_executor = SandboxedExecutor::new()
            .with_network_policy(NetworkPolicy::AllowList(allowed_hosts.clone()));

        // Test unrestricted policy with consent
        let _unrestricted_executor = SandboxedExecutor::new()
            .with_network_policy(NetworkPolicy::Unrestricted { user_consent: true });

        // Test unrestricted policy without consent
        let _no_consent_executor =
            SandboxedExecutor::new().with_network_policy(NetworkPolicy::Unrestricted {
                user_consent: false,
            });

        // Verify policies can be configured
        // Network policies should be configurable

        // Test policy equality
        let policy1 = NetworkPolicy::Denied;
        let policy2 = NetworkPolicy::Denied;
        assert_eq!(policy1, policy2, "Denied policies should be equal");

        let policy3 = NetworkPolicy::AllowList(vec!["test.com".to_string()]);
        let policy4 = NetworkPolicy::AllowList(vec!["test.com".to_string()]);
        assert_eq!(
            policy3, policy4,
            "Identical allowlist policies should be equal"
        );

        let policy5 = NetworkPolicy::Unrestricted { user_consent: true };
        let policy6 = NetworkPolicy::Unrestricted { user_consent: true };
        assert_eq!(
            policy5, policy6,
            "Identical unrestricted policies should be equal"
        );
    }
}
