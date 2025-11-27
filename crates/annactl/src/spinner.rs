//! Terminal spinner for thinking animation
//! v0.15.8: Old-school hacker aesthetic

use owo_colors::OwoColorize;
use std::io::{self, Write};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};

/// Braille spinner frames for smooth animation
const SPINNER_FRAMES: &[&str] = &["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧", "⠇", "⠏"];

/// ASCII fallback spinner
const ASCII_FRAMES: &[&str] = &["|", "/", "-", "\\"];

/// Spinner for showing thinking state
pub struct Spinner {
    running: Arc<AtomicBool>,
    handle: Option<std::thread::JoinHandle<()>>,
    start_time: Instant,
}

impl Spinner {
    /// Start a new spinner with message
    pub fn new(message: &str) -> Self {
        let running = Arc::new(AtomicBool::new(true));
        let running_clone = running.clone();
        let message = message.to_string();

        // Print the initial line
        print!("\r{}  {} {}", "[anna]".bright_cyan(), SPINNER_FRAMES[0].bright_yellow(), message.dimmed());
        let _ = io::stdout().flush();

        let handle = std::thread::spawn(move || {
            let mut frame = 0;
            let frames = SPINNER_FRAMES;

            while running_clone.load(Ordering::Relaxed) {
                frame = (frame + 1) % frames.len();
                print!("\r{}  {} {}", "[anna]".bright_cyan(), frames[frame].bright_yellow(), message.dimmed());
                let _ = io::stdout().flush();
                std::thread::sleep(Duration::from_millis(80));
            }
        });

        Self {
            running,
            handle: Some(handle),
            start_time: Instant::now(),
        }
    }

    /// Stop spinner and return elapsed time
    pub fn stop(mut self) -> Duration {
        self.running.store(false, Ordering::Relaxed);
        if let Some(handle) = self.handle.take() {
            let _ = handle.join();
        }

        let elapsed = self.start_time.elapsed();

        // Clear the spinner line
        print!("\r{}\r", " ".repeat(80));
        let _ = io::stdout().flush();

        elapsed
    }

    /// Stop spinner and show completion with timing
    pub fn finish(self) -> Duration {
        let elapsed = self.stop();

        // Print completion line
        println!(
            "{}  {} {}",
            "[anna]".bright_cyan(),
            "✓".bright_green(),
            format!("({:.1}s)", elapsed.as_secs_f64()).dimmed()
        );
        println!();

        elapsed
    }
}

impl Drop for Spinner {
    fn drop(&mut self) {
        self.running.store(false, Ordering::Relaxed);
    }
}

/// Print user question in styled format
pub fn print_question(question: &str) {
    println!(
        "{}  {}",
        "[you]".bright_green(),
        question.white()
    );
}
