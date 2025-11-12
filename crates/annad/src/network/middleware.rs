//! Request middleware for body limits and rate limiting (Phase 1.14)

use axum::{
    body::Body,
    extract::{Request, State},
    http::StatusCode,
    middleware::Next,
    response::{IntoResponse, Response},
};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::RwLock;
use tracing::{debug, warn};

/// Maximum body size: 64 KiB
pub const MAX_BODY_SIZE: usize = 64 * 1024;

/// Rate limit tiers (Phase 1.16)
/// Burst: 20 requests in 10 seconds
pub const RATE_LIMIT_BURST_REQUESTS: usize = 20;
pub const RATE_LIMIT_BURST_WINDOW: Duration = Duration::from_secs(10);

/// Sustained: 100 requests per minute
pub const RATE_LIMIT_SUSTAINED_REQUESTS: usize = 100;
pub const RATE_LIMIT_SUSTAINED_WINDOW: Duration = Duration::from_secs(60);

/// Rate limiter state
#[derive(Clone)]
pub struct RateLimiter {
    // Track requests per peer IP
    peer_requests: Arc<RwLock<HashMap<String, Vec<Instant>>>>,
    // Track requests per auth token (Phase 1.15)
    token_requests: Arc<RwLock<HashMap<String, Vec<Instant>>>>,
    // Metrics for tracking violations
    metrics: Option<Arc<super::metrics::ConsensusMetrics>>,
}

impl RateLimiter {
    pub fn new() -> Self {
        Self {
            peer_requests: Arc::new(RwLock::new(HashMap::new())),
            token_requests: Arc::new(RwLock::new(HashMap::new())),
            metrics: None,
        }
    }

    pub fn new_with_metrics(metrics: Arc<super::metrics::ConsensusMetrics>) -> Self {
        Self {
            peer_requests: Arc::new(RwLock::new(HashMap::new())),
            token_requests: Arc::new(RwLock::new(HashMap::new())),
            metrics: Some(metrics),
        }
    }

    /// Check if peer is rate limited (by IP address)
    ///
    /// Implements dual-tier rate limiting (Phase 1.16):
    /// - Burst: 20 requests in 10 seconds
    /// - Sustained: 100 requests in 60 seconds
    pub async fn check_peer_rate_limit(&self, peer_addr: &str) -> bool {
        let mut requests = self.peer_requests.write().await;

        // Get or create entry for this peer
        let peer_reqs = requests.entry(peer_addr.to_string()).or_insert_with(Vec::new);

        let now = Instant::now();

        // Check burst limit (short window)
        let burst_count = peer_reqs.iter()
            .filter(|&&timestamp| now.duration_since(timestamp) < RATE_LIMIT_BURST_WINDOW)
            .count();

        if burst_count >= RATE_LIMIT_BURST_REQUESTS {
            warn!("Peer burst rate limit exceeded for: {} ({}/{})",
                  peer_addr, burst_count, RATE_LIMIT_BURST_REQUESTS);
            if let Some(ref metrics) = self.metrics {
                metrics.record_rate_limit_violation("peer_burst");
            }
            return false;
        }

        // Check sustained limit (long window)
        let sustained_count = peer_reqs.iter()
            .filter(|&&timestamp| now.duration_since(timestamp) < RATE_LIMIT_SUSTAINED_WINDOW)
            .count();

        if sustained_count >= RATE_LIMIT_SUSTAINED_REQUESTS {
            warn!("Peer sustained rate limit exceeded for: {} ({}/{})",
                  peer_addr, sustained_count, RATE_LIMIT_SUSTAINED_REQUESTS);
            if let Some(ref metrics) = self.metrics {
                metrics.record_rate_limit_violation("peer_sustained");
            }
            return false;
        }

        // Remove stale entries older than sustained window
        peer_reqs.retain(|&timestamp| now.duration_since(timestamp) < RATE_LIMIT_SUSTAINED_WINDOW);

        // Record this request
        peer_reqs.push(now);
        true
    }

