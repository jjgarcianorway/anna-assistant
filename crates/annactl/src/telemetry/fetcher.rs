//! Telemetry Fetcher - Fetch real-time system telemetry
//!
//! Beta.200: Core telemetry fetching logic
//!
//! Responsibilities:
//! - Fetch telemetry from annad daemon via RPC
//! - Cache telemetry data with TTL
//! - Provide structured telemetry access
//! - Handle daemon connectivity issues gracefully

use anyhow::Result;
use serde_json::Value;

/// Telemetry fetcher for system data
pub struct TelemetryFetcher {
    /// Cached telemetry data
    cache: Option<Value>,

    /// Cache timestamp for TTL
    cache_time: Option<std::time::Instant>,

    /// Cache TTL in seconds (default: 5 seconds)
    ttl_seconds: u64,
}

impl TelemetryFetcher {
    /// Create a new telemetry fetcher
    pub fn new() -> Self {
        Self {
            cache: None,
            cache_time: None,
            ttl_seconds: 5,
        }
    }

    /// Create a fetcher with custom TTL
    pub fn with_ttl(ttl_seconds: u64) -> Self {
        Self {
            cache: None,
            cache_time: None,
            ttl_seconds,
        }
    }

    /// Fetch fresh telemetry from daemon
    ///
    /// Beta.200: This will integrate with existing RPC client
    pub async fn fetch_fresh(&mut self) -> Result<Value> {
        // TODO: Integrate with crate::rpc_client to fetch telemetry
        // For now, return a placeholder
        let telemetry = serde_json::json!({
            "system": {
                "hostname": "placeholder",
                "uptime": 0,
                "load": [0.0, 0.0, 0.0]
            },
            "cpu": {
                "model": "Unknown",
                "cores": 0,
                "usage": 0.0
            },
            "memory": {
                "total": 0,
                "available": 0,
                "used": 0
            }
        });

        // Update cache
        self.cache = Some(telemetry.clone());
        self.cache_time = Some(std::time::Instant::now());

        Ok(telemetry)
    }

    /// Get telemetry (uses cache if valid)
    pub async fn get_telemetry(&mut self) -> Result<Value> {
        // Check if cache is valid
        if let (Some(cache), Some(cache_time)) = (&self.cache, self.cache_time) {
            if cache_time.elapsed().as_secs() < self.ttl_seconds {
                return Ok(cache.clone());
            }
        }

        // Cache miss or expired - fetch fresh
        self.fetch_fresh().await
    }

    /// Invalidate cache
    pub fn invalidate_cache(&mut self) {
        self.cache = None;
        self.cache_time = None;
    }

    /// Get specific telemetry field
    pub async fn get_field(&mut self, field_path: &str) -> Result<Option<Value>> {
        let telemetry = self.get_telemetry().await?;

        // Navigate JSON path
        let parts: Vec<&str> = field_path.split('.').collect();
        let mut current = &telemetry;

        for part in parts {
            match current.get(part) {
                Some(value) => current = value,
                None => return Ok(None),
            }
        }

        Ok(Some(current.clone()))
    }
}

impl Default for TelemetryFetcher {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_fetch_telemetry() {
        let mut fetcher = TelemetryFetcher::new();
        let result = fetcher.fetch_fresh().await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_cache() {
        let mut fetcher = TelemetryFetcher::with_ttl(10);

        // First fetch
        let t1 = fetcher.get_telemetry().await.unwrap();

        // Second fetch should use cache
        let t2 = fetcher.get_telemetry().await.unwrap();

        assert_eq!(t1, t2);
    }

    #[tokio::test]
    async fn test_get_field() {
        let mut fetcher = TelemetryFetcher::new();
        let hostname = fetcher.get_field("system.hostname").await;
        assert!(hostname.is_ok());
    }

    #[test]
    fn test_invalidate_cache() {
        let mut fetcher = TelemetryFetcher::new();
        fetcher.cache = Some(serde_json::json!({}));
        fetcher.invalidate_cache();
        assert!(fetcher.cache.is_none());
    }
}
