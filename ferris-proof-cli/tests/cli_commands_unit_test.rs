use ferris_proof_cli::commands::{config, explain, init};
use ferris_proof_core::VerificationLevel;
use std::fs;
use tempfile::TempDir;
use tokio;

#[cfg(test)]
mod init_command_tests {
    use super::*;

    #[tokio::test]
    async fn test_init_command_creates_config_file() {
        let temp_dir = TempDir::new().unwrap();
        let temp_path = temp_dir.path().to_path_buf();
        let original_dir = std::env::current_dir().unwrap();

        // Small delay to avoid race conditions
        tokio::time::sleep(std::time::Duration::from_millis(10)).await;

        std::env::set_current_dir(&temp_path).unwrap();

        // Verify we're actually in the new directory
        let current_dir = std::env::current_dir().unwrap();
        assert_eq!(
            current_dir, temp_path,
            "Failed to change to temporary directory"
        );

        let result = init::run(VerificationLevel::Standard, false, None).await;

        // Check that config file was created while still in temp directory
        assert!(
            std::path::Path::new("ferrisproof.toml").exists(),
            "Config file not found in: {:?}",
            temp_path
        );

        let config_content = fs::read_to_string("ferrisproof.toml").unwrap();
        assert!(config_content.contains("level = \"standard\""));

        // Restore directory before returning result
        std::env::set_current_dir(&original_dir).unwrap();

        // Check result
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 0);
    }

    #[tokio::test]
    async fn test_init_command_creates_directories_for_minimal_level() {
        let temp_dir = TempDir::new().unwrap();
        let temp_path = temp_dir.path().to_path_buf();
        let original_dir = std::env::current_dir().unwrap();

        std::env::set_current_dir(&temp_path).unwrap();

        let result = init::run(VerificationLevel::Minimal, false, None).await;

        // Check basic directories exist while still in temp directory
        assert!(std::path::Path::new("specs").exists());
        assert!(std::path::Path::new("tests").exists());

        // Minimal level should not create additional directories
        assert!(!std::path::Path::new("tests/property").exists());

        std::env::set_current_dir(&original_dir).unwrap();

        // Check result
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 0);
    }

    #[tokio::test]
    async fn test_init_command_creates_directories_for_standard_level() {
        let temp_dir = TempDir::new().unwrap();
        let temp_path = temp_dir.path().to_path_buf();
        let original_dir = std::env::current_dir().unwrap();

        std::env::set_current_dir(&temp_path).unwrap();

        let result = init::run(VerificationLevel::Standard, false, None).await;

        // Check basic directories exist while still in temp directory
        assert!(std::path::Path::new("specs").exists());
        assert!(std::path::Path::new("tests").exists());

        // Standard level should create property test directory
        assert!(std::path::Path::new("tests/property").exists());

        std::env::set_current_dir(&original_dir).unwrap();

        // Check result
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 0);
    }

    #[tokio::test]
    async fn test_init_command_creates_directories_for_formal_level() {
        let temp_dir = TempDir::new().unwrap();
        let temp_path = temp_dir.path().to_path_buf();
        let original_dir = std::env::current_dir().unwrap();

        std::env::set_current_dir(&temp_path).unwrap();

        let result = init::run(VerificationLevel::Formal, false, None).await;

        // Check all directories for formal level exist while still in temp directory
        assert!(std::path::Path::new("specs").exists());
        assert!(std::path::Path::new("tests").exists());
        assert!(std::path::Path::new("tests/property").exists());
        assert!(std::path::Path::new("specs/session-types").exists());
        assert!(std::path::Path::new("specs/refinement-types").exists());
        assert!(std::path::Path::new("specs/formal").exists());
        assert!(std::path::Path::new("specs/formal/tla").exists());
        assert!(std::path::Path::new("specs/formal/alloy").exists());

        std::env::set_current_dir(&original_dir).unwrap();

        // Check result
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 0);
    }

    #[tokio::test]
    async fn test_init_command_with_template() {
        let temp_dir = TempDir::new().unwrap();
        let temp_path = temp_dir.path().to_path_buf();
        let original_dir = std::env::current_dir().unwrap();

        std::env::set_current_dir(&temp_path).unwrap();

        let result = init::run(
            VerificationLevel::Standard,
            false,
            Some("standard".to_string()),
        )
        .await;

        // Check that config file was created while still in temp directory
        assert!(std::path::Path::new("ferrisproof.toml").exists());

        // Check that template files were created while still in temp directory
        assert!(std::path::Path::new("README.md").exists());
        assert!(std::path::Path::new("tests/property/example_properties.rs").exists());

        std::env::set_current_dir(&original_dir).unwrap();

        // Check result
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 0);
    }
}

#[cfg(test)]
mod config_command_tests {
    use super::*;

