//! Model Registry Fetcher v0.16.1
//!
//! Async fetching of the model registry from GitHub.
//! Uses a 24-hour cache to avoid repeated fetches.

use anna_common::model_registry::{
    cache_path, ModelRegistry, CACHE_DURATION_SECS, REGISTRY_URL,
};
use std::time::{Duration, SystemTime};
use tracing::{debug, info, warn};

/// Async registry fetcher
pub struct RegistryFetcher {
    client: reqwest::Client,
    /// Timeout for HTTP requests
    timeout: Duration,
}

impl Default for RegistryFetcher {
    fn default() -> Self {
        Self::new()
    }
}

impl RegistryFetcher {
    /// Create a new registry fetcher
    pub fn new() -> Self {
        Self {
            client: reqwest::Client::builder()
                .timeout(Duration::from_secs(30))
                .build()
                .unwrap_or_default(),
            timeout: Duration::from_secs(30),
        }
    }

    /// Create with custom timeout
    pub fn with_timeout(mut self, timeout: Duration) -> Self {
        self.timeout = timeout;
        self.client = reqwest::Client::builder()
            .timeout(timeout)
            .build()
            .unwrap_or_default();
        self
    }

    /// Check if cache is still valid
    pub fn is_cache_valid() -> bool {
        let path = cache_path();
        if !path.exists() {
            return false;
        }

        if let Ok(metadata) = std::fs::metadata(&path) {
            if let Ok(modified) = metadata.modified() {
                let age = SystemTime::now()
                    .duration_since(modified)
                    .unwrap_or(Duration::MAX);
                return age < Duration::from_secs(CACHE_DURATION_SECS);
            }
        }

        false
    }

    /// Fetch registry from remote URL
    pub async fn fetch_remote(&self) -> Result<ModelRegistry, String> {
        info!("📡  Fetching model registry from {}", REGISTRY_URL);

        let response = self
            .client
            .get(REGISTRY_URL)
            .send()
            .await
            .map_err(|e| format!("HTTP request failed: {}", e))?;

        if !response.status().is_success() {
            return Err(format!("HTTP error: {}", response.status()));
        }

        let text = response
            .text()
            .await
            .map_err(|e| format!("Failed to read response: {}", e))?;

        let registry: ModelRegistry = serde_json::from_str(&text)
            .map_err(|e| format!("Failed to parse registry JSON: {}", e))?;

        info!(
            "✅  Model registry fetched: v{}, {} tiers, {} known models",
            registry.schema_version,
            registry.recommended_by_tier.len(),
            registry.known_good_models.len()
        );

        // Cache the result
        if let Err(e) = registry.save_to_cache() {
            warn!("Failed to cache registry: {}", e);
        }

        Ok(registry)
    }

    /// Load registry (cache first, then remote, then builtin)
    pub async fn load(&self) -> ModelRegistry {
        // Try cache first
        if Self::is_cache_valid() {
            debug!("Loading registry from cache");
            if let Some(cached) = Self::load_from_cache() {
                return cached;
            }
        }

        // Try remote fetch
        match self.fetch_remote().await {
            Ok(registry) => registry,
            Err(e) => {
                warn!("Failed to fetch remote registry: {} - using builtin", e);
                ModelRegistry::builtin()
            }
        }
    }

    /// Load from cache file
    fn load_from_cache() -> Option<ModelRegistry> {
        let path = cache_path();
        std::fs::read_to_string(&path)
            .ok()
            .and_then(|content| serde_json::from_str(&content).ok())
    }

    /// Force refresh from remote (ignore cache)
    pub async fn refresh(&self) -> ModelRegistry {
        match self.fetch_remote().await {
            Ok(registry) => registry,
            Err(e) => {
                warn!("Failed to refresh registry: {} - using builtin", e);
                ModelRegistry::builtin()
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fetcher_creation() {
        let fetcher = RegistryFetcher::new();
        assert_eq!(fetcher.timeout, Duration::from_secs(30));
    }

    #[test]
    fn test_builtin_fallback() {
        let registry = ModelRegistry::builtin();
        assert!(!registry.recommended_by_tier.is_empty());
    }
}
