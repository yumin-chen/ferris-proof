use ferris_proof_cli::commands::init;
use ferris_proof_core::VerificationLevel;
use proptest::prelude::*;
use std::fs;
use std::sync::Mutex;
use tempfile::TempDir;

// Mutex to serialize tests that change the current working directory
static WORKING_DIR_MUTEX: Mutex<()> = Mutex::new(());

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
        let _guard = WORKING_DIR_MUTEX.lock().unwrap();

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

        // Verify the command succeeded
        prop_assert!(result.is_ok());
        prop_assert_eq!(result.unwrap(), 0);

        // Verify ferrisproof.toml was created while still in temp directory
        prop_assert!(std::path::Path::new("ferrisproof.toml").exists());

        // Parse the configuration file
        let config_content = fs::read_to_string("ferrisproof.toml").unwrap();
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

        // Verify appropriate directories were created while still in temp directory
        prop_assert!(std::path::Path::new("specs").exists());
        prop_assert!(std::path::Path::new("tests").exists());

        // Verify level-specific directories
        match level {
            VerificationLevel::Minimal => {
                // No additional directories expected
            }
            VerificationLevel::Standard => {
                prop_assert!(std::path::Path::new("tests/property").exists());
            }
            VerificationLevel::Strict => {
                prop_assert!(std::path::Path::new("tests/property").exists());
                prop_assert!(std::path::Path::new("specs/session-types").exists());
                prop_assert!(std::path::Path::new("specs/refinement-types").exists());
            }
            VerificationLevel::Formal => {
                prop_assert!(std::path::Path::new("tests/property").exists());
                prop_assert!(std::path::Path::new("specs/session-types").exists());
                prop_assert!(std::path::Path::new("specs/refinement-types").exists());
                prop_assert!(std::path::Path::new("specs/formal").exists());
                prop_assert!(std::path::Path::new("specs/formal/tla").exists());
                prop_assert!(std::path::Path::new("specs/formal/alloy").exists());
            }
        }

        // Restore original directory
        std::env::set_current_dir(original_dir).unwrap();
    }
}

#[cfg(test)]
mod unit_tests {
    use super::*;
    use tempfile::TempDir;

    #[tokio::test]
    async fn test_init_creates_config_file() {
        let _guard = WORKING_DIR_MUTEX.lock().unwrap();

        let temp_dir = TempDir::new().unwrap();
        let temp_path = temp_dir.path().to_path_buf();
        let original_dir = std::env::current_dir().unwrap();

        std::env::set_current_dir(&temp_path).unwrap();

        let result = init::run(VerificationLevel::Standard, false, None).await;

        // Check that config file was created while still in temp directory
        assert!(std::path::Path::new("ferrisproof.toml").exists());

        // Restore directory before checking result to avoid issues with temp dir cleanup
        std::env::set_current_dir(original_dir).unwrap();

        // Check result
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 0);
    }

    #[tokio::test]
    async fn test_init_creates_directories() {
        let _guard = WORKING_DIR_MUTEX.lock().unwrap();

        let temp_dir = TempDir::new().unwrap();
        let temp_path = temp_dir.path().to_path_buf();
        let original_dir = std::env::current_dir().unwrap();

        std::env::set_current_dir(&temp_path).unwrap();

        let result = init::run(VerificationLevel::Formal, false, None).await;

        // Check that all expected directories exist while still in temp directory
        assert!(std::path::Path::new("specs").exists());
        assert!(std::path::Path::new("tests").exists());
        assert!(std::path::Path::new("tests/property").exists());
        assert!(std::path::Path::new("specs/formal").exists());
        assert!(std::path::Path::new("specs/formal/tla").exists());
        assert!(std::path::Path::new("specs/formal/alloy").exists());

        // Restore directory before checking result to avoid issues with temp dir cleanup
        std::env::set_current_dir(original_dir).unwrap();

        // Check result
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 0);
    }
}