    #[tokio::test]
    async fn test_config_command_without_ferrisproof_toml() {
        let temp_dir = TempDir::new().unwrap();
        let original_dir = std::env::current_dir().unwrap();

        std::env::set_current_dir(temp_dir.path()).unwrap();

        let result = config::run(None, false).await;

        std::env::set_current_dir(&original_dir).unwrap();

        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 1); // Should return 1 when no config found
    }

    #[tokio::test]
    async fn test_config_command_with_valid_config() {
        let temp_dir = TempDir::new().unwrap();
        let temp_path = temp_dir.path().to_path_buf();
        let original_dir = std::env::current_dir().unwrap();

        std::env::set_current_dir(&temp_path).unwrap();

        // First create a config file
        let init_result = init::run(VerificationLevel::Standard, false, None).await;
        assert!(init_result.is_ok());
        assert_eq!(init_result.unwrap(), 0);

        // Then test the config command
        let result = config::run(None, false).await;

        std::env::set_current_dir(&original_dir).unwrap();

        // Check result
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 0);
    }

    #[tokio::test]
    async fn test_config_validation_with_valid_config() {
        let temp_dir = TempDir::new().unwrap();
        let temp_path = temp_dir.path().to_path_buf();
        let original_dir = std::env::current_dir().unwrap();

        std::env::set_current_dir(&temp_path).unwrap();

        // First create a config file
        let init_result = init::run(VerificationLevel::Standard, false, None).await;
        assert!(init_result.is_ok());
        assert_eq!(init_result.unwrap(), 0);

        // Then test validation
        let result = config::run(None, true).await;

        std::env::set_current_dir(&original_dir).unwrap();

        // Check result
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 0); // Should pass validation
    }

    #[tokio::test]
    async fn test_config_validation_without_config() {
        // Don't change directory, just test that config validation works without a config file
        // This avoids issues with temp directories in CI environments
        let result = config::run(None, true).await;

        // Check result - validation without config should still work (uses default config)
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 0);
    }

    #[tokio::test]
    async fn test_config_command_for_specific_file() {
        let temp_dir = TempDir::new().unwrap();
        let temp_path = temp_dir.path().to_path_buf();
        let original_dir = std::env::current_dir().unwrap();

        std::env::set_current_dir(&temp_path).unwrap();

        // Create a config file
        let init_result = init::run(VerificationLevel::Standard, false, None).await;

        // Create a test file
        let test_file = temp_path.join("src").join("main.rs");
        fs::create_dir_all(test_file.parent().unwrap()).unwrap();
        fs::write(&test_file, "fn main() {}").unwrap();

        // Test config for specific file
        let result = config::run(Some(test_file), false).await;

        std::env::set_current_dir(&original_dir).unwrap();

        // Check results
        assert!(init_result.is_ok());
        assert_eq!(init_result.unwrap(), 0);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 0);
    }
}

#[cfg(test)]
mod explain_command_tests {
    use super::*;

    // These don't require changing directories, so no additional setup needed
    #[tokio::test]
    async fn test_explain_command_with_known_error_code() {
        let result = explain::run("FP-CF-001".to_string()).await;

        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 0);
    }

    #[tokio::test]
    async fn test_explain_command_with_unknown_error_code() {
        let result = explain::run("FP-XX-999".to_string()).await;

        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 1); // Should return 1 for unknown codes
    }

    #[tokio::test]
    async fn test_explain_command_with_configuration_error() {
        let result = explain::run("FP-CF-002".to_string()).await;

        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 0);
    }

    #[tokio::test]
    async fn test_explain_command_with_verification_error() {
        let result = explain::run("FP-VR-001".to_string()).await;

        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 0);
    }

    #[tokio::test]
    async fn test_explain_command_with_tool_error() {
        let result = explain::run("FP-TL-001".to_string()).await;

        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 0);
    }

    #[tokio::test]
    async fn test_explain_command_with_io_error() {
        let result = explain::run("FP-IO-001".to_string()).await;

        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 0);
    }

    #[tokio::test]
    async fn test_explain_command_with_empty_code() {
        let result = explain::run("".to_string()).await;

        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 1); // Should return 1 for empty/invalid codes
    }

    #[tokio::test]
    async fn test_explain_command_with_partial_code() {
        let result = explain::run("FP-CF".to_string()).await;

        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 1); // Should return 1 for partial codes
    }
}

#[cfg(test)]
mod integration_tests {
    use super::*;

    #[tokio::test]
    async fn test_init_then_config_workflow() {
        let temp_dir = TempDir::new().unwrap();
        let temp_path = temp_dir.path().to_path_buf();
        let original_dir = std::env::current_dir().unwrap();

        std::env::set_current_dir(&temp_path).unwrap();

        // Initialize project
        let init_result = init::run(VerificationLevel::Strict, false, None).await;
        assert!(init_result.is_ok());
        assert_eq!(init_result.unwrap(), 0);

        // Check configuration
        let config_result = config::run(None, false).await;
        assert!(config_result.is_ok());
        assert_eq!(config_result.unwrap(), 0);

        // Validate configuration
        let validate_result = config::run(None, true).await;
        assert!(validate_result.is_ok());
        assert_eq!(validate_result.unwrap(), 0);

        std::env::set_current_dir(&original_dir).unwrap();
    }

    #[tokio::test]
    async fn test_different_verification_levels_create_correct_configs() {
        let levels = vec![
            VerificationLevel::Minimal,
            VerificationLevel::Standard,
            VerificationLevel::Strict,
            VerificationLevel::Formal,
        ];

        for level in levels {
            let temp_dir = TempDir::new().unwrap();
            let temp_path = temp_dir.path().to_path_buf();
            let original_dir = std::env::current_dir().unwrap();

            std::env::set_current_dir(&temp_path).unwrap();

            let result = init::run(level, false, None).await;

            // Verify config contains correct level while still in temp directory
            assert!(std::path::Path::new("ferrisproof.toml").exists());

            let config_content = fs::read_to_string("ferrisproof.toml").unwrap();
            let level_str = format!("{:?}", level).to_lowercase();
            assert!(config_content.contains(&format!("level = \"{}\"", level_str)));

            std::env::set_current_dir(&original_dir).unwrap();

            // Check result
            assert!(result.is_ok());
            assert_eq!(result.unwrap(), 0);
        }
    }
}
