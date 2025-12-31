use crate::config::Config;
use crate::schema::SchemaValidator;
use crate::attributes::parse_verification_attributes;
use anyhow::{Result, anyhow};
use serde_json::Value;
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use tracing::{debug, info, warn};
use walkdir::WalkDir;
use globset::{Glob, GlobSetBuilder};

#[derive(Debug)]
pub struct ConfigManager {
    root_config: Config,
    module_overrides: HashMap<PathBuf, Config>,
    cache: ConfigCache,
    project_root: PathBuf,
    schema_validator: SchemaValidator,
}

#[derive(Debug, Default)]
struct ConfigCache {
    entries: HashMap<PathBuf, CachedConfig>,
}

#[derive(Debug, Clone)]
struct CachedConfig {
    config: Config,
    timestamp: std::time::SystemTime,
    modified_time: std::time::SystemTime,
}

impl ConfigManager {
    pub fn from_project_root(root: &Path) -> Result<Self> {
        info!("Loading configuration from project root: {:?}", root);
        
        let schema_validator = SchemaValidator::new()?;
        
        let config_path = root.join("ferrisproof.toml");
        let root_config = if config_path.exists() {
            let content = std::fs::read_to_string(&config_path)?;
            
            // Validate with schema before parsing
            let json_value: Value = serde_json::from_str(&content)
                .map_err(|e| anyhow!("Failed to convert TOML to JSON for validation: {}", e))?;
            schema_validator.validate(&json_value)
                .map_err(|e| anyhow!("Schema validation failed: {}", e))?;
            
            toml::from_str(&content).map_err(|e| {
                anyhow!("Failed to parse root config at {:?}: {}", config_path, e)
            })?
        } else {
            debug!("No ferrisproof.toml found, using default configuration");
            Config::default()
        };

        let mut manager = Self {
            root_config,
            module_overrides: HashMap::new(),
            cache: ConfigCache::default(),
            project_root: root.to_path_buf(),
            schema_validator,
        };

        // Discover and load module configuration files
        manager.discover_module_configs()?;
        
        Ok(manager)
    }

    /// Recursively discover all ferrisproof.toml files in subdirectories
    fn discover_module_configs(&mut self) -> Result<()> {
        info!("Discovering module configuration files");
        
        let mut discovered_count = 0;
        
        // Collect all config paths first to avoid borrowing issues
        let config_paths: Vec<PathBuf> = WalkDir::new(&self.project_root)
            .into_iter()
            .filter_entry(|e| {
                // Skip hidden directories and target/
                let path = e.path();
                !path.file_name()
                    .map(|name| name.to_string_lossy().starts_with('.'))
                    .unwrap_or(false)
                    && path.file_name() != Some(std::ffi::OsStr::new("target"))
            })
            .filter_map(|e| e.ok())
            .filter(|e| {
                e.file_type().is_file() && 
                e.file_name() == "ferrisproof.toml" &&
                e.path().parent() != Some(&self.project_root)
            })
            .map(|e| e.path().to_path_buf())
            .collect();
        
        // Now load each config
        for config_path in config_paths {
            debug!("Found module config: {:?}", config_path);
            
            match self.load_module_config(&config_path) {
                Ok(_) => {
                    discovered_count += 1;
                    debug!("Successfully loaded module config: {:?}", config_path);
                }
                Err(e) => {
                    warn!("Failed to load module config {:?}: {}", config_path, e);
                    // Continue loading other configs rather than failing completely
                }
            }
        }
        
        info!("Discovered and loaded {} module configuration files", discovered_count);
        Ok(())
    }

    /// Load a single module configuration file
    fn load_module_config(&mut self, config_path: &Path) -> Result<()> {
        let content = std::fs::read_to_string(config_path)?;
        let config: Config = toml::from_str(&content)
            .map_err(|e| anyhow!("Failed to parse module config at {:?}: {}", config_path, e))?;
        
        let metadata = std::fs::metadata(config_path)?;
        let modified_time = metadata.modified()?;
        
        // Store the config with its directory as the key
        let config_dir = config_path.parent()
            .ok_or_else(|| anyhow!("Config file has no parent directory: {:?}", config_path))?
            .to_path_buf();
            
        // Clone config before moving it
        let config_clone = config.clone();
        self.module_overrides.insert(config_dir, config);
        
        // Update cache
        self.cache.entries.insert(
            config_path.to_path_buf(),
            CachedConfig {
                config: config_clone,
                timestamp: std::time::SystemTime::now(),
                modified_time,
            }
        );
        
        Ok(())
    }

