//! ASCII spinner animation for waiting states.
//! v0.3.4: Initial implementation

use std::io::{Write, stdout};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::thread;
use std::time::Duration;

/// Spinner animation frames (braille dots)
const FRAMES: &[&str] = &["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧", "⠇", "⠏"];

/// ANSI escape codes
const DIM: &str = "\x1b[2m";
const RESET: &str = "\x1b[0m";

/// Spinner handle that stops when dropped
pub struct Spinner {
    running: Arc<AtomicBool>,
    handle: Option<thread::JoinHandle<()>>,
}

impl Spinner {
    /// Start a new spinner with optional message
    pub fn new(message: &str) -> Self {
        let running = Arc::new(AtomicBool::new(true));
        let running_clone = running.clone();
        let message = message.to_string();

        let handle = thread::spawn(move || {
            let mut frame_idx = 0;
            while running_clone.load(Ordering::Relaxed) {
                // Clear line and print spinner
                print!("\r{}{} {}{}\x1b[K",
                    DIM,
                    FRAMES[frame_idx],
                    message,
                    RESET
                );
                let _ = stdout().flush();

                frame_idx = (frame_idx + 1) % FRAMES.len();
                thread::sleep(Duration::from_millis(80));
            }
            // Clear spinner line
            print!("\r\x1b[K");
            let _ = stdout().flush();
        });

        Self {
            running,
            handle: Some(handle),
        }
    }

    /// Stop the spinner
    pub fn stop(&mut self) {
        self.running.store(false, Ordering::Relaxed);
        if let Some(handle) = self.handle.take() {
            let _ = handle.join();
        }
    }
}

impl Drop for Spinner {
    fn drop(&mut self) {
        self.stop();
    }
}
