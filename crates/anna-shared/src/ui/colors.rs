//! ANSI color codes using true color (24-bit) (v0.0.213).

pub const HEADER: &str = "\x1b[38;2;255;210;120m";
pub const OK: &str = "\x1b[38;2;120;255;120m";
pub const ERR: &str = "\x1b[38;2;255;100;100m";
pub const WARN: &str = "\x1b[38;2;255;200;100m";
pub const DIM: &str = "\x1b[38;2;140;140;140m";
pub const CYAN: &str = "\x1b[38;2;100;200;255m";
pub const BOLD: &str = "\x1b[1m";
pub const RESET: &str = "\x1b[0m";
