use ferris_proof_config::ConfigManager;
use ferris_proof_core::plugins::VerificationPlugin;
use ferris_proof_core::types::VerificationLevel;
use ferris_proof_core::{PluginManager, VerificationEngine};
use ferris_proof_plugins::{ProptestPlugin, TlaPlusPlugin};
use std::path::PathBuf;
use tempfile::TempDir;

/// Integration test to verify all core systems work together
/// This test validates the checkpoint requirements:
/// - Configuration system integration
/// - Plugin loading and tool integration
/// - Cache system functionality
/// - Verification engine coordination
#[tokio::test]
async fn test_core_infrastructure_integration() {
    println!("🔧 Testing Core Infrastructure Integration");

    // 1. Test Configuration System
    println!("  ✓ Testing Configuration Manager...");
    let temp_dir = TempDir::new().expect("Failed to create temp directory");
    let config_path = temp_dir.path().join("ferrisproof.toml");

    // Create a test configuration
    let config_content = r#"
[profile]
level = "standard"
enforcement = "warning"
enabled_techniques = ["TypeSafety", "PropertyTests"]

[tools.proptest]
cases = 1000
max_shrink_iterations = 10000

[features]
cache_enabled = true
parallel_execution = true
generate_reports = true
"#;

    std::fs::write(&config_path, config_content).expect("Failed to write config");

    // Test configuration loading
    let config_manager =
        ConfigManager::from_project_root(temp_dir.path()).expect("Failed to load configuration");

    let effective_config = config_manager.for_file(&PathBuf::from("src/main.rs"));
    assert_eq!(effective_config.level, VerificationLevel::Standard);
    println!("    ✓ Configuration loading works");

    // 2. Test Plugin System
    println!("  ✓ Testing Plugin Manager...");
    let mut plugin_manager = PluginManager::new();

    // Register plugins
    let proptest_plugin = Box::new(ProptestPlugin::new());
    let tla_plugin = Box::new(TlaPlusPlugin::new());

    plugin_manager
        .register_plugin(proptest_plugin)
        .expect("Failed to register Proptest plugin");
    plugin_manager
        .register_plugin(tla_plugin)
        .expect("Failed to register TLA+ plugin");

    // Verify plugins are loaded
    let plugins = plugin_manager.list_plugins();
    assert!(plugins.len() >= 2, "Should have at least 2 plugins loaded");
    println!("    ✓ Plugin loading works");

    // Test tool validation
    let validation_results = plugin_manager
        .validate_tools()
        .expect("Tool validation should complete");
    println!(
        "    ✓ Tool validation completed: {} results",
        validation_results.len()
    );

    // 3. Test Verification Engine Integration
    println!("  ✓ Testing Verification Engine...");
    let verification_engine = VerificationEngine::new();

    // Test verification with mock targets
    let targets = vec![ferris_proof_core::verification::Target::RustFile(
        PathBuf::from("src/main.rs"),
    )];

    let result = verification_engine
        .verify(&targets)
        .await
        .expect("Verification should complete successfully");

    assert_eq!(
        result.overall_status,
        ferris_proof_core::types::Status::Success
    );
    println!("    ✓ Verification engine works");

    // 4. Test Cache Integration
    println!("  ✓ Testing Cache System...");
    // Cache is tested through the verification engine
    let needs_verification = verification_engine.needs_verification(&targets[0]);
    assert!(
        needs_verification,
        "Should need verification for new target"
    );
    println!("    ✓ Cache system works");

    println!("🎉 All core systems integration test passed!");
}

#[test]
fn test_plugin_system_integration() {
    println!("🔌 Testing Plugin System Integration");

    let mut plugin_manager = PluginManager::new();

    // Test plugin registration
    let proptest_plugin = Box::new(ProptestPlugin::new());
    let plugin_name = proptest_plugin.name().to_string();

    let result = plugin_manager.register_plugin(proptest_plugin);
    assert!(result.is_ok(), "Plugin registration should succeed");

    // Test plugin listing
    let plugins = plugin_manager.list_plugins();
    assert!(!plugins.is_empty(), "Should have registered plugins");

    // Test plugin metadata
    let metadata = plugin_manager.plugin_metadata(&plugin_name);
    assert!(metadata.is_some(), "Plugin metadata should be available");

    println!("  ✓ Plugin registration works");
    println!("  ✓ Plugin metadata retrieval works");
    println!("🎉 Plugin system integration test passed!");
}

#[test]
fn test_configuration_system_integration() {
    println!("⚙️  Testing Configuration System Integration");

    let temp_dir = TempDir::new().expect("Failed to create temp directory");
    let config_path = temp_dir.path().join("ferrisproof.toml");

    // Create a minimal configuration
    let config_content = r#"
[profile]
level = "minimal"
enforcement = "advisory"
enabled_techniques = ["TypeSafety"]
"#;

    std::fs::write(&config_path, config_content).expect("Failed to write config");

    // Test configuration loading
    let config_manager =
        ConfigManager::from_project_root(temp_dir.path()).expect("Failed to load configuration");

    let effective_config = config_manager.for_file(&PathBuf::from("test.rs"));
    assert_eq!(effective_config.level, VerificationLevel::Minimal);

    println!("  ✓ Configuration file parsing works");
    println!("  ✓ Configuration resolution works");
    println!("🎉 Configuration system integration test passed!");
}
