//! Terminal spinner for thinking animation
//! v0.15.8: Old-school hacker aesthetic
//! v0.27.0: SSH-friendly with TTY detection and slower updates

use owo_colors::OwoColorize;
use std::io::{self, IsTerminal, Write};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};

/// Braille spinner frames for smooth animation
const SPINNER_FRAMES: &[&str] = &["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧", "⠇", "⠏"];

/// ASCII fallback spinner
const ASCII_FRAMES: &[&str] = &["|", "/", "-", "\\"];

/// Spinner update interval (ms) - slower for SSH stability
const SPINNER_INTERVAL_MS: u64 = 200;

/// Spinner for showing thinking state
pub struct Spinner {
    running: Arc<AtomicBool>,
    handle: Option<std::thread::JoinHandle<()>>,
    start_time: Instant,
    is_tty: bool,
}

impl Spinner {
    /// Start a new spinner with message
    pub fn new(message: &str) -> Self {
        let running = Arc::new(AtomicBool::new(true));
        let running_clone = running.clone();
        let message = message.to_string();
        let is_tty = io::stdout().is_terminal();

        // For non-TTY (piped output, scripts), just print once without spinner
        if !is_tty {
            println!("[anna]  ... {}", message);
            return Self {
                running,
                handle: None,
                start_time: Instant::now(),
                is_tty: false,
            };
        }

        // Print the initial line
        print!(
            "\r{}  {} {}",
            "[anna]".bright_cyan(),
            SPINNER_FRAMES[0].bright_yellow(),
            message.dimmed()
        );
        let _ = io::stdout().flush();

        let handle = std::thread::spawn(move || {
            let mut frame = 0;
            let frames = SPINNER_FRAMES;

            while running_clone.load(Ordering::Relaxed) {
                frame = (frame + 1) % frames.len();
                print!(
                    "\r{}  {} {}",
                    "[anna]".bright_cyan(),
                    frames[frame].bright_yellow(),
                    message.dimmed()
                );
                let _ = io::stdout().flush();
                // v0.27.0: Slower updates for SSH stability
                std::thread::sleep(Duration::from_millis(SPINNER_INTERVAL_MS));
            }
        });

        Self {
            running,
            handle: Some(handle),
            start_time: Instant::now(),
            is_tty,
        }
    }

    /// Stop spinner and return elapsed time
    pub fn stop(mut self) -> Duration {
        self.running.store(false, Ordering::Relaxed);
        if let Some(handle) = self.handle.take() {
            let _ = handle.join();
        }

        let elapsed = self.start_time.elapsed();

        // Only clear line if we have a TTY
        if self.is_tty {
            // Clear the spinner line
            print!("\r{}\r", " ".repeat(80));
            let _ = io::stdout().flush();
        }

        elapsed
    }

    /// Stop spinner and show completion with timing
    pub fn finish(self) -> Duration {
        let is_tty = self.is_tty;
        let elapsed = self.stop();

        // Print completion line
        if is_tty {
            println!(
                "{}  {} {}",
                "[anna]".bright_cyan(),
                "✓".bright_green(),
                format!("({:.1}s)", elapsed.as_secs_f64()).dimmed()
            );
        } else {
            println!("[anna]  done ({:.1}s)", elapsed.as_secs_f64());
        }
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
    println!("{}  {}", "[you]".bright_green(), question.white());
}
