use ferris_proof_cli::commands::{init, config, explain};
use ferris_proof_core::VerificationLevel;
use tempfile::TempDir;
use std::fs;

#[cfg(test)]
mod init_command_tests {
    use super::*;

    #[tokio::test]
    async fn test_init_command_creates_config_file() {
        let temp_dir = TempDir::new().unwrap();
        let temp_path = temp_dir.path().to_path_buf();
        let original_dir = std::env::current_dir().unwrap();
        
        std::env::set_current_dir(&temp_path).unwrap();
        
        let result = init::run(VerificationLevel::Standard, false, None).await;
        
        // Check result while still in temp directory
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 0);
        
        // Check that config file was created
        let config_path = temp_path.join("ferrisproof.toml");
        assert!(config_path.exists());
        
        let config_content = fs::read_to_string(&config_path).unwrap();
        assert!(config_content.contains("level = \"standard\""));
        
        // Restore directory
        std::env::set_current_dir(original_dir).unwrap();
    }

    #[tokio::test]
    async fn test_init_command_creates_directories_for_minimal_level() {
        let temp_dir = TempDir::new().unwrap();
        let temp_path = temp_dir.path().to_path_buf();
        let original_dir = std::env::current_dir().unwrap();
        
        std::env::set_current_dir(&temp_path).unwrap();
        
        let result = init::run(VerificationLevel::Minimal, false, None).await;
        
        // Check result while still in temp directory
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 0);
        
        // Check basic directories exist
        assert!(temp_path.join("specs").exists());
        assert!(temp_path.join("tests").exists());
        
        // Minimal level should not create additional directories
        assert!(!temp_path.join("tests/property").exists());
        
        // Restore directory
        std::env::set_current_dir(original_dir).unwrap();
    }

    #[tokio::test]
    async fn test_init_command_creates_directories_for_standard_level() {
        let temp_dir = TempDir::new().unwrap();
        let temp_path = temp_dir.path().to_path_buf();
        let original_dir = std::env::current_dir().unwrap();
        
        std::env::set_current_dir(&temp_path).unwrap();
        
        let result = init::run(VerificationLevel::Standard, false, None).await;
        
        // Check result while still in temp directory
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 0);
        
        // Check basic directories exist
        assert!(temp_path.join("specs").exists());
        assert!(temp_path.join("tests").exists());
        
        // Standard level should create property test directory
        assert!(temp_path.join("tests/property").exists());
        
        // Restore directory
        std::env::set_current_dir(original_dir).unwrap();
    }

    #[tokio::test]
    async fn test_init_command_creates_directories_for_formal_level() {
        let temp_dir = TempDir::new().unwrap();
        let temp_path = temp_dir.path().to_path_buf();
        let original_dir = std::env::current_dir().unwrap();
        
        std::env::set_current_dir(&temp_path).unwrap();
        
        let result = init::run(VerificationLevel::Formal, false, None).await;
        
        // Check result while still in temp directory
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 0);
        
        // Check all directories for formal level exist
        assert!(temp_path.join("specs").exists());
        assert!(temp_path.join("tests").exists());
        assert!(temp_path.join("tests/property").exists());
        assert!(temp_path.join("specs/session-types").exists());
        assert!(temp_path.join("specs/refinement-types").exists());
        assert!(temp_path.join("specs/formal").exists());
        assert!(temp_path.join("specs/formal/tla").exists());
        assert!(temp_path.join("specs/formal/alloy").exists());
        
        // Restore directory
        std::env::set_current_dir(original_dir).unwrap();
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
            Some("standard".to_string())
        ).await;
        
        // Check result while still in temp directory
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 0);
        
        // Check that config file was created
        let config_path = temp_path.join("ferrisproof.toml");
        assert!(config_path.exists());
        
        // Check that template files were created
        let readme_path = temp_path.join("README.md");
        assert!(readme_path.exists());
        
        let property_test_path = temp_path.join("tests/property/example_properties.rs");
        assert!(property_test_path.exists());
        
        // Restore directory
        std::env::set_current_dir(original_dir).unwrap();
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
        
        std::env::set_current_dir(original_dir).unwrap();
        
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
        
        // Check result while still in temp directory
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 0);
        
        // Restore directory
        std::env::set_current_dir(original_dir).unwrap();
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
        
        // Check result while still in temp directory
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 0); // Should pass validation
        
        // Restore directory
        std::env::set_current_dir(original_dir).unwrap();
    }

    #[tokio::test]
    async fn test_config_validation_without_config() {
        let temp_dir = TempDir::new().unwrap();
        let temp_path = temp_dir.path().to_path_buf();
        let original_dir = std::env::current_dir().unwrap();
        
        std::env::set_current_dir(&temp_path).unwrap();
        
        let result = config::run(None, true).await;
        
        // Check result while still in temp directory
        assert!(result.is_ok());
        // Validation without config should still work (uses default config)
        assert_eq!(result.unwrap(), 0);
        
        // Restore directory
        std::env::set_current_dir(original_dir).unwrap();
    }

    #[tokio::test]
    async fn test_config_command_for_specific_file() {
        let temp_dir = TempDir::new().unwrap();
        let temp_path = temp_dir.path().to_path_buf();
        let original_dir = std::env::current_dir().unwrap();
        
        std::env::set_current_dir(&temp_path).unwrap();
        
        // Create a config file
        let init_result = init::run(VerificationLevel::Standard, false, None).await;
        assert!(init_result.is_ok());
        assert_eq!(init_result.unwrap(), 0);
        
        // Create a test file
        let test_file = temp_path.join("src").join("main.rs");
        fs::create_dir_all(test_file.parent().unwrap()).unwrap();
        fs::write(&test_file, "fn main() {}").unwrap();
        
        // Test config for specific file
        let result = config::run(Some(test_file), false).await;
        
        // Check result while still in temp directory
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 0);
        
        // Restore directory
        std::env::set_current_dir(original_dir).unwrap();
    }
}

#[cfg(test)]
mod explain_command_tests {
    use super::*;

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
        
        // Restore directory
        std::env::set_current_dir(original_dir).unwrap();
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
            
            // Check result while still in temp directory
            assert!(result.is_ok());
            assert_eq!(result.unwrap(), 0);
            
            // Verify config contains correct level
            let config_path = temp_path.join("ferrisproof.toml");
            assert!(config_path.exists());
            
            let config_content = fs::read_to_string(&config_path).unwrap();
            let level_str = format!("{:?}", level).to_lowercase();
            assert!(config_content.contains(&format!("level = \"{}\"", level_str)));
            
            // Restore directory
            std::env::set_current_dir(&original_dir).unwrap();
        }
    }
}