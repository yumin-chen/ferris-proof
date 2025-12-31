use crate::types::*;
use anyhow::{Result, anyhow};
use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};
use std::sync::{Arc, RwLock};
use semver::Version;
use tracing::{debug, info, warn, error};
use uuid::Uuid;

/// Stable ABI trait for verification plugins
/// 
/// This trait defines the interface that all verification plugins must implement.
/// The ABI is designed to be stable across minor version updates to ensure
/// plugin compatibility.
pub trait VerificationPlugin: Send + Sync {
    /// Plugin name and identifier
    fn name(&self) -> &str;
    
    /// Plugin version (semantic versioning)
    fn version(&self) -> &str;
    
    /// List of verification techniques supported by this plugin
    fn supported_techniques(&self) -> Vec<Technique>;
    
    /// Minimum and maximum supported FerrisProof versions
    fn supported_versions(&self) -> VersionRange;
    
    /// Check if the tool is available and properly configured
    fn check_availability(&self) -> Result<ToolInfo>;
    
    /// Execute verification with given input
    fn verify(&self, input: VerificationInput) -> Result<VerificationOutput>;
    
    /// Parse tool output into structured results
    fn parse_output(&self, raw_output: &str) -> Result<StructuredResult>;
    
    /// Get plugin metadata and capabilities
    fn metadata(&self) -> PluginMetadata;
    
    /// Initialize plugin with configuration
    fn initialize(&mut self, config: &serde_json::Value) -> Result<()>;
    
    /// Cleanup plugin resources
    fn cleanup(&mut self) -> Result<()>;
}

#[derive(Debug, Clone)]
pub struct ToolValidationResult {
    pub plugin_name: String,
    pub tool_info: Option<ToolInfo>,
    pub status: ValidationStatus,
    pub issues: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ValidationStatus {
    Valid,
    Unavailable,
    VersionIncompatible,
    Error,
}

#[derive(Debug, Clone)]
pub struct ToolInfo {
    pub name: String,
    pub version: String,
    pub path: PathBuf,
    pub available: bool,
    pub capabilities: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct VerificationInput {
    pub target: crate::verification::Target,
    pub config: EffectiveConfig,
    pub context: VerificationContext,
}

#[derive(Debug, Clone)]
pub struct VerificationOutput {
    pub status: Status,
    pub violations: Vec<Violation>,
    pub artifacts: Vec<Artifact>,
    pub tool_output: ToolOutput,
    pub metrics: VerificationMetrics,
}

#[derive(Debug, Clone)]
pub struct EffectiveConfig {
    pub level: VerificationLevel,
    pub enforcement: EnforcementMode,
    pub enabled_techniques: Vec<Technique>,
    pub tool_config: serde_json::Value,
}

#[derive(Debug, Clone)]
pub struct VerificationContext {
    pub session_id: String,
    pub working_dir: PathBuf,
    pub cache_dir: PathBuf,
    pub timeout: Option<std::time::Duration>,
    pub parallel_id: Option<u32>,
}

#[derive(Debug, Clone)]
pub struct PluginMetadata {
    pub name: String,
    pub version: String,
    pub description: String,
    pub author: String,
    pub license: String,
    pub homepage: Option<String>,
    pub techniques: Vec<Technique>,
    pub supported_platforms: Vec<String>,
    pub dependencies: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct VersionRange {
    pub min: Option<Version>,
    pub max: Option<Version>,
    pub requires_exact: Option<Version>,
}

#[derive(Debug, Clone)]
pub struct StructuredResult {
    pub status: Status,
    pub violations: Vec<Violation>,
    pub statistics: serde_json::Value,
    pub performance: PerformanceMetrics,
}

#[derive(Debug, Clone)]
pub struct PerformanceMetrics {
    pub execution_time: std::time::Duration,
    pub memory_usage: u64,
    pub cpu_usage: f64,
    pub cache_hits: u32,
}

pub struct PluginManager {
    plugins: HashMap<String, Arc<RwLock<Box<dyn VerificationPlugin>>>>,
    plugin_registry: PluginRegistry,
    version_checker: VersionChecker,
    discovery: PluginDiscovery,
}

#[derive(Debug, Default)]
struct PluginRegistry {
    registered_plugins: HashMap<String, PluginRegistration>,
    loaded_plugins: HashSet<String>,
}

#[derive(Debug, Clone)]
struct PluginRegistration {
    metadata: PluginMetadata,
    library_path: PathBuf,
    version_compatible: bool,
}

#[derive(Debug)]
struct VersionChecker {
    current_version: Version,
}

#[derive(Debug)]
struct PluginDiscovery {
    search_paths: Vec<PathBuf>,
}

impl PluginManager {
    pub fn new() -> Self {
        let current_version = Version::parse(env!("CARGO_PKG_VERSION"))
            .expect("Failed to parse current FerrisProof version");
        
        Self {
            plugins: HashMap::new(),
            plugin_registry: PluginRegistry::default(),
            version_checker: VersionChecker::new(current_version),
            discovery: PluginDiscovery::new(),
        }
    }

