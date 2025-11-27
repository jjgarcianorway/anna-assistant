//! Rate Limiter v0.15.0
//!
//! Token bucket rate limiter for request throttling.

use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

/// Rate limiter using token bucket algorithm
#[derive(Debug)]
pub struct RateLimiter {
    /// Buckets per client identifier
    buckets: Arc<Mutex<HashMap<String, TokenBucket>>>,
    /// Tokens per period
    tokens_per_period: u32,
    /// Period duration
    period: Duration,
}

/// Token bucket for a single client
#[derive(Debug, Clone)]
struct TokenBucket {
    /// Current token count
    tokens: f64,
    /// Last refill time
    last_refill: Instant,
    /// Maximum tokens
    max_tokens: u32,
    /// Refill rate (tokens per second)
    refill_rate: f64,
}

impl TokenBucket {
    fn new(max_tokens: u32, period: Duration) -> Self {
        Self {
            tokens: max_tokens as f64,
            last_refill: Instant::now(),
            max_tokens,
            refill_rate: max_tokens as f64 / period.as_secs_f64(),
        }
    }

    fn refill(&mut self) {
        let now = Instant::now();
        let elapsed = now.duration_since(self.last_refill).as_secs_f64();
        self.tokens = (self.tokens + elapsed * self.refill_rate).min(self.max_tokens as f64);
        self.last_refill = now;
    }

    fn try_consume(&mut self, tokens: u32) -> bool {
        self.refill();
        if self.tokens >= tokens as f64 {
            self.tokens -= tokens as f64;
            true
        } else {
            false
        }
    }

    fn time_until_available(&self, tokens: u32) -> Duration {
        if self.tokens >= tokens as f64 {
            Duration::ZERO
        } else {
            let needed = tokens as f64 - self.tokens;
            Duration::from_secs_f64(needed / self.refill_rate)
        }
    }
}

impl RateLimiter {
    /// Create a new rate limiter
    pub fn new(tokens_per_period: u32, period: Duration) -> Self {
        Self {
            buckets: Arc::new(Mutex::new(HashMap::new())),
            tokens_per_period,
            period,
        }
    }

    /// Create a rate limiter with default settings (20 req/min)
    pub fn default_limiter() -> Self {
        Self::new(20, Duration::from_secs(60))
    }

    /// Try to acquire a token for the given client
    pub fn try_acquire(&self, client_id: &str) -> RateLimitResult {
        let mut buckets = self.buckets.lock().unwrap();

        let bucket = buckets
            .entry(client_id.to_string())
            .or_insert_with(|| TokenBucket::new(self.tokens_per_period, self.period));

        if bucket.try_consume(1) {
            RateLimitResult::Allowed {
                remaining: bucket.tokens as u32,
            }
        } else {
            RateLimitResult::Limited {
                retry_after: bucket.time_until_available(1),
            }
        }
    }

    /// Check if a client can make a request without consuming
    pub fn check(&self, client_id: &str) -> RateLimitResult {
        let mut buckets = self.buckets.lock().unwrap();

        let bucket = buckets
            .entry(client_id.to_string())
            .or_insert_with(|| TokenBucket::new(self.tokens_per_period, self.period));

        bucket.refill();

        if bucket.tokens >= 1.0 {
            RateLimitResult::Allowed {
                remaining: bucket.tokens as u32,
            }
        } else {
            RateLimitResult::Limited {
                retry_after: bucket.time_until_available(1),
            }
        }
    }

    /// Clean up old buckets (call periodically)
    pub fn cleanup(&self, max_age: Duration) {
        let mut buckets = self.buckets.lock().unwrap();
        let now = Instant::now();

        buckets.retain(|_, bucket| now.duration_since(bucket.last_refill) < max_age);
    }

    /// Get current bucket count (for monitoring)
    pub fn bucket_count(&self) -> usize {
        self.buckets.lock().unwrap().len()
    }
}

impl Default for RateLimiter {
    fn default() -> Self {
        Self::default_limiter()
    }
}

/// Result of a rate limit check
#[derive(Debug, Clone)]
pub enum RateLimitResult {
    /// Request is allowed
    Allowed {
        /// Remaining tokens
        remaining: u32,
    },
    /// Request is rate limited
    Limited {
        /// Time until token available
        retry_after: Duration,
    },
}

impl RateLimitResult {
    pub fn is_allowed(&self) -> bool {
        matches!(self, RateLimitResult::Allowed { .. })
    }

    pub fn is_limited(&self) -> bool {
        matches!(self, RateLimitResult::Limited { .. })
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rate_limiter_creation() {
        let limiter = RateLimiter::new(10, Duration::from_secs(60));
        assert_eq!(limiter.bucket_count(), 0);
    }

    #[test]
    fn test_allow_within_limit() {
        let limiter = RateLimiter::new(10, Duration::from_secs(60));

        for _ in 0..10 {
            assert!(limiter.try_acquire("client1").is_allowed());
        }
    }

    #[test]
    fn test_limit_exceeded() {
        let limiter = RateLimiter::new(2, Duration::from_secs(60));

        assert!(limiter.try_acquire("client1").is_allowed());
        assert!(limiter.try_acquire("client1").is_allowed());
        assert!(limiter.try_acquire("client1").is_limited());
    }

    #[test]
    fn test_separate_clients() {
        let limiter = RateLimiter::new(2, Duration::from_secs(60));

        assert!(limiter.try_acquire("client1").is_allowed());
        assert!(limiter.try_acquire("client1").is_allowed());
        assert!(limiter.try_acquire("client1").is_limited());

        // Different client should have its own bucket
        assert!(limiter.try_acquire("client2").is_allowed());
        assert!(limiter.try_acquire("client2").is_allowed());
    }

    #[test]
    fn test_check_does_not_consume() {
        let limiter = RateLimiter::new(1, Duration::from_secs(60));

        // Check without consuming
        assert!(limiter.check("client1").is_allowed());
        assert!(limiter.check("client1").is_allowed());

        // Now consume
        assert!(limiter.try_acquire("client1").is_allowed());
        assert!(limiter.try_acquire("client1").is_limited());
    }

    #[test]
    fn test_cleanup() {
        let limiter = RateLimiter::new(10, Duration::from_secs(60));

        limiter.try_acquire("client1");
        limiter.try_acquire("client2");
        assert_eq!(limiter.bucket_count(), 2);

        // Cleanup with zero max age removes all
        limiter.cleanup(Duration::ZERO);
        assert_eq!(limiter.bucket_count(), 0);
    }
}
