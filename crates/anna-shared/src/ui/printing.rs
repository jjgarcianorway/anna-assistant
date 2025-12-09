//! Terminal print functions for styled output (v0.0.213).

use super::colors;
use super::symbols;

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
