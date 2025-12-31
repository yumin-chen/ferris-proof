use crate::types::*;
use anyhow::{anyhow, Result};
use blake3::Hasher;
use quote::ToTokens;
use regex::Regex;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use syn::{parse_file, Attribute, File, Item, ItemEnum, ItemFn, ItemMod, ItemStruct};

#[derive(Debug, Clone, Hash, Eq, PartialEq, Serialize, Deserialize)]
pub struct CacheKey {
    pub content_hash: ContentHash,
    pub config_hash: ConfigHash,
    pub tool_versions: ToolVersions,
    pub layer: Layer,
}

impl CacheKey {
    /// Create a new cache key for the given target and layer
    pub fn new(
        target: &crate::verification::Target,
        layer: Layer,
        config_hash: &str,
    ) -> Result<Self> {
        let content_hash = Self::compute_content_hash(target)?;
        let tool_versions = Self::get_tool_versions()?;

        Ok(Self {
            content_hash,
            config_hash: ConfigHash(config_hash.to_string()),
            tool_versions,
            layer,
        })
    }

    /// Compute content hash for a verification target
    fn compute_content_hash(target: &crate::verification::Target) -> Result<ContentHash> {
        let mut hasher = Hasher::new();

        match target {
            crate::verification::Target::RustFile(path) => {
                let ast = parse_file(
                    &std::fs::read_to_string(path)
                        .map_err(|e| anyhow!("Failed to read Rust file {:?}: {}", path, e))?,
                )?;
                let normalized = Self::normalize_ast(&ast);
                hasher.update(normalized.as_bytes());
            }
            crate::verification::Target::FormalSpec(path) => {
                let spec = std::fs::read_to_string(path)
                    .map_err(|e| anyhow!("Failed to read formal spec {:?}: {}", path, e))?;
                let normalized = Self::normalize_spec(&spec);
                hasher.update(normalized.as_bytes());
            }
            crate::verification::Target::Module(module_path) => {
                // For modules, we hash the module path directly
                // TODO: Implement module content hashing
                hasher.update(module_path.as_bytes());
            }
        }

        Ok(ContentHash(hex::encode(hasher.finalize().as_bytes())))
    }

    /// Normalize Rust AST by removing comments, whitespace, and other irrelevant details
    fn normalize_ast(ast: &File) -> String {
        let mut normalized_items = Vec::new();

        for item in &ast.items {
            match item {
                Item::Fn(item_fn) => {
                    // Normalize function signature and body structure
                    let normalized_item = Self::normalize_function(item_fn);
                    normalized_items.push(Item::Fn(normalized_item));
                }
                Item::Struct(item_struct) => {
                    // Normalize struct definition
                    let normalized_item = Self::normalize_struct(item_struct);
                    normalized_items.push(Item::Struct(normalized_item));
                }
                Item::Enum(item_enum) => {
                    // Normalize enum definition
                    let normalized_item = Self::normalize_enum(item_enum);
                    normalized_items.push(Item::Enum(normalized_item));
                }
                Item::Mod(item_mod) => {
                    // Normalize module declaration
                    let normalized_item = Self::normalize_module(item_mod);
                    normalized_items.push(Item::Mod(normalized_item));
                }
                _ => {
                    // Include other items as-is for now
                    normalized_items.push(item.clone());
                }
            }
        }

        // Reconstruct the normalized AST and convert to string
        let normalized_file = File {
            shebang: ast.shebang.clone(),
            attrs: ast.attrs.clone(),
            items: normalized_items,
        };

        normalized_file.to_token_stream().to_string()
    }

