//! UI utilities for clean terminal experience.
//!
//! Provides:
//! - Animated spinners for long operations
//! - Color support (ANSI 256 and true color)
//! - Clean section headers (no box drawing)
//! - Status indicators
//!
//! v0.1.0: Initial implementation
//! v0.1.1: Removed box drawing for cleaner look

use std::io::{self, Write};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::Duration;

// =============================================================================
// TRUE COLOR SUPPORT
// =============================================================================

/// RGB color
pub struct Rgb(pub u8, pub u8, pub u8);

/// Print text with true color (24-bit)
pub fn print_rgb(text: &str, fg: &Rgb) {
    print!("\x1b[38;2;{};{};{}m{}\x1b[0m", fg.0, fg.1, fg.2, text);
}

/// Print text with true color and newline
pub fn println_rgb(text: &str, fg: &Rgb) {
    println!("\x1b[38;2;{};{};{}m{}\x1b[0m", fg.0, fg.1, fg.2, text);
}

/// Anna brand colors
pub mod colors {
    use super::Rgb;

    pub const CYAN: Rgb = Rgb(0, 200, 200);
    pub const CYAN_DARK: Rgb = Rgb(0, 150, 150);
    pub const GREEN: Rgb = Rgb(100, 220, 100);
    pub const YELLOW: Rgb = Rgb(220, 200, 100);
    pub const RED: Rgb = Rgb(220, 80, 80);
    pub const MAGENTA: Rgb = Rgb(180, 100, 200);
    pub const WHITE: Rgb = Rgb(240, 240, 240);
    pub const GRAY: Rgb = Rgb(128, 128, 128);
    pub const DIM: Rgb = Rgb(80, 80, 80);
}

// =============================================================================
// SPINNER ANIMATION
// =============================================================================

/// Spinner frames (Braille animation - smooth)
const SPINNER_FRAMES: &[&str] = &["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧", "⠇", "⠏"];

/// Spinner handle for async operations
pub struct Spinner {
    running: Arc<AtomicBool>,
    handle: Option<std::thread::JoinHandle<()>>,
}

impl Spinner {
    /// Start a new spinner with a message
    pub fn start(message: &str) -> Self {
        let running = Arc::new(AtomicBool::new(true));
        let running_clone = running.clone();
        let msg = message.to_string();

        let handle = std::thread::spawn(move || {
            let mut frame_idx = 0;
            let frames = SPINNER_FRAMES;

            while running_clone.load(Ordering::SeqCst) {
                print!("\r\x1b[K");
                print!("\x1b[36m{}\x1b[0m ", frames[frame_idx]);
                print!("\x1b[2m{}\x1b[0m", msg);
                io::stdout().flush().ok();

                frame_idx = (frame_idx + 1) % frames.len();
                std::thread::sleep(Duration::from_millis(80));
            }

            print!("\r\x1b[K");
            io::stdout().flush().ok();
        });

        Self {
            running,
            handle: Some(handle),
        }
    }

    fn stop_internal(&mut self) {
        self.running.store(false, Ordering::SeqCst);
        if let Some(handle) = self.handle.take() {
            handle.join().ok();
        }
    }

    pub fn stop(mut self) {
        self.stop_internal();
    }

    pub fn success(mut self, message: &str) {
        self.stop_internal();
        println!("\x1b[32m✓\x1b[0m {}", message);
    }

    pub fn fail(mut self, message: &str) {
        self.stop_internal();
        println!("\x1b[31m✗\x1b[0m {}", message);
    }

    pub fn info(mut self, message: &str) {
        self.stop_internal();
        println!("\x1b[36m→\x1b[0m {}", message);
    }
}

impl Drop for Spinner {
    fn drop(&mut self) {
        self.stop_internal();
    }
}

// =============================================================================
// PROGRESS BAR
// =============================================================================

/// Simple progress bar
pub fn print_progress_bar(progress: f32, width: usize, label: &str) {
    let filled = (progress * width as f32) as usize;
    let empty = width.saturating_sub(filled);

    print!("\r\x1b[K");
    print!("{} ", label);
    print!("\x1b[32m{}\x1b[0m", "=".repeat(filled));
    print!("\x1b[2m{}\x1b[0m", "-".repeat(empty));
    print!(" {:.0}%", progress * 100.0);
    io::stdout().flush().ok();
}

