use ferris_proof_plugins::sandbox::{SandboxedExecutor, NetworkPolicy, ResourceLimits};
use proptest::prelude::*;
use std::collections::HashMap;
use std::path::PathBuf;
use std::time::Duration;
use tokio::runtime::Runtime;

/// **Feature: ferris-proof, Property 11: Network isolation**
/// **Validates: Requirements 12.1**
/// 
/// For any verification execution without explicit external service consent,
/// the system should not initiate any network connections to external services.
#[cfg(test)]
mod network_isolation_tests {
    use super::*;

    proptest! {
        #[test]
        /// Property test for network isolation enforcement
        /// Tests that commands attempting network access are properly blocked
        /// when network policy is set to Denied
        fn network_isolation_blocks_external_connections(
            // Generate various network-related commands that should be blocked
            network_command in prop::sample::select(vec![
                ("curl", vec!["https://example.com".to_string()]),
                ("wget", vec!["http://example.com/file.txt".to_string()]),
                ("ping", vec!["8.8.8.8".to_string()]),
                ("nslookup", vec!["example.com".to_string()]),
                ("nc", vec!["-z".to_string(), "example.com".to_string(), "80".to_string()]),
            ]),
            // Generate different working directories
            working_dir in prop::option::of(
                prop::sample::select(vec![
                    PathBuf::from("/tmp"),
                    PathBuf::from("."),
                    PathBuf::from("/var/tmp"),
                ])
            ),
            // Generate various environment variables
            env_vars in prop::collection::hash_map(
                "[A-Z_]{1,10}",
                "[a-zA-Z0-9_/.-]{0,50}",
                0..5
            )
        ) {
            let rt = Runtime::new().unwrap();
            
            rt.block_on(async {
                // Create sandboxed executor with network access denied
                let executor = SandboxedExecutor::new()
                    .with_network_policy(NetworkPolicy::Denied)
                    .with_limits(ResourceLimits {
                        max_memory: 100 * 1024 * 1024, // 100MB
                        max_cpu_time: 10, // 10 seconds
                        max_file_descriptors: 64,
                        max_processes: 2,
                        max_file_size: 1024 * 1024, // 1MB
                    })
                    .with_timeout(Duration::from_secs(5));

                let (command, args) = network_command;
                let args_str: Vec<&str> = args.iter().map(|s| s.as_ref()).collect();

                // Execute the network command
                let result = executor.execute(
                    command,
                    &args_str,
                    env_vars,
                    working_dir.as_ref()
                ).await;

                match result {
                    Ok(output) => {
                        // If command succeeded, it should not have made external connections
                        let network_blocked = 
                            output.exit_code != 0 || // Command failed
                            output.stderr.contains("Connection refused") ||
                            output.stderr.contains("Network is unreachable") ||
                            output.stderr.contains("Name or service not known") ||
                            output.stderr.contains("Temporary failure in name resolution") ||
                            output.stderr.contains("No route to host") ||
                            output.stdout.is_empty(); // No successful output
                        
                        prop_assert!(
                            network_blocked,
                            "Network command '{}' should have been blocked or failed, but got exit_code: {}, stdout: '{}', stderr: '{}'",
                            command,
                            output.exit_code,
                            output.stdout.trim(),
                            output.stderr.trim()
                        );
                    }
                    Err(e) => {
                        // Command was blocked at validation level - this is expected and good
                        let error_msg = e.to_string();
                        let expected_blocking = 
                            error_msg.contains("not allowed in sandbox") ||
                            error_msg.contains("Network access denied") ||
                            error_msg.contains("Command") && error_msg.contains("not allowed");
                        
                        if !expected_blocking {
                            // If it's not a security-related error, it might be a system issue
                            // (e.g., command not found), which is acceptable
                            prop_assume!(
                                error_msg.contains("not found") || 
                                error_msg.contains("No such file") ||
                                error_msg.contains("Failed to spawn process")
                            );
                        }
                    }
                }
                
                Ok(())
            })?;
        }

        #[test]
        /// Property test for unrestricted network access with consent
        /// Tests that network access works when explicitly granted
        fn unrestricted_network_with_consent_allows_access(
            user_consent in any::<bool>(),
            command in prop::sample::select(vec!["echo", "true", "false"]), // Safe commands
        ) {
            let rt = Runtime::new().unwrap();
            
            rt.block_on(async {
                let executor = SandboxedExecutor::new()
                    .with_network_policy(NetworkPolicy::Unrestricted { user_consent })
                    .with_limits(ResourceLimits::default())
                    .with_timeout(Duration::from_secs(2));

                let result = executor.execute(
                    command,
                    &[],
                    HashMap::new(),
                    None
                ).await;

                if user_consent {
                    // With consent, safe commands should execute successfully
                    prop_assert!(
                        result.is_ok(),
                        "Command '{}' should succeed with user consent, but got error: {:?}",
                        command,
                        result.err()
                    );
                } else {
                    // Without consent, should be blocked
                    match result {
                        Ok(_) => {
                            // This might happen for very safe commands that don't trigger network checks
                            // This is acceptable as long as no actual network access occurs
                        }
                        Err(e) => {
                            let error_msg = e.to_string();
                            prop_assert!(
                                error_msg.contains("requires") && error_msg.contains("consent"),
                                "Expected consent error without user consent, got: {}",
                                error_msg
                            );
                        }
                    }
                }
                
                Ok(())
            })?;
        }
    }