    /// Normalize a function item
    fn normalize_function(item_fn: &ItemFn) -> ItemFn {
        // Remove attributes that don't affect verification
        let attrs: Vec<Attribute> = item_fn
            .attrs
            .iter()
            .filter(|attr| {
                // Keep verification attributes, remove others like #[cfg(test)]
                attr.path().is_ident("verification")
                    || !attr
                        .path()
                        .segments
                        .first()
                        .map(|seg| seg.ident == "cfg")
                        .unwrap_or(false)
            })
            .cloned()
            .collect();

        // Normalize function body by removing comments and normalizing whitespace
        let normalized_block = Self::normalize_block(&item_fn.block);

        ItemFn {
            attrs,
            vis: item_fn.vis.clone(),
            sig: item_fn.sig.clone(),
            block: Box::new(normalized_block),
        }
    }

    /// Normalize a block by removing comments and normalizing structure
    fn normalize_block(block: &syn::Block) -> syn::Block {
        use syn::Stmt;

        let mut normalized_stmts = Vec::new();

        for stmt in &block.stmts {
            match stmt {
                Stmt::Local(_local) => {
                    // Keep local variable declarations as-is for now
                    normalized_stmts.push(stmt.clone());
                }
                Stmt::Item(_item) => {
                    // Recursively normalize nested items
                    normalized_stmts.push(stmt.clone());
                }
                Stmt::Expr(expr, semi) => {
                    // Keep expressions but could normalize further
                    normalized_stmts.push(Stmt::Expr(expr.clone(), *semi));
                }
                Stmt::Macro(_mac) => {
                    // Keep macro calls as-is
                    normalized_stmts.push(stmt.clone());
                }
            }
        }

        syn::Block {
            brace_token: block.brace_token,
            stmts: normalized_stmts,
        }
    }

    /// Normalize a struct item
    fn normalize_struct(item_struct: &ItemStruct) -> ItemStruct {
        // Remove attributes that don't affect verification
        let attrs: Vec<Attribute> = item_struct
            .attrs
            .iter()
            .filter(|attr| attr.path().is_ident("verification"))
            .cloned()
            .collect();

        ItemStruct {
            attrs,
            vis: item_struct.vis.clone(),
            struct_token: item_struct.struct_token,
            ident: item_struct.ident.clone(),
            generics: item_struct.generics.clone(),
            fields: item_struct.fields.clone(),
            semi_token: item_struct.semi_token,
        }
    }

    /// Normalize an enum item
    fn normalize_enum(item_enum: &ItemEnum) -> ItemEnum {
        // Remove attributes that don't affect verification
        let attrs: Vec<Attribute> = item_enum
            .attrs
            .iter()
            .filter(|attr| attr.path().is_ident("verification"))
            .cloned()
            .collect();

        ItemEnum {
            attrs,
            vis: item_enum.vis.clone(),
            enum_token: item_enum.enum_token,
            ident: item_enum.ident.clone(),
            generics: item_enum.generics.clone(),
            brace_token: item_enum.brace_token,
            variants: item_enum.variants.clone(),
        }
    }

    /// Normalize a module item
    fn normalize_module(item_mod: &ItemMod) -> ItemMod {
        // Remove attributes that don't affect verification
        let attrs: Vec<Attribute> = item_mod
            .attrs
            .iter()
            .filter(|attr| attr.path().is_ident("verification"))
            .cloned()
            .collect();

        ItemMod {
            attrs,
            vis: item_mod.vis.clone(),
            unsafety: item_mod.unsafety,
            mod_token: item_mod.mod_token,
            ident: item_mod.ident.clone(),
            content: item_mod.content.clone(),
            semi: item_mod.semi,
        }
    }

