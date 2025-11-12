//! Exponential backoff with jitter

use rand::Rng;
use serde::{Deserialize, Serialize};
use std::time::Duration;

/// Backoff configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BackoffConfig {
    /// Initial backoff duration (milliseconds)
    pub floor_ms: u64,
    /// Maximum backoff duration (milliseconds)
    pub ceiling_ms: u64,
    /// Jitter percentage (0.0-1.0)
    pub jitter: f64,
    /// Multiplier for exponential backoff
    pub multiplier: f64,
}

impl Default for BackoffConfig {
    fn default() -> Self {
        Self {
            floor_ms: 100,      // 100ms minimum
            ceiling_ms: 30_000, // 30s maximum
            jitter: 0.25,       // ±25% jitter
            multiplier: 2.0,    // Double each time
        }
    }
}

/// Backoff state for a task
#[derive(Debug, Clone)]
pub struct BackoffState {
    config: BackoffConfig,
    current_ms: u64,
    attempt: u32,
}

impl BackoffState {
    pub fn new(config: BackoffConfig) -> Self {
        Self {
            current_ms: config.floor_ms,
            attempt: 0,
            config,
        }
    }

    /// Calculate next backoff duration with jitter
    pub fn next_backoff(&mut self) -> Duration {
        self.attempt += 1;

        // Exponential backoff
        let base_ms = (self.config.floor_ms as f64
            * self.config.multiplier.powi(self.attempt as i32 - 1))
            .min(self.config.ceiling_ms as f64);

        // Add jitter
        let jitter_range = base_ms * self.config.jitter;
        let mut rng = rand::thread_rng();
        let jitter = rng.gen_range(-jitter_range..=jitter_range);

        let duration_ms = (base_ms + jitter)
            .max(self.config.floor_ms as f64)
            .min(self.config.ceiling_ms as f64);

        self.current_ms = duration_ms as u64;

        Duration::from_millis(self.current_ms)
    }

    /// Reset backoff state (after successful operation)
    pub fn reset(&mut self) {
        self.attempt = 0;
        self.current_ms = self.config.floor_ms;
    }

    /// Get current attempt count
    pub fn attempts(&self) -> u32 {
        self.attempt
    }

    /// Get current backoff duration (without advancing)
    pub fn current(&self) -> Duration {
        Duration::from_millis(self.current_ms)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_exponential_backoff() {
        let config = BackoffConfig {
            floor_ms: 100,
            ceiling_ms: 1000,
            jitter: 0.0, // Disable jitter for deterministic test
            multiplier: 2.0,
        };

        let mut backoff = BackoffState::new(config);

        // First backoff: floor_ms * 2^0 = 100ms
        let d1 = backoff.next_backoff();
        assert_eq!(d1.as_millis(), 100);

        // Second backoff: floor_ms * 2^1 = 200ms
        let d2 = backoff.next_backoff();
        assert_eq!(d2.as_millis(), 200);

        // Third backoff: floor_ms * 2^2 = 400ms
        let d3 = backoff.next_backoff();
        assert_eq!(d3.as_millis(), 400);

        // Fourth backoff: floor_ms * 2^3 = 800ms
        let d4 = backoff.next_backoff();
        assert_eq!(d4.as_millis(), 800);

        // Fifth backoff: floor_ms * 2^4 = 1600ms, capped at ceiling_ms = 1000ms
        let d5 = backoff.next_backoff();
        assert_eq!(d5.as_millis(), 1000);
    }

    #[test]
    fn test_backoff_reset() {
        let config = BackoffConfig::default();
        let mut backoff = BackoffState::new(config.clone());

        backoff.next_backoff();
        backoff.next_backoff();
        assert!(backoff.attempts() > 0);

        backoff.reset();
        assert_eq!(backoff.attempts(), 0);
        assert_eq!(backoff.current().as_millis(), config.floor_ms as u128);
    }

    #[test]
    fn test_jitter_within_bounds() {
        let config = BackoffConfig {
            floor_ms: 1000,
            ceiling_ms: 10000,
            jitter: 0.25,
            multiplier: 2.0,
        };

        let mut backoff = BackoffState::new(config.clone());

        for _ in 0..10 {
            let duration = backoff.next_backoff();
            let ms = duration.as_millis() as f64;

            // Should be within bounds
            assert!(ms >= config.floor_ms as f64);
            assert!(ms <= config.ceiling_ms as f64);
        }
    }
}
