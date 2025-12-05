//! Terminal UI helpers for consistent output styling.
//! v0.0.43: Added Spinner for stage progress animation.

use std::io::{self, Write};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};

/// ANSI color codes using true color (24-bit)
pub mod colors {
    pub const HEADER: &str = "\x1b[38;2;255;210;120m";
    pub const OK: &str = "\x1b[38;2;120;255;120m";
    pub const ERR: &str = "\x1b[38;2;255;100;100m";
    pub const WARN: &str = "\x1b[38;2;255;200;100m";
    pub const DIM: &str = "\x1b[38;2;140;140;140m";
    pub const CYAN: &str = "\x1b[38;2;100;200;255m";
    pub const BOLD: &str = "\x1b[1m";
    pub const RESET: &str = "\x1b[0m";
}

/// Unicode symbols
pub mod symbols {
    pub const OK: &str = "✓";
    pub const ERR: &str = "✗";
    pub const ARROW: &str = "›";
    pub const SPINNER: [&str; 4] = ["⠋", "⠙", "⠹", "⠸"];
    pub const PROGRESS_FULL: &str = "█";
    pub const PROGRESS_EMPTY: &str = "░";
}

/// Horizontal rule
pub const HR: &str =
    "──────────────────────────────────────────────────────────────────────────────";

/// Print a styled header with version
pub fn print_header(name: &str, version: &str) {
    println!();
    println!("{}{} v{}{}", colors::HEADER, name, version, colors::RESET);
    println!("{}{}{}", colors::DIM, HR, colors::RESET);
}

/// Print a footer with horizontal rule
pub fn print_footer() {
    println!("{}{}{}", colors::DIM, HR, colors::RESET);
    println!();
}

/// Print a section header like [section] description
pub fn print_section(section: &str, description: &str) {
    println!(
        "{}[{}{}{}]{} {}",
        colors::DIM,
        colors::RESET,
        section,
        colors::DIM,
        colors::RESET,
        description
    );
}

/// Print an OK line with checkmark
pub fn print_ok(message: &str) {
    println!(
        "  {}{}{} {}",
        colors::OK,
        symbols::OK,
        colors::RESET,
        message
    );
}

/// Print an error line with X
pub fn print_err(message: &str) {
    println!(
        "  {}{}{} {}",
        colors::ERR,
        symbols::ERR,
        colors::RESET,
        message
    );
}

/// Print a key-value pair with alignment
pub fn print_kv(key: &str, value: &str, key_width: usize) {
    println!("  {:width$} {}", key, value, width = key_width);
}

/// Print a key-value pair with colored value
pub fn print_kv_status(key: &str, value: &str, status_color: &str, key_width: usize) {
    println!(
        "  {:width$} {}{}{}",
        key,
        status_color,
        value,
        colors::RESET,
        width = key_width
    );
}

/// Format a progress bar
pub fn progress_bar(progress: f32, width: usize) -> String {
    let filled = (progress * width as f32) as usize;
    let empty = width.saturating_sub(filled);
    format!(
        "[{}{}]",
        symbols::PROGRESS_FULL.repeat(filled),
        symbols::PROGRESS_EMPTY.repeat(empty)
    )
}

/// Format bytes as human readable
pub fn format_bytes(bytes: u64) -> String {
    const KB: u64 = 1024;
    const MB: u64 = KB * 1024;
    const GB: u64 = MB * 1024;

    if bytes >= GB {
        format!("{:.1} GiB", bytes as f64 / GB as f64)
    } else if bytes >= MB {
        format!("{:.1} MiB", bytes as f64 / MB as f64)
    } else if bytes >= KB {
        format!("{:.1} KiB", bytes as f64 / KB as f64)
    } else {
        format!("{} B", bytes)
    }
}

