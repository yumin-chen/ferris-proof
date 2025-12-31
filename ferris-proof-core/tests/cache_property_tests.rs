use ferris_proof_core::cache::{VerificationCache, CacheKey, ContentHash, ConfigHash, ToolVersions, CacheEntry, CacheMetadata};
use ferris_proof_core::types::*;
use ferris_proof_core::verification::Target;
use proptest::prelude::*;
use std::time::Duration;
use tempfile::TempDir;

// Property test generators
prop_compose! {
    fn arb_content_hash()(hash in "[a-f0-9]{64}") -> ContentHash {
        ContentHash(hash)
    }
}

prop_compose! {
    fn arb_config_hash()(hash in "[a-f0-9]{32}") -> ConfigHash {
        ConfigHash(hash)
    }
}

prop_compose! {
    fn arb_tool_versions()(
        ferris_proof in "[0-9]+\\.[0-9]+\\.[0-9]+",
        external_tools in prop::collection::vec(
            ("[a-zA-Z]+", "[0-9]+\\.[0-9]+\\.[0-9]+"),
            0..5
        )
    ) -> ToolVersions {
        ToolVersions {
            ferris_proof,
            external_tools,
        }
    }
}

prop_compose! {
    fn arb_layer()(layer in prop::sample::select(vec![
        Layer::Formal,
        Layer::TypeLevel,
        Layer::PropertyBased,
        Layer::Monitoring,
    ])) -> Layer {
        layer
    }
}

prop_compose! {
    fn arb_status()(status in prop::sample::select(vec![
        Status::Success,
        Status::Warning,
        Status::Error,
        Status::Skipped,
    ])) -> Status {
        status
    }
}

prop_compose! {
    fn arb_cache_key()(
        content_hash in arb_content_hash(),
        config_hash in arb_config_hash(),
        tool_versions in arb_tool_versions(),
        layer in arb_layer(),
    ) -> CacheKey {
        CacheKey {
            content_hash,
            config_hash,
            tool_versions,
            layer,
        }
    }
}

prop_compose! {
    fn arb_layer_result()(
        layer in arb_layer(),
        status in arb_status(),
        execution_time_ms in 1u64..10000u64,
    ) -> LayerResult {
        LayerResult {
            layer,
            status,
            violations: vec![], // Simplified for property testing
            execution_time: Duration::from_millis(execution_time_ms),
            tool_outputs: vec![], // Simplified for property testing
        }
    }
}

prop_compose! {
    fn arb_cache_metadata()(
        file_size in 1u64..1_000_000u64,
        execution_time_ms in 1u64..10000u64,
        memory_usage in 1u64..1_000_000_000u64,
        cache_hit_count in 0u32..1000u32,
    ) -> CacheMetadata {
        CacheMetadata {
            file_size,
            execution_time: Duration::from_millis(execution_time_ms),
            memory_usage,
            cache_hit_count,
        }
    }
}

prop_compose! {
    fn arb_cache_entry()(
        result in arb_layer_result(),
        ttl_secs in 1u64..86400u64, // 1 second to 1 day
        metadata in arb_cache_metadata(),
    ) -> CacheEntry {
        CacheEntry {
            result,
            timestamp: chrono::Utc::now(),
            ttl: Duration::from_secs(ttl_secs),
            metadata,
        }
    }
}

