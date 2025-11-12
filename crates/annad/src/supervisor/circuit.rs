//! Circuit breaker pattern for task supervision

use std::time::{Duration, Instant};

/// Circuit breaker state
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CircuitState {
    /// Circuit is closed, requests flow normally
    Closed,
    /// Circuit is open, requests are rejected
    Open,
    /// Circuit is half-open, testing if service recovered
    HalfOpen,
}

/// Circuit breaker for task supervision
#[derive(Debug, Clone)]
pub struct CircuitBreaker {
    /// Current state
    state: CircuitState,
    /// Failure count in current window
    failure_count: u32,
    /// Success count in half-open state
    success_count: u32,
    /// Threshold for opening circuit
    failure_threshold: u32,
    /// Success threshold for closing circuit from half-open
    success_threshold: u32,
    /// Time when circuit was opened
    opened_at: Option<Instant>,
    /// Duration to wait before half-open
    timeout: Duration,
}

impl CircuitBreaker {
    pub fn new(failure_threshold: u32, timeout: Duration) -> Self {
        Self {
            state: CircuitState::Closed,
            failure_count: 0,
            success_count: 0,
            failure_threshold,
            success_threshold: 3, // Require 3 successes to close
            opened_at: None,
            timeout,
        }
    }

    /// Record a failure
    pub fn record_failure(&mut self) {
        match self.state {
            CircuitState::Closed => {
                self.failure_count += 1;
                if self.failure_count >= self.failure_threshold {
                    self.open();
                }
            }
            CircuitState::HalfOpen => {
                // Failure in half-open state reopens circuit
                self.open();
            }
            CircuitState::Open => {
                // Already open, no-op
            }
        }
    }

    /// Record a success
    pub fn record_success(&mut self) {
        match self.state {
            CircuitState::Closed => {
                self.failure_count = 0; // Reset failure count
            }
            CircuitState::HalfOpen => {
                self.success_count += 1;
                if self.success_count >= self.success_threshold {
                    self.close();
                }
            }
            CircuitState::Open => {
                // Success in open state shouldn't happen (calls rejected)
            }
        }
    }

    /// Check if circuit is open
    pub fn is_open(&mut self) -> bool {
        // Check if we should transition to half-open
        if self.state == CircuitState::Open {
            if let Some(opened_at) = self.opened_at {
                if opened_at.elapsed() >= self.timeout {
                    self.half_open();
                }
            }
        }

        self.state == CircuitState::Open
    }

    /// Get current state
    pub fn state(&self) -> CircuitState {
        self.state
    }

    /// Open the circuit
    fn open(&mut self) {
        self.state = CircuitState::Open;
        self.opened_at = Some(Instant::now());
        self.failure_count = 0;
        self.success_count = 0;
    }

    /// Transition to half-open
    fn half_open(&mut self) {
        self.state = CircuitState::HalfOpen;
        self.success_count = 0;
    }

    /// Close the circuit
    fn close(&mut self) {
        self.state = CircuitState::Closed;
        self.failure_count = 0;
        self.success_count = 0;
        self.opened_at = None;
    }
}

impl Default for CircuitBreaker {
    fn default() -> Self {
        Self::new(5, Duration::from_secs(60))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_circuit_breaker_opens() {
        let mut cb = CircuitBreaker::new(3, Duration::from_secs(60));

        assert_eq!(cb.state(), CircuitState::Closed);

        // Record failures
        cb.record_failure();
        assert_eq!(cb.state(), CircuitState::Closed);

        cb.record_failure();
        assert_eq!(cb.state(), CircuitState::Closed);

        cb.record_failure();
        assert_eq!(cb.state(), CircuitState::Open);
    }

    #[test]
    fn test_circuit_breaker_half_open() {
        let mut cb = CircuitBreaker::new(2, Duration::from_millis(10));

        // Open circuit
        cb.record_failure();
        cb.record_failure();
        assert_eq!(cb.state(), CircuitState::Open);

        // Wait for timeout
        std::thread::sleep(Duration::from_millis(20));

        // Check should transition to half-open
        assert!(cb.is_open()); // Still technically open, but...
        assert_eq!(cb.state(), CircuitState::HalfOpen);
    }

    #[test]
    fn test_circuit_breaker_closes() {
        let mut cb = CircuitBreaker::new(2, Duration::from_millis(10));

        // Open circuit
        cb.record_failure();
        cb.record_failure();
        assert_eq!(cb.state(), CircuitState::Open);

        // Wait and transition to half-open
        std::thread::sleep(Duration::from_millis(20));
        cb.is_open();
        assert_eq!(cb.state(), CircuitState::HalfOpen);

        // Record successes to close
        cb.record_success();
        cb.record_success();
        cb.record_success();
        assert_eq!(cb.state(), CircuitState::Closed);
    }

    #[test]
    fn test_success_resets_failure_count() {
        let mut cb = CircuitBreaker::new(3, Duration::from_secs(60));

        cb.record_failure();
        assert_eq!(cb.failure_count, 1);

        cb.record_success();
        assert_eq!(cb.failure_count, 0);
        assert_eq!(cb.state(), CircuitState::Closed);
    }
}
