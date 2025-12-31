use ferris_proof_core::{EnforcementMode, Technique, VerificationLevel};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub profile: ProfileConfig,
    #[serde(default)]
    pub tools: ToolConfig,
    #[serde(default)]
    pub modules: HashMap<String, ModuleConfig>,
    #[serde(default)]
    pub features: FeatureConfig,
    #[serde(default)]
    pub thresholds: Thresholds,
    #[serde(default)]
    pub ci: CiConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProfileConfig {
    pub level: VerificationLevel,
    pub enforcement: EnforcementMode,
    pub enabled_techniques: Vec<Technique>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolConfig {
    pub tla_plus: Option<TlaPlusConfig>,
    pub alloy: Option<AlloyConfig>,
    pub proptest: Option<ProptestConfig>,
    pub kani: Option<KaniConfig>,
}

impl Default for ToolConfig {
    fn default() -> Self {
        Self {
            tla_plus: None,
            alloy: None,
            proptest: Some(ProptestConfig {
                cases: Some(1000),
                max_shrink_iters: Some(10000),
            }),
            kani: None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TlaPlusConfig {
    pub tlc_path: Option<PathBuf>,
    pub timeout: Option<u64>,
    pub workers: Option<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AlloyConfig {
    pub analyzer_path: Option<PathBuf>,
    pub scope: Option<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProptestConfig {
    pub cases: Option<u32>,
    pub max_shrink_iters: Option<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KaniConfig {
    pub cbmc_path: Option<PathBuf>,
    pub unwind: Option<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModuleConfig {
    pub level: Option<VerificationLevel>,
    pub enforcement: Option<EnforcementMode>,
    pub enabled_techniques: Option<Vec<Technique>>,
    pub spec_file: Option<PathBuf>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FeatureConfig {
    pub cache_enabled: bool,
    pub parallel_execution: bool,
    pub generate_reports: bool,
}

impl Default for FeatureConfig {
    fn default() -> Self {
        Self {
            cache_enabled: true,
            parallel_execution: true,
            generate_reports: true,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Thresholds {
    pub max_verification_time: u64,
    pub max_memory_usage: u64,
    pub cache_ttl: u64,
}

impl Default for Thresholds {
    fn default() -> Self {
        Self {
            max_verification_time: 300,               // 5 minutes
            max_memory_usage: 2 * 1024 * 1024 * 1024, // 2GB
            cache_ttl: 24 * 60 * 60,                  // 24 hours
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CiConfig {
    pub fail_on_violations: bool,
    pub generate_artifacts: bool,
    pub upload_reports: bool,
}

impl Default for CiConfig {
    fn default() -> Self {
        Self {
            fail_on_violations: true,
            generate_artifacts: true,
            upload_reports: false,
        }
    }
}

impl Default for Config {
    fn default() -> Self {
        Self {
            profile: ProfileConfig {
                level: VerificationLevel::Standard,
                enforcement: EnforcementMode::Warning,
                enabled_techniques: vec![Technique::TypeSafety, Technique::PropertyTests],
            },
            tools: ToolConfig {
                tla_plus: None,
                alloy: None,
                proptest: Some(ProptestConfig {
                    cases: Some(1000),
                    max_shrink_iters: Some(10000),
                }),
                kani: None,
            },
            modules: HashMap::new(),
            features: FeatureConfig {
                cache_enabled: true,
                parallel_execution: true,
                generate_reports: true,
            },
            thresholds: Thresholds {
                max_verification_time: 300,               // 5 minutes
                max_memory_usage: 2 * 1024 * 1024 * 1024, // 2GB
                cache_ttl: 24 * 60 * 60,                  // 24 hours
            },
            ci: CiConfig {
                fail_on_violations: true,
                generate_artifacts: true,
                upload_reports: false,
            },
        }
    }
}
