use ferris_proof_config::attributes::parse_verification_attributes;
use ferris_proof_config::ConfigManager;
use ferris_proof_core::{EnforcementMode, Technique, VerificationLevel};
use std::fs;
use tempfile::TempDir;

#[test]
fn test_config_discovery_and_merging() {
    // Create temporary project structure
    let temp_dir = TempDir::new().unwrap();
    let project_root = temp_dir.path();

    // Create root config
    let root_config = r#"
[profile]
level = "standard"
enforcement = "warning"
enabled_techniques = ["TypeSafety", "PropertyTests"]

[tools.proptest]
cases = 1000

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

    // Create subdirectory with module override
    let crypto_dir = project_root.join("src/crypto");
    fs::create_dir_all(&crypto_dir).unwrap();

    let module_config = r#"
[profile]
level = "formal"
enforcement = "error"
enabled_techniques = ["TypeSafety", "PropertyTests", "FormalSpecs"]

[modules]
"crypto::*" = { level = "formal", enforcement = "error" }
"#;

    fs::write(crypto_dir.join("ferrisproof.toml"), module_config).unwrap();

    // Create a Rust file in crypto directory
    let rust_file = crypto_dir.join("cipher.rs");
    fs::write(&rust_file, "pub fn encrypt() {}").unwrap();

    // Load config manager
    let config_manager = ConfigManager::from_project_root(project_root).unwrap();

    // Test configuration resolution
    let effective_config = config_manager.for_file(&rust_file);

    // Should inherit formal level from module config
    assert_eq!(effective_config.level, VerificationLevel::Formal);
    assert_eq!(effective_config.enforcement, EnforcementMode::Error);
    assert!(effective_config
        .enabled_techniques
        .contains(&Technique::FormalSpecs));
}

#[test]
fn test_glob_pattern_matching() {
    let temp_dir = TempDir::new().unwrap();
    let project_root = temp_dir.path();

    // Create root config with glob patterns
    let root_config = r#"
[profile]
level = "standard"
enforcement = "warning"
enabled_techniques = ["TypeSafety", "PropertyTests"]

[modules]
"consensus::*" = { level = "formal" }
"api::public::*" = { level = "strict", enforcement = "error" }
"utils::*" = { level = "minimal" }
"#;

    fs::write(project_root.join("ferrisproof.toml"), root_config).unwrap();

    // Create directory structure
    fs::create_dir_all(project_root.join("src/consensus/raft")).unwrap();
    fs::create_dir_all(project_root.join("src/api/public")).unwrap();
    fs::create_dir_all(project_root.join("src/utils")).unwrap();
    fs::create_dir_all(project_root.join("src/core")).unwrap();

    let config_manager = ConfigManager::from_project_root(project_root).unwrap();

    // Test glob pattern matching
    let raft_file = project_root.join("src/consensus/raft/state_machine.rs");
    let effective_config = config_manager.for_file(&raft_file);
    assert_eq!(effective_config.level, VerificationLevel::Formal);

    let api_file = project_root.join("src/api/public/handler.rs");
    let effective_config = config_manager.for_file(&api_file);
    assert_eq!(effective_config.level, VerificationLevel::Strict);
    assert_eq!(effective_config.enforcement, EnforcementMode::Error);

    let utils_file = project_root.join("src/utils/helpers.rs");
    let effective_config = config_manager.for_file(&utils_file);
    assert_eq!(effective_config.level, VerificationLevel::Minimal);

    let core_file = project_root.join("src/core/types.rs");
    let effective_config = config_manager.for_file(&core_file);
    assert_eq!(effective_config.level, VerificationLevel::Standard); // Default from root
}

