//! Dialogue Consumer - Terminal consumer for streaming dialogue.
//!
//! Implements DialogueConsumer for real-time terminal output.

use anna_shared::timeline::{DialogueConsumer, DialogueLine};
use crate::spinner::Spinner;
use super::renderer::render_line;

/// Terminal dialogue consumer with spinner support.
pub struct TerminalConsumer {
    /// Active spinner.
    spinner: Option<Spinner>,
    /// Lines rendered so far.
    line_count: usize,
    /// Debug mode.
    debug: bool,
}

impl TerminalConsumer {
    /// Create a new terminal consumer.
    pub fn new(debug: bool) -> Self {
        Self {
            spinner: None,
            line_count: 0,
            debug,
        }
    }

    /// Stop any active spinner.
    fn stop_spinner(&mut self) {
        if let Some(spinner) = self.spinner.take() {
            drop(spinner);
        }
    }

    /// Get the number of lines rendered.
    pub fn lines_rendered(&self) -> usize {
        self.line_count
    }
}

impl DialogueConsumer for TerminalConsumer {
    fn on_line(&mut self, line: &DialogueLine) {
        self.stop_spinner();
        render_line(line);
        self.line_count += 1;
    }

    fn on_spinner_start(&mut self, message: &str) {
        self.stop_spinner();
        self.spinner = Some(Spinner::new(message));
    }

    fn on_spinner_stop(&mut self) {
        self.stop_spinner();
    }

    fn on_partial(&mut self, key: &str, value: &str) {
        if self.debug {
            self.stop_spinner();
            crate::display::print_colored(&format!("[{}] ", key), crate::display::DIM);
            println!("{}", value);
        }
    }

    fn on_complete(&mut self) {
        self.stop_spinner();
    }
}

impl Drop for TerminalConsumer {
    fn drop(&mut self) {
        self.stop_spinner();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_consumer_creation() {
        let consumer = TerminalConsumer::new(false);
        assert_eq!(consumer.lines_rendered(), 0);
    }

    #[test]
    fn test_consumer_line_counting() {
        let mut consumer = TerminalConsumer::new(false);
        // Note: actual rendering would need stdout capture
        consumer.line_count = 5;
        assert_eq!(consumer.lines_rendered(), 5);
    }
}
