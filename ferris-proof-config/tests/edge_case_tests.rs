use ferris_proof_config::{ConfigManager, Config, ProfileConfig, ToolConfig, FeatureConfig, Thresholds, CiConfig};
use ferris_proof_core::{VerificationLevel, EnforcementMode, Technique};
use std::path::PathBuf;
use tempfile::TempDir;
use std::fs;

#[test]
fn test_invalid_toml_syntax_handling() {
    let temp_dir = TempDir::new().unwrap();
    let project_root = temp_dir.path();
    
    // Create config with invalid TOML syntax
    let invalid_config = r#"
[profile
level = "standard"  # Missing closing bracket
enforcement = "warning"
enabled_techniques = ["TypeSafety", "PropertyTests"]
"#;
    
    fs::write(project_root.join("ferrisproof.toml"), invalid_config).unwrap();
    
    // Should fail to load with descriptive error
    let result = ConfigManager::from_project_root(project_root);
    assert!(result.is_err());
    
    let error_message = result.unwrap_err().to_string();
    assert!(error_message.contains("Failed to parse root config"));
}

#[test]
fn test_missing_required_fields() {
    let temp_dir = TempDir::new().unwrap();
    let project_root = temp_dir.path();
    
    // Create config missing required fields
    let incomplete_config = r#"
[profile]
level = "standard"
# Missing enforcement and enabled_techniques
"#;
    
    fs::write(project_root.join("ferrisproof.toml"), incomplete_config).unwrap();
    
    // Should fail to load
    let result = ConfigManager::from_project_root(project_root);
    assert!(result.is_err());
}

#[test]
fn test_invalid_verification_level() {
    let temp_dir = TempDir::new().unwrap();
    let project_root = temp_dir.path();
    
    // Create config with invalid verification level
    let invalid_config = r#"
[profile]
level = "invalid_level"
enforcement = "warning"
enabled_techniques = ["TypeSafety"]

[tools]

[modules]

[features]
cache_enabled = true
parallel_execution = true
generate_reports = true

[thresholds]
max_verification_time = 300
max_memory_usage = 2147483648
cache_ttl = 86400

[ci]
fail_on_violations = true
generate_artifacts = true
upload_reports = false
"#;
    
    fs::write(project_root.join("ferrisproof.toml"), invalid_config).unwrap();
    
    // Should fail schema validation
    let result = ConfigManager::from_project_root(project_root);
    assert!(result.is_err());
    
    let error_message = result.unwrap_err().to_string();
    assert!(error_message.contains("validation failed") || error_message.contains("invalid_level"));
}

#[test]
fn test_conflicting_glob_pattern_resolution() {
    let temp_dir = TempDir::new().unwrap();
    let project_root = temp_dir.path();
    
    // Create root config with overlapping glob patterns
    let root_config = r#"
[profile]
level = "standard"
enforcement = "warning"
enabled_techniques = ["TypeSafety", "PropertyTests"]

[tools]

[features]
cache_enabled = true
parallel_execution = true
generate_reports = true

[thresholds]
max_verification_time = 300
max_memory_usage = 2147483648
cache_ttl = 86400

[ci]
fail_on_violations = true
generate_artifacts = true
upload_reports = false

[modules]
# More specific pattern should win
"crypto::aes::*" = { level = "formal", enforcement = "error" }
"crypto::*" = { level = "strict", enforcement = "warning" }
"*" = { level = "minimal", enforcement = "advisory" }
"#;
    
    fs::write(project_root.join("ferrisproof.toml"), root_config).unwrap();
    
    // Create directory structure
    fs::create_dir_all(project_root.join("src/crypto/aes")).unwrap();
    
    let config_manager = ConfigManager::from_project_root(project_root).unwrap();
    
    // Test that most specific pattern wins
    let aes_file = project_root.join("src/crypto/aes/cipher.rs");
    let effective_config = config_manager.for_file(&aes_file);
    
    // Should use the most specific pattern (crypto::aes::*)
    assert_eq!(effective_config.level, VerificationLevel::Formal);
    assert_eq!(effective_config.enforcement, EnforcementMode::Error);
    
    // Test less specific pattern
    let crypto_file = project_root.join("src/crypto/hash.rs");
    let effective_config = config_manager.for_file(&crypto_file);
    
    // Should use crypto::* pattern
    assert_eq!(effective_config.level, VerificationLevel::Strict);
    assert_eq!(effective_config.enforcement, EnforcementMode::Warning);
}

#[test]
fn test_configuration_schema_validation_errors() {
    let temp_dir = TempDir::new().unwrap();
    let project_root = temp_dir.path();
    
    // Create config with schema violations
    let invalid_config = r#"
[profile]
level = "standard"
enforcement = "warning"
enabled_techniques = ["TypeSafety", "PropertyTests"]

[tools]

[modules]

[features]
cache_enabled = true
parallel_execution = true
generate_reports = true

[thresholds]
max_verification_time = 0  # Invalid: must be > 0
max_memory_usage = 0       # Invalid: must be > 0
cache_ttl = 0             # Invalid: must be > 0

[ci]
fail_on_violations = true
generate_artifacts = true
upload_reports = false
"#;
    
    fs::write(project_root.join("ferrisproof.toml"), invalid_config).unwrap();
    
    let config_manager = ConfigManager::from_project_root(project_root).unwrap();
    
    // Validation should fail
    let validation_result = config_manager.validate();
    assert!(validation_result.is_err());
    
    let error_message = validation_result.unwrap_err().to_string();
    assert!(error_message.contains("must be > 0"));
}

#[test]
fn test_inconsistent_level_techniques_validation() {
    let temp_dir = TempDir::new().unwrap();
    let project_root = temp_dir.path();
    
    // Create config with inconsistent level and techniques
    let inconsistent_config = r#"
[profile]
level = "minimal"
enforcement = "warning"
enabled_techniques = ["PropertyTests"]  # Minimal should have TypeSafety, not PropertyTests

[tools]

[modules]

[features]
cache_enabled = true
parallel_execution = true
generate_reports = true

[thresholds]
max_verification_time = 300
max_memory_usage = 2147483648
cache_ttl = 86400

[ci]
fail_on_violations = true
generate_artifacts = true
upload_reports = false
"#;
    
    fs::write(project_root.join("ferrisproof.toml"), inconsistent_config).unwrap();
    
    let config_manager = ConfigManager::from_project_root(project_root).unwrap();
    
    // Validation should fail
    let validation_result = config_manager.validate();
    assert!(validation_result.is_err());
    
    let error_message = validation_result.unwrap_err().to_string();
    assert!(error_message.contains("Minimal level must include TypeSafety"));
}

#[test]
fn test_malformed_glob_patterns() {
    let temp_dir = TempDir::new().unwrap();
    let project_root = temp_dir.path();
    
    // Create config with malformed glob patterns
    let malformed_config = r#"
[profile]
level = "standard"
enforcement = "warning"
enabled_techniques = ["TypeSafety", "PropertyTests"]

[tools]

[features]
cache_enabled = true
parallel_execution = true
generate_reports = true

[thresholds]
max_verification_time = 300
max_memory_usage = 2147483648
cache_ttl = 86400

[ci]
fail_on_violations = true
generate_artifacts = true
upload_reports = false

[modules]
# Invalid glob patterns
"[invalid" = { level = "formal" }
"***/invalid" = { level = "strict" }
"" = { level = "minimal" }
"#;
    
    fs::write(project_root.join("ferrisproof.toml"), malformed_config).unwrap();
    
    // Should still load (malformed patterns are ignored with warnings)
    let config_manager = ConfigManager::from_project_root(project_root).unwrap();
    
    // Test that malformed patterns don't match anything
    let test_file = project_root.join("src/test.rs");
    let effective_config = config_manager.for_file(&test_file);
    
    // Should fall back to root config
    assert_eq!(effective_config.level, VerificationLevel::Standard);
}

#[test]
fn test_circular_configuration_references() {
    let temp_dir = TempDir::new().unwrap();
    let project_root = temp_dir.path();
    
    // Create root config
    let root_config = r#"
[profile]
level = "standard"
enforcement = "warning"
enabled_techniques = ["TypeSafety", "PropertyTests"]

[tools]

[modules]

[features]
cache_enabled = true
parallel_execution = true
generate_reports = true

[thresholds]
max_verification_time = 300
max_memory_usage = 2147483648
cache_ttl = 86400

[ci]
fail_on_violations = true
generate_artifacts = true
upload_reports = false
"#;
    
    fs::write(project_root.join("ferrisproof.toml"), root_config).unwrap();
    
    // Create subdirectory configs that could create circular references
    let crypto_dir = project_root.join("src/crypto");
    fs::create_dir_all(&crypto_dir).unwrap();
    
    let crypto_config = r#"
[profile]
level = "formal"
enforcement = "error"
enabled_techniques = ["TypeSafety", "PropertyTests", "FormalSpecs"]

[modules]
"../api::*" = { level = "standard" }  # Reference to sibling directory
"#;
    
    fs::write(crypto_dir.join("ferrisproof.toml"), crypto_config).unwrap();
    
    let api_dir = project_root.join("src/api");
    fs::create_dir_all(&api_dir).unwrap();
    
    let api_config = r#"
[profile]
level = "strict"
enforcement = "warning"
enabled_techniques = ["TypeSafety", "PropertyTests", "SessionTypes"]

[modules]
"../crypto::*" = { level = "formal" }  # Reference back to crypto
"#;
    
    fs::write(api_dir.join("ferrisproof.toml"), api_config).unwrap();
    
    // Should handle circular references gracefully
    let config_manager = ConfigManager::from_project_root(project_root).unwrap();
    
    let crypto_file = project_root.join("src/crypto/cipher.rs");
    let effective_config = config_manager.for_file(&crypto_file);
    
    // Should resolve to the local config without infinite recursion
    assert_eq!(effective_config.level, VerificationLevel::Formal);
}

#[test]
fn test_empty_configuration_file() {
    let temp_dir = TempDir::new().unwrap();
    let project_root = temp_dir.path();
    
    // Create empty config file
    fs::write(project_root.join("ferrisproof.toml"), "").unwrap();
    
    // Should fail to load due to missing required fields
    let result = ConfigManager::from_project_root(project_root);
    assert!(result.is_err());
}

#[test]
fn test_configuration_with_unknown_fields() {
    let temp_dir = TempDir::new().unwrap();
    let project_root = temp_dir.path();
    
    // Create config with unknown fields
    let config_with_unknown = r#"
[profile]
level = "standard"
enforcement = "warning"
enabled_techniques = ["TypeSafety", "PropertyTests"]
unknown_field = "should_be_ignored"

[tools]
unknown_tool = { path = "/unknown" }

[modules]

[features]
cache_enabled = true
parallel_execution = true
generate_reports = true

[thresholds]
max_verification_time = 300
max_memory_usage = 2147483648
cache_ttl = 86400

[ci]
fail_on_violations = true
generate_artifacts = true
upload_reports = false

[unknown_section]
field = "value"
"#;
    
    fs::write(project_root.join("ferrisproof.toml"), config_with_unknown).unwrap();
    
    // Should fail schema validation due to additionalProperties: false
    let result = ConfigManager::from_project_root(project_root);
    assert!(result.is_err());
}

#[test]
fn test_deeply_nested_directory_structure() {
    let temp_dir = TempDir::new().unwrap();
    let project_root = temp_dir.path();
    
    // Create root config
    let root_config = r#"
[profile]
level = "minimal"
enforcement = "advisory"
enabled_techniques = ["TypeSafety"]

[tools]

[modules]

[features]
cache_enabled = true
parallel_execution = true
generate_reports = true

[thresholds]
max_verification_time = 300
max_memory_usage = 2147483648
cache_ttl = 86400

[ci]
fail_on_violations = true
generate_artifacts = true
upload_reports = false
"#;
    
    fs::write(project_root.join("ferrisproof.toml"), root_config).unwrap();
    
    // Create deeply nested structure with configs at each level
    let deep_path = project_root.join("src/level1/level2/level3/level4");
    fs::create_dir_all(&deep_path).unwrap();
    
    // Level 1 config
    let level1_config = r#"
[profile]
level = "standard"
enforcement = "warning"
enabled_techniques = ["TypeSafety", "PropertyTests"]
"#;
    fs::write(project_root.join("src/level1/ferrisproof.toml"), level1_config).unwrap();
    
    // Level 3 config (skip level 2)
    let level3_config = r#"
[profile]
level = "formal"
enforcement = "error"
enabled_techniques = ["TypeSafety", "PropertyTests", "FormalSpecs"]
"#;
    fs::write(project_root.join("src/level1/level2/level3/ferrisproof.toml"), level3_config).unwrap();
    
    let config_manager = ConfigManager::from_project_root(project_root).unwrap();
    
    // Test file at deepest level
    let deep_file = deep_path.join("deep_file.rs");
    let effective_config = config_manager.for_file(&deep_file);
    
    // Should inherit from level3 config (closest ancestor)
    assert_eq!(effective_config.level, VerificationLevel::Formal);
    assert_eq!(effective_config.enforcement, EnforcementMode::Error);
    
    // Test file at level2 (no config at this level)
    let level2_file = project_root.join("src/level1/level2/file.rs");
    let effective_config = config_manager.for_file(&level2_file);
    
    // Should inherit from level1 config
    assert_eq!(effective_config.level, VerificationLevel::Standard);
    assert_eq!(effective_config.enforcement, EnforcementMode::Warning);
}