/// Format duration as human readable
pub fn format_duration(seconds: u64) -> String {
    if seconds >= 3600 {
        let hours = seconds / 3600;
        let mins = (seconds % 3600) / 60;
        format!("{:02}:{:02}:{:02}", hours, mins, seconds % 60)
    } else if seconds >= 60 {
        let mins = seconds / 60;
        format!("{:02}:{:02}", mins, seconds % 60)
    } else {
        format!("00:00:{:02}", seconds)
    }
}

/// Print without newline and flush
pub fn print_inline(message: &str) {
    print!("{}", message);
    io::stdout().flush().ok();
}

/// Clear current line
pub fn clear_line() {
    print!("\r\x1b[K");
    io::stdout().flush().ok();
}

/// Move cursor up n lines
pub fn cursor_up(n: usize) {
    print!("\x1b[{}A", n);
    io::stdout().flush().ok();
}

// === v0.0.43: Spinner for stage progress ===

/// Spinner state for animated progress display
#[derive(Clone)]
pub struct Spinner {
    message: String,
    frame: usize,
    start: Instant,
    running: Arc<AtomicBool>,
}

impl Spinner {
    /// Create a new spinner with message
    pub fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
            frame: 0,
            running: Arc::new(AtomicBool::new(true)),
            start: Instant::now(),
        }
    }

    /// Get the current spinner frame character
    pub fn frame_char(&self) -> &'static str {
        symbols::SPINNER[self.frame % symbols::SPINNER.len()]
    }

    /// Advance to next frame
    pub fn tick(&mut self) {
        self.frame = self.frame.wrapping_add(1);
    }

    /// Render current state (call in loop)
    pub fn render(&self) {
        let elapsed = self.start.elapsed().as_secs();
        let frame = symbols::SPINNER[self.frame % symbols::SPINNER.len()];
        print!(
            "\r{}{}{} {} {}({}s){}",
            colors::CYAN, frame, colors::RESET,
            self.message,
            colors::DIM, elapsed, colors::RESET
        );
        io::stdout().flush().ok();
    }

    /// Mark as complete with success
    pub fn success(&self, final_msg: Option<&str>) {
        clear_line();
        let msg = final_msg.unwrap_or(&self.message);
        let elapsed = self.start.elapsed().as_millis();
        println!(
            "{}{}{} {} {}({}ms){}",
            colors::OK, symbols::OK, colors::RESET,
            msg,
            colors::DIM, elapsed, colors::RESET
        );
    }

    /// Mark as complete with error
    pub fn error(&self, final_msg: Option<&str>) {
        clear_line();
        let msg = final_msg.unwrap_or(&self.message);
        let elapsed = self.start.elapsed().as_millis();
        println!(
            "{}{}{} {} {}({}ms){}",
            colors::ERR, symbols::ERR, colors::RESET,
            msg,
            colors::DIM, elapsed, colors::RESET
        );
    }

    /// Mark as skipped
    pub fn skip(&self, reason: &str) {
        clear_line();
        println!(
            "{}-{} {} {}({}){}",
            colors::DIM, colors::RESET,
            self.message,
            colors::DIM, reason, colors::RESET
        );
    }

    /// Check if still running
    pub fn is_running(&self) -> bool {
        self.running.load(Ordering::Relaxed)
    }

    /// Stop the spinner
    pub fn stop(&self) {
        self.running.store(false, Ordering::Relaxed);
    }

    /// Get elapsed time
    pub fn elapsed(&self) -> Duration {
        self.start.elapsed()
    }
}

/// Stage progress tracker for pipeline visualization
pub struct StageProgress { stages: Vec<StageInfo>, current: Option<usize> }

struct StageInfo { name: String, status: StageStatus, duration_ms: Option<u64> }

#[derive(Clone, Copy, PartialEq)]
pub enum StageStatus { Pending, Running, Complete, Skipped, Error }

impl StageProgress {
    /// Create with stage names
    pub fn new(stage_names: &[&str]) -> Self {
        Self {
            stages: stage_names.iter().map(|n| StageInfo {
                name: n.to_string(),
                status: StageStatus::Pending,
                duration_ms: None,
            }).collect(),
            current: None,
        }
    }

