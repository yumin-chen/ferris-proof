use ferris_proof_core::cache::{VerificationCache, CacheKey, ContentHash, ConfigHash, ToolVersions};
use ferris_proof_core::types::*;
use tempfile::TempDir;
use std::time::Duration;

#[test]
fn test_cache_basic_operations() {
    let temp_dir = TempDir::new().unwrap();
    let cache_dir = temp_dir.path().join("cache");
    
    let mut cache = VerificationCache::with_cache_dir(cache_dir);
    
    // Create test cache key
    let cache_key = CacheKey {
        content_hash: ContentHash("test_hash".to_string()),
        config_hash: ConfigHash("config_hash".to_string()),
        tool_versions: ToolVersions {
            ferris_proof: "0.1.0".to_string(),
            external_tools: vec![],
        },
        layer: Layer::PropertyBased,
    };
    
    // Create test cache entry
    let cache_entry = ferris_proof_core::cache::CacheEntry {
        result: LayerResult {
            layer: Layer::PropertyBased,
            status: Status::Success,
            violations: vec![],
            execution_time: Duration::from_millis(100),
            tool_outputs: vec![],
        },
        timestamp: chrono::Utc::now(),
        ttl: Duration::from_secs(3600),
        metadata: ferris_proof_core::cache::CacheMetadata {
            file_size: 1024,
            execution_time: Duration::from_millis(100),
            memory_usage: 512 * 1024 * 1024, // 512MB
            cache_hit_count: 0,
        },
    };
    
    // Test store operation
    cache.store(cache_key.clone(), cache_entry.clone());
    
    // Test get operation
    let retrieved_entry = cache.get(&cache_key);
    assert!(retrieved_entry.is_some());
    
    let retrieved = retrieved_entry.unwrap();
    assert_eq!(retrieved.result.status, Status::Success);
    assert_eq!(retrieved.result.layer, Layer::PropertyBased);
}

#[test]
fn test_cache_expiration() {
    let temp_dir = TempDir::new().unwrap();
    let cache_dir = temp_dir.path().join("cache");
    
    let mut cache = VerificationCache::with_cache_dir(cache_dir);
    
    let cache_key = CacheKey {
        content_hash: ContentHash("test_hash".to_string()),
        config_hash: ConfigHash("config_hash".to_string()),
        tool_versions: ToolVersions {
            ferris_proof: "0.1.0".to_string(),
            external_tools: vec![],
        },
        layer: Layer::PropertyBased,
    };
    
    // Create cache entry with very short TTL
    let cache_entry = ferris_proof_core::cache::CacheEntry {
        result: LayerResult {
            layer: Layer::PropertyBased,
            status: Status::Success,
            violations: vec![],
            execution_time: Duration::from_millis(100),
            tool_outputs: vec![],
        },
        timestamp: chrono::Utc::now() - chrono::Duration::seconds(10), // 10 seconds ago
        ttl: Duration::from_secs(5), // 5 second TTL (expired)
        metadata: ferris_proof_core::cache::CacheMetadata {
            file_size: 1024,
            execution_time: Duration::from_millis(100),
            memory_usage: 512 * 1024 * 1024,
            cache_hit_count: 0,
        },
    };
    
    cache.store(cache_key.clone(), cache_entry);
    
    // Should not retrieve expired entry
    let retrieved_entry = cache.get(&cache_key);
    assert!(retrieved_entry.is_none());
}

#[test]
fn test_cache_persistence() {
    let temp_dir = TempDir::new().unwrap();
    let cache_dir = temp_dir.path().join("cache");
    
    {
        let mut cache1 = VerificationCache::with_cache_dir(cache_dir.clone());
        
        let cache_key = CacheKey {
            content_hash: ContentHash("persistent_test".to_string()),
            config_hash: ConfigHash("config_hash".to_string()),
            tool_versions: ToolVersions {
                ferris_proof: "0.1.0".to_string(),
                external_tools: vec![],
            },
            layer: Layer::PropertyBased,
        };
        
        let cache_entry = ferris_proof_core::cache::CacheEntry {
            result: LayerResult {
                layer: Layer::PropertyBased,
                status: Status::Success,
                violations: vec![],
                execution_time: Duration::from_millis(100),
                tool_outputs: vec![],
            },
            timestamp: chrono::Utc::now(),
            ttl: Duration::from_secs(3600),
            metadata: ferris_proof_core::cache::CacheMetadata {
                file_size: 2048,
                execution_time: Duration::from_millis(200),
                memory_usage: 1024 * 1024 * 1024, // 1GB
                cache_hit_count: 0,
            },
        };
        
        cache1.store(cache_key.clone(), cache_entry);
        
        // Save to disk
        cache1.save_to_disk().unwrap();
    }
    
    // Create new cache instance and load from disk
    {
        let mut cache2 = VerificationCache::with_cache_dir(cache_dir);
        cache2.load_from_disk().unwrap();
        
        let cache_key = CacheKey {
            content_hash: ContentHash("persistent_test".to_string()),
            config_hash: ConfigHash("config_hash".to_string()),
            tool_versions: ToolVersions {
                ferris_proof: "0.1.0".to_string(),
                external_tools: vec![],
            },
            layer: Layer::PropertyBased,
        };
        
        let retrieved_entry = cache2.get(&cache_key);
        assert!(retrieved_entry.is_some());
        
        let retrieved = retrieved_entry.unwrap();
        assert_eq!(retrieved.result.status, Status::Success);
        assert_eq!(retrieved.metadata.file_size, 2048);
    }
}

