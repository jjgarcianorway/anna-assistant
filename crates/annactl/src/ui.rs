//! UI utilities for Hollywood-style terminal experience.
//!
//! Provides:
//! - Animated spinners for long operations
//! - Gradient colors and true color support
//! - Box-drawing and panel utilities
//! - Real-time progress indicators
//!
//! v0.1.0: Initial implementation

use std::io::{self, Write};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::Duration;

// ═══════════════════════════════════════════════════════════════════════════
// TRUE COLOR SUPPORT
// ═══════════════════════════════════════════════════════════════════════════

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

// ═══════════════════════════════════════════════════════════════════════════
// SPINNER ANIMATION
// ═══════════════════════════════════════════════════════════════════════════

/// Spinner frames (Braille animation - smooth)
const SPINNER_FRAMES: &[&str] = &["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧", "⠇", "⠏"];

/// Alternative spinner (dots)
const SPINNER_DOTS: &[&str] = &["⣾", "⣽", "⣻", "⢿", "⡿", "⣟", "⣯", "⣷"];

/// Minimalist spinner
const SPINNER_SIMPLE: &[&str] = &["|", "/", "-", "\\"];

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
                // Clear line and print spinner
                print!("\r\x1b[K");  // Clear line
                print!("\x1b[36m{}\x1b[0m ", frames[frame_idx]);  // Cyan spinner
                print!("\x1b[2m{}\x1b[0m", msg);  // Dim message
                io::stdout().flush().ok();

                frame_idx = (frame_idx + 1) % frames.len();
                std::thread::sleep(Duration::from_millis(80));
            }

            // Clear the spinner line when done
            print!("\r\x1b[K");
            io::stdout().flush().ok();
        });

        Self {
            running,
            handle: Some(handle),
        }
    }

    /// Stop the spinner (internal helper)
    fn stop_internal(&mut self) {
        self.running.store(false, Ordering::SeqCst);
        if let Some(handle) = self.handle.take() {
            handle.join().ok();
        }
    }

    /// Stop the spinner
    pub fn stop(mut self) {
        self.stop_internal();
    }

    /// Stop with a success message
    pub fn success(mut self, message: &str) {
        self.stop_internal();
        println!("\x1b[32m✓\x1b[0m {}", message);
    }

    /// Stop with a failure message
    pub fn fail(mut self, message: &str) {
        self.stop_internal();
        println!("\x1b[31m✗\x1b[0m {}", message);
    }

    /// Stop with an info message
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

// ═══════════════════════════════════════════════════════════════════════════
// PROGRESS BAR
// ═══════════════════════════════════════════════════════════════════════════

/// Simple progress bar
pub fn print_progress_bar(progress: f32, width: usize, label: &str) {
    let filled = (progress * width as f32) as usize;
    let empty = width.saturating_sub(filled);

    print!("\r\x1b[K");  // Clear line
    print!("{} ", label);
    print!("\x1b[2m[\x1b[0m");
    print!("\x1b[32m{}\x1b[0m", "█".repeat(filled));
    print!("\x1b[2m{}\x1b[0m", "░".repeat(empty));
    print!("\x1b[2m]\x1b[0m");
    print!(" {:.0}%", progress * 100.0);
    io::stdout().flush().ok();
}

/// Complete the progress bar with newline
pub fn complete_progress_bar() {
    println!();
}

// ═══════════════════════════════════════════════════════════════════════════
// BOX DRAWING
// ═══════════════════════════════════════════════════════════════════════════

/// Draw a box around content
pub fn draw_box(title: &str, content: &[&str], width: usize) {
    let inner_width = width - 4;

    // Top border with title
    print!("\x1b[2m┌─ \x1b[0m");
    print!("\x1b[36m{}\x1b[0m", title);
    let title_len = title.chars().count();
    if title_len + 4 < width {
        print!("\x1b[2m{}\x1b[0m", "─".repeat(width - title_len - 4));
    }
    println!("\x1b[2m┐\x1b[0m");

    // Content
    for line in content {
        print!("\x1b[2m│\x1b[0m ");
        let line_len = line.chars().count();
        print!("{}", line);
        if line_len < inner_width {
            print!("{}", " ".repeat(inner_width - line_len));
        }
        println!(" \x1b[2m│\x1b[0m");
    }

    // Bottom border
    println!("\x1b[2m└{}┘\x1b[0m", "─".repeat(width - 2));
}