proptest! {
    /// **Feature: ferris-proof, Property 10: Verification result caching**
    /// **Validates: Requirements 11.3**
    /// 
    /// For any verification target that has not changed since the last verification,
    /// the system should reuse cached results instead of re-running verification
    #[test]
    fn verification_result_caching_property(
        cache_key in arb_cache_key(),
        cache_entry in arb_cache_entry(),
    ) {
        let temp_dir = TempDir::new().unwrap();
        let cache_dir = temp_dir.path().join("cache");
        let mut cache = VerificationCache::with_cache_dir(cache_dir);
        
        // Store the cache entry
        cache.store(cache_key.clone(), cache_entry.clone());
        
        // Retrieve the same cache entry
        let retrieved = cache.get(&cache_key);
        
        // Property: If we store a cache entry, we should be able to retrieve it
        // as long as it hasn't expired
        prop_assert!(retrieved.is_some(), "Stored cache entry should be retrievable");
        
        let retrieved_entry = retrieved.unwrap();
        
        // Property: Retrieved entry should have the same result status and layer
        prop_assert_eq!(retrieved_entry.result.status, cache_entry.result.status);
        prop_assert_eq!(retrieved_entry.result.layer, cache_entry.result.layer);
        
        // Property: Retrieved entry should have the same metadata
        prop_assert_eq!(retrieved_entry.metadata.file_size, cache_entry.metadata.file_size);
        prop_assert_eq!(retrieved_entry.metadata.memory_usage, cache_entry.metadata.memory_usage);
    }
    
    /// Property: Cache keys with different content should produce different cache entries
    #[test]
    fn cache_key_uniqueness_property(
        key1 in arb_cache_key(),
        key2 in arb_cache_key(),
        entry1 in arb_cache_entry(),
        entry2 in arb_cache_entry(),
    ) {
        // Skip if keys are identical
        prop_assume!(key1 != key2);
        
        let temp_dir = TempDir::new().unwrap();
        let cache_dir = temp_dir.path().join("cache");
        let mut cache = VerificationCache::with_cache_dir(cache_dir);
        
        // Store two different entries with different keys
        cache.store(key1.clone(), entry1.clone());
        cache.store(key2.clone(), entry2.clone());
        
        // Property: Different keys should retrieve different entries
        let retrieved1 = cache.get(&key1);
        let retrieved2 = cache.get(&key2);
        
        prop_assert!(retrieved1.is_some());
        prop_assert!(retrieved2.is_some());
        
        // If the entries were different, they should remain different
        if entry1.result.status != entry2.result.status || 
           entry1.result.layer != entry2.result.layer {
            let r1 = retrieved1.unwrap();
            let r2 = retrieved2.unwrap();
            prop_assert!(
                r1.result.status != r2.result.status || 
                r1.result.layer != r2.result.layer,
                "Different cache keys should preserve different entries"
            );
        }
    }
    
    /// Property: Cache invalidation should remove entries
    #[test]
    fn cache_invalidation_property(
        cache_key in arb_cache_key(),
        cache_entry in arb_cache_entry(),
    ) {
        let temp_dir = TempDir::new().unwrap();
        let cache_dir = temp_dir.path().join("cache");
        let mut cache = VerificationCache::with_cache_dir(cache_dir);
        
        // Store entry
        cache.store(cache_key.clone(), cache_entry);
        
        // Verify it exists
        prop_assert!(cache.get(&cache_key).is_some());
        
        // Invalidate
        cache.invalidate(&cache_key);
        
        // Property: After invalidation, entry should not be retrievable
        prop_assert!(cache.get(&cache_key).is_none(), "Invalidated entry should not be retrievable");
    }
    
    /// Property: Cache statistics should be consistent with stored entries
    #[test]
    fn cache_statistics_consistency_property(
        cache_entries in prop::collection::vec(
            (arb_cache_key(), arb_cache_entry()),
            1..10
        )
    ) {
        let temp_dir = TempDir::new().unwrap();
        let cache_dir = temp_dir.path().join("cache");
        let mut cache = VerificationCache::with_cache_dir(cache_dir);
        
        // Store all entries
        let mut expected_total_size = 0u64;
        let mut unique_keys = std::collections::HashSet::new();
        
        for (key, entry) in &cache_entries {
            if unique_keys.insert(key.clone()) {
                cache.store(key.clone(), entry.clone());
                expected_total_size += entry.metadata.file_size;
            }
        }
        
        let stats = cache.statistics();
        
        // Property: Statistics should reflect the actual stored entries
        prop_assert_eq!(stats.total_entries, unique_keys.len());
        prop_assert_eq!(stats.total_size_bytes, expected_total_size);
        
        // Property: Valid entries + expired entries should equal total entries
        prop_assert_eq!(stats.valid_entries + stats.expired_entries, stats.total_entries);
    }
    
    /// Property: Cache persistence should preserve entries across instances
    #[test]
    fn cache_persistence_property(
        cache_key in arb_cache_key(),
        cache_entry in arb_cache_entry(),
    ) {
        let temp_dir = TempDir::new().unwrap();
        let cache_dir = temp_dir.path().join("cache");
        
        // First cache instance - store and save
        {
            let mut cache1 = VerificationCache::with_cache_dir(cache_dir.clone());
            cache1.store(cache_key.clone(), cache_entry.clone());
            cache1.save_to_disk().unwrap();
        }
        
        // Second cache instance - load and retrieve
        {
            let mut cache2 = VerificationCache::with_cache_dir(cache_dir);
            cache2.load_from_disk().unwrap();
            
            let retrieved = cache2.get(&cache_key);
            
            // Property: Persisted entries should be retrievable after loading
            prop_assert!(retrieved.is_some(), "Persisted entry should be retrievable after loading");
            
            let retrieved_entry = retrieved.unwrap();
            prop_assert_eq!(retrieved_entry.result.status, cache_entry.result.status);
            prop_assert_eq!(retrieved_entry.result.layer, cache_entry.result.layer);
        }
    }
}