    /// Normalize formal specification content
    fn normalize_spec(spec: &str) -> String {
        // Remove comments and normalize whitespace for TLA+ and Alloy specs
        let mut normalized = String::new();
        let mut chars = spec.chars().peekable();
        let mut in_line_comment = false;
        let mut in_block_comment = false;
        let mut in_string = false;
        let mut escape_next = false;

        while let Some(char) = chars.next() {
            if escape_next {
                if in_string {
                    normalized.push('\\');
                    normalized.push(char);
                }
                escape_next = false;
                continue;
            }

            match char {
                '\\' if in_string => {
                    escape_next = true;
                    continue;
                }
                '"' if !in_line_comment && !in_block_comment => {
                    in_string = !in_string;
                    normalized.push(char);
                }
                '/' if !in_string && !in_line_comment && !in_block_comment => {
                    if let Some(&next_char) = chars.peek() {
                        if next_char == '/' {
                            // Start of line comment
                            chars.next(); // consume the second '/'
                            in_line_comment = true;
                            continue;
                        } else if next_char == '*' {
                            // Start of block comment
                            chars.next(); // consume the '*'
                            in_block_comment = true;
                            continue;
                        }
                    }
                    normalized.push(char);
                }
                '*' if in_block_comment && !in_string => {
                    if let Some(&next_char) = chars.peek() {
                        if next_char == '/' {
                            // End of block comment
                            chars.next(); // consume the '/'
                            in_block_comment = false;
                            continue;
                        }
                    }
                }
                '\n' | '\r' => {
                    if in_line_comment {
                        in_line_comment = false;
                    }
                    if !in_block_comment && !in_string {
                        // Normalize line breaks to single space
                        if !normalized.ends_with(' ') && !normalized.is_empty() {
                            normalized.push(' ');
                        }
                    }
                }
                c if c.is_whitespace() => {
                    if !in_line_comment && !in_block_comment && !in_string {
                        // Normalize whitespace to single space
                        if !normalized.ends_with(' ') && !normalized.is_empty() {
                            normalized.push(' ');
                        }
                    }
                }
                c => {
                    if !in_line_comment && !in_block_comment {
                        normalized.push(c);
                    }
                }
            }
        }

        normalized.trim().to_string()
    }

    /// Get current tool versions for cache invalidation
    fn get_tool_versions() -> Result<ToolVersions> {
        let ferris_proof_version = env!("CARGO_PKG_VERSION").to_string();
        let mut external_tools = Vec::new();

        // Check for common external tools with their version commands
        let tools_to_check = vec![
            ("tlc", "TLA+ TLC", vec!["--version", "-version"]),
            ("java", "Java (for TLA+)", vec!["-version"]),
            ("alloy", "Alloy Analyzer", vec!["--version"]),
            ("kani", "Kani Verifier", vec!["--version"]),
            ("cargo", "Cargo", vec!["--version"]),
            ("rustc", "Rust Compiler", vec!["--version"]),
        ];

        for (tool_name, display_name, version_args) in tools_to_check {
            if let Ok(version) = Self::get_tool_version_with_args(tool_name, &version_args) {
                external_tools.push((display_name.to_string(), version));
            }
        }

        // Add Rust toolchain information
        if let Ok(rustc_version) = Self::get_rustc_commit_hash() {
            external_tools.push(("Rust Commit".to_string(), rustc_version));
        }

        Ok(ToolVersions {
            ferris_proof: ferris_proof_version,
            external_tools,
        })
    }

    /// Get version of an external tool with multiple possible version arguments
    fn get_tool_version_with_args(tool_name: &str, version_args: &[&str]) -> Result<String> {
        for &arg in version_args {
            if let Ok(version) = Self::get_tool_version_with_arg(tool_name, arg) {
                return Ok(version);
            }
        }
        Err(anyhow!(
            "Could not determine version for tool: {}",
            tool_name
        ))
    }