    /// Get effective configuration for a specific file
    pub fn for_file(&self, file_path: &Path) -> EffectiveConfig {
        debug!("Resolving configuration for file: {:?}", file_path);
        
        // Start with root config
        let mut config = self.root_config.clone();
        
        // Apply ancestor module configurations (bottom-up)
        let ancestor_configs = self.find_ancestor_configs(file_path);
        for (config_dir, module_config) in ancestor_configs {
            debug!("Applying module config from {:?}", config_dir);
            config = self.merge_configs(config, module_config);
        }
        
        // Apply glob pattern matches from modules
        let module_path = self.file_to_module_path(file_path);
        let matching_configs = self.find_matching_configs(&module_path);
        for (pattern, module_config) in matching_configs {
            debug!("Applying glob pattern '{}' from config", pattern);
            config = self.merge_configs(config, module_config);
        }
        
        // Apply item-level attributes (TODO: implement AST parsing)
        if let Some(attr_config) = self.parse_item_attributes(file_path) {
            debug!("Applying item-level attributes");
            config = self.merge_configs(config, attr_config);
        }
        
        EffectiveConfig {
            level: config.profile.level,
            enforcement: config.profile.enforcement,
            enabled_techniques: config.profile.enabled_techniques,
        }
    }

    /// Find all ancestor module configurations for a file
    fn find_ancestor_configs(&self, file_path: &Path) -> Vec<(PathBuf, Config)> {
        let mut ancestors = Vec::new();
        let mut current_dir = file_path.parent();
        
        while let Some(dir) = current_dir {
            if let Some(module_config) = self.module_overrides.get(dir) {
                ancestors.push((dir.to_path_buf(), module_config.clone()));
            }
            
            // Stop at project root
            if dir == self.project_root {
                break;
            }
            
            current_dir = dir.parent();
        }
        
        // Return in bottom-up order (closest to file first)
        ancestors.reverse();
        ancestors
    }

    /// Find all module configurations with glob patterns matching the module path
    fn find_matching_configs(&self, module_path: &str) -> Vec<(String, Config)> {
        let mut matches = Vec::new();
        
        for (config_dir, module_config) in &self.module_overrides {
            // Check if this module config has glob patterns
            for (pattern_str, module_override) in &module_config.modules {
                // Try to compile the glob pattern
                if let Ok(glob) = Glob::new(pattern_str) {
                    if let Ok(glob_set) = GlobSetBuilder::new().add(glob).build() {
                        if glob_set.is_match(module_path) {
                            debug!("Glob pattern '{}' matches module path '{}'", pattern_str, module_path);
                            // Create a temporary config with just this module override
                            let mut temp_config = Config::default();
                            temp_config.modules.insert(pattern_str.clone(), module_override.clone());
                            matches.push((pattern_str.clone(), temp_config));
                        }
                    }
                }
            }
        }
        
        // Sort by specificity (longer patterns are more specific)
        matches.sort_by(|a, b| b.0.len().cmp(&a.0.len()));
        matches
    }

    /// Convert file path to module path string
    fn file_to_module_path(&self, file_path: &Path) -> String {
        // Get relative path from project root
        let relative_path = file_path.strip_prefix(&self.project_root)
            .unwrap_or(file_path);
        
        // Convert to module path (e.g., "src/main.rs" -> "src::main")
        let path_str = relative_path.to_string_lossy();
        
        // Remove file extension and replace path separators with ::
        path_str
            .strip_suffix(".rs")
            .unwrap_or(&path_str)
            .replace('/', "::")
            .replace('\\', "::")
    }

    /// Parse item-level attributes from a Rust file
    fn parse_item_attributes(&self, file_path: &Path) -> Option<Config> {
        debug!("Parsing item-level attributes from: {:?}", file_path);
        
        match parse_verification_attributes(file_path) {
            Ok(Some(config)) => Some(config),
            Ok(None) => None,
            Err(e) => {
                warn!("Failed to parse verification attributes from {:?}: {}", file_path, e);
                None
            }
        }
    }