    /// Register a plugin instance
    pub fn register_plugin(&mut self, plugin: Box<dyn VerificationPlugin>) -> Result<()> {
        let name = plugin.name().to_string();
        let version = plugin.version().to_string();
        
        // Check version compatibility
        let version_range = plugin.supported_versions();
        if !self.version_checker.is_compatible(&version_range) {
            return Err(anyhow!(
                "Plugin {} version {} is not compatible with FerrisProof {}",
                name, version, self.version_checker.current_version
            ));
        }
        
        let plugin_arc = Arc::new(RwLock::new(plugin));
        self.plugins.insert(name.clone(), plugin_arc);
        
        info!("Registered plugin: {} v{}", name, version);
        Ok(())
    }

    /// Discover and load plugins from search paths
    pub fn discover_plugins(&mut self) -> Result<usize> {
        info!("Discovering plugins in search paths");
        
        let mut discovered_count = 0;
        
        // Collect search paths to avoid borrowing issues
        let search_paths: Vec<PathBuf> = self.discovery.search_paths.clone();
        
        for search_path in &search_paths {
            if search_path.exists() {
                discovered_count += self.discover_in_directory(search_path)?;
            }
        }
        
        info!("Discovered {} plugins", discovered_count);
        Ok(discovered_count)
    }

    /// Discover plugins in a specific directory
    fn discover_in_directory(&mut self, dir: &Path) -> Result<usize> {
        let mut discovered_count = 0;
        
        for entry in std::fs::read_dir(dir)? {
            let entry = entry?;
            let path = entry.path();
            
            // Look for plugin manifest files
            if path.extension().and_then(|s| s.to_str()) == Some("json") {
                match self.load_plugin_from_manifest(&path) {
                    Ok(_) => discovered_count += 1,
                    Err(e) => {
                        warn!("Failed to load plugin from manifest {:?}: {}", path, e);
                    }
                }
            }
        }
        
        Ok(discovered_count)
    }

    /// Load plugin from manifest file
    fn load_plugin_from_manifest(&mut self, manifest_path: &Path) -> Result<()> {
        let manifest_content = std::fs::read_to_string(manifest_path)?;
        let manifest: serde_json::Value = serde_json::from_str(&manifest_content)?;
        
        let metadata = PluginMetadata::from_json(&manifest)?;
        
        // Check if already loaded
        if self.plugin_registry.loaded_plugins.contains(&metadata.name) {
            return Ok(());
        }
        
        // For now, just register the metadata
        // TODO: Implement dynamic loading with libloading
        let registration = PluginRegistration {
            metadata: metadata.clone(),
            library_path: manifest_path.parent()
                .unwrap_or_else(|| Path::new("."))
                .join(format!("lib{}.so", metadata.name)),
            version_compatible: self.version_checker.is_metadata_compatible(&metadata),
        };
        
        self.plugin_registry.registered_plugins
            .insert(metadata.name.clone(), registration);
        
        info!("Discovered plugin: {} v{}", metadata.name, metadata.version);
        Ok(())
    }