    /// Check if auth token is rate limited (Phase 1.15 + 1.16)
    ///
    /// Implements dual-tier rate limiting (Phase 1.16):
    /// - Burst: 20 requests in 10 seconds
    /// - Sustained: 100 requests in 60 seconds
    pub async fn check_token_rate_limit(&self, token: &str) -> bool {
        let mut requests = self.token_requests.write().await;

        // Get or create entry for this token
        let token_reqs = requests.entry(token.to_string()).or_insert_with(Vec::new);

        let now = Instant::now();

        // Check burst limit (short window)
        let burst_count = token_reqs.iter()
            .filter(|&&timestamp| now.duration_since(timestamp) < RATE_LIMIT_BURST_WINDOW)
            .count();

        if burst_count >= RATE_LIMIT_BURST_REQUESTS {
            warn!("Token burst rate limit exceeded for: {} ({}/{})",
                  Self::mask_token(token), burst_count, RATE_LIMIT_BURST_REQUESTS);
            if let Some(ref metrics) = self.metrics {
                metrics.record_rate_limit_violation("token_burst");
            }
            return false;
        }

        // Check sustained limit (long window)
        let sustained_count = token_reqs.iter()
            .filter(|&&timestamp| now.duration_since(timestamp) < RATE_LIMIT_SUSTAINED_WINDOW)
            .count();

        if sustained_count >= RATE_LIMIT_SUSTAINED_REQUESTS {
            warn!("Token sustained rate limit exceeded for: {} ({}/{})",
                  Self::mask_token(token), sustained_count, RATE_LIMIT_SUSTAINED_REQUESTS);
            if let Some(ref metrics) = self.metrics {
                metrics.record_rate_limit_violation("token_sustained");
            }
            return false;
        }

        // Remove stale entries older than sustained window
        token_reqs.retain(|&timestamp| now.duration_since(timestamp) < RATE_LIMIT_SUSTAINED_WINDOW);

        // Record this request
        token_reqs.push(now);
        true
    }

    /// Get current request count for peer
    pub async fn get_peer_request_count(&self, peer_addr: &str) -> usize {
        let requests = self.peer_requests.read().await;
        requests.get(peer_addr).map(|v| v.len()).unwrap_or(0)
    }

    /// Get current request count for token
    pub async fn get_token_request_count(&self, token: &str) -> usize {
        let requests = self.token_requests.read().await;
        requests.get(token).map(|v| v.len()).unwrap_or(0)
    }

    /// Clean up old entries (call periodically)
    pub async fn cleanup(&self) {
        let now = Instant::now();

        // Clean peer requests
        let mut peer_requests = self.peer_requests.write().await;
        peer_requests.retain(|_, timestamps| {
            timestamps.retain(|&ts| now.duration_since(ts) < RATE_LIMIT_SUSTAINED_WINDOW);
            !timestamps.is_empty()
        });
        let peer_count = peer_requests.len();
        drop(peer_requests);

        // Clean token requests
        let mut token_requests = self.token_requests.write().await;
        token_requests.retain(|_, timestamps| {
            timestamps.retain(|&ts| now.duration_since(ts) < RATE_LIMIT_SUSTAINED_WINDOW);
            !timestamps.is_empty()
        });
        let token_count = token_requests.len();
        drop(token_requests);

        debug!(
            "Rate limiter cleanup: {} active peers, {} active tokens",
            peer_count, token_count
        );
    }

    /// Mask token for logging (show first 8 chars only)
    fn mask_token(token: &str) -> String {
        if token.len() > 8 {
            format!("{}...", &token[..8])
        } else {
            "***".to_string()
        }
    }
}