#[test]
fn test_cache_cleanup() {
    let temp_dir = TempDir::new().unwrap();
    let cache_dir = temp_dir.path().join("cache");
    
    let mut cache = VerificationCache::with_cache_dir(cache_dir);
    
    // Add multiple cache entries
    for i in 0..5 {
        let cache_key = CacheKey {
            content_hash: ContentHash(format!("test_hash_{}", i)),
            config_hash: ConfigHash("config_hash".to_string()),
            tool_versions: ToolVersions {
                ferris_proof: "0.1.0".to_string(),
                external_tools: vec![],
            },
            layer: Layer::PropertyBased,
        };
        
        let cache_entry = ferris_proof_core::cache::CacheEntry {
            result: LayerResult {
                layer: Layer::PropertyBased,
                status: Status::Success,
                violations: vec![],
                execution_time: Duration::from_millis(100),
                tool_outputs: vec![],
            },
            timestamp: chrono::Utc::now() - chrono::Duration::seconds(i * 10), // Varying ages
            ttl: Duration::from_secs((5 + i * 5) as u64), // Some expired, some not
            metadata: ferris_proof_core::cache::CacheMetadata {
                file_size: 1024,
                execution_time: Duration::from_millis(100),
                memory_usage: 512 * 1024 * 1024,
                cache_hit_count: 0,
            },
        };
        
        cache.store(cache_key, cache_entry);
    }
    
    // Cleanup expired entries
    let expired_count = cache.cleanup_expired().unwrap();
    
    // Should have cleaned up some entries
    assert!(expired_count > 0);
}

#[test]
fn test_cache_statistics() {
    let temp_dir = TempDir::new().unwrap();
    let cache_dir = temp_dir.path().join("cache");
    
    let mut cache = VerificationCache::with_cache_dir(cache_dir);
    
    // Add some test entries
    for i in 0..3 {
        let cache_key = CacheKey {
            content_hash: ContentHash(format!("stats_test_{}", i)),
            config_hash: ConfigHash("config_hash".to_string()),
            tool_versions: ToolVersions {
                ferris_proof: "0.1.0".to_string(),
                external_tools: vec![],
            },
            layer: Layer::PropertyBased,
        };
        
        let cache_entry = ferris_proof_core::cache::CacheEntry {
            result: LayerResult {
                layer: Layer::PropertyBased,
                status: Status::Success,
                violations: vec![],
                execution_time: Duration::from_millis(100),
                tool_outputs: vec![],
            },
            timestamp: chrono::Utc::now(),
            ttl: Duration::from_secs(3600),
            metadata: ferris_proof_core::cache::CacheMetadata {
                file_size: 1024 * (i + 1),
                execution_time: Duration::from_millis(100 * (i + 1)),
                memory_usage: 512 * 1024 * 1024 * (i + 1),
                cache_hit_count: 0,
            },
        };
        
        cache.store(cache_key, cache_entry);
    }
    
    // Get statistics
    let stats = cache.statistics();
    
    assert_eq!(stats.total_entries, 3);
    assert_eq!(stats.expired_entries, 0); // All should be valid
    assert_eq!(stats.valid_entries, 3);
    assert_eq!(stats.total_size_bytes, 1024 + 2048 + 3072); // Sum of file sizes
}