    /// Get all plugins that support a specific technique
    pub fn plugins_for_technique(&self, technique: &Technique) -> Vec<Arc<RwLock<Box<dyn VerificationPlugin>>>> {
        self.plugins
            .values()
            .filter(|plugin| {
                let plugin_guard = plugin.read().ok();
                if let Some(plugin) = plugin_guard {
                    plugin.supported_techniques().contains(technique)
                } else {
                    false
                }
            })
            .cloned()
            .collect()
    }

    /// Validate all registered tools and their versions
    pub fn validate_tools(&self) -> Result<Vec<ToolValidationResult>> {
        let mut validation_results = Vec::new();
        
        for (name, plugin) in &self.plugins {
            match plugin.read() {
                Ok(plugin) => {
                    let validation_result = match plugin.check_availability() {
                        Ok(tool_info) => {
                            // Validate tool version if plugin specifies requirements
                            let version_validation = self.version_checker
                                .validate_tool_version(&tool_info, None);
                            
                            match version_validation {
                                Ok(_) => {
                                    debug!("Tool {} is available and compatible", name);
                                    ToolValidationResult {
                                        plugin_name: name.clone(),
                                        tool_info: Some(tool_info),
                                        status: ValidationStatus::Valid,
                                        issues: Vec::new(),
                                    }
                                }
                                Err(e) => {
                                    warn!("Tool {} version incompatible: {}", name, e);
                                    ToolValidationResult {
                                        plugin_name: name.clone(),
                                        tool_info: Some(tool_info),
                                        status: ValidationStatus::VersionIncompatible,
                                        issues: vec![e.to_string()],
                                    }
                                }
                            }
                        }
                        Err(e) => {
                            warn!("Tool {} not available: {}", name, e);
                            ToolValidationResult {
                                plugin_name: name.clone(),
                                tool_info: None,
                                status: ValidationStatus::Unavailable,
                                issues: vec![e.to_string()],
                            }
                        }
                    };
                    
                    validation_results.push(validation_result);
                }
                Err(e) => {
                    error!("Failed to lock plugin {}: {}", name, e);
                    validation_results.push(ToolValidationResult {
                        plugin_name: name.clone(),
                        tool_info: None,
                        status: ValidationStatus::Error,
                        issues: vec![format!("Plugin lock error: {}", e)],
                    });
                }
            }
        }
        
        Ok(validation_results)
    }

    /// Execute verification using appropriate plugin
    pub async fn verify(&self, technique: &Technique, input: VerificationInput) -> Result<VerificationOutput> {
        let plugins = self.plugins_for_technique(technique);
        
        if plugins.is_empty() {
            return Err(anyhow!("No plugins available for technique: {:?}", technique));
        }
        
        // Use first available plugin
        // TODO: Implement plugin selection strategy
        if let Some(plugin_arc) = plugins.first() {
            let plugin = plugin_arc.read()
                .map_err(|e| anyhow!("Failed to acquire plugin lock: {}", e))?;
            
            // Create verification context
            let context = VerificationContext {
                session_id: Uuid::new_v4().to_string(),
                working_dir: std::env::current_dir().unwrap_or_else(|_| PathBuf::from("/tmp")),
                cache_dir: std::env::temp_dir(),
                timeout: Some(std::time::Duration::from_secs(300)), // 5 minutes default
                parallel_id: None,
            };
            
            let enhanced_input = VerificationInput {
                target: input.target,
                config: input.config,
                context,
            };
            
            plugin.verify(enhanced_input)
        } else {
            Err(anyhow!("No plugin available for technique: {:?}", technique))
        }
    }

