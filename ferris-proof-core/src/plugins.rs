use crate::types::*;
use anyhow::Result;
use std::collections::HashMap;

pub struct PluginManager {
    plugins: HashMap<String, Box<dyn VerificationPlugin>>,
}

pub trait VerificationPlugin: Send + Sync {
    fn name(&self) -> &str;
    fn supported_techniques(&self) -> Vec<Technique>;
    fn check_availability(&self) -> Result<ToolInfo>;
    fn verify(&self, input: VerificationInput) -> Result<VerificationOutput>;
}

#[derive(Debug, Clone)]
pub struct ToolInfo {
    pub name: String,
    pub version: String,
    pub path: std::path::PathBuf,
}

#[derive(Debug, Clone)]
pub struct VerificationInput {
    pub target: crate::verification::Target,
    pub config: EffectiveConfig,
}

#[derive(Debug, Clone)]
pub struct VerificationOutput {
    pub status: Status,
    pub violations: Vec<Violation>,
    pub artifacts: Vec<Artifact>,
    pub tool_output: ToolOutput,
}

#[derive(Debug, Clone)]
pub struct EffectiveConfig {
    pub level: VerificationLevel,
    pub enforcement: EnforcementMode,
    pub enabled_techniques: Vec<Technique>,
}

impl PluginManager {
    pub fn new() -> Self {
        Self {
            plugins: HashMap::new(),
        }
    }

    pub fn register_plugin(&mut self, plugin: Box<dyn VerificationPlugin>) {
        let name = plugin.name().to_string();
        self.plugins.insert(name, plugin);
    }

    pub fn plugins_for_technique(&self, technique: &Technique) -> Vec<&dyn VerificationPlugin> {
        self.plugins
            .values()
            .filter(|plugin| plugin.supported_techniques().contains(technique))
            .map(|plugin| plugin.as_ref())
            .collect()
    }

    pub fn validate_tools(&self) -> Result<Vec<ToolInfo>> {
        let mut tools = Vec::new();
        for plugin in self.plugins.values() {
            match plugin.check_availability() {
                Ok(tool_info) => tools.push(tool_info),
                Err(e) => tracing::warn!("Tool {} not available: {}", plugin.name(), e),
            }
        }
        Ok(tools)
    }
}

impl Default for PluginManager {
    fn default() -> Self {
        Self::new()
    }
}