    /// Start a stage
    pub fn start(&mut self, name: &str) {
        if let Some(idx) = self.stages.iter().position(|s| s.name == name) {
            self.stages[idx].status = StageStatus::Running;
            self.current = Some(idx);
        }
    }

    /// Complete current stage
    pub fn complete(&mut self, duration_ms: u64) {
        if let Some(idx) = self.current {
            self.stages[idx].status = StageStatus::Complete;
            self.stages[idx].duration_ms = Some(duration_ms);
        }
    }

    /// Skip a stage
    pub fn skip(&mut self, name: &str) {
        if let Some(idx) = self.stages.iter().position(|s| s.name == name) {
            self.stages[idx].status = StageStatus::Skipped;
        }
    }

    /// Mark stage as error
    pub fn error(&mut self, duration_ms: u64) {
        if let Some(idx) = self.current {
            self.stages[idx].status = StageStatus::Error;
            self.stages[idx].duration_ms = Some(duration_ms);
        }
    }

    /// Render progress line
    pub fn render_line(&self) -> String {
        self.stages.iter().map(|s| {
            match s.status {
                StageStatus::Pending => format!("{}○{}", colors::DIM, colors::RESET),
                StageStatus::Running => format!("{}◉{}", colors::CYAN, colors::RESET),
                StageStatus::Complete => format!("{}●{}", colors::OK, colors::RESET),
                StageStatus::Skipped => format!("{}-{}", colors::DIM, colors::RESET),
                StageStatus::Error => format!("{}●{}", colors::ERR, colors::RESET),
            }
        }).collect::<Vec<_>>().join(" ")
    }

    /// Get summary string
    pub fn summary(&self) -> String {
        let completed = self.stages.iter().filter(|s| s.status == StageStatus::Complete).count();
        let total = self.stages.len();
        let total_ms: u64 = self.stages.iter().filter_map(|s| s.duration_ms).sum();
        format!("{}/{} stages ({}ms)", completed, total, total_ms)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_bytes() {
        assert_eq!(format_bytes(500), "500 B");
        assert_eq!(format_bytes(1024), "1.0 KiB");
        assert_eq!(format_bytes(1024 * 1024), "1.0 MiB");
        assert_eq!(format_bytes(1024 * 1024 * 1024), "1.0 GiB");
    }

    #[test]
    fn test_progress_bar() {
        assert_eq!(progress_bar(0.5, 10), "[█████░░░░░]");
        assert_eq!(progress_bar(1.0, 10), "[██████████]");
        assert_eq!(progress_bar(0.0, 10), "[░░░░░░░░░░]");
    }

    #[test]
    fn test_format_duration() {
        assert_eq!(format_duration(5), "00:00:05");
        assert_eq!(format_duration(65), "01:05");
        assert_eq!(format_duration(3665), "01:01:05");
    }

    // v0.0.43 Spinner tests
    #[test]
    fn test_spinner_new() {
        let spinner = Spinner::new("Loading...");
        assert!(spinner.is_running());
        assert_eq!(spinner.frame_char(), symbols::SPINNER[0]);
    }

    #[test]
    fn test_spinner_tick() {
        let mut spinner = Spinner::new("Loading...");
        spinner.tick();
        assert_eq!(spinner.frame_char(), symbols::SPINNER[1]);
        spinner.tick();
        assert_eq!(spinner.frame_char(), symbols::SPINNER[2]);
    }

    #[test]
    fn test_stage_progress() {
        let mut progress = StageProgress::new(&["translator", "probes", "specialist"]);
        progress.start("translator");
        progress.complete(100);
        progress.start("probes");
        progress.complete(200);
        progress.skip("specialist");

        assert!(progress.summary().contains("2/3"));
    }

    #[test]
    fn test_stage_status_render() {
        let mut progress = StageProgress::new(&["a", "b"]);
        progress.start("a");
        let line = progress.render_line();
        assert!(line.contains("◉")); // Running indicator
    }
}
