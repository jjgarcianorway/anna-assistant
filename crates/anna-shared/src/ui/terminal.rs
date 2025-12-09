//! Terminal control functions (v0.0.213).

use std::io::{self, Write};

/// Print without newline and flush
pub fn print_inline(message: &str) {
    print!("{}", message);
    io::stdout().flush().ok();
}

/// Clear current line
pub fn clear_line() {
    print!("\r\x1b[K");
    io::stdout().flush().ok();
}

/// Move cursor up n lines
pub fn cursor_up(n: usize) {
    print!("\x1b[{}A", n);
    io::stdout().flush().ok();
}