/// Body size limit middleware
///
/// Checks Content-Length header and rejects requests exceeding MAX_BODY_SIZE
pub async fn body_size_limit(request: Request, next: Next) -> Result<Response, StatusCode> {
    // Check Content-Length header
    if let Some(content_length) = request.headers().get("content-length") {
        if let Ok(length_str) = content_length.to_str() {
            if let Ok(length) = length_str.parse::<usize>() {
                if length > MAX_BODY_SIZE {
                    warn!(
                        "Request body too large: {} bytes (max: {})",
                        length, MAX_BODY_SIZE
                    );
                    return Err(StatusCode::PAYLOAD_TOO_LARGE);
                }
            }
        }
    }

    Ok(next.run(request).await)
}

/// Rate limit middleware (Phase 1.15 - dual scope)
///
/// Enforces rate limits for both:
/// 1. Per-peer (by IP address)
/// 2. Per-auth-token (if Authorization header present)
pub async fn rate_limit_middleware(
    State(rate_limiter): State<RateLimiter>,
    request: Request,
    next: Next,
) -> Result<Response, StatusCode> {
    // Extract peer address from connection info or X-Forwarded-For header
    let peer_addr = extract_peer_addr(&request);

    // Check peer rate limit
    if !rate_limiter.check_peer_rate_limit(&peer_addr).await {
        warn!("Peer rate limit exceeded for {}", peer_addr);
        return Err(StatusCode::TOO_MANY_REQUESTS);
    }

    // Check auth token rate limit (if present)
    if let Some(auth_token) = extract_auth_token(&request) {
        if !rate_limiter.check_token_rate_limit(&auth_token).await {
            warn!(
                "Token rate limit exceeded for peer {}, token: {}",
                peer_addr,
                RateLimiter::mask_token(&auth_token)
            );
            return Err(StatusCode::TOO_MANY_REQUESTS);
        }

        debug!(
            "Rate limit OK for {}, token: {} - {}/{} requests",
            peer_addr,
            RateLimiter::mask_token(&auth_token),
            rate_limiter.get_token_request_count(&auth_token).await,
            RATE_LIMIT_SUSTAINED_REQUESTS
        );
    } else {
        debug!(
            "Rate limit OK for {}: {}/{} requests (no token)",
            peer_addr,
            rate_limiter.get_peer_request_count(&peer_addr).await,
            RATE_LIMIT_SUSTAINED_REQUESTS
        );
    }

    Ok(next.run(request).await)
}

/// Extract peer address from request
fn extract_peer_addr(request: &Request) -> String {
    // Try X-Forwarded-For header first (for proxies)
    if let Some(forwarded) = request.headers().get("x-forwarded-for") {
        if let Ok(forwarded_str) = forwarded.to_str() {
            // Take first IP in the list
            if let Some(first_ip) = forwarded_str.split(',').next() {
                return first_ip.trim().to_string();
            }
        }
    }

    // Fall back to connection remote address (would need ConnectInfo extension)
    // For now, use a placeholder - in production, this would come from connection info
    "unknown".to_string()
}