    /// Get plugin metadata
    pub fn plugin_metadata(&self, name: &str) -> Option<PluginMetadata> {
        if let Some(plugin_arc) = self.plugins.get(name) {
            match plugin_arc.read() {
                Ok(plugin) => Some(plugin.metadata()),
                Err(_) => None,
            }
        } else if let Some(registration) = self.plugin_registry.registered_plugins.get(name) {
            Some(registration.metadata.clone())
        } else {
            None
        }
    }

    /// List all registered plugins
    pub fn list_plugins(&self) -> Vec<PluginMetadata> {
        let mut metadata_list = Vec::new();
        
        // Add loaded plugins
        for (name, plugin_arc) in &self.plugins {
            match plugin_arc.read() {
                Ok(plugin) => {
                    metadata_list.push(plugin.metadata());
                }
                Err(_) => {
                    warn!("Failed to read plugin metadata for: {}", name);
                }
            }
        }
        
        // Add discovered but not loaded plugins
        for registration in self.plugin_registry.registered_plugins.values() {
            if !self.plugin_registry.loaded_plugins.contains(&registration.metadata.name) {
                metadata_list.push(registration.metadata.clone());
            }
        }
        
        metadata_list.sort_by(|a, b| a.name.cmp(&b.name));
        metadata_list
    }

    /// Initialize all plugins with configuration
    pub fn initialize_plugins(&mut self, config: &serde_json::Value) -> Result<()> {
        for (name, plugin_arc) in &self.plugins {
            match plugin_arc.write() {
                Ok(mut plugin) => {
                    if let Some(plugin_config) = config.get(name) {
                        if let Err(e) = plugin.initialize(plugin_config) {
                            warn!("Failed to initialize plugin {}: {}", name, e);
                        }
                    }
                }
                Err(e) => {
                    error!("Failed to acquire write lock for plugin {}: {}", name, e);
                }
            }
        }
        Ok(())
    }

    /// Cleanup all plugins
    pub fn cleanup_plugins(&mut self) -> Result<()> {
        for (name, plugin_arc) in &self.plugins {
            match plugin_arc.write() {
                Ok(mut plugin) => {
                    if let Err(e) = plugin.cleanup() {
                        warn!("Failed to cleanup plugin {}: {}", name, e);
                    }
                }
                Err(e) => {
                    error!("Failed to acquire write lock for plugin {}: {}", name, e);
                }
            }
        }
        Ok(())
    }
}

impl PluginMetadata {
    fn from_json(value: &serde_json::Value) -> Result<Self> {
        Ok(Self {
            name: value.get("name")
                .and_then(|v| v.as_str())
                .ok_or_else(|| anyhow!("Missing plugin name"))?
                .to_string(),
            version: value.get("version")
                .and_then(|v| v.as_str())
                .ok_or_else(|| anyhow!("Missing plugin version"))?
                .to_string(),
            description: value.get("description")
                .and_then(|v| v.as_str())
                .unwrap_or("No description available")
                .to_string(),
            author: value.get("author")
                .and_then(|v| v.as_str())
                .unwrap_or("Unknown")
                .to_string(),
            license: value.get("license")
                .and_then(|v| v.as_str())
                .unwrap_or("Unknown")
                .to_string(),
            homepage: value.get("homepage")
                .and_then(|v| v.as_str())
                .map(|s| s.to_string()),
            techniques: value.get("techniques")
                .and_then(|v| v.as_array())
                .map(|arr| {
                    arr.iter()
                        .filter_map(|v| v.as_str())
                        .filter_map(|s| {
                            match s {
                                "TypeSafety" => Some(Technique::TypeSafety),
                                "PropertyTests" => Some(Technique::PropertyTests),
                                "SessionTypes" => Some(Technique::SessionTypes),
                                "RefinementTypes" => Some(Technique::RefinementTypes),
                                "ConcurrencyTesting" => Some(Technique::ConcurrencyTesting),
                                "FormalSpecs" => Some(Technique::FormalSpecs),
                                "ModelChecking" => Some(Technique::ModelChecking),
                                _ => None,
                            }
                        })
                        .collect()
                })
                .unwrap_or_default(),
            supported_platforms: value.get("platforms")
                .and_then(|v| v.as_array())
                .map(|arr| {
                    arr.iter()
                        .filter_map(|v| v.as_str())
                        .map(|s| s.to_string())
                        .collect()
                })
                .unwrap_or_default(),
            dependencies: value.get("dependencies")
                .and_then(|v| v.as_array())
                .map(|arr| {
                    arr.iter()
                        .filter_map(|v| v.as_str())
                        .map(|s| s.to_string())
                        .collect()
                })
                .unwrap_or_default(),
        })
    }
}

impl PluginRegistry {
    fn register(&mut self, name: String, registration: PluginRegistration) {
        self.registered_plugins.insert(name, registration);
    }
}

impl VersionChecker {
    fn new(current_version: Version) -> Self {
        Self { current_version }
    }
    
