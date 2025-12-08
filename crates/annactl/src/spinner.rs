//! Animated spinner for async operations.
//! v0.0.142: Real-time animated spinner during LLM calls.

use anna_shared::ui::colors;
use std::io::{self, Write};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::thread::{self, JoinHandle};
use std::time::{Duration, Instant};

/// ASCII spinner frames for animation
const SPINNER_FRAMES: [&str; 8] = ["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧"];

/// Animated spinner that runs in a background thread
pub struct AnimatedSpinner {
    running: Arc<AtomicBool>,
    handle: Option<JoinHandle<()>>,
}

// v0.0.148: Spinner unused now that live_request shows progress events
// Kept for future use (e.g., non-interactive mode or fallback)
#[allow(dead_code)]
impl AnimatedSpinner {
    /// Start a new animated spinner with the given message
    pub fn start(message: impl Into<String>) -> Self {
        let message = message.into();
        let running = Arc::new(AtomicBool::new(true));
        let running_clone = Arc::clone(&running);

        let handle = thread::spawn(move || {
            let start = Instant::now();
            let mut frame = 0;

            while running_clone.load(Ordering::Relaxed) {
                let elapsed = start.elapsed().as_secs();
                let spinner = SPINNER_FRAMES[frame % SPINNER_FRAMES.len()];

                // Clear line and print spinner
                print!(
                    "\r\x1b[K{}{}{} {} {}({}s){}",
                    colors::CYAN,
                    spinner,
                    colors::RESET,
                    message,
                    colors::DIM,
                    elapsed,
                    colors::RESET
                );
                let _ = io::stdout().flush();

                frame += 1;
                thread::sleep(Duration::from_millis(80));
            }

            // Clear the spinner line when done
            print!("\r\x1b[K");
            let _ = io::stdout().flush();
        });

        Self {
            running,
            handle: Some(handle),
        }
    }

    /// Stop the spinner
    pub fn stop(mut self) {
        self.running.store(false, Ordering::Relaxed);
        if let Some(handle) = self.handle.take() {
            let _ = handle.join();
        }
    }

    /// Stop with a success message
    #[allow(dead_code)]
    pub fn success(mut self, message: &str) {
        self.running.store(false, Ordering::Relaxed);
        if let Some(handle) = self.handle.take() {
            let _ = handle.join();
        }
        println!(
            "{}✓{} {}",
            colors::OK,
            colors::RESET,
            message
        );
    }

    /// Stop with an error message
    #[allow(dead_code)]
    pub fn error(mut self, message: &str) {
        self.running.store(false, Ordering::Relaxed);
        if let Some(handle) = self.handle.take() {
            let _ = handle.join();
        }
        println!(
            "{}✗{} {}",
            colors::ERR,
            colors::RESET,
            message
        );
    }
}

impl Drop for AnimatedSpinner {
    fn drop(&mut self) {
        self.running.store(false, Ordering::Relaxed);
        if let Some(handle) = self.handle.take() {
            let _ = handle.join();
        }
    }
}

/// Spinner with stage updates - shows what Anna is doing
/// TODO: Use this for streaming stage updates
#[allow(dead_code)]
pub struct StageSpinner {
    running: Arc<AtomicBool>,
    stage: Arc<std::sync::Mutex<String>>,
    handle: Option<JoinHandle<()>>,
}

#[allow(dead_code)]
impl StageSpinner {
    /// Start a stage-aware spinner
    pub fn start() -> Self {
        let running = Arc::new(AtomicBool::new(true));
        let stage = Arc::new(std::sync::Mutex::new("Thinking".to_string()));
        let running_clone = Arc::clone(&running);
        let stage_clone = Arc::clone(&stage);

        let handle = thread::spawn(move || {
            let start = Instant::now();
            let mut frame = 0;

            while running_clone.load(Ordering::Relaxed) {
                let elapsed = start.elapsed().as_secs();
                let spinner = SPINNER_FRAMES[frame % SPINNER_FRAMES.len()];
                let current_stage = stage_clone.lock().map(|s| s.clone()).unwrap_or_default();

                print!(
                    "\r\x1b[K{}{}{} {}... {}({}s){}",
                    colors::CYAN,
                    spinner,
                    colors::RESET,
                    current_stage,
                    colors::DIM,
                    elapsed,
                    colors::RESET
                );
                let _ = io::stdout().flush();

                frame += 1;
                thread::sleep(Duration::from_millis(80));
            }

            print!("\r\x1b[K");
            let _ = io::stdout().flush();
        });

        Self {
            running,
            stage,
            handle: Some(handle),
        }
    }

    /// Update the current stage message
    pub fn set_stage(&self, new_stage: &str) {
        if let Ok(mut s) = self.stage.lock() {
            *s = new_stage.to_string();
        }
    }

    /// Stop the spinner
    pub fn stop(mut self) {
        self.running.store(false, Ordering::Relaxed);
        if let Some(handle) = self.handle.take() {
            let _ = handle.join();
        }
    }
}

impl Drop for StageSpinner {
    fn drop(&mut self) {
        self.running.store(false, Ordering::Relaxed);
        if let Some(handle) = self.handle.take() {
            let _ = handle.join();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_spinner_creates_and_stops() {
        let spinner = AnimatedSpinner::start("Testing");
        thread::sleep(Duration::from_millis(200));
        spinner.stop();
    }

    #[test]
    fn test_stage_spinner() {
        let spinner = StageSpinner::start();
        spinner.set_stage("Loading");
        thread::sleep(Duration::from_millis(100));
        spinner.set_stage("Processing");
        thread::sleep(Duration::from_millis(100));
        spinner.stop();
    }
}
