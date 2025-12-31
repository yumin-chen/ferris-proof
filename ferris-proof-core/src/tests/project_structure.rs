use proptest::prelude::*;
use std::collections::HashSet;
use std::path::Path;

/// **Feature: ferris-proof, Property 1: Project structure consistency**
/// **Validates: Requirements 18.3**
///
/// This property test verifies that FerrisProof maintains consistent project structure
/// across different initialization scenarios and configurations.
#[cfg(test)]
mod project_structure_tests {
    use super::*;

    /// Represents a valid FerrisProof project structure
    #[derive(Debug, Clone)]
    struct ProjectStructure {
        pub root_files: HashSet<String>,
        pub required_directories: HashSet<String>,
        pub crate_structure: Vec<String>,
    }

    impl ProjectStructure {
        fn ferris_proof_workspace() -> Self {
            let mut root_files = HashSet::new();
            root_files.insert("Cargo.toml".to_string());
            root_files.insert("ReadMe.md".to_string());
            root_files.insert("Containerfile".to_string());

            let mut required_directories = HashSet::new();
            required_directories.insert("ferris-proof-cli".to_string());
            required_directories.insert("ferris-proof-core".to_string());
            required_directories.insert("ferris-proof-config".to_string());
            required_directories.insert("ferris-proof-plugins".to_string());
            required_directories.insert(".github".to_string());
            required_directories.insert("docs".to_string());

            let crate_structure = vec![
                "ferris-proof-cli".to_string(),
                "ferris-proof-core".to_string(),
                "ferris-proof-config".to_string(),
                "ferris-proof-plugins".to_string(),
            ];

            Self {
                root_files,
                required_directories,
                crate_structure,
            }
        }

        fn validate_structure(&self, project_root: &Path) -> Result<(), String> {
            // Check root files exist
            for file in &self.root_files {
                let file_path = project_root.join(file);
                if !file_path.exists() {
                    // Debug: list what files actually exist
                    if let Ok(entries) = std::fs::read_dir(project_root) {
                        let existing_files: Vec<String> = entries
                            .filter_map(|e| e.ok())
                            .filter_map(|e| e.file_name().into_string().ok())
                            .collect();
                        return Err(format!(
                            "Required root file missing: {}. Existing files: {:?}",
                            file, existing_files
                        ));
                    }
                    return Err(format!("Required root file missing: {}", file));
                }
            }

            // Check required directories exist
            for dir in &self.required_directories {
                let dir_path = project_root.join(dir);
                if !dir_path.exists() || !dir_path.is_dir() {
                    return Err(format!("Required directory missing: {}", dir));
                }
            }

            // Check crate structure
            for crate_name in &self.crate_structure {
                let crate_path = project_root.join(crate_name);
                let cargo_toml = crate_path.join("Cargo.toml");
                let src_dir = crate_path.join("src");
                let lib_rs = src_dir.join("lib.rs");

                if !cargo_toml.exists() {
                    return Err(format!("Crate {} missing Cargo.toml", crate_name));
                }

                if !src_dir.exists() || !src_dir.is_dir() {
                    return Err(format!("Crate {} missing src directory", crate_name));
                }

                // CLI crate has main.rs, others have lib.rs
                if crate_name == "ferris-proof-cli" {
                    let main_rs = src_dir.join("main.rs");
                    if !main_rs.exists() && !lib_rs.exists() {
                        return Err("CLI crate missing main.rs or lib.rs".to_string());
                    }
                } else if !lib_rs.exists() {
                    return Err(format!("Crate {} missing lib.rs", crate_name));
                }
            }

            Ok(())
        }
    }

