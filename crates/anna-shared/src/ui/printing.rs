//! Terminal print functions for styled output (v0.0.344).
//!
//! v0.0.213: Initial implementation.
//! v0.0.337: Enhanced with consistent section/label formatting.
//! v0.0.344: Added print_title() for headers without version.

use super::colors;
use super::symbols;

/// Horizontal rule
pub const HR: &str =
    "──────────────────────────────────────────────────────────────────────────────";

/// Standard key width for alignment (22 chars)
pub const KEY_WIDTH: usize = 22;

/// Print a styled header with version
pub fn print_header(name: &str, version: &str) {
    println!();
    println!("{}{} v{}{}", colors::HEADER, name, version, colors::RESET);
    println!("{}{}{}", colors::DIM, HR, colors::RESET);
}

/// Print a title bar (HR + title + HR) - use for headers without version
pub fn print_title(title: &str) {
    println!("{}{}{}", colors::DIM, HR, colors::RESET);
    println!("{}{}{}", colors::HEADER, title, colors::RESET);
    println!("{}{}{}", colors::DIM, HR, colors::RESET);
}

/// Print a footer with horizontal rule and newline
pub fn print_footer() {
    println!("{}{}{}", colors::DIM, HR, colors::RESET);
    println!();
}

/// Print just a horizontal rule (no newline after)
pub fn print_hr() {
    println!("{}{}{}", colors::DIM, HR, colors::RESET);
}

/// Print a section header like [section] with HEADER color (orange)
/// This is the standard format for all section labels
pub fn print_section_header(section: &str) {
    println!("{}[{}]{}", colors::HEADER, section, colors::RESET);
}

/// Print a labeled section like [label] message with contextual color
pub fn print_label(label: &str, message: &str, color: &str) {
    println!("{}[{}]{} {}", color, label, colors::RESET, message);
}

/// Print a section header like [section] description (legacy - uses DIM)
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

/// Print a warning line
pub fn print_warn(message: &str) {
    println!(
        "  {}{}{} {}",
        colors::WARN,
        symbols::WARN,
        colors::RESET,
        message
    );
}

/// Print a dim/hint line
pub fn print_hint(message: &str) {
    println!("  {}{}{}", colors::DIM, message, colors::RESET);
}

/// Print a key-value pair with standard alignment
pub fn kv(key: &str, value: &str) {
    println!("  {:width$}{}", key, value, width = KEY_WIDTH);
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

/// Print a key-value pair with standard width and colored value
pub fn kv_colored(key: &str, value: &str, color: &str) {
    println!(
        "  {:width$}{}{}{}",
        key,
        color,
        value,
        colors::RESET,
        width = KEY_WIDTH
    );
}
