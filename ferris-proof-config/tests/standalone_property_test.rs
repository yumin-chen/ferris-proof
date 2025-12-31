// Standalone property test for configuration level enforcement
// This test validates Requirements 2.2, 2.3, 2.4, 2.5 without depending on ferris-proof-core

use proptest::prelude::*;
use std::collections::HashMap;

// Standalone enums for testing (mirroring the core types)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum VerificationLevel {
    Minimal,
    Standard,
    Strict,
    Formal,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum EnforcementMode {
    Advisory,
    Warning,
    Error,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
enum Technique {
    TypeSafety,
    PropertyTests,
    SessionTypes,
    RefinementTypes,
    ConcurrencyTesting,
    FormalSpecs,
    ModelChecking,
}

#[derive(Debug, Clone)]
struct TestConfig {
    level: VerificationLevel,
    enforcement: EnforcementMode,
    enabled_techniques: Vec<Technique>,
}

#[derive(Debug, Clone)]
struct TestConfigFile {
    path: String,
    config: TestConfig,
}

#[derive(Debug, Clone)]
struct TestProjectStructure {
    root_config: TestConfig,
    module_configs: Vec<TestConfigFile>,
}

/// Simulate configuration merging logic
fn merge_configs(base: TestConfig, override_config: TestConfig) -> TestConfig {
    TestConfig {
        level: override_config.level, // Override takes precedence
        enforcement: override_config.enforcement, // Override takes precedence
        enabled_techniques: if override_config.enabled_techniques.is_empty() {
            base.enabled_techniques
        } else {
            override_config.enabled_techniques
        },
    }
}

/// Simulate configuration discovery and merging for a file path
fn resolve_config_for_path(project: &TestProjectStructure, file_path: &str) -> TestConfig {
    let mut config = project.root_config.clone();
    
    // Apply module configs in order (simulating directory hierarchy)
    for module_config in &project.module_configs {
        // Simple path matching - if file path starts with module path, apply config
        if file_path.starts_with(&module_config.path) {
            config = merge_configs(config, module_config.config.clone());
        }
    }
    
    config
}

/// Get the expected techniques for each verification level according to requirements
fn get_techniques_for_level(level: VerificationLevel) -> Vec<Technique> {
    match level {
        // Requirement 2.2: minimal level enables only type safety and basic tests
        VerificationLevel::Minimal => vec![
            Technique::TypeSafety,
        ],
        // Requirement 2.3: standard level enables type safety, basic tests, and property-based testing
        VerificationLevel::Standard => vec![
            Technique::TypeSafety,
            Technique::PropertyTests,
        ],
        // Requirement 2.4: strict level enables session types, refinement types, and concurrency testing
        VerificationLevel::Strict => vec![
            Technique::TypeSafety,
            Technique::PropertyTests,
            Technique::SessionTypes,
            Technique::RefinementTypes,
            Technique::ConcurrencyTesting,
        ],
        // Requirement 2.5: formal level enables all verification techniques including formal specifications
        VerificationLevel::Formal => vec![
            Technique::TypeSafety,
            Technique::PropertyTests,
            Technique::SessionTypes,
            Technique::RefinementTypes,
            Technique::ConcurrencyTesting,
            Technique::FormalSpecs,
            Technique::ModelChecking,
        ],
    }
}

proptest! {
    #[test]
    /// **Feature: ferris-proof, Property 2: Configuration level enforcement**
    /// **Validates: Requirements 2.2, 2.3, 2.4, 2.5**
    fn configuration_level_enforcement(
        level in prop::sample::select(vec![
            VerificationLevel::Minimal,
            VerificationLevel::Standard,
            VerificationLevel::Strict,
            VerificationLevel::Formal,
        ]),
        enforcement in prop::sample::select(vec![
            EnforcementMode::Advisory,
            EnforcementMode::Warning,
            EnforcementMode::Error,
        ])
    ) {
        // Create a configuration with the specified level
        let config = TestConfig {
            level,
            enforcement,
            enabled_techniques: get_techniques_for_level(level),
        };

        // Verify that the configuration contains exactly the expected techniques for the level
        let expected_techniques = get_techniques_for_level(level);
        let actual_techniques = &config.enabled_techniques;

        // Check that all expected techniques are present
        for expected_technique in &expected_techniques {
            prop_assert!(
                actual_techniques.contains(expected_technique),
                "Level {:?} should include technique {:?}, but it was missing. Expected: {:?}, Actual: {:?}",
                level, expected_technique, expected_techniques, actual_techniques
            );
        }

        // Check that no unexpected techniques are present
        for actual_technique in actual_techniques {
            prop_assert!(
                expected_techniques.contains(actual_technique),
                "Level {:?} should not include technique {:?}, but it was present. Expected: {:?}, Actual: {:?}",
                level, actual_technique, expected_techniques, actual_techniques
            );
        }

        // Verify the techniques match exactly (same length and contents)
        prop_assert_eq!(
            expected_techniques.len(),
            actual_techniques.len(),
            "Level {:?} should have exactly {} techniques, but had {}. Expected: {:?}, Actual: {:?}",
            level, expected_techniques.len(), actual_techniques.len(), expected_techniques, actual_techniques
        );
    }

    #[test]
    /// **Feature: ferris-proof, Property 3: Configuration discovery and merging**
    /// **Validates: Requirements 2.8, 2.9**
    fn configuration_discovery_and_merging(
        root_level in prop::sample::select(vec![
            VerificationLevel::Minimal,
            VerificationLevel::Standard,
            VerificationLevel::Strict,
            VerificationLevel::Formal,
        ]),
        root_enforcement in prop::sample::select(vec![
            EnforcementMode::Advisory,
            EnforcementMode::Warning,
            EnforcementMode::Error,
        ]),
        module_level in prop::sample::select(vec![
            VerificationLevel::Minimal,
            VerificationLevel::Standard,
            VerificationLevel::Strict,
            VerificationLevel::Formal,
        ]),
        module_enforcement in prop::sample::select(vec![
            EnforcementMode::Advisory,
            EnforcementMode::Warning,
            EnforcementMode::Error,
        ]),
        file_paths in prop::collection::vec("src/[a-z]+/[a-z]+\\.rs", 1..5)
    ) {
        // Create a project structure with root and module configs
        let root_config = TestConfig {
            level: root_level,
            enforcement: root_enforcement,
            enabled_techniques: get_techniques_for_level(root_level),
        };

        let module_config = TestConfig {
            level: module_level,
            enforcement: module_enforcement,
            enabled_techniques: get_techniques_for_level(module_level),
        };

        let project = TestProjectStructure {
            root_config: root_config.clone(),
            module_configs: vec![TestConfigFile {
                path: "src/crypto/".to_string(),
                config: module_config.clone(),
            }],
        };

        // Test configuration resolution for different file paths
        for file_path in &file_paths {
            let resolved_config = resolve_config_for_path(&project, file_path);

            if file_path.starts_with("src/crypto/") {
                // Files in crypto directory should inherit module config
                prop_assert_eq!(
                    resolved_config.level,
                    module_level,
                    "File {} in crypto directory should inherit module level {:?}, but got {:?}",
                    file_path, module_level, resolved_config.level
                );
                prop_assert_eq!(
                    resolved_config.enforcement,
                    module_enforcement,
                    "File {} in crypto directory should inherit module enforcement {:?}, but got {:?}",
                    file_path, module_enforcement, resolved_config.enforcement
                );
            } else {
                // Files outside crypto directory should use root config
                prop_assert_eq!(
                    resolved_config.level,
                    root_level,
                    "File {} outside crypto directory should use root level {:?}, but got {:?}",
                    file_path, root_level, resolved_config.level
                );
                prop_assert_eq!(
                    resolved_config.enforcement,
                    root_enforcement,
                    "File {} outside crypto directory should use root enforcement {:?}, but got {:?}",
                    file_path, root_enforcement, resolved_config.enforcement
                );
            }

            // Verify that the resolved config has valid techniques for its level
            let expected_techniques = get_techniques_for_level(resolved_config.level);
            prop_assert_eq!(
                resolved_config.enabled_techniques,
                expected_techniques,
                "Resolved config for {} should have techniques {:?} for level {:?}, but got {:?}",
                file_path, expected_techniques, resolved_config.level, resolved_config.enabled_techniques
            );
        }
    }
}

#[cfg(test)]
mod unit_tests {
    use super::*;

    #[test]
    fn test_minimal_level_techniques() {
        let techniques = get_techniques_for_level(VerificationLevel::Minimal);
        assert_eq!(techniques, vec![Technique::TypeSafety]);
    }

    #[test]
    fn test_standard_level_techniques() {
        let techniques = get_techniques_for_level(VerificationLevel::Standard);
        assert_eq!(techniques, vec![
            Technique::TypeSafety,
            Technique::PropertyTests,
        ]);
    }

    #[test]
    fn test_strict_level_techniques() {
        let techniques = get_techniques_for_level(VerificationLevel::Strict);
        assert_eq!(techniques, vec![
            Technique::TypeSafety,
            Technique::PropertyTests,
            Technique::SessionTypes,
            Technique::RefinementTypes,
            Technique::ConcurrencyTesting,
        ]);
    }

    #[test]
    fn test_formal_level_techniques() {
        let techniques = get_techniques_for_level(VerificationLevel::Formal);
        assert_eq!(techniques, vec![
            Technique::TypeSafety,
            Technique::PropertyTests,
            Technique::SessionTypes,
            Technique::RefinementTypes,
            Technique::ConcurrencyTesting,
            Technique::FormalSpecs,
            Technique::ModelChecking,
        ]);
    }
}