/// Draw a simple horizontal divider
pub fn draw_divider(width: usize) {
    println!("\x1b[2m{}\x1b[0m", "─".repeat(width));
}

/// Draw a double horizontal divider
pub fn draw_double_divider(width: usize) {
    println!("\x1b[2m{}\x1b[0m", "═".repeat(width));
}

// ═══════════════════════════════════════════════════════════════════════════
// STATUS INDICATORS
// ═══════════════════════════════════════════════════════════════════════════

/// Print a success indicator
pub fn print_success(message: &str) {
    println!("\x1b[32m✓\x1b[0m {}", message);
}

/// Print an error indicator
pub fn print_error(message: &str) {
    println!("\x1b[31m✗\x1b[0m {}", message);
}

/// Print a warning indicator
pub fn print_warning(message: &str) {
    println!("\x1b[33m⚠\x1b[0m {}", message);
}

/// Print an info indicator
pub fn print_info(message: &str) {
    println!("\x1b[36m→\x1b[0m {}", message);
}

/// Print a bullet point
pub fn print_bullet(message: &str) {
    println!("\x1b[2m•\x1b[0m {}", message);
}

// ═══════════════════════════════════════════════════════════════════════════
// TYPING EFFECT (for dramatic answers)
// ═══════════════════════════════════════════════════════════════════════════

/// Print text with typing effect
pub fn print_typing(text: &str, delay_ms: u64) {
    for c in text.chars() {
        print!("{}", c);
        io::stdout().flush().ok();
        std::thread::sleep(Duration::from_millis(delay_ms));
    }
}

/// Print text with typing effect and color
pub fn print_typing_colored(text: &str, color: &str, delay_ms: u64) {
    print!("{}", color);
    for c in text.chars() {
        print!("{}", c);
        io::stdout().flush().ok();
        std::thread::sleep(Duration::from_millis(delay_ms));
    }
    print!("\x1b[0m");
}

// ═══════════════════════════════════════════════════════════════════════════
// HEADER / BANNER
// ═══════════════════════════════════════════════════════════════════════════

/// Print Anna header banner
pub fn print_banner() {
    println!();
    println!("\x1b[36m╔═══════════════════════════════════════════════════════════╗\x1b[0m");
    println!("\x1b[36m║\x1b[0m              \x1b[1;36mANNA\x1b[0m - IT Department Assistant              \x1b[36m║\x1b[0m");
    println!("\x1b[36m╚═══════════════════════════════════════════════════════════╝\x1b[0m");
    println!();
}

/// Print a section header
pub fn print_section(title: &str) {
    println!();
    print!("\x1b[2m─── \x1b[0m");
    print!("\x1b[1;36m{}\x1b[0m", title);
    println!("\x1b[2m ───────────────────────────────────────────\x1b[0m");
    println!();
}

// ═══════════════════════════════════════════════════════════════════════════
// TEAM DIALOGUE FORMATTING
// ═══════════════════════════════════════════════════════════════════════════

/// Format team member dialogue
pub fn print_team_dialogue(name: &str, role: &str, message: &str) {
    print!("\x1b[2m  │ \x1b[0m");
    print!("\x1b[35m{}\x1b[0m", name);
    print!("\x1b[2m ({})\x1b[0m", role);
    print!(": {}", message);
    println!();
}

/// Format Anna speaking
pub fn print_anna_speaks(message: &str) {
    print!("\x1b[35mAnna\x1b[0m → ");
    println!("{}", message);
}

/// Format ticket creation
pub fn print_ticket(ticket_id: &str, department: &str) {
    println!();
    print!("\x1b[2m┌─ \x1b[0m");
    print!("\x1b[36mTICKET\x1b[0m ");
    print!("\x1b[1;37m{}\x1b[0m", ticket_id);
    print!(" → ");
    print!("\x1b[36m{}\x1b[0m", department);
    println!(" department");
    println!("\x1b[2m└─\x1b[0m");
}

/// Format escalation
pub fn print_escalation(from: &str, to: &str, reason: &str) {
    println!();
    print!("\x1b[33m  ⇈ \x1b[0m");
    print!("Escalating from {} to {}", from, to);
    if !reason.is_empty() {
        print!(": {}", reason);
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
