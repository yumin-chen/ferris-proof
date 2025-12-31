use ferris_proof_cli::commands::init;
use ferris_proof_core::models::config::VerificationLevel;
use proptest::prelude::*;
use std::fs;
use std::path::PathBuf;
use tempfile::{tempdir, TempDir};
use tokio::sync::Mutex;

lazy_static::lazy_static! {
    // Use a tokio Mutex to make it async-aware
    static ref WORKING_DIR_MUTEX: Mutex<()> = Mutex::new(());
}

/// Represents the outcome of an `init` command execution for property testing.
#[derive(Debug, Clone)]
struct InitCommandOutcome {
    level: VerificationLevel,
    created_config: bool,
    created_specs_dir: bool,
    created_tests_dir: bool,
    config_content_valid: bool,
    temp_dir: Option<PathBuf>, // Keep track of the temp dir path for cleanup
}

/// Strategy to generate different verification levels for testing.
fn verification_level_strategy() -> impl Strategy<Value = VerificationLevel> {
    prop_oneof![
        Just(VerificationLevel::Minimal),
        Just(VerificationLevel::Standard),
        Just(VerificationLevel::Strict),
        Just(VerificationLevel::Formal),
    ]
}

/// Runs the `init` command in a temporary directory and captures the outcome.
async fn run_init_and_capture_outcome(level: VerificationLevel) -> (InitCommandOutcome, TempDir) {
    let dir = tempdir().unwrap();
    let project_path = dir.path().to_path_buf();

    // Set current directory for the test
    std::env::set_current_dir(&project_path).unwrap();

    let result = init::run(level.clone(), false, None).await;
    assert!(result.is_ok());

    let config_path = project_path.join("ferrisproof.toml");
    let created_config = config_path.exists();
    let created_specs_dir = project_path.join("specs").is_dir();
    let created_tests_dir = project_path.join("tests").is_dir();

    let config_content_valid = if created_config {
        let content = fs::read_to_string(&config_path).unwrap();
        content.contains(&format!("level = \"{}\"", level.to_string().to_lowercase()))
    } else {
        false
    };

    (
        InitCommandOutcome {
            level,
            created_config,
            created_specs_dir,
            created_tests_dir,
            config_content_valid,
            temp_dir: Some(project_path),
        },
        dir, // Return the TempDir to keep it in scope
    )
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(10))]

    /// Property: The `init` command should always create the configuration file and standard directories.
    #[test]
    fn prop_init_creates_required_files(level in verification_level_strategy()) {
        tokio::runtime::Runtime::new().unwrap().block_on(async {
            // Lock the async mutex
            let _guard = WORKING_DIR_MUTEX.lock().await;

            let (outcome, _dir) = run_init_and_capture_outcome(level).await;

            prop_assert!(outcome.created_config, "ferrisproof.toml should be created");
            prop_assert!(outcome.created_specs_dir, "specs directory should be created");
            prop_assert!(outcome.created_tests_dir, "tests directory should be created");
            prop_assert!(outcome.config_content_valid, "ferrisproof.toml should have the correct level");
        });
    }

    /// Property: The generated `ferrisproof.toml` should always contain the correct verification level.
    #[test]
    fn prop_init_config_contains_correct_level(level in verification_level_strategy()) {
        tokio::runtime::Runtime::new().unwrap().block_on(async {
            // Lock the async mutex
            let _guard = WORKING_DIR_MUTEX.lock().await;

            let (outcome, _dir) = run_init_and_capture_outcome(level).await;

            prop_assert!(outcome.config_content_valid, "Generated config must contain the correct verification level");
        });
    }
}

#[tokio::test]
async fn test_init_standard_level() {
    // Lock the async mutex
    let _guard = WORKING_DIR_MUTEX.lock().await;
    let dir = tempdir().unwrap();
    std::env::set_current_dir(dir.path()).unwrap();

    let result = init::run(VerificationLevel::Standard, false, None).await;
    assert!(result.is_ok());

    let config_path = dir.path().join("ferrisproof.toml");
    assert!(config_path.exists());

    let content = fs::read_to_string(config_path).unwrap();
    assert!(content.contains("level = \"standard\""));
    assert!(content.contains("cache_enabled = true"));

    dir.close().unwrap();
}

#[tokio::test]
async fn test_init_formal_level() {
    // Lock the async mutex
    let _guard = WORKING_DIR_MUTEX.lock().await;
    let dir = tempdir().unwrap();
    std::env::set_current_dir(dir.path()).unwrap();

    let result = init::run(VerificationLevel::Formal, false, None).await;
    assert!(result.is_ok());

    let config_path = dir.path().join("ferrisproof.toml");
    assert!(config_path.exists());

    let content = fs::read_to_string(config_path).unwrap();
    assert!(content.contains("level = \"formal\""));
    assert!(content.contains("[[profile.enabled_techniques]]"));

    dir.close().unwrap();
}