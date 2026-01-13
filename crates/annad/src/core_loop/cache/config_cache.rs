//! Configuration caching (performance and wiki).

use anna_shared::config::{AnnaConfig, PerformanceConfig, WikiConfig};
use tracing::info;

use super::types::{PERF_CONFIG, WIKI_CONFIG};

/// Get performance config (loads from disk once, caches in memory).
pub fn get_perf_config() -> PerformanceConfig {
    if let Ok(guard) = PERF_CONFIG.read() {
        if let Some(ref config) = *guard {
            return config.clone();
        }
    }
    let config = AnnaConfig::load()
        .map(|c| c.performance)
        .unwrap_or_default();
    if let Ok(mut guard) = PERF_CONFIG.write() {
        *guard = Some(config.clone());
    }
    config
}

/// Reload performance config from disk.
pub fn reload_perf_config() {
    if let Ok(mut guard) = PERF_CONFIG.write() {
        let config = AnnaConfig::load()
            .map(|c| c.performance)
            .unwrap_or_default();
        *guard = Some(config);
        info!("Reloaded performance config");
    }
}

/// Get cached wiki config.
pub fn get_wiki_config() -> WikiConfig {
    if let Ok(guard) = WIKI_CONFIG.read() {
        if let Some(ref config) = *guard {
            return config.clone();
        }
    }
    let config = AnnaConfig::load()
        .map(|c| c.wiki)
        .unwrap_or_default();
    if let Ok(mut guard) = WIKI_CONFIG.write() {
        *guard = Some(config.clone());
    }
    config
}
