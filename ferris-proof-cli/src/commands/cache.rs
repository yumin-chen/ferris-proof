use crate::CacheAction;
use anyhow::Result;
use ferris_proof_core::CacheManager;
use std::path::PathBuf;

pub async fn run(action: CacheAction) -> Result<i32> {
    let cache_dir = get_cache_dir()?;
    let mut cache_manager = CacheManager::with_cache_dir(cache_dir);

    match action {
        CacheAction::Info => {
            let info = cache_manager.info()?;

            println!("Cache Information:");
            println!("  Directory: {}", info.cache_dir.display());
            println!("  Total entries: {}", info.total_entries);
            println!("  Valid entries: {}", info.valid_entries);
            println!("  Expired entries: {}", info.expired_entries);
            println!("  Memory size: {}", format_bytes(info.total_size_bytes));
            println!("  Disk size: {}", format_bytes(info.disk_size_bytes));

            if info.total_entries == 0 {
                println!("\nCache is empty.");
            } else if info.expired_entries > 0 {
                println!(
                    "\n⚠️  {} expired entries found. Consider running 'ferris-proof cache cleanup'",
                    info.expired_entries
                );
            } else {
                println!("\n✅ Cache is healthy.");
            }
        }

        CacheAction::Cleanup => {
            println!("Cleaning up expired cache entries...");
            let result = cache_manager.cleanup()?;

            println!("Cleanup completed:");
            println!("  Entries removed: {}", result.entries_removed);
            println!("  Size freed: {}", format_bytes(result.size_freed));
            println!("  Entries before: {}", result.entries_before);
            println!("  Entries after: {}", result.entries_after);

            if result.entries_removed == 0 {
                println!("✅ No expired entries found.");
            } else {
                println!(
                    "✅ Successfully cleaned up {} expired entries.",
                    result.entries_removed
                );
            }
        }

        CacheAction::Clear => {
            println!("⚠️  This will remove ALL cache entries. Are you sure? (y/N)");

            let mut input = String::new();
            std::io::stdin().read_line(&mut input)?;

            if input.trim().to_lowercase() != "y" {
                println!("Operation cancelled.");
                return Ok(0);
            }

            println!("Clearing all cache entries...");
            let result = cache_manager.clear()?;

            println!("Clear completed:");
            println!("  Entries removed: {}", result.entries_removed);
            println!("  Size freed: {}", format_bytes(result.size_freed));

            println!("✅ Cache cleared successfully.");
        }

        CacheAction::Compact => {
            println!("Compacting cache...");
            let result = cache_manager.compact()?;

            println!("Compaction completed:");
            println!("  Entries before: {}", result.entries_before);
            println!("  Entries after: {}", result.entries_after);
            println!("  Entries removed: {}", result.entries_removed);
            println!("  Size before: {}", format_bytes(result.size_before));
            println!("  Size after: {}", format_bytes(result.size_after));
            println!("  Size saved: {}", format_bytes(result.size_saved));

            if result.entries_removed == 0 && result.size_saved == 0 {
                println!("✅ Cache is already optimized.");
            } else {
                println!("✅ Cache compacted successfully.");
            }
        }

        CacheAction::Health => {
            println!("Checking cache health...");
            let health_report = cache_manager.health_check()?;

            println!("Cache Health Report:");
            println!("  Directory: {}", health_report.info.cache_dir.display());
            println!("  Total entries: {}", health_report.info.total_entries);
            println!("  Valid entries: {}", health_report.info.valid_entries);
            println!("  Expired entries: {}", health_report.info.expired_entries);
            println!(
                "  Disk size: {}",
                format_bytes(health_report.info.disk_size_bytes)
            );

            if !health_report.integrity_errors.is_empty() {
                println!("\n❌ Integrity Issues:");
                for error in &health_report.integrity_errors {
                    println!("  • {}", error);
                }
            } else {
                println!("\n✅ No integrity issues found.");
            }

            if !health_report.recommendations.is_empty() {
                println!("\n💡 Recommendations:");
                for recommendation in &health_report.recommendations {
                    println!("  • {}", recommendation);
                }
            }
        }

        CacheAction::Repair => {
            println!("Repairing cache...");
            let result = cache_manager.repair()?;

            println!("Repair completed:");
            println!(
                "  Corrupted entries removed: {}",
                result.corrupted_entries_removed
            );
            println!("  Entries before: {}", result.entries_before);
            println!("  Entries after: {}", result.entries_after);
            println!("  Size freed: {}", format_bytes(result.size_freed));

            if result.corrupted_entries_removed == 0 {
                println!("✅ No corrupted entries found.");
            } else {
                println!("✅ Cache repaired successfully.");
            }
        }
    }

    Ok(0)
}

fn get_cache_dir() -> Result<PathBuf> {
    // Try to get cache directory from environment or use default
    if let Ok(cache_dir) = std::env::var("FERRIS_PROOF_CACHE_DIR") {
        Ok(PathBuf::from(cache_dir))
    } else {
        // Use default cache directory
        let cache_dir = dirs::cache_dir()
            .unwrap_or_else(std::env::temp_dir)
            .join("ferris-proof");
        Ok(cache_dir)
    }
}

fn format_bytes(bytes: u64) -> String {
    const UNITS: &[&str] = &["B", "KB", "MB", "GB", "TB"];

    if bytes == 0 {
        return "0 B".to_string();
    }

    let mut size = bytes as f64;
    let mut unit_index = 0;

    while size >= 1024.0 && unit_index < UNITS.len() - 1 {
        size /= 1024.0;
        unit_index += 1;
    }

    if unit_index == 0 {
        format!("{} {}", bytes, UNITS[unit_index])
    } else {
        format!("{:.1} {}", size, UNITS[unit_index])
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_bytes() {
        assert_eq!(format_bytes(0), "0 B");
        assert_eq!(format_bytes(512), "512 B");
        assert_eq!(format_bytes(1024), "1.0 KB");
        assert_eq!(format_bytes(1536), "1.5 KB");
        assert_eq!(format_bytes(1048576), "1.0 MB");
        assert_eq!(format_bytes(1073741824), "1.0 GB");
    }
}
