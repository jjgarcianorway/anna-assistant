//! Color constants and terminal utilities for Anna TUI.
//! Hollywood-style truecolor output with no icons.

use std::io::{self, Write};

// ANSI color codes for truecolor terminals
pub const GREEN: &str = "\x1b[32m";
pub const YELLOW: &str = "\x1b[33m";
pub const RED: &str = "\x1b[31m";
pub const BLUE: &str = "\x1b[34m";
pub const CYAN: &str = "\x1b[36m";
pub const MAGENTA: &str = "\x1b[35m";
pub const WHITE: &str = "\x1b[37;1m";
pub const DIM: &str = "\x1b[2m";
pub const BOLD: &str = "\x1b[1m";
pub const RESET: &str = "\x1b[0m";

// RGB colors for specific UI elements
pub type Rgb = (u8, u8, u8);
pub const ANNA_BLUE: Rgb = (100, 149, 237);
pub const SUCCESS_GREEN: Rgb = (50, 205, 50);
pub const WARNING_AMBER: Rgb = (255, 191, 0);
pub const ERROR_RED: Rgb = (220, 20, 60);

/// Print text with color, no newline
pub fn print_colored(text: &str, color: &str) {
    print!("{}{}{}", color, text, RESET);
}

/// Print text with color, with newline
pub fn println_colored(text: &str, color: &str) {
    println!("{}{}{}", color, text, RESET);
}

/// Print RGB colored text (truecolor)
pub fn print_rgb(text: &str, rgb: Rgb) {
    print!("\x1b[38;2;{};{};{}m{}{}", rgb.0, rgb.1, rgb.2, text, RESET);
}

/// Flush stdout immediately (for streaming output)
pub fn flush_stdout() {
    let _ = io::stdout().flush();
}