/// Extract auth token from Authorization header (Phase 1.15)
///
/// Supports Bearer token format: "Authorization: Bearer <token>"
fn extract_auth_token(request: &Request) -> Option<String> {
    let auth_header = request.headers().get("authorization")?;
    let auth_str = auth_header.to_str().ok()?;

    // Parse "Bearer <token>" format
    if auth_str.starts_with("Bearer ") {
        Some(auth_str[7..].trim().to_string())
    } else {
        // Also support plain token (for backwards compatibility)
        Some(auth_str.trim().to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_peer_rate_limiter() {
        let limiter = RateLimiter::new();

        // First 20 requests should succeed (burst limit)
        for i in 1..=RATE_LIMIT_BURST_REQUESTS {
            assert!(
                limiter.check_peer_rate_limit("127.0.0.1").await,
                "Request {} should succeed within burst limit", i
            );
        }

        // 21st request should fail (burst exceeded)
        assert!(!limiter.check_peer_rate_limit("127.0.0.1").await);

        // Different peer should succeed
        assert!(limiter.check_peer_rate_limit("127.0.0.2").await);
    }

    #[tokio::test]
    async fn test_token_rate_limiter() {
        let limiter = RateLimiter::new();

        // First 20 requests should succeed (burst limit)
        for i in 1..=RATE_LIMIT_BURST_REQUESTS {
            assert!(
                limiter.check_token_rate_limit("test-token-123").await,
                "Token request {} should succeed within burst limit", i
            );
        }

        // 21st request should fail (burst exceeded)
        assert!(!limiter.check_token_rate_limit("test-token-123").await);

        // Different token should succeed
        assert!(limiter.check_token_rate_limit("test-token-456").await);
    }

    #[tokio::test]
    async fn test_rate_limiter_window() {
        let limiter = RateLimiter::new();

        // Fill up the burst rate limit
        for _ in 0..RATE_LIMIT_BURST_REQUESTS {
            assert!(limiter.check_peer_rate_limit("127.0.0.1").await);
        }

        // Should be rate limited (burst exceeded)
        assert!(!limiter.check_peer_rate_limit("127.0.0.1").await);

        // Manually clear old entries (in production, this happens via cleanup())
        tokio::time::sleep(Duration::from_millis(100)).await;
        limiter.cleanup().await;

        // Still rate limited (window not expired - burst window is 10s)
        assert!(!limiter.check_peer_rate_limit("127.0.0.1").await);
    }

    #[tokio::test]
    async fn test_cleanup() {
        let limiter = RateLimiter::new();

        // Add requests for multiple peers
        for i in 0..5 {
            limiter.check_peer_rate_limit(&format!("127.0.0.{}", i)).await;
        }

        // Cleanup should keep all (they're recent)
        limiter.cleanup().await;

        let requests = limiter.peer_requests.read().await;
        assert_eq!(requests.len(), 5);
    }

    #[test]
    fn test_mask_token() {
        assert_eq!(RateLimiter::mask_token("short"), "***");
        assert_eq!(RateLimiter::mask_token("12345678"), "***");
        assert_eq!(RateLimiter::mask_token("1234567890abcdef"), "12345678...");
    }

    #[tokio::test]
    async fn test_burst_rate_limiter() {
        let limiter = RateLimiter::new();

        // First 20 requests should succeed (burst limit)
        for i in 1..=RATE_LIMIT_BURST_REQUESTS {
            assert!(
                limiter.check_peer_rate_limit("127.0.0.1").await,
                "Request {} should succeed within burst limit", i
            );
        }

        // 21st request should fail (burst exceeded)
        assert!(
            !limiter.check_peer_rate_limit("127.0.0.1").await,
            "Request 21 should fail - burst limit exceeded"
        );

        // Different peer should still succeed
        assert!(limiter.check_peer_rate_limit("127.0.0.2").await);
    }

    #[tokio::test]
    async fn test_dual_tier_rate_limiting() {
        let limiter = RateLimiter::new();

        // Fill up burst limit (20 requests)
        for _ in 0..RATE_LIMIT_BURST_REQUESTS {
            assert!(limiter.check_peer_rate_limit("10.0.0.1").await);
        }

        // Next request should be blocked by burst limit
        assert!(!limiter.check_peer_rate_limit("10.0.0.1").await);

        // Wait for burst window to expire
        tokio::time::sleep(RATE_LIMIT_BURST_WINDOW + Duration::from_millis(100)).await;

        // Should now be able to make requests again (burst window reset)
        assert!(limiter.check_peer_rate_limit("10.0.0.1").await);
    }

    #[tokio::test]
    async fn test_token_burst_rate_limiter() {
        let limiter = RateLimiter::new();

        // First 20 requests should succeed (burst limit)
        for i in 1..=RATE_LIMIT_BURST_REQUESTS {
            assert!(
                limiter.check_token_rate_limit("test-token-burst").await,
                "Token request {} should succeed within burst limit", i
            );
        }

        // 21st request should fail (burst exceeded)
        assert!(
            !limiter.check_token_rate_limit("test-token-burst").await,
            "Token request 21 should fail - burst limit exceeded"
        );

        // Different token should still succeed
        assert!(limiter.check_token_rate_limit("test-token-other").await);
    }
}
