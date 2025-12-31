use crate::cache::{CompactionResult, VerificationCache};
use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// Cache management operations for CLI and programmatic use
pub struct CacheManager {
    cache: VerificationCache,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CacheInfo {
    pub cache_dir: PathBuf,
    pub total_entries: usize,
    pub valid_entries: usize,
    pub expired_entries: usize,
    pub total_size_bytes: u64,
    pub disk_size_bytes: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CacheHealthReport {
    pub info: CacheInfo,
    pub integrity_errors: Vec<String>,
    pub recommendations: Vec<String>,
}

impl CacheManager {
    /// Create a new cache manager with default cache directory
    pub fn new() -> Self {
        Self {
            cache: VerificationCache::new(),
        }
    }

    /// Create a new cache manager with custom cache directory
    pub fn with_cache_dir(cache_dir: PathBuf) -> Self {
        Self {
            cache: VerificationCache::with_cache_dir(cache_dir),
        }
    }

    /// Get comprehensive cache information
    pub fn info(&self) -> Result<CacheInfo> {
        let stats = self.cache.statistics();
        let disk_size = self.cache.disk_size().unwrap_or(0);

        Ok(CacheInfo {
            cache_dir: stats.cache_dir,
            total_entries: stats.total_entries,
            valid_entries: stats.valid_entries,
            expired_entries: stats.expired_entries,
            total_size_bytes: stats.total_size_bytes,
            disk_size_bytes: disk_size,
        })
    }

    /// Clean up expired cache entries
    pub fn cleanup(&mut self) -> Result<CleanupResult> {
        let initial_info = self.info()?;
        let expired_removed = self.cache.cleanup_expired()?;
        let final_info = self.info()?;

        Ok(CleanupResult {
            entries_removed: expired_removed,
            size_freed: initial_info
                .disk_size_bytes
                .saturating_sub(final_info.disk_size_bytes),
            entries_before: initial_info.total_entries,
            entries_after: final_info.total_entries,
        })
    }

    /// Clear all cache entries
    pub fn clear(&mut self) -> Result<ClearResult> {
        let initial_info = self.info()?;
        self.cache.clear();

        Ok(ClearResult {
            entries_removed: initial_info.total_entries,
            size_freed: initial_info.disk_size_bytes,
        })
    }

    /// Compact cache by removing expired entries and optimizing storage
    pub fn compact(&mut self) -> Result<CompactionResult> {
        self.cache.compact()
    }

    /// Validate cache integrity and provide health report
    pub fn health_check(&self) -> Result<CacheHealthReport> {
        let info = self.info()?;
        let integrity_errors = self.cache.validate_integrity()?;
        let mut recommendations = Vec::new();

        // Generate recommendations based on cache state
        if info.expired_entries > 0 {
            recommendations.push(format!(
                "Consider running cleanup to remove {} expired entries",
                info.expired_entries
            ));
        }

        if info.disk_size_bytes > 1_000_000_000 {
            // > 1GB
            recommendations.push(
                "Cache size is large (>1GB). Consider running compact to optimize storage"
                    .to_string(),
            );
        }

        if !integrity_errors.is_empty() {
            recommendations.push(
                "Cache integrity issues detected. Consider clearing and rebuilding cache"
                    .to_string(),
            );
        }

        if info.total_entries == 0 {
            recommendations.push("Cache is empty. No action needed".to_string());
        }

        Ok(CacheHealthReport {
            info,
            integrity_errors,
            recommendations,
        })
    }

    /// Repair cache by removing corrupted entries
    pub fn repair(&mut self) -> Result<RepairResult> {
        let initial_info = self.info()?;
        let integrity_errors = self.cache.validate_integrity()?;

        if integrity_errors.is_empty() {
            return Ok(RepairResult {
                corrupted_entries_removed: 0,
                entries_before: initial_info.total_entries,
                entries_after: initial_info.total_entries,
                size_freed: 0,
            });
        }

        // For now, we'll clear the entire cache if there are integrity issues
        // In a more sophisticated implementation, we could selectively remove corrupted entries
        let clear_result = self.clear()?;

        Ok(RepairResult {
            corrupted_entries_removed: clear_result.entries_removed,
            entries_before: initial_info.total_entries,
            entries_after: 0,
            size_freed: clear_result.size_freed,
        })
    }

    /// Get cache statistics for monitoring
    pub fn statistics(&self) -> CacheStatistics {
        let stats = self.cache.statistics();
        let disk_size = self.cache.disk_size().unwrap_or(0);

        CacheStatistics {
            total_entries: stats.total_entries,
            valid_entries: stats.valid_entries,
            expired_entries: stats.expired_entries,
            memory_size_bytes: stats.total_size_bytes,
            disk_size_bytes: disk_size,
            cache_dir: stats.cache_dir,
        }
    }

    /// Load cache from disk
    pub fn load(&mut self) -> Result<()> {
        self.cache.load_from_disk()
    }

    /// Save cache to disk
    pub fn save(&self) -> Result<()> {
        self.cache.save_to_disk()
    }

    /// Get the underlying cache for advanced operations
    pub fn cache(&self) -> &VerificationCache {
        &self.cache
    }

    /// Get mutable access to the underlying cache
    pub fn cache_mut(&mut self) -> &mut VerificationCache {
        &mut self.cache
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CleanupResult {
    pub entries_removed: usize,
    pub size_freed: u64,
    pub entries_before: usize,
    pub entries_after: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClearResult {
    pub entries_removed: usize,
    pub size_freed: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RepairResult {
    pub corrupted_entries_removed: usize,
    pub entries_before: usize,
    pub entries_after: usize,
    pub size_freed: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CacheStatistics {
    pub total_entries: usize,
    pub valid_entries: usize,
    pub expired_entries: usize,
    pub memory_size_bytes: u64,
    pub disk_size_bytes: u64,
    pub cache_dir: PathBuf,
}

impl Default for CacheManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cache::{
        CacheEntry, CacheKey, CacheMetadata, ConfigHash, ContentHash, ToolVersions,
    };
    use crate::types::{Layer, LayerResult, Status};
    use std::time::Duration;
    use tempfile::TempDir;

    #[test]
    fn test_cache_manager_info() {
        let temp_dir = TempDir::new().unwrap();
        let cache_dir = temp_dir.path().join("cache");

        let manager = CacheManager::with_cache_dir(cache_dir.clone());
        let info = manager.info().unwrap();

        assert_eq!(info.cache_dir, cache_dir);
        assert_eq!(info.total_entries, 0);
        assert_eq!(info.valid_entries, 0);
        assert_eq!(info.expired_entries, 0);
    }

    #[test]
    fn test_cache_manager_cleanup() {
        let temp_dir = TempDir::new().unwrap();
        let cache_dir = temp_dir.path().join("cache");

        let mut manager = CacheManager::with_cache_dir(cache_dir);

        // Add some test entries (some expired)
        let cache_key = CacheKey {
            content_hash: ContentHash("test_hash".to_string()),
            config_hash: ConfigHash("config_hash".to_string()),
            tool_versions: ToolVersions {
                ferris_proof: "0.1.0".to_string(),
                external_tools: vec![],
            },
            layer: Layer::PropertyBased,
        };

        let expired_entry = CacheEntry {
            result: LayerResult {
                layer: Layer::PropertyBased,
                status: Status::Success,
                violations: vec![],
                execution_time: Duration::from_millis(100),
                tool_outputs: vec![],
            },
            timestamp: chrono::Utc::now() - chrono::Duration::seconds(10),
            ttl: Duration::from_secs(5), // Expired
            metadata: CacheMetadata {
                file_size: 1024,
                execution_time: Duration::from_millis(100),
                memory_usage: 512 * 1024 * 1024,
                cache_hit_count: 0,
            },
        };

        manager.cache_mut().store(cache_key, expired_entry);

        let cleanup_result = manager.cleanup().unwrap();
        assert_eq!(cleanup_result.entries_removed, 1);
        assert_eq!(cleanup_result.entries_before, 1);
        assert_eq!(cleanup_result.entries_after, 0);
    }

    #[test]
    fn test_cache_manager_clear() {
        let temp_dir = TempDir::new().unwrap();
        let cache_dir = temp_dir.path().join("cache");

        let mut manager = CacheManager::with_cache_dir(cache_dir);

        // Add a test entry
        let cache_key = CacheKey {
            content_hash: ContentHash("test_hash".to_string()),
            config_hash: ConfigHash("config_hash".to_string()),
            tool_versions: ToolVersions {
                ferris_proof: "0.1.0".to_string(),
                external_tools: vec![],
            },
            layer: Layer::PropertyBased,
        };

        let cache_entry = CacheEntry {
            result: LayerResult {
                layer: Layer::PropertyBased,
                status: Status::Success,
                violations: vec![],
                execution_time: Duration::from_millis(100),
                tool_outputs: vec![],
            },
            timestamp: chrono::Utc::now(),
            ttl: Duration::from_secs(3600),
            metadata: CacheMetadata {
                file_size: 1024,
                execution_time: Duration::from_millis(100),
                memory_usage: 512 * 1024 * 1024,
                cache_hit_count: 0,
            },
        };

        manager.cache_mut().store(cache_key, cache_entry);

        let clear_result = manager.clear().unwrap();
        assert_eq!(clear_result.entries_removed, 1);

        let info_after = manager.info().unwrap();
        assert_eq!(info_after.total_entries, 0);
    }

    #[test]
    fn test_cache_manager_health_check() {
        let temp_dir = TempDir::new().unwrap();
        let cache_dir = temp_dir.path().join("cache");

        let manager = CacheManager::with_cache_dir(cache_dir);
        let health_report = manager.health_check().unwrap();

        assert!(health_report.integrity_errors.is_empty());
        assert!(!health_report.recommendations.is_empty());
        assert!(health_report.recommendations[0].contains("Cache is empty"));
    }
}
