pub mod types;
pub mod verification;
pub mod cache;
pub mod cache_manager;
pub mod plugins;
pub mod errors;

#[cfg(test)]
mod tests;

pub use types::{
    VerificationLevel, Layer, EnforcementMode, Technique,
    VerificationResult, LayerResult, Status, Severity,
};

pub use verification::VerificationEngine;
pub use cache::VerificationCache;
pub use cache_manager::CacheManager;
pub use plugins::PluginManager;
pub use errors::FerrisProofError;