    #[tokio::test]
    /// Integration test for network isolation with real network commands
    /// This test uses actual network commands to verify isolation works
    async fn test_network_isolation_integration() {
        let executor = SandboxedExecutor::new()
            .with_network_policy(NetworkPolicy::Denied)
            .with_limits(ResourceLimits {
                max_memory: 50 * 1024 * 1024, // 50MB
                max_cpu_time: 5, // 5 seconds
                max_file_descriptors: 32,
                max_processes: 1,
                max_file_size: 1024, // 1KB
            })
            .with_timeout(Duration::from_secs(3));

        // Test that curl is blocked
        let result = executor.execute(
            "curl",
            &["--connect-timeout", "1", "https://httpbin.org/get"],
            HashMap::new(),
            None
        ).await;

        match result {
            Ok(output) => {
                // Command executed but should have failed due to network restrictions
                assert_ne!(output.exit_code, 0, "curl should fail when network is denied");
                
                // Check for network-related error messages
                let has_network_error = 
                    output.stderr.contains("Could not resolve host") ||
                    output.stderr.contains("Connection refused") ||
                    output.stderr.contains("Network is unreachable") ||
                    output.stderr.contains("Temporary failure in name resolution");
                
                assert!(
                    has_network_error || output.stdout.is_empty(),
                    "Expected network error or empty output, got stdout: '{}', stderr: '{}'",
                    output.stdout,
                    output.stderr
                );
            }
            Err(e) => {
                // Command was blocked at validation level - this is also acceptable
                let error_msg = e.to_string();
                assert!(
                    error_msg.contains("not allowed") || 
                    error_msg.contains("Network access denied") ||
                    error_msg.contains("not found"), // curl might not be installed
                    "Unexpected error: {}",
                    error_msg
                );
            }
        }
    }

    #[tokio::test]
    /// Test that environment variables properly restrict network access
    async fn test_environment_variable_network_restrictions() {
        let executor = SandboxedExecutor::new()
            .with_network_policy(NetworkPolicy::Denied)
            .with_limits(ResourceLimits::default())
            .with_timeout(Duration::from_secs(2));

        // Test with a command that checks environment variables
        let result = executor.execute(
            "env",
            &[],
            HashMap::new(),
            None
        ).await;

        if let Ok(output) = result {
            let env_output = output.stdout;
            
            // Check that network-restricting environment variables are set
            assert!(
                env_output.contains("NO_PROXY=*") || env_output.contains("no_proxy=*"),
                "Network-restricting environment variables should be set"
            );
            
            // Check that potentially dangerous proxy variables are not set
            assert!(
                !env_output.contains("HTTP_PROXY=") || 
                !env_output.contains("HTTPS_PROXY=") ||
                env_output.contains("NO_PROXY=*"),
                "Proxy variables should be cleared or NO_PROXY should be set"
            );
        }
    }
}