    /// Merge two configurations, with override taking precedence
    fn merge_configs(&self, base: Config, override_config: Config) -> Config {
        Config {
            profile: crate::config::ProfileConfig {
                level: override_config.profile.level,
                enforcement: override_config.profile.enforcement,
                enabled_techniques: if override_config.profile.enabled_techniques.is_empty() {
                    base.profile.enabled_techniques
                } else {
                    override_config.profile.enabled_techniques
                },
            },
            tools: self.merge_tool_configs(&base.tools, &override_config.tools),
            modules: self.merge_module_configs(&base.modules, &override_config.modules),
            features: crate::config::FeatureConfig {
                cache_enabled: override_config.features.cache_enabled || base.features.cache_enabled,
                parallel_execution: override_config.features.parallel_execution || base.features.parallel_execution,
                generate_reports: override_config.features.generate_reports || base.features.generate_reports,
            },
            thresholds: crate::config::Thresholds {
                max_verification_time: override_config.thresholds.max_verification_time,
                max_memory_usage: override_config.thresholds.max_memory_usage,
                cache_ttl: override_config.thresholds.cache_ttl,
            },
            ci: crate::config::CiConfig {
                fail_on_violations: override_config.ci.fail_on_violations || base.ci.fail_on_violations,
                generate_artifacts: override_config.ci.generate_artifacts || base.ci.generate_artifacts,
                upload_reports: override_config.ci.upload_reports || base.ci.upload_reports,
            },
        }
    }

    fn merge_tool_configs(&self, base: &crate::config::ToolConfig, override_config: &crate::config::ToolConfig) -> crate::config::ToolConfig {
        crate::config::ToolConfig {
            tla_plus: override_config.tla_plus.clone().or(base.tla_plus.clone()),
            alloy: override_config.alloy.clone().or(base.alloy.clone()),
            proptest: override_config.proptest.clone().or(base.proptest.clone()),
            kani: override_config.kani.clone().or(base.kani.clone()),
        }
    }

    fn merge_module_configs(&self, base: &HashMap<String, crate::config::ModuleConfig>, override_config: &HashMap<String, crate::config::ModuleConfig>) -> HashMap<String, crate::config::ModuleConfig> {
        let mut merged = base.clone();
        for (key, value) in override_config {
            merged.insert(key.clone(), value.clone());
        }
        merged
    }

    pub fn validate(&self) -> Result<()> {
        // Validate root config
        self.validate_config(&self.root_config, "root")?;
        
        // Validate all module configs
        for (config_dir, module_config) in &self.module_overrides {
            self.validate_config(module_config, &format!("module at {:?}", config_dir))?;
        }
        
        Ok(())
    }

    fn validate_config(&self, config: &Config, context: &str) -> Result<()> {
        // Validate verification level and enabled techniques consistency
        match config.profile.level {
            ferris_proof_core::VerificationLevel::Minimal => {
                if !config.profile.enabled_techniques.iter().any(|t| matches!(t, ferris_proof_core::Technique::TypeSafety)) {
                    return Err(anyhow!("Minimal level must include TypeSafety technique in {}", context));
                }
            }
            ferris_proof_core::VerificationLevel::Standard => {
                if !config.profile.enabled_techniques.iter().any(|t| matches!(t, ferris_proof_core::Technique::PropertyTests)) {
                    return Err(anyhow!("Standard level must include PropertyTests technique in {}", context));
                }
            }
            ferris_proof_core::VerificationLevel::Strict => {
                if !config.profile.enabled_techniques.iter().any(|t| matches!(t, ferris_proof_core::Technique::SessionTypes)) {
                    return Err(anyhow!("Strict level must include SessionTypes technique in {}", context));
                }
            }
            ferris_proof_core::VerificationLevel::Formal => {
                if !config.profile.enabled_techniques.iter().any(|t| matches!(t, ferris_proof_core::Technique::FormalSpecs)) {
                    return Err(anyhow!("Formal level must include FormalSpecs technique in {}", context));
                }
            }
        }
        
        // Validate thresholds
        if config.thresholds.max_verification_time == 0 {
            return Err(anyhow!("max_verification_time must be > 0 in {}", context));
        }
        if config.thresholds.max_memory_usage == 0 {
            return Err(anyhow!("max_memory_usage must be > 0 in {}", context));
        }
        if config.thresholds.cache_ttl == 0 {
            return Err(anyhow!("cache_ttl must be > 0 in {}", context));
        }
        
        // Validate tool configurations
        if let Some(proptest_config) = &config.tools.proptest {
            if let Some(cases) = proptest_config.cases {
                if cases == 0 {
                    return Err(anyhow!("proptest.cases must be > 0 in {}", context));
                }
            }
        }
        
        Ok(())
    }
}

#[derive(Debug, Clone)]
pub struct EffectiveConfig {
    pub level: ferris_proof_core::VerificationLevel,
    pub enforcement: ferris_proof_core::EnforcementMode,
    pub enabled_techniques: Vec<ferris_proof_core::Technique>,
}