    /// Get version of an external tool with a specific argument
    fn get_tool_version_with_arg(tool_name: &str, version_arg: &str) -> Result<String> {
        let output = std::process::Command::new(tool_name)
            .arg(version_arg)
            .output();

        match output {
            Ok(result) => {
                let stdout = String::from_utf8_lossy(&result.stdout);
                let stderr = String::from_utf8_lossy(&result.stderr);

                // Try stdout first, then stderr (some tools output version to stderr)
                let version_text = if !stdout.trim().is_empty() {
                    stdout
                } else {
                    stderr
                };

                // Extract version string (improved parsing)
                let version_line = version_text.lines().next().unwrap_or("");
                let version = Self::extract_version_from_line(version_line);

                if !version.is_empty() {
                    Ok(version)
                } else {
                    Err(anyhow!(
                        "Could not parse version from output: {}",
                        version_line
                    ))
                }
            }
            Err(e) => Err(anyhow!("Failed to execute tool {}: {}", tool_name, e)),
        }
    }

    /// Extract version string from a line of text
    fn extract_version_from_line(line: &str) -> String {
        // Look for semantic version patterns (x.y.z)
        if let Ok(version_regex) = Regex::new(r"\b(\d+\.\d+\.\d+(?:-[a-zA-Z0-9.-]+)?)\b") {
            if let Some(captures) = version_regex.captures(line) {
                return captures.get(1).unwrap().as_str().to_string();
            }
        }

        // Look for simpler version patterns (x.y)
        if let Ok(simple_version_regex) = Regex::new(r"\b(\d+\.\d+)\b") {
            if let Some(captures) = simple_version_regex.captures(line) {
                return captures.get(1).unwrap().as_str().to_string();
            }
        }

        // Fallback: look for any sequence of digits and dots
        if let Ok(fallback_regex) = Regex::new(r"\b(\d+(?:\.\d+)*)\b") {
            if let Some(captures) = fallback_regex.captures(line) {
                return captures.get(1).unwrap().as_str().to_string();
            }
        }

        "unknown".to_string()
    }

