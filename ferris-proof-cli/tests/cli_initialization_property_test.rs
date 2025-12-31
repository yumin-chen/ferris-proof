use proptest::prelude::*;
use tempfile::TempDir;
use std::fs;
use std::path::Path;
use ferris_proof_core::VerificationLevel;
use ferris_proof_cli::commands::init;

proptest! {
    #[test]
    /// **Feature: ferris-proof, Property 6: CLI verification level initialization**
    /// **Validates: Requirements 6.1**
    fn cli_verification_level_initialization(
        level in prop::sample::select(vec![
            VerificationLevel::Minimal,
            VerificationLevel::Standard,
            VerificationLevel::Strict,
            VerificationLevel::Formal,
        ])
    ) {
        // Create a temporary directory for the test
        let temp_dir = TempDir::new().unwrap();
        let original_dir = std::env::current_dir().unwrap();
        
        // Change to the temporary directory
        std::env::set_current_dir(temp_dir.path()).unwrap();
        
        // Run the init command
        let rt = tokio::runtime::Runtime::new().unwrap();
        let result = rt.block_on(async {
            init::run(level, false, None).await
        });
        
        // Restore original directory
        std::env::set_current_dir(original_dir).unwrap();
        
        // Verify the command succeeded
        prop_assert!(result.is_ok());
        prop_assert_eq!(result.unwrap(), 0);
        
        // Verify ferrisproof.toml was created
        let config_path = temp_dir.path().join("ferrisproof.toml");
        prop_assert!(config_path.exists());
        
        // Parse the configuration file
        let config_content = fs::read_to_string(&config_path).unwrap();
        let config: ferris_proof_config::Config = toml::from_str(&config_content).unwrap();
        
        // Verify the configuration has the correct verification level
        prop_assert_eq!(config.profile.level, level);
        
        // Verify the enabled techniques match the verification level
        let expected_techniques = match level {
            VerificationLevel::Minimal => vec![
                ferris_proof_core::Technique::TypeSafety
            ],
            VerificationLevel::Standard => vec![
                ferris_proof_core::Technique::TypeSafety,
                ferris_proof_core::Technique::PropertyTests,
            ],
            VerificationLevel::Strict => vec![
                ferris_proof_core::Technique::TypeSafety,
                ferris_proof_core::Technique::PropertyTests,
                ferris_proof_core::Technique::SessionTypes,
                ferris_proof_core::Technique::RefinementTypes,
                ferris_proof_core::Technique::ConcurrencyTesting,
            ],
            VerificationLevel::Formal => vec![
                ferris_proof_core::Technique::TypeSafety,
                ferris_proof_core::Technique::PropertyTests,
                ferris_proof_core::Technique::SessionTypes,
                ferris_proof_core::Technique::RefinementTypes,
                ferris_proof_core::Technique::ConcurrencyTesting,
                ferris_proof_core::Technique::FormalSpecs,
                ferris_proof_core::Technique::ModelChecking,
            ],
        };
        
        prop_assert_eq!(config.profile.enabled_techniques, expected_techniques);
        
        // Verify appropriate directories were created
        let specs_dir = temp_dir.path().join("specs");
        let tests_dir = temp_dir.path().join("tests");
        
        prop_assert!(specs_dir.exists());
        prop_assert!(tests_dir.exists());
        
        // Verify level-specific directories
        match level {
            VerificationLevel::Minimal => {
                // No additional directories expected
            }
            VerificationLevel::Standard => {
                let property_tests_dir = temp_dir.path().join("tests/property");
                prop_assert!(property_tests_dir.exists());
            }
            VerificationLevel::Strict => {
                let property_tests_dir = temp_dir.path().join("tests/property");
                let session_types_dir = temp_dir.path().join("specs/session-types");
                let refinement_types_dir = temp_dir.path().join("specs/refinement-types");
                
                prop_assert!(property_tests_dir.exists());
                prop_assert!(session_types_dir.exists());
                prop_assert!(refinement_types_dir.exists());
            }
            VerificationLevel::Formal => {
                let property_tests_dir = temp_dir.path().join("tests/property");
                let session_types_dir = temp_dir.path().join("specs/session-types");
                let refinement_types_dir = temp_dir.path().join("specs/refinement-types");
                let formal_dir = temp_dir.path().join("specs/formal");
                let tla_dir = temp_dir.path().join("specs/formal/tla");
                let alloy_dir = temp_dir.path().join("specs/formal/alloy");
                
                prop_assert!(property_tests_dir.exists());
                prop_assert!(session_types_dir.exists());
                prop_assert!(refinement_types_dir.exists());
                prop_assert!(formal_dir.exists());
                prop_assert!(tla_dir.exists());
                prop_assert!(alloy_dir.exists());
            }
        }
    }
}

#[cfg(test)]
mod unit_tests {
    use super::*;
    use tempfile::TempDir;
    
    #[tokio::test]
    async fn test_init_creates_config_file() {
        let temp_dir = TempDir::new().unwrap();
        let original_dir = std::env::current_dir().unwrap();
        
        std::env::set_current_dir(temp_dir.path()).unwrap();
        
        let result = init::run(VerificationLevel::Standard, false, None).await;
        
        std::env::set_current_dir(original_dir).unwrap();
        
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 0);
        
        let config_path = temp_dir.path().join("ferrisproof.toml");
        assert!(config_path.exists());
    }
    
    #[tokio::test]
    async fn test_init_creates_directories() {
        let temp_dir = TempDir::new().unwrap();
        let original_dir = std::env::current_dir().unwrap();
        
        std::env::set_current_dir(temp_dir.path()).unwrap();
        
        let result = init::run(VerificationLevel::Formal, false, None).await;
        
        std::env::set_current_dir(original_dir).unwrap();
        
        assert!(result.is_ok());
        
        // Check that all expected directories exist
        assert!(temp_dir.path().join("specs").exists());
        assert!(temp_dir.path().join("tests").exists());
        assert!(temp_dir.path().join("tests/property").exists());
        assert!(temp_dir.path().join("specs/formal").exists());
        assert!(temp_dir.path().join("specs/formal/tla").exists());
        assert!(temp_dir.path().join("specs/formal/alloy").exists());
    }
}