//! Watch mode infrastructure for live-updating displays
//!
//! Provides terminal management and refresh loop for watch-style commands

use anyhow::{Context, Result};
use std::io::{self, Write};
use std::time::{Duration, Instant};
use tokio::signal;
use tokio::time;

/// Watch mode configuration
pub struct WatchConfig {
    /// Refresh interval
    pub interval: Duration,

    /// Enable double-buffering to reduce flicker
    pub use_alternate_screen: bool,

    /// Clear screen between updates
    pub clear_screen: bool,
}

impl Default for WatchConfig {
    fn default() -> Self {
        Self {
            interval: Duration::from_secs(2),
            use_alternate_screen: true,
            clear_screen: true,
        }
    }
}

/// Watch mode controller
pub struct WatchMode {
    config: WatchConfig,
    start_time: Instant,
    iteration_count: u64,
}

impl WatchMode {
    /// Create a new watch mode controller
    pub fn new(config: WatchConfig) -> Self {
        Self {
            config,
            start_time: Instant::now(),
            iteration_count: 0,
        }
    }

    /// Enter alternate screen buffer (reduces flicker)
    pub fn enter_alternate_screen(&self) -> Result<()> {
        if self.config.use_alternate_screen {
            print!("\x1b[?1049h"); // Enter alternate screen
            print!("\x1b[?25l");   // Hide cursor
            io::stdout().flush()?;
        }
        Ok(())
    }

    /// Exit alternate screen buffer
    pub fn exit_alternate_screen(&self) -> Result<()> {
        if self.config.use_alternate_screen {
            print!("\x1b[?25h");   // Show cursor
            print!("\x1b[?1049l"); // Exit alternate screen
            io::stdout().flush()?;
        }
        Ok(())
    }

    /// Clear screen and move cursor to top-left
    pub fn clear_screen(&self) -> Result<()> {
        if self.config.clear_screen {
            print!("\x1b[2J");     // Clear entire screen
            print!("\x1b[H");      // Move cursor to home (0, 0)
            io::stdout().flush()?;
        }
        Ok(())
    }

    /// Run watch loop with async update function
    pub async fn run<F, Fut>(&mut self, mut update_fn: F) -> Result<()>
    where
        F: FnMut(u64) -> Fut,
        Fut: std::future::Future<Output = Result<()>>,
    {
        // Enter alternate screen
        self.enter_alternate_screen()?;

        // Setup Ctrl+C handler
        let mut interval = time::interval(self.config.interval);

        loop {
            tokio::select! {
                // Ctrl+C signal
                _ = signal::ctrl_c() => {
                    self.exit_alternate_screen()?;
                    println!("\nWatch mode interrupted.");
                    println!("Total iterations: {}", self.iteration_count);
                    println!("Total time: {:?}", self.start_time.elapsed());
                    break;
                }

                // Periodic update
                _ = interval.tick() => {
                    self.clear_screen()?;

                    // Call update function
                    if let Err(e) = update_fn(self.iteration_count).await {
                        self.exit_alternate_screen()?;
                        eprintln!("Error during update: {}", e);
                        return Err(e);
                    }

                    self.iteration_count += 1;
                }
            }
        }

        Ok(())
    }

    /// Get elapsed time since watch started
    pub fn elapsed(&self) -> Duration {
        self.start_time.elapsed()
    }

    /// Get iteration count
    pub fn iterations(&self) -> u64 {
        self.iteration_count
    }
}

/// Display watch mode header
pub fn print_watch_header(title: &str, iteration: u64, elapsed: Duration) {
    let dim = "\x1b[2m";
    let reset = "\x1b[0m";
    let bold = "\x1b[1m";

    println!("{}╭─ {} (Live) ───────────────────────────────────────{}", dim, title, reset);
    println!("{}│{}", dim, reset);
    println!(
        "{}│{}  {}Iteration:{} {}  {}Elapsed:{} {:?}  {}Refresh:{} 2s",
        dim, reset, bold, reset, iteration, bold, reset, elapsed, bold, reset
    );
    println!("{}│{}  Press {}Ctrl+C{} to exit", dim, reset, bold, reset);
    println!("{}│{}", dim, reset);
}

/// Display watch mode footer
pub fn print_watch_footer() {
    let dim = "\x1b[2m";
    let reset = "\x1b[0m";

    println!("{}│{}", dim, reset);
    println!("{}╰──────────────────────────────────────────────────────────{}", dim, reset);
}

/// Display delta indicator for numeric values
pub fn format_delta(old_value: f64, new_value: f64) -> String {
    let delta = new_value - old_value;
    let color = if delta > 0.0 {
        "\x1b[32m" // Green
    } else if delta < 0.0 {
        "\x1b[31m" // Red
    } else {
        "\x1b[2m"  // Dim
    };
    let reset = "\x1b[0m";

    if delta.abs() < 0.01 {
        format!("{}→{}", "\x1b[2m", reset) // No change
    } else {
        format!("{}{:+.2}{}", color, delta, reset)
    }
}

/// Display delta indicator for count values
pub fn format_count_delta(old_value: u64, new_value: u64) -> String {
    let delta = new_value as i64 - old_value as i64;
    let color = if delta > 0 {
        "\x1b[32m" // Green (increase)
    } else if delta < 0 {
        "\x1b[31m" // Red (decrease)
    } else {
        "\x1b[2m"  // Dim (no change)
    };
    let reset = "\x1b[0m";

    if delta == 0 {
        format!("{}→{}", "\x1b[2m", reset) // No change
    } else {
        format!("{}{:+}{}", color, delta, reset)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_watch_config_default() {
        let config = WatchConfig::default();
        assert_eq!(config.interval, Duration::from_secs(2));
        assert!(config.use_alternate_screen);
        assert!(config.clear_screen);
    }

    #[test]
    fn test_watch_mode_creation() {
        let config = WatchConfig::default();
        let watch = WatchMode::new(config);

        assert_eq!(watch.iterations(), 0);
        assert!(watch.elapsed() < Duration::from_millis(100));
    }

    #[test]
    fn test_format_delta_positive() {
        let delta_str = format_delta(10.0, 15.5);
        assert!(delta_str.contains("+5.50"));
    }

    #[test]
    fn test_format_delta_negative() {
        let delta_str = format_delta(10.0, 5.0);
        assert!(delta_str.contains("-5.00"));
    }

    #[test]
    fn test_format_delta_zero() {
        let delta_str = format_delta(10.0, 10.0);
        assert!(delta_str.contains("→"));
    }

    #[test]
    fn test_format_count_delta_positive() {
        let delta_str = format_count_delta(10, 15);
        assert!(delta_str.contains("+5"));
    }

    #[test]
    fn test_format_count_delta_negative() {
        let delta_str = format_count_delta(15, 10);
        assert!(delta_str.contains("-5"));
    }

    #[test]
    fn test_format_count_delta_zero() {
        let delta_str = format_count_delta(10, 10);
        assert!(delta_str.contains("→"));
    }

    #[tokio::test]
    async fn test_watch_mode_elapsed() {
        let config = WatchConfig::default();
        let watch = WatchMode::new(config);

        tokio::time::sleep(Duration::from_millis(50)).await;

        assert!(watch.elapsed() >= Duration::from_millis(50));
    }
}
