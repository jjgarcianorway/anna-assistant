//! Circuit Breaker v0.15.0
//!
//! Protection against cascading failures when backends are unavailable.

use std::sync::atomic::{AtomicU32, AtomicU64, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};

/// Circuit breaker state
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CircuitState {
    /// Circuit is closed, requests flow through
    Closed,
    /// Circuit is open, requests fail immediately
    Open,
    /// Circuit is half-open, testing if backend recovered
    HalfOpen,
}

/// Circuit breaker configuration
#[derive(Debug, Clone)]
pub struct CircuitBreakerConfig {
    /// Number of failures before opening circuit
    pub failure_threshold: u32,
    /// Number of successes needed to close circuit from half-open
    pub success_threshold: u32,
    /// Time to wait before testing again (half-open)
    pub reset_timeout: Duration,
    /// Time window for counting failures
    pub failure_window: Duration,
}

impl Default for CircuitBreakerConfig {
    fn default() -> Self {
        Self {
            failure_threshold: 5,
            success_threshold: 2,
            reset_timeout: Duration::from_secs(30),
            failure_window: Duration::from_secs(60),
        }
    }
}

/// Circuit breaker for a single backend
#[derive(Debug)]
pub struct CircuitBreaker {
    /// Configuration
    config: CircuitBreakerConfig,
    /// Current failure count
    failure_count: AtomicU32,
    /// Success count (for half-open state)
    success_count: AtomicU32,
    /// Time when circuit was opened (epoch millis, 0 if closed)
    opened_at: AtomicU64,
    /// Last failure time (epoch millis)
    last_failure: AtomicU64,
}

impl CircuitBreaker {
    /// Create a new circuit breaker
    pub fn new(config: CircuitBreakerConfig) -> Self {
        Self {
            config,
            failure_count: AtomicU32::new(0),
            success_count: AtomicU32::new(0),
            opened_at: AtomicU64::new(0),
            last_failure: AtomicU64::new(0),
        }
    }

    /// Create with default configuration
    pub fn default_breaker() -> Self {
        Self::new(CircuitBreakerConfig::default())
    }

    /// Get current state
    pub fn state(&self) -> CircuitState {
        let opened_at = self.opened_at.load(Ordering::Relaxed);

        if opened_at == 0 {
            return CircuitState::Closed;
        }

        let now = current_time_millis();
        let elapsed = Duration::from_millis(now.saturating_sub(opened_at));

        if elapsed >= self.config.reset_timeout {
            CircuitState::HalfOpen
        } else {
            CircuitState::Open
        }
    }

    /// Check if a request can proceed
    pub fn can_proceed(&self) -> CircuitBreakerResult {
        match self.state() {
            CircuitState::Closed => CircuitBreakerResult::Allowed,
            CircuitState::Open => {
                let opened_at = self.opened_at.load(Ordering::Relaxed);
                let now = current_time_millis();
                let elapsed = Duration::from_millis(now.saturating_sub(opened_at));
                let remaining = self.config.reset_timeout.saturating_sub(elapsed);

                CircuitBreakerResult::Rejected {
                    retry_after: remaining,
                }
            }
            CircuitState::HalfOpen => CircuitBreakerResult::AllowedHalfOpen,
        }
    }

    /// Record a successful request
    pub fn record_success(&self) {
        let state = self.state();

        match state {
            CircuitState::Closed => {
                // Reset failure count on success
                self.failure_count.store(0, Ordering::Relaxed);
            }
            CircuitState::HalfOpen => {
                let successes = self.success_count.fetch_add(1, Ordering::Relaxed) + 1;
                if successes >= self.config.success_threshold {
                    // Close the circuit
                    self.opened_at.store(0, Ordering::Relaxed);
                    self.failure_count.store(0, Ordering::Relaxed);
                    self.success_count.store(0, Ordering::Relaxed);
                }
            }
            CircuitState::Open => {
                // Shouldn't happen, but ignore
            }
        }
    }