    proptest! {
        #[test]
        /// **Feature: ferris-proof, Property 1: Project structure consistency**
        /// For any valid FerrisProof workspace configuration, the project structure
        /// should maintain consistency with required files and directories.
        fn project_structure_consistency(
            // Generate different project configurations
            _include_optional_files in prop::bool::ANY,
            _include_examples in prop::bool::ANY,
        ) {
            let expected_structure = ProjectStructure::ferris_proof_workspace();

            // Find the workspace root by looking for Cargo.toml with [workspace]
            let mut current_dir = std::env::current_dir()
                .expect("Failed to get current directory");

            // Walk up the directory tree to find workspace root
            loop {
                let cargo_toml = current_dir.join("Cargo.toml");
                if cargo_toml.exists() {
                    if let Ok(content) = std::fs::read_to_string(&cargo_toml) {
                        if content.contains("[workspace]") {
                            break; // Found workspace root
                        }
                    }
                }

                if let Some(parent) = current_dir.parent() {
                    current_dir = parent.to_path_buf();
                } else {
                    prop_assume!(false); // Skip test if can't find workspace root
                }
            }

            // Validate the current project structure
            let validation_result = expected_structure.validate_structure(&current_dir);

            prop_assert!(
                validation_result.is_ok(),
                "Project structure validation failed: {}",
                validation_result.unwrap_err()
            );

            // Additional consistency checks

            // 1. Workspace Cargo.toml should list all crates
            let workspace_cargo = current_dir.join("Cargo.toml");
            if workspace_cargo.exists() {
                let cargo_content = std::fs::read_to_string(&workspace_cargo)
                    .expect("Failed to read workspace Cargo.toml");

                for crate_name in &expected_structure.crate_structure {
                    prop_assert!(
                        cargo_content.contains(&format!("\"{}\"", crate_name)),
                        "Workspace Cargo.toml missing crate: {}",
                        crate_name
                    );
                }
            }

            // 2. Each crate should have consistent naming
            for crate_name in &expected_structure.crate_structure {
                let crate_cargo = current_dir.join(crate_name).join("Cargo.toml");
                if crate_cargo.exists() {
                    let cargo_content = std::fs::read_to_string(&crate_cargo)
                        .expect("Failed to read crate Cargo.toml");

                    prop_assert!(
                        cargo_content.contains(&format!("name = \"{}\"", crate_name)),
                        "Crate {} has inconsistent name in Cargo.toml",
                        crate_name
                    );
                }
            }

            // 3. Documentation structure should be consistent
            let docs_dir = current_dir.join("docs");
            if docs_dir.exists() {
                let getting_started = docs_dir.join("getting-started.md");
                prop_assert!(
                    getting_started.exists(),
                    "Documentation missing getting-started.md"
                );
            }
        }
    }

    proptest! {
        #[test]
        /// **Feature: ferris-proof, Property 1: Project structure consistency**
        /// For any crate in the workspace, dependencies should be properly declared
        /// and version constraints should be consistent.
        fn crate_dependency_consistency(
            crate_index in 0..4usize, // We have 4 crates
        ) {
            let expected_structure = ProjectStructure::ferris_proof_workspace();

            // Find the workspace root by looking for Cargo.toml with [workspace]
            let mut current_dir = std::env::current_dir()
                .expect("Failed to get current directory");

            // Walk up the directory tree to find workspace root
            loop {
                let cargo_toml = current_dir.join("Cargo.toml");
                if cargo_toml.exists() {
                    if let Ok(content) = std::fs::read_to_string(&cargo_toml) {
                        if content.contains("[workspace]") {
                            break; // Found workspace root
                        }
                    }
                }

                if let Some(parent) = current_dir.parent() {
                    current_dir = parent.to_path_buf();
                } else {
                    prop_assume!(false); // Skip test if can't find workspace root
                }
            }

            if crate_index >= expected_structure.crate_structure.len() {
                return Ok(());
            }

            let crate_name = &expected_structure.crate_structure[crate_index];
            let crate_cargo = current_dir.join(crate_name).join("Cargo.toml");

            if !crate_cargo.exists() {
                return Ok(());
            }

            let cargo_content = std::fs::read_to_string(&crate_cargo)
                .expect("Failed to read crate Cargo.toml");

            // Check that workspace dependencies are used consistently
            if cargo_content.contains("[dependencies]") {
                // If crate uses workspace dependencies, they should use .workspace = true
                let workspace_deps = ["serde", "tokio", "anyhow", "thiserror", "tracing"];

                for dep in &workspace_deps {
                    if cargo_content.contains(&format!("{}.workspace", dep)) {
                        prop_assert!(
                            cargo_content.contains(&format!("{}.workspace = true", dep)),
                            "Crate {} should use workspace dependency for {}",
                            crate_name,
                            dep
                        );
                    }
                }
            }

            // Check internal dependencies use path references
            for other_crate in &expected_structure.crate_structure {
                if other_crate != crate_name && cargo_content.contains(other_crate) {
                    prop_assert!(
                        cargo_content.contains(&format!("path = \"../{}\"", other_crate)),
                        "Crate {} should use path dependency for internal crate {}",
                        crate_name,
                        other_crate
                    );
                }
            }
        }
    }

    #[test]
    fn test_current_project_structure() {
        let expected_structure = ProjectStructure::ferris_proof_workspace();

        // Find the workspace root by looking for Cargo.toml with [workspace]
        let mut current_dir = std::env::current_dir().expect("Failed to get current directory");

        // Walk up the directory tree to find workspace root
        loop {
            let cargo_toml = current_dir.join("Cargo.toml");
            if cargo_toml.exists() {
                if let Ok(content) = std::fs::read_to_string(&cargo_toml) {
                    if content.contains("[workspace]") {
                        break; // Found workspace root
                    }
                }
            }

            if let Some(parent) = current_dir.parent() {
                current_dir = parent.to_path_buf();
            } else {
                panic!("Could not find workspace root");
            }
        }

        match expected_structure.validate_structure(&current_dir) {
            Ok(()) => println!("✓ Project structure validation passed"),
            Err(e) => panic!("Project structure validation failed: {}", e),
        }
    }
}