    /// Check if a plugin's version range is compatible with current FerrisProof version
    fn is_compatible(&self, version_range: &VersionRange) -> bool {
        if let Some(exact_version) = &version_range.requires_exact {
            return &self.current_version == exact_version;
        }
        
        if let Some(min_version) = &version_range.min {
            if self.current_version < *min_version {
                return false;
            }
        }
        
        if let Some(max_version) = &version_range.max {
            if self.current_version > *max_version {
                return false;
            }
        }
        
        true
    }
    
    /// Check if plugin metadata indicates compatibility
    fn is_metadata_compatible(&self, metadata: &PluginMetadata) -> bool {
        // Check plugin metadata version compatibility
        if let Ok(plugin_version) = metadata.version.parse::<Version>() {
            // Check if plugin version is within reasonable bounds
            // Allow plugins that are at most 1 major version behind or ahead
            let major_diff = (self.current_version.major as i64) - (plugin_version.major as i64);
            major_diff.abs() <= 1
        } else {
            false
        }
    }
    
    /// Validate tool version compatibility
    fn validate_tool_version(&self, tool_info: &ToolInfo, expected_range: Option<&VersionRange>) -> Result<()> {
        if let Some(range) = expected_range {
            if let Ok(tool_version) = tool_info.version.parse::<Version>() {
                if let Some(min_version) = &range.min {
                    if tool_version < *min_version {
                        return Err(anyhow!(
                            "Tool {} version {} is below minimum required version {}",
                            tool_info.name, tool_version, min_version
                        ));
                    }
                }
                
                if let Some(max_version) = &range.max {
                    if tool_version > *max_version {
                        return Err(anyhow!(
                            "Tool {} version {} is above maximum supported version {}",
                            tool_info.name, tool_version, max_version
                        ));
                    }
                }
                
                if let Some(exact_version) = &range.requires_exact {
                    if tool_version != *exact_version {
                        return Err(anyhow!(
                            "Tool {} version {} does not match required exact version {}",
                            tool_info.name, tool_version, exact_version
                        ));
                    }
                }
            } else {
                warn!("Could not parse tool version: {}", tool_info.version);
            }
        }
        
        Ok(())
    }
}

impl PluginDiscovery {
    fn new() -> Self {
        let mut search_paths = Vec::new();
        
        // Add default search paths
        if let Some(home_dir) = dirs::home_dir() {
            search_paths.push(home_dir.join(".ferris-proof/plugins"));
            search_paths.push(home_dir.join(".local/share/ferris-proof/plugins"));
        }
        
        if let Some(current_dir) = std::env::current_dir().ok() {
            search_paths.push(current_dir.join(".ferris-proof/plugins"));
        }
        
        // Add system-wide paths
        search_paths.push(PathBuf::from("/usr/local/lib/ferris-proof/plugins"));
        search_paths.push(PathBuf::from("/usr/lib/ferris-proof/plugins"));
        
        Self { search_paths }
    }
}

impl Default for PluginManager {
    fn default() -> Self {
        Self::new()
    }
}