#[cfg(test)]
mod cache_content_hash_tests {
    use super::*;
    use std::fs;
    
    proptest! {
        /// Property: Identical Rust file content should produce identical cache keys
        #[test]
        fn rust_file_content_hash_consistency(
            rust_code in r#"fn [a-z_][a-z0-9_]*\(\) \{[^}]*\}"#,
        ) {
            let temp_dir = TempDir::new().unwrap();
            
            // Create two identical files
            let file1 = temp_dir.path().join("test1.rs");
            let file2 = temp_dir.path().join("test2.rs");
            
            fs::write(&file1, &rust_code).unwrap();
            fs::write(&file2, &rust_code).unwrap();
            
            let target1 = Target::RustFile(file1);
            let target2 = Target::RustFile(file2);
            
            let key1_result = CacheKey::new(&target1, Layer::PropertyBased, "test_config");
            let key2_result = CacheKey::new(&target2, Layer::PropertyBased, "test_config");
            
            // Both should succeed or both should fail
            match (key1_result, key2_result) {
                (Ok(key1), Ok(key2)) => {
                    // Property: Identical content should produce identical content hashes
                    prop_assert_eq!(key1.content_hash, key2.content_hash);
                }
                (Err(_), Err(_)) => {
                    // Both failed - this is acceptable for invalid Rust code
                }
                _ => {
                    prop_assert!(false, "Inconsistent parsing results for identical content");
                }
            }
        }
        
        /// Property: Different Rust file content should produce different cache keys
        #[test]
        fn rust_file_content_hash_uniqueness(
            rust_code1 in r#"fn [a-z_][a-z0-9_]*\(\) \{[^}]*\}"#,
            rust_code2 in r#"fn [a-z_][a-z0-9_]*\(\) \{[^}]*\}"#,
        ) {
            prop_assume!(rust_code1 != rust_code2);
            
            let temp_dir = TempDir::new().unwrap();
            
            let file1 = temp_dir.path().join("test1.rs");
            let file2 = temp_dir.path().join("test2.rs");
            
            fs::write(&file1, &rust_code1).unwrap();
            fs::write(&file2, &rust_code2).unwrap();
            
            let target1 = Target::RustFile(file1);
            let target2 = Target::RustFile(file2);
            
            let key1_result = CacheKey::new(&target1, Layer::PropertyBased, "test_config");
            let key2_result = CacheKey::new(&target2, Layer::PropertyBased, "test_config");
            
            if let (Ok(key1), Ok(key2)) = (key1_result, key2_result) {
                // Property: Different content should produce different content hashes
                prop_assert_ne!(key1.content_hash, key2.content_hash);
            }
        }
    }
}