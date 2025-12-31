pub mod cache;
pub mod cache_manager;
pub mod errors;
pub mod plugins;
pub mod types;
pub mod verification;

#[cfg(test)]
mod tests;

pub use types::{
    EnforcementMode, Layer, LayerResult, Severity, Status, Technique, VerificationLevel,
    VerificationResult,
};

pub use cache::VerificationCache;
pub use cache_manager::CacheManager;
pub use errors::FerrisProofError;
pub use plugins::PluginManager;
pub use verification::VerificationEngine;