/// Complete the progress bar with newline
pub fn complete_progress_bar() {
    println!();
}

// =============================================================================
// SECTION HEADERS (clean, no boxes)
// =============================================================================

/// Print a section with title and content (clean style, no boxes)
pub fn draw_box(title: &str, content: &[&str], _width: usize) {
    // Just print section header and indented content
    println!();
    println!("\x1b[1;36m{}\x1b[0m", title.to_uppercase());
    for line in content {
        println!("  {}", line);
    }
}

/// Draw a subtle divider (just blank line)
pub fn draw_divider(_width: usize) {
    println!();
}

/// Draw a double divider (just blank line)
pub fn draw_double_divider(_width: usize) {
    println!();
}

// =============================================================================
// STATUS INDICATORS
// =============================================================================

pub fn print_success(message: &str) {
    println!("\x1b[32m✓\x1b[0m {}", message);
}

pub fn print_error(message: &str) {
    println!("\x1b[31m✗\x1b[0m {}", message);
}

pub fn print_warning(message: &str) {
    println!("\x1b[33m⚠\x1b[0m {}", message);
}

pub fn print_info(message: &str) {
    println!("\x1b[36m→\x1b[0m {}", message);
}

pub fn print_bullet(message: &str) {
    println!("  {}", message);
}

// =============================================================================
// TYPING EFFECT
// =============================================================================

pub fn print_typing(text: &str, delay_ms: u64) {
    for c in text.chars() {
        print!("{}", c);
        io::stdout().flush().ok();
        std::thread::sleep(Duration::from_millis(delay_ms));
    }
}

pub fn print_typing_colored(text: &str, color: &str, delay_ms: u64) {
    print!("{}", color);
    for c in text.chars() {
        print!("{}", c);
        io::stdout().flush().ok();
        std::thread::sleep(Duration::from_millis(delay_ms));
    }
    print!("\x1b[0m");
}

// =============================================================================
// HEADER / BANNER (clean style)
// =============================================================================

/// Print Anna header banner (clean, no boxes)
pub fn print_banner() {
    println!();
    println!("\x1b[1;36mANNA\x1b[0m - IT Department Assistant");
    println!();
}

/// Print a section header
pub fn print_section(title: &str) {
    println!();
    println!("\x1b[1;36m{}\x1b[0m", title.to_uppercase());
}

// =============================================================================
// TEAM DIALOGUE FORMATTING (clean style)
// =============================================================================

/// Format team member dialogue
pub fn print_team_dialogue(name: &str, role: &str, message: &str) {
    print!("  \x1b[35m{}\x1b[0m", name);
    print!("\x1b[2m ({})\x1b[0m", role);
    println!(": {}", message);
}

/// Format Anna speaking
pub fn print_anna_speaks(message: &str) {
    println!("\x1b[35mAnna\x1b[0m: {}", message);
}

/// Format ticket creation (clean style)
pub fn print_ticket(ticket_id: &str, department: &str) {
    println!();
    print!("\x1b[36mTicket\x1b[0m ");
    print!("\x1b[1;37m{}\x1b[0m", ticket_id);
    print!(" → \x1b[36m{}\x1b[0m", department);
    println!(" department");
}

/// Format escalation
pub fn print_escalation(from: &str, to: &str, reason: &str) {
    println!();
    print!("\x1b[33m↑\x1b[0m Escalating: {} → {}", from, to);
    if !reason.is_empty() {
        print!(" ({})", reason);
    }
    println!();
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_spinner_create() {
        let spinner = Spinner::start("Testing...");
        std::thread::sleep(Duration::from_millis(200));
        spinner.stop();
    }

    #[test]
    fn test_progress_bar() {
        for i in 0..=10 {
            print_progress_bar(i as f32 / 10.0, 30, "Progress");
            std::thread::sleep(Duration::from_millis(50));
        }
        complete_progress_bar();
    }
}
