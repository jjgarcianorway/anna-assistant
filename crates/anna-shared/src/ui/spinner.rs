//! Spinner for animated progress display (v0.0.213).

use std::io::{self, Write};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};

use super::colors;
use super::symbols;
use super::terminal::clear_line;

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
            colors::CYAN,
            frame,
            colors::RESET,
            self.message,
            colors::DIM,
            elapsed,
            colors::RESET
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
            colors::OK,
            symbols::OK,
            colors::RESET,
            msg,
            colors::DIM,
            elapsed,
            colors::RESET
        );
    }

    /// Mark as complete with error
    pub fn error(&self, final_msg: Option<&str>) {
        clear_line();
        let msg = final_msg.unwrap_or(&self.message);
        let elapsed = self.start.elapsed().as_millis();
        println!(
            "{}{}{} {} {}({}ms){}",
            colors::ERR,
            symbols::ERR,
            colors::RESET,
            msg,
            colors::DIM,
            elapsed,
            colors::RESET
        );
    }

    /// Mark as skipped
    pub fn skip(&self, reason: &str) {
        clear_line();
        println!(
            "{}-{} {} {}({}){}",
            colors::DIM,
            colors::RESET,
            self.message,
            colors::DIM,
            reason,
            colors::RESET
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
