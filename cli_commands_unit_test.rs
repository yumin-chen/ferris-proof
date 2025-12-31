use ferris_proof_cli::commands::init;
use ferris_proof_core::models::config::VerificationLevel;
use std::fs;
use tempfile::tempdir;

#[tokio::test]
async fn test_init_command_creates_files() {
    let dir = tempdir().unwrap();
    let project_path = dir.path();

    // Change current directory to the temporary one
    std::env::set_current_dir(project_path).unwrap();

    let result = init::run(VerificationLevel::Standard, false, None).await;
    assert!(result.is_ok());

    // Verify that the expected files and directories were created
    assert!(project_path.join("ferrisproof.toml").exists());
    assert!(project_path.join("specs").is_dir());
    assert!(project_path.join("tests").is_dir());
    assert!(project_path.join("tests/property").is_dir());

    // Verify content of ferrisproof.toml
    let content = fs::read_to_string(project_path.join("ferrisproof.toml")).unwrap();
    assert!(content.contains("level = \"standard\""));

    dir.close().unwrap();
}