    /// Get Rust compiler commit hash for more precise cache invalidation
    fn get_rustc_commit_hash() -> Result<String> {
        let output = std::process::Command::new("rustc")
            .arg("--version")
            .arg("--verbose")
            .output();

        match output {
            Ok(result) if result.status.success() => {
                let stdout = String::from_utf8_lossy(&result.stdout);
                for line in stdout.lines() {
                    if line.starts_with("commit-hash:") {
                        return Ok(line
                            .split(':')
                            .nth(1)
                            .unwrap_or("unknown")
                            .trim()
                            .to_string());
                    }
                }
                Err(anyhow!("Could not find commit hash in rustc output"))
            }
            Ok(result) => Err(anyhow!(
                "rustc returned non-zero exit code: {}",
                result.status
            )),
            Err(e) => Err(anyhow!("Failed to execute rustc: {}", e)),
        }
    }
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
    persistent_storage: Option<PersistentStorage>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CacheEntry {
    pub result: LayerResult,
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub ttl: std::time::Duration,
    pub metadata: CacheMetadata,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CacheMetadata {
    pub file_size: u64,
    pub execution_time: std::time::Duration,
    pub memory_usage: u64,
    pub cache_hit_count: u32,
}

pub struct PersistentStorage {
    cache_dir: PathBuf,
}

impl VerificationCache {
    pub fn new() -> Self {
        let cache_dir = std::env::temp_dir().join("ferris-proof-cache");
        std::fs::create_dir_all(&cache_dir).ok();

        let persistent_storage = PersistentStorage::new(&cache_dir);

        Self {
            cache_dir: cache_dir.clone(),
            entries: HashMap::new(),
            persistent_storage: Some(persistent_storage),
        }
    }

    pub fn with_cache_dir(cache_dir: PathBuf) -> Self {
        std::fs::create_dir_all(&cache_dir).ok();

        let persistent_storage = PersistentStorage::new(&cache_dir);

        Self {
            cache_dir: cache_dir.clone(),
            entries: HashMap::new(),
            persistent_storage: Some(persistent_storage),
        }
    }

    /// Get cache entry, checking for expiration and validity
    pub fn get(&self, key: &CacheKey) -> Option<&CacheEntry> {
        if let Some(entry) = self.entries.get(key) {
            if self.is_entry_valid(entry) {
                return Some(entry);
            }
        }
        None
    }

    /// Check if a cache entry is still valid (not expired)
    fn is_entry_valid(&self, entry: &CacheEntry) -> bool {
        let now = chrono::Utc::now();
        let age = now.signed_duration_since(entry.timestamp);
        age.to_std().unwrap_or(std::time::Duration::MAX) < entry.ttl
    }

    /// Store cache entry with TTL and automatic persistence
    pub fn store(&mut self, key: CacheKey, entry: CacheEntry) {
        self.entries.insert(key.clone(), entry.clone());

        // Persist to disk if persistent storage is available
        if let Some(storage) = &self.persistent_storage {
            if let Err(e) = storage.store(&key, &entry) {
                tracing::warn!("Failed to persist cache entry: {}", e);
            }
        }
    }

    /// Invalidate cache entry (remove from memory and disk)
    pub fn invalidate(&mut self, key: &CacheKey) {
        self.entries.remove(key);

        if let Some(storage) = &self.persistent_storage {
            if let Err(e) = storage.remove(key) {
                tracing::warn!("Failed to remove cache entry from disk: {}", e);
            }
        }
    }

    /// Clear all cache entries (memory and disk)
    pub fn clear(&mut self) {
        self.entries.clear();

        if let Some(storage) = &self.persistent_storage {
            if let Err(e) = storage.clear() {
                tracing::warn!("Failed to clear cache from disk: {}", e);
            }
        }
    }

    /// Load cache from persistent storage with validation
    pub fn load_from_disk(&mut self) -> Result<()> {
        if let Some(storage) = &self.persistent_storage {
            let loaded_entries = storage.load_all()?;

            // Filter out expired entries during load
            let _now = chrono::Utc::now();
            for (key, entry) in loaded_entries {
                if self.is_entry_valid(&entry) {
                    self.entries.insert(key, entry);
                } else {
                    // Remove expired entries from disk
                    let _ = storage.remove(&key);
                }
            }
        }
        Ok(())
    }

    /// Save cache to persistent storage
    pub fn save_to_disk(&self) -> Result<()> {
        if let Some(storage) = &self.persistent_storage {
            storage.save_all(&self.entries)?;
        }
        Ok(())
    }

    /// Cleanup expired entries from memory and disk
    pub fn cleanup_expired(&mut self) -> Result<usize> {
        let now = chrono::Utc::now();
        let mut expired_keys = Vec::new();

        for (key, entry) in &self.entries {
            let age = now.signed_duration_since(entry.timestamp);
            if age.to_std().unwrap_or(std::time::Duration::MAX) >= entry.ttl {
                expired_keys.push(key.clone());
            }
        }

        let expired_count = expired_keys.len();
        for key in expired_keys {
            self.invalidate(&key);
        }

        Ok(expired_count)
    }

    /// Get comprehensive cache statistics
    pub fn statistics(&self) -> CacheStatistics {
        let total_entries = self.entries.len();
        let mut expired_entries = 0;
        let mut total_size = 0u64;
        let now = chrono::Utc::now();

        for entry in self.entries.values() {
            let age = now.signed_duration_since(entry.timestamp);
            if age.to_std().unwrap_or(std::time::Duration::MAX) >= entry.ttl {
                expired_entries += 1;
            }
            total_size += entry.metadata.file_size;
        }

        CacheStatistics {
            total_entries,
            expired_entries,
            valid_entries: total_entries - expired_entries,
            total_size_bytes: total_size,
            cache_dir: self.cache_dir.clone(),
        }
    }

    /// Calculate cache hit rate
    pub fn hit_rate(&self, hits: u64, misses: u64) -> f64 {
        if hits + misses == 0 {
            0.0
        } else {
            hits as f64 / (hits + misses) as f64
        }
    }

    /// Validate cache integrity and return any errors found
    pub fn validate_integrity(&self) -> Result<Vec<String>> {
        if let Some(storage) = &self.persistent_storage {
            storage.validate()
        } else {
            Ok(Vec::new())
        }
    }

    /// Get total cache size on disk
    pub fn disk_size(&self) -> Result<u64> {
        if let Some(storage) = &self.persistent_storage {
            storage.cache_size()
        } else {
            Ok(0)
        }
    }

    /// Compact cache by removing expired entries and optimizing storage
    pub fn compact(&mut self) -> Result<CompactionResult> {
        let initial_entries = self.entries.len();
        let initial_size = self.disk_size().unwrap_or(0);

        // Remove expired entries
        let expired_removed = self.cleanup_expired()?;

        // Save compacted cache to disk
        self.save_to_disk()?;

        let final_entries = self.entries.len();
        let final_size = self.disk_size().unwrap_or(0);

        Ok(CompactionResult {
            entries_before: initial_entries,
            entries_after: final_entries,
            entries_removed: expired_removed,
            size_before: initial_size,
            size_after: final_size,
            size_saved: initial_size.saturating_sub(final_size),
        })
    }
}

impl PersistentStorage {
    fn new(cache_dir: &Path) -> Self {
        Self {
            cache_dir: cache_dir.to_path_buf(),
        }
    }

    fn store(&self, key: &CacheKey, entry: &CacheEntry) -> Result<()> {
        let file_name = self.key_to_filename(key);
        let file_path = self.cache_dir.join(file_name);

        // Create cache directory if it doesn't exist
        std::fs::create_dir_all(&self.cache_dir)?;

        let serialized = bincode::serialize(&(key, entry))?;

        // Use zstd compression with level 3 for good balance of speed/compression
        let compressed = zstd::encode_all(serialized.as_slice(), 3)?;

        // Write atomically using a temporary file
        let temp_path = file_path.with_extension("tmp");
        std::fs::write(&temp_path, compressed)?;
        std::fs::rename(temp_path, file_path)?;

        Ok(())
    }

    fn remove(&self, key: &CacheKey) -> Result<()> {
        let file_name = self.key_to_filename(key);
        let file_path = self.cache_dir.join(file_name);

        if file_path.exists() {
            std::fs::remove_file(file_path)?;
        }
        Ok(())
    }

    fn load_all(&self) -> Result<HashMap<CacheKey, CacheEntry>> {
        let mut entries = HashMap::new();

        if !self.cache_dir.exists() {
            return Ok(entries);
        }

        // Recursively search for cache files in subdirectories
        fn visit_dir(
            dir: &Path,
            entries: &mut HashMap<CacheKey, CacheEntry>,
            storage: &PersistentStorage,
        ) -> Result<()> {
            for entry in std::fs::read_dir(dir)? {
                let entry = entry?;
                let path = entry.path();

                if path.is_dir() {
                    // Recursively visit subdirectories
                    visit_dir(&path, entries, storage)?;
                } else if path.is_file()
                    && path.extension().and_then(|s| s.to_str()) == Some("cache")
                {
                    match storage.load_entry(&path) {
                        Ok((key, cache_entry)) => {
                            entries.insert(key, cache_entry);
                        }
                        Err(e) => {
                            tracing::warn!("Failed to load cache entry {:?}: {}", path, e);
                            // Optionally remove corrupted cache files
                            let _ = std::fs::remove_file(&path);
                        }
                    }
                }
            }
            Ok(())
        }

        visit_dir(&self.cache_dir, &mut entries, self)?;
        Ok(entries)
    }

    fn save_all(&self, entries: &HashMap<CacheKey, CacheEntry>) -> Result<()> {
        std::fs::create_dir_all(&self.cache_dir)?;

        for (key, entry) in entries {
            if let Err(e) = self.store(key, entry) {
                tracing::warn!("Failed to save cache entry: {}", e);
                // Continue with other entries even if one fails
            }
        }
        Ok(())
    }

    fn clear(&self) -> Result<()> {
        if !self.cache_dir.exists() {
            return Ok(());
        }

        // Recursively remove all cache files
        fn visit_dir(dir: &Path) -> Result<()> {
            for entry in std::fs::read_dir(dir)? {
                let entry = entry?;
                let path = entry.path();

                if path.is_dir() {
                    // Recursively visit subdirectories
                    visit_dir(&path)?;
                    // Try to remove empty directory
                    let _ = std::fs::remove_dir(&path);
                } else if path.is_file()
                    && path.extension().and_then(|s| s.to_str()) == Some("cache")
                {
                    if let Err(e) = std::fs::remove_file(&path) {
                        tracing::warn!("Failed to remove cache file {:?}: {}", path, e);
                    }
                }
            }
            Ok(())
        }

        visit_dir(&self.cache_dir)?;
        Ok(())
    }

    fn load_entry(&self, path: &Path) -> Result<(CacheKey, CacheEntry)> {
        let compressed = std::fs::read(path)?;
        let serialized = zstd::decode_all(compressed.as_slice())?;
        let (key, entry): (CacheKey, CacheEntry) = bincode::deserialize(&serialized)?;
        Ok((key, entry))
    }

    /// Generate content-addressed filename from cache key
    fn key_to_filename(&self, key: &CacheKey) -> String {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};

        let mut hasher = DefaultHasher::new();
        key.hash(&mut hasher);
        let hash = hasher.finish();

        // Use content-addressed storage: first two hex digits as subdirectory
        let subdir = format!("{:02x}", (hash >> 56) & 0xFF);
        let filename = format!("{:016x}.cache", hash);

        // Create subdirectory path
        let subdir_path = self.cache_dir.join(&subdir);
        if let Err(e) = std::fs::create_dir_all(&subdir_path) {
            tracing::warn!("Failed to create cache subdirectory {}: {}", subdir, e);
        }

        format!("{}/{}", subdir, filename)
    }

    /// Get cache directory size in bytes
    pub fn cache_size(&self) -> Result<u64> {
        let mut total_size = 0u64;

        if !self.cache_dir.exists() {
            return Ok(0);
        }

        fn visit_dir(dir: &Path, total: &mut u64) -> Result<()> {
            for entry in std::fs::read_dir(dir)? {
                let entry = entry?;
                let path = entry.path();

                if path.is_dir() {
                    visit_dir(&path, total)?;
                } else if path.is_file() {
                    if let Ok(metadata) = entry.metadata() {
                        *total += metadata.len();
                    }
                }
            }
            Ok(())
        }

        visit_dir(&self.cache_dir, &mut total_size)?;
        Ok(total_size)
    }

    /// Validate cache integrity
    pub fn validate(&self) -> Result<Vec<String>> {
        let mut errors = Vec::new();

        if !self.cache_dir.exists() {
            return Ok(errors);
        }

        for entry in std::fs::read_dir(&self.cache_dir)? {
            let entry = entry?;
            let path = entry.path();

            if path.is_file() && path.extension().and_then(|s| s.to_str()) == Some("cache") {
                if let Err(e) = self.load_entry(&path) {
                    errors.push(format!("Corrupted cache file {:?}: {}", path, e));
                }
            }
        }

        Ok(errors)
    }
}

#[derive(Debug, Clone)]
pub struct CacheStatistics {
    pub total_entries: usize,
    pub expired_entries: usize,
    pub valid_entries: usize,
    pub total_size_bytes: u64,
    pub cache_dir: PathBuf,
}

#[derive(Debug, Clone)]
pub struct CompactionResult {
    pub entries_before: usize,
    pub entries_after: usize,
    pub entries_removed: usize,
    pub size_before: u64,
    pub size_after: u64,
    pub size_saved: u64,
}

impl Default for VerificationCache {
    fn default() -> Self {
        Self::new()
    }
}