#[test]
fn test_cache_hit_rate() {
    let cache = VerificationCache::new();
    
    // Test hit rate calculation
    assert_eq!(cache.hit_rate(80, 20), 0.8);
    assert_eq!(cache.hit_rate(0, 100), 0.0);
    assert_eq!(cache.hit_rate(100, 0), 1.0);
    assert_eq!(cache.hit_rate(0, 0), 0.0);
}

#[test]
fn test_cache_key_hash() {
    // Test that cache keys with different content produce different hashes
    let key1 = CacheKey {
        content_hash: ContentHash("hash1".to_string()),
        config_hash: ConfigHash("config_hash".to_string()),
        tool_versions: ToolVersions {
            ferris_proof: "0.1.0".to_string(),
            external_tools: vec![],
        },
        layer: Layer::PropertyBased,
    };
    
    let key2 = CacheKey {
        content_hash: ContentHash("hash2".to_string()),
        config_hash: ConfigHash("config_hash".to_string()),
        tool_versions: ToolVersions {
            ferris_proof: "0.1.0".to_string(),
            external_tools: vec![],
        },
        layer: Layer::PropertyBased,
    };
    
    // Different content hashes should produce different keys
    assert_ne!(key1, key2);
    
    use std::hash::{Hash, Hasher};
    let mut hasher1 = std::collections::hash_map::DefaultHasher::new();
    let mut hasher2 = std::collections::hash_map::DefaultHasher::new();
    key1.hash(&mut hasher1);
    key2.hash(&mut hasher2);
    assert_ne!(hasher1.finish(), hasher2.finish());
}

#[test]
fn test_cache_invalidation() {
    let temp_dir = TempDir::new().unwrap();
    let cache_dir = temp_dir.path().join("cache");
    
    let mut cache = VerificationCache::with_cache_dir(cache_dir);
    
    let cache_key = CacheKey {
        content_hash: ContentHash("invalidate_test".to_string()),
        config_hash: ConfigHash("config_hash".to_string()),
        tool_versions: ToolVersions {
            ferris_proof: "0.1.0".to_string(),
            external_tools: vec![],
        },
        layer: Layer::PropertyBased,
    };
    
    let cache_entry = ferris_proof_core::cache::CacheEntry {
        result: LayerResult {
            layer: Layer::PropertyBased,
            status: Status::Success,
            violations: vec![],
            execution_time: Duration::from_millis(100),
            tool_outputs: vec![],
        },
        timestamp: chrono::Utc::now(),
        ttl: Duration::from_secs(3600),
        metadata: ferris_proof_core::cache::CacheMetadata {
            file_size: 1024,
            execution_time: Duration::from_millis(100),
            memory_usage: 512 * 1024 * 1024,
            cache_hit_count: 0,
        },
    };
    
    // Store and verify
    cache.store(cache_key.clone(), cache_entry);
    assert!(cache.get(&cache_key).is_some());
    
    // Invalidate and verify removal
    cache.invalidate(&cache_key);
    assert!(cache.get(&cache_key).is_none());
}

#[test]
fn test_cache_clear() {
    let temp_dir = TempDir::new().unwrap();
    let cache_dir = temp_dir.path().join("cache");
    
    let mut cache = VerificationCache::with_cache_dir(cache_dir);
    
    // Add some entries
    for i in 0..5 {
        let cache_key = CacheKey {
            content_hash: ContentHash(format!("clear_test_{}", i)),
            config_hash: ConfigHash("config_hash".to_string()),
            tool_versions: ToolVersions {
                ferris_proof: "0.1.0".to_string(),
                external_tools: vec![],
            },
            layer: Layer::PropertyBased,
        };
        
        let cache_entry = ferris_proof_core::cache::CacheEntry {
            result: LayerResult {
                layer: Layer::PropertyBased,
                status: Status::Success,
                violations: vec![],
                execution_time: Duration::from_millis(100),
                tool_outputs: vec![],
            },
            timestamp: chrono::Utc::now(),
            ttl: Duration::from_secs(3600),
            metadata: ferris_proof_core::cache::CacheMetadata {
                file_size: 1024,
                execution_time: Duration::from_millis(100),
                memory_usage: 512 * 1024 * 1024,
                cache_hit_count: 0,
            },
        };
        
        cache.store(cache_key, cache_entry);
    }
    
    // Verify entries exist
    let stats_before = cache.statistics();
    assert_eq!(stats_before.total_entries, 5);
    
    // Clear all
    cache.clear();
    
    // Verify all entries are gone
    let stats_after = cache.statistics();
    assert_eq!(stats_after.total_entries, 0);
}
