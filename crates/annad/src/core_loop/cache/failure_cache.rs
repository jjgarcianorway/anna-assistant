//! Command failure tracking (session-level and global).

use std::collections::HashMap;
use std::time::Instant;
use tracing::debug;

use super::command_cache::normalize_command;
use super::types::{
    CommandFailure, CommandFailureRecord, CMD_FAILURE_THRESHOLD, CMD_FAILURE_TTL_SECS,
    COMMAND_FAILURE_CACHE, FAILURE_CACHE, FAILURE_CACHE_TTL_SECS, MAX_CMD_FAILURE_CACHE_SIZE,
};

/// Check if a command is a known failure (session-level negative learning).
pub fn is_known_failed_command(cmd: &str) -> Option<String> {
    if let Ok(guard) = FAILURE_CACHE.read() {
        if let Some(ref cache) = *guard {
            let base_cmd = cmd.split_whitespace().next().unwrap_or(cmd);

            if let Some(failure) = cache.get(cmd) {
                if failure.failed_at.elapsed().as_secs() < FAILURE_CACHE_TTL_SECS {
                    debug!("Skipping known-failed command: {}", cmd);
                    return Some(failure.error_type.clone());
                }
            }

            if let Some(failure) = cache.get(base_cmd) {
                if failure.failed_at.elapsed().as_secs() < FAILURE_CACHE_TTL_SECS {
                    if failure.error_type.contains("NotFound") {
                        debug!("Skipping command with known-failed base: {} (base: {})", cmd, base_cmd);
                        return Some(failure.error_type.clone());
                    }
                }
            }
        }
    }
    None
}

/// Record a command failure (session-level negative learning).
pub fn record_command_failure_cache(cmd: &str, error_type: &str) {
    if let Ok(mut guard) = FAILURE_CACHE.write() {
        let cache = guard.get_or_insert_with(HashMap::new);

        cache.insert(cmd.to_string(), CommandFailure {
            error_type: error_type.to_string(),
            failed_at: Instant::now(),
        });

        if error_type.contains("NotFound") {
            if let Some(base_cmd) = cmd.split_whitespace().next() {
                if base_cmd != cmd {
                    cache.insert(base_cmd.to_string(), CommandFailure {
                        error_type: error_type.to_string(),
                        failed_at: Instant::now(),
                    });
                }
            }
        }

        if cache.len() > 100 {
            cache.retain(|_, v| v.failed_at.elapsed().as_secs() < FAILURE_CACHE_TTL_SECS);
        }

        debug!("Recorded command failure: {} ({})", cmd, error_type);
    }
}

/// Clear session-level failure cache.
pub fn clear_failure_cache() {
    if let Ok(mut guard) = FAILURE_CACHE.write() {
        *guard = Some(HashMap::new());
        debug!("Failure cache cleared");
    }
}

/// Normalize command for global failure cache key.
fn normalize_command_for_cache(command: &str) -> String {
    let cmd = command.trim();
    if cmd.len() < 2 { return String::new(); }

    let parts: Vec<&str> = cmd.split_whitespace().collect();
    if parts.is_empty() { return String::new(); }

    match parts[0] {
        "cat" | "head" | "tail" | "less" | "grep" | "find" | "ls" => parts[0].to_string(),
        "systemctl" | "journalctl" => {
            if parts.len() > 1 { format!("{} {}", parts[0], parts[1]) }
            else { parts[0].to_string() }
        }
        "pacman" | "yay" | "paru" => {
            if parts.len() > 1 && parts[1].starts_with('-') { format!("{} {}", parts[0], parts[1]) }
            else { parts[0].to_string() }
        }
        _ => cmd.chars().take(50).collect(),
    }
}

/// Record a command failure globally.
pub fn record_command_failure(command: &str, error_type: &str) {
    let normalized = normalize_command_for_cache(command);
    if normalized.is_empty() { return; }

    if let Ok(mut guard) = COMMAND_FAILURE_CACHE.write() {
        let cache = guard.get_or_insert_with(HashMap::new);
        let now = Instant::now();

        if let Some(record) = cache.get_mut(&normalized) {
            record.failure_count += 1;
            record.last_error_type = error_type.to_string();
            record.last_failed_at = now;

            if record.failure_count >= CMD_FAILURE_THRESHOLD {
                debug!(
                    "Command '{}' has failed {} times ({})",
                    normalized, record.failure_count, error_type
                );
            }
        } else {
            cache.insert(normalized.clone(), CommandFailureRecord {
                failure_count: 1,
                last_error_type: error_type.to_string(),
                first_failed_at: now,
                last_failed_at: now,
            });
        }

        if cache.len() > MAX_CMD_FAILURE_CACHE_SIZE {
            cache.retain(|_, v| v.last_failed_at.elapsed().as_secs() < CMD_FAILURE_TTL_SECS);

            if cache.len() > MAX_CMD_FAILURE_CACHE_SIZE {
                let mut entries: Vec<_> = cache.iter().collect();
                entries.sort_by(|a, b| b.1.failure_count.cmp(&a.1.failure_count));
                let keys_to_remove: Vec<String> = entries.iter()
                    .skip(MAX_CMD_FAILURE_CACHE_SIZE / 2)
                    .map(|(k, _)| (*k).clone())
                    .collect();
                for key in keys_to_remove { cache.remove(&key); }
            }
        }
    }
}

/// Check if a command is known to fail frequently (global).
pub fn check_command_failure(command: &str) -> Option<u32> {
    let normalized = normalize_command_for_cache(command);
    if normalized.is_empty() { return None; }

    if let Ok(guard) = COMMAND_FAILURE_CACHE.read() {
        if let Some(ref cache) = *guard {
            if let Some(record) = cache.get(&normalized) {
                if record.last_failed_at.elapsed().as_secs() < CMD_FAILURE_TTL_SECS
                    && record.failure_count >= CMD_FAILURE_THRESHOLD
                {
                    return Some(record.failure_count);
                }
            }
        }
    }
    None
}

/// Record a command success (resets failure count).
pub fn record_command_success(command: &str) {
    let normalized = normalize_command_for_cache(command);
    if normalized.is_empty() { return; }

    if let Ok(mut guard) = COMMAND_FAILURE_CACHE.write() {
        if let Some(ref mut cache) = *guard {
            if cache.remove(&normalized).is_some() {
                debug!("Command '{}' succeeded, removed from failure cache", normalized);
            }
        }
    }
}

/// Get list of frequently failing commands for diagnostic purposes.
pub fn get_failing_commands() -> Vec<(String, u32, String)> {
    let mut result = Vec::new();

    if let Ok(guard) = COMMAND_FAILURE_CACHE.read() {
        if let Some(ref cache) = *guard {
            for (cmd, record) in cache.iter() {
                if record.failure_count >= CMD_FAILURE_THRESHOLD
                    && record.last_failed_at.elapsed().as_secs() < CMD_FAILURE_TTL_SECS
                {
                    result.push((cmd.clone(), record.failure_count, record.last_error_type.clone()));
                }
            }
        }
    }

    result.sort_by(|a, b| b.1.cmp(&a.1));
    result
}