    /// Record a failed request
    pub fn record_failure(&self) {
        let now = current_time_millis();
        let last = self.last_failure.load(Ordering::Relaxed);

        // Check if we're within the failure window
        let window_start = now.saturating_sub(self.config.failure_window.as_millis() as u64);

        if last < window_start {
            // Outside window, reset count
            self.failure_count.store(1, Ordering::Relaxed);
        } else {
            let failures = self.failure_count.fetch_add(1, Ordering::Relaxed) + 1;

            if failures >= self.config.failure_threshold {
                // Open the circuit
                self.opened_at.store(now, Ordering::Relaxed);
                self.success_count.store(0, Ordering::Relaxed);
            }
        }

        self.last_failure.store(now, Ordering::Relaxed);
    }

    /// Reset the circuit breaker to closed state
    pub fn reset(&self) {
        self.opened_at.store(0, Ordering::Relaxed);
        self.failure_count.store(0, Ordering::Relaxed);
        self.success_count.store(0, Ordering::Relaxed);
        self.last_failure.store(0, Ordering::Relaxed);
    }

    /// Get statistics
    pub fn stats(&self) -> CircuitBreakerStats {
        CircuitBreakerStats {
            state: self.state(),
            failure_count: self.failure_count.load(Ordering::Relaxed),
            success_count: self.success_count.load(Ordering::Relaxed),
        }
    }
}

impl Default for CircuitBreaker {
    fn default() -> Self {
        Self::default_breaker()
    }
}

/// Result of circuit breaker check
#[derive(Debug, Clone)]
pub enum CircuitBreakerResult {
    /// Request is allowed (circuit closed)
    Allowed,
    /// Request is allowed but circuit is testing (half-open)
    AllowedHalfOpen,
    /// Request is rejected (circuit open)
    Rejected {
        /// Time until circuit enters half-open
        retry_after: Duration,
    },
}

impl CircuitBreakerResult {
    pub fn is_allowed(&self) -> bool {
        matches!(
            self,
            CircuitBreakerResult::Allowed | CircuitBreakerResult::AllowedHalfOpen
        )
    }
}

/// Circuit breaker statistics
#[derive(Debug, Clone)]
pub struct CircuitBreakerStats {
    pub state: CircuitState,
    pub failure_count: u32,
    pub success_count: u32,
}

/// Get current time in milliseconds since epoch
fn current_time_millis() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis() as u64
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_initial_state_closed() {
        let cb = CircuitBreaker::default_breaker();
        assert_eq!(cb.state(), CircuitState::Closed);
        assert!(cb.can_proceed().is_allowed());
    }

    #[test]
    fn test_opens_after_failures() {
        let config = CircuitBreakerConfig {
            failure_threshold: 3,
            success_threshold: 1,
            reset_timeout: Duration::from_secs(30),
            failure_window: Duration::from_secs(60),
        };
        let cb = CircuitBreaker::new(config);

        assert_eq!(cb.state(), CircuitState::Closed);

        cb.record_failure();
        cb.record_failure();
        assert_eq!(cb.state(), CircuitState::Closed);

        cb.record_failure();
        assert_eq!(cb.state(), CircuitState::Open);
        assert!(!cb.can_proceed().is_allowed());
    }

    #[test]
    fn test_success_resets_failures() {
        let config = CircuitBreakerConfig {
            failure_threshold: 3,
            success_threshold: 1,
            reset_timeout: Duration::from_secs(30),
            failure_window: Duration::from_secs(60),
        };
        let cb = CircuitBreaker::new(config);

        cb.record_failure();
        cb.record_failure();
        cb.record_success();

        // Failures should be reset
        assert_eq!(cb.stats().failure_count, 0);
    }

    #[test]
    fn test_reset() {
        let config = CircuitBreakerConfig {
            failure_threshold: 2,
            ..Default::default()
        };
        let cb = CircuitBreaker::new(config);

        cb.record_failure();
        cb.record_failure();
        assert_eq!(cb.state(), CircuitState::Open);

        cb.reset();
        assert_eq!(cb.state(), CircuitState::Closed);
    }

    #[test]
    fn test_stats() {
        let cb = CircuitBreaker::default_breaker();
        let stats = cb.stats();

        assert_eq!(stats.state, CircuitState::Closed);
        assert_eq!(stats.failure_count, 0);
        assert_eq!(stats.success_count, 0);
    }
}
