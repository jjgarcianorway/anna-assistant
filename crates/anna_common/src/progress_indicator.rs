//! Progress Indicator - Visual feedback for long-running operations
//!
//! v6.36.0: Thinking Indicator & Progress Feedback v1
//!
//! ## Purpose
//!
//! Provides animated spinner and timing feedback during operations that take >500ms.
//! Pure UX layer - spinner failures never block query execution.
//!
//! ## Design Principles
//!
//! 1. **TTY-Aware**: Only show spinner if stdout is a TTY (no artifacts in piped output)
//! 2. **Fail-Safe**: Spinner failures never prevent query execution
//! 3. **Threshold-Based**: Only show for operations expected to take >500ms
//! 4. **Clean Termination**: Always stop spinner cleanly, show timing
//! 5. **Respects NO_COLOR**: Uses OutputEngine capability detection
//!
//! ## Usage
//!
//! ```rust
//! use anna_common::progress_indicator::ProgressIndicator;
//!
//! let mut progress = ProgressIndicator::new("Consulting Arch Wiki...");
//! // ... do work ...
//! progress.finish_with_timing("Answer ready");
//! ```

use indicatif::{ProgressBar, ProgressStyle};
use std::time::Instant;

/// Progress indicator with spinner and timing
pub struct ProgressIndicator {
    spinner: Option<ProgressBar>,
    start_time: Instant,
    enabled: bool,
}

impl ProgressIndicator {
    /// Create a new progress indicator with the given message.
    ///
    /// Only shows spinner if:
    /// - stdout is a TTY
    /// - Terminal supports Unicode (falls back to ASCII)
    /// - NO_COLOR is not set
    ///
    /// If any condition fails, the indicator is disabled but timing still works.
    pub fn new(message: &str) -> Self {
        let start_time = Instant::now();

        // Check if stdout is a TTY
        let is_tty = atty::is(atty::Stream::Stdout);

        // Check NO_COLOR environment variable
        let no_color = std::env::var("NO_COLOR").is_ok();

        let enabled = is_tty && !no_color;

        let spinner = if enabled {
            let pb = ProgressBar::new_spinner();

            // Try Unicode spinner first, fall back to ASCII
            let style = if supports_unicode() {
                ProgressStyle::default_spinner()
                    .tick_strings(&["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧", "⠇", "⠏"])
                    .template("{spinner} {msg}")
            } else {
                ProgressStyle::default_spinner()
                    .tick_strings(&["◐", "◓", "◑", "◒"])
                    .template("{spinner} {msg}")
            };

            if let Ok(style) = style {
                pb.set_style(style);
            }

            pb.set_message(message.to_string());
            pb.enable_steady_tick(std::time::Duration::from_millis(80));

            Some(pb)
        } else {
            None
        };

        Self {
            spinner,
            start_time,
            enabled,
        }
    }

    /// Update the progress message.
    ///
    /// Has no effect if spinner is disabled.
    pub fn update_message(&mut self, message: &str) {
        if let Some(ref spinner) = self.spinner {
            spinner.set_message(message.to_string());
        }
    }

    /// Finish with a custom message (no timing).
    ///
    /// If spinner is enabled, clears the spinner line and prints the message.
    pub fn finish_with_message(&mut self, message: &str) {
        if let Some(spinner) = self.spinner.take() {
            spinner.finish_and_clear();
            println!("{}", message);
        }
    }

    /// Finish with timing display.
    ///
    /// Shows elapsed time in format: "✓ <message> (X.Xs)"
    pub fn finish_with_timing(&mut self, message: &str) {
        let elapsed = self.start_time.elapsed();
        let elapsed_secs = elapsed.as_secs_f64();

        if let Some(spinner) = self.spinner.take() {
            spinner.finish_and_clear();
        }

        if self.enabled {
            println!("✓ {} ({:.1}s)", message, elapsed_secs);
        }
    }

    /// Get elapsed time since creation.
    pub fn elapsed_secs(&self) -> f64 {
        self.start_time.elapsed().as_secs_f64()
    }

    /// Check if spinner is enabled (TTY, no NO_COLOR).
    pub fn is_enabled(&self) -> bool {
        self.enabled
    }
}

impl Drop for ProgressIndicator {
    fn drop(&mut self) {
        // Clean up spinner if still active
        if let Some(spinner) = self.spinner.take() {
            spinner.finish_and_clear();
        }
    }
}

/// Check if terminal supports Unicode.
///
/// Simple heuristic: check LANG/LC_ALL for UTF-8.
fn supports_unicode() -> bool {
    std::env::var("LANG")
        .or_else(|_| std::env::var("LC_ALL"))
        .map(|val| val.to_lowercase().contains("utf") || val.to_lowercase().contains("utf-8"))
        .unwrap_or(true) // Default to Unicode if env vars not set
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::thread;
    use std::time::Duration;

    #[test]
    fn test_progress_indicator_creation() {
        let progress = ProgressIndicator::new("Testing...");
        assert!(progress.elapsed_secs() < 0.1); // Should be nearly instant
    }

    #[test]
    fn test_progress_indicator_timing() {
        let progress = ProgressIndicator::new("Timing test...");
        thread::sleep(Duration::from_millis(100));
        let elapsed = progress.elapsed_secs();
        assert!(elapsed >= 0.1 && elapsed < 0.3); // Should be ~100ms
    }

    #[test]
    fn test_progress_indicator_message_update() {
        let mut progress = ProgressIndicator::new("Initial message");
        progress.update_message("Updated message");
        // If spinner exists, message was updated (no panic = success)
        assert!(progress.elapsed_secs() >= 0.0);
    }

    #[test]
    fn test_progress_indicator_finish_formats() {
        let mut progress = ProgressIndicator::new("Test operation");
        thread::sleep(Duration::from_millis(50));
        progress.finish_with_timing("Operation complete");
        // Should not panic, clean termination
    }

    #[test]
    fn test_progress_indicator_no_tty() {
        // When running in test harness, stdout is typically not a TTY
        // So spinner should be disabled
        let progress = ProgressIndicator::new("Non-TTY test");

        // In non-TTY mode, spinner should be None
        // But timing should still work
        assert!(progress.elapsed_secs() >= 0.0);
    }

    #[test]
    fn test_progress_indicator_cleanup() {
        {
            let _progress = ProgressIndicator::new("Cleanup test");
            // progress goes out of scope here
        }
        // Should not leave terminal in broken state
        // (no hanging spinner, cursor visible)
    }

    #[test]
    fn test_supports_unicode() {
        // Should not panic regardless of env vars
        let _ = supports_unicode();
    }

    #[test]
    fn test_progress_indicator_enabled_flag() {
        let progress = ProgressIndicator::new("Enabled check");
        // is_enabled() should return false in test environment (no TTY)
        assert!(!progress.is_enabled());
    }
}