#[test]
fn test_verification_attribute_parsing() {
    // Test simple level attribute
    let simple_attr_code = r#"#[verification(formal)]
pub fn critical_function() {}
"#;

    let temp_file = tempfile::NamedTempFile::with_suffix(".rs").unwrap();
    fs::write(temp_file.path(), simple_attr_code).unwrap();

    let result = parse_verification_attributes(temp_file.path()).unwrap();
    assert!(result.is_some());

    let config = result.unwrap();
    assert_eq!(config.profile.level, VerificationLevel::Formal);

    // Test full specification attribute
    let full_attr_code = r#"
#[verification(
    level = "strict",
    enforcement = "error",
    techniques = [TypeSafety, SessionTypes, PropertyTests]
)]
pub fn strict_function() {}
"#;

    let temp_file2 = tempfile::NamedTempFile::with_suffix(".rs").unwrap();
    fs::write(temp_file2.path(), full_attr_code).unwrap();

    let result2 = parse_verification_attributes(temp_file2.path()).unwrap();
    assert!(result2.is_some());

    let config2 = result2.unwrap();
    assert_eq!(config2.profile.level, VerificationLevel::Strict);
    assert_eq!(config2.profile.enforcement, EnforcementMode::Error);
    assert!(config2
        .profile
        .enabled_techniques
        .contains(&Technique::SessionTypes));
    assert!(config2
        .profile
        .enabled_techniques
        .contains(&Technique::PropertyTests));
}

#[test]
fn test_configuration_validation() {
    let temp_dir = TempDir::new().unwrap();
    let project_root = temp_dir.path();

    // Test valid configuration
    let valid_config = r#"
[profile]
level = "standard"
enforcement = "warning"
enabled_techniques = ["TypeSafety", "PropertyTests"]

[tools.proptest]
cases = 1000

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

    fs::write(project_root.join("ferrisproof.toml"), valid_config).unwrap();

    let config_manager = ConfigManager::from_project_root(project_root).unwrap();
    assert!(config_manager.validate().is_ok());

    // Test invalid configuration (minimal level without TypeSafety)
    let temp_dir2 = TempDir::new().unwrap();
    let project_root2 = temp_dir2.path();

    let invalid_config = r#"
[profile]
level = "minimal"
enforcement = "warning"
# Missing TypeSafety
enabled_techniques = ["PropertyTests"]
"#;

    fs::write(project_root2.join("ferrisproof.toml"), invalid_config).unwrap();

    let config_manager2 = ConfigManager::from_project_root(project_root2).unwrap();
    assert!(config_manager2.validate().is_err());
}

#[test]
fn test_configuration_precedence() {
    let temp_dir = TempDir::new().unwrap();
    let project_root = temp_dir.path();

    // Create root config
    let root_config = r#"
[profile]
level = "standard"
enforcement = "warning"
enabled_techniques = ["TypeSafety", "PropertyTests"]
"#;

    fs::write(project_root.join("ferrisproof.toml"), root_config).unwrap();

    // Create subdirectory with module override
    let crypto_dir = project_root.join("src/crypto");
    fs::create_dir_all(&crypto_dir).unwrap();

    let module_config = r#"
[profile]
level = "formal"
enforcement = "error"
"#;

    fs::write(crypto_dir.join("ferrisproof.toml"), module_config).unwrap();

    // Create a Rust file with verification attribute
    let rust_file = crypto_dir.join("cipher.rs");
    let rust_code = r#"
#[verification(strict)]
pub fn encrypt() {}
"#;

    fs::write(&rust_file, rust_code).unwrap();

    let config_manager = ConfigManager::from_project_root(project_root).unwrap();

    // Test precedence: attribute > module config > root config
    // Even though module config sets formal level, attribute should override to strict
    let effective_config = config_manager.for_file(&rust_file);
    assert_eq!(effective_config.level, VerificationLevel::Strict); // From attribute
}

#[test]
fn test_config_caching() {
    let temp_dir = TempDir::new().unwrap();
    let project_root = temp_dir.path();

    let config_content = r#"
[profile]
level = "standard"
enforcement = "warning"
enabled_techniques = ["TypeSafety", "PropertyTests"]
"#;

    fs::write(project_root.join("ferrisproof.toml"), config_content).unwrap();

    let config_manager = ConfigManager::from_project_root(project_root).unwrap();

    // First load should read from file
    let rust_file = project_root.join("src/main.rs");
    let effective_config1 = config_manager.for_file(&rust_file);

    // Second load should use cached config (if cache is working)
    let effective_config2 = config_manager.for_file(&rust_file);

    // Both should be the same
    assert_eq!(effective_config1.level, effective_config2.level);
    assert_eq!(effective_config1.enforcement, effective_config2.enforcement);
}
