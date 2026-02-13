//! Event-driven system command cache with intelligent invalidation.
//!
//! This module provides a shared cache for system commands with:
//! - Event-driven invalidation (hardware changes, config changes, etc.)
//! - TTL fallback for safety
//! - Pre-warming of common queries
//! - Thread-safe access via Arc<RwLock>

mod types;
mod watcher;
mod warmer;

pub use types::{CacheStats, InvalidationTag, SystemCache};
pub use watcher::watcher_loop;
pub use warmer::warmer_loop;

use std::process::Command;
use std::time::Duration;
use tracing::debug;

/// Get a command result from cache, or run and cache it.
///
/// # Arguments
/// * `cache` - The shared cache
/// * `key` - Unique key for this command (e.g., "lsblk_devices")
/// * `cmd` - Command to run
/// * `args` - Command arguments
/// * `ttl_secs` - TTL in seconds
/// * `tags` - Invalidation tags
///
/// # Returns
/// Command output, either from cache or freshly executed.
pub fn get_or_run(
    cache: &SystemCache,
    key: &str,
    cmd: &str,
    args: &[&str],
    ttl_secs: u64,
    tags: &[InvalidationTag],
) -> Result<String, std::io::Error> {
    // Check cache first
    if let Some(cached) = cache.get(key) {
        debug!("Cache hit: {}", key);
        return Ok(cached);
    }

    // Cache miss - run command
    debug!("Cache miss: {} (running {})", key, cmd);
    let output = Command::new(cmd).args(args).output()?;
    let result = String::from_utf8_lossy(&output.stdout).to_string();

    // Store in cache
    if !result.is_empty() {
        cache.set(
            key.to_string(),
            result.clone(),
            Duration::from_secs(ttl_secs),
            tags.to_vec(),
        );
    }

    Ok(result)
}

/// Start all cache background tasks.
///
/// Call this once at daemon startup to spawn watcher and warmer tasks.
pub fn start_cache_tasks(cache: SystemCache) {
    // Spawn watcher task
    let cache_for_watcher = cache.clone();
    tokio::spawn(async move {
        watcher_loop(cache_for_watcher).await;
    });

    // Spawn warmer task
    tokio::spawn(async move {
        warmer_loop(cache).await;
    });
}
