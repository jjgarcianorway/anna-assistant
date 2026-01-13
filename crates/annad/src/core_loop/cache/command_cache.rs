//! Command output caching.

use std::collections::HashMap;
use std::time::Instant;
use tracing::debug;

use crate::state::STATIC_COMMANDS;
use super::config_cache::get_perf_config;
use super::types::{CachedOutput, COMMAND_CACHE};

/// Check if a command is cacheable (static system info).
pub fn is_static_command(cmd: &str) -> bool {
    let cmd_trimmed = cmd.trim();
    STATIC_COMMANDS
        .iter()
        .any(|&static_cmd| cmd_trimmed == static_cmd || cmd_trimmed.starts_with(static_cmd))
}

/// Normalize command for cache key.
pub fn normalize_command(cmd: &str) -> String {
    cmd.split_whitespace().collect::<Vec<&str>>().join(" ")
}

/// Get cached command output if not expired.
pub fn get_cached_command(cmd: &str) -> Option<String> {
    let perf = get_perf_config();
    if let Ok(guard) = COMMAND_CACHE.read() {
        if let Some(ref cache) = *guard {
            let key = normalize_command(cmd);
            if let Some(cached) = cache.get(&key) {
                let ttl = if cached.is_static {
                    perf.static_command_cache_ttl_secs
                } else {
                    perf.command_cache_ttl_secs
                };
                if cached.cached_at.elapsed().as_secs() < ttl {
                    debug!("Command cache hit: {}", cmd);
                    return Some(cached.output.clone());
                }
            }
        }
    }
    None
}

/// Cache a command's output.
pub fn cache_command(cmd: &str, output: &str) {
    let perf = get_perf_config();
    if let Ok(mut guard) = COMMAND_CACHE.write() {
        let cache = guard.get_or_insert_with(HashMap::new);
        let key = normalize_command(cmd);
        let is_static = is_static_command(cmd);
        cache.insert(
            key,
            CachedOutput {
                output: output.to_string(),
                cached_at: Instant::now(),
                is_static,
            },
        );
        if cache.len() > 100 {
            cache.retain(|_, v| {
                let ttl = if v.is_static {
                    perf.static_command_cache_ttl_secs
                } else {
                    perf.command_cache_ttl_secs
                };
                v.cached_at.elapsed().as_secs() < ttl
            });
        }
    }
}
