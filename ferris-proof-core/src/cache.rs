use crate::types::*;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;

#[derive(Debug, Clone, Hash, Eq, PartialEq, Serialize, Deserialize)]
pub struct CacheKey {
    pub content_hash: ContentHash,
    pub config_hash: ConfigHash,
    pub tool_versions: ToolVersions,
    pub layer: Layer,
}

#[derive(Debug, Clone, Hash, Eq, PartialEq, Serialize, Deserialize)]
pub struct ContentHash(pub String);

#[derive(Debug, Clone, Hash, Eq, PartialEq, Serialize, Deserialize)]
pub struct ConfigHash(pub String);

#[derive(Debug, Clone, Hash, Eq, PartialEq, Serialize, Deserialize)]
pub struct ToolVersions {
    pub ferris_proof: String,
    pub external_tools: Vec<(String, String)>, // Use Vec instead of HashMap for Hash trait
}

pub struct VerificationCache {
    cache_dir: PathBuf,
    entries: HashMap<CacheKey, CacheEntry>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CacheEntry {
    pub result: LayerResult,
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub ttl: std::time::Duration,
}

impl VerificationCache {
    pub fn new() -> Self {
        Self {
            cache_dir: std::env::temp_dir().join("ferris-proof-cache"),
            entries: HashMap::new(),
        }
    }

    pub fn get(&self, key: &CacheKey) -> Option<&CacheEntry> {
        self.entries.get(key)
    }

    pub fn store(&mut self, key: CacheKey, entry: CacheEntry) {
        self.entries.insert(key, entry);
    }

    pub fn invalidate(&mut self, key: &CacheKey) {
        self.entries.remove(key);
    }

    pub fn clear(&mut self) {
        self.entries.clear();
    }
}

impl Default for VerificationCache {
    fn default() -> Self {
        Self::new()
    }
}