//! Snapshot tests for UX rendering contract.
//! These tests verify that output format matches docs/UX_SPEC.md.

use std::io::Write;

use super::colors::*;

/// Test-only render context that captures output to a buffer
pub struct RenderBuffer {
    pub buffer: Vec<u8>,
}

impl RenderBuffer {
    pub fn new() -> Self {
        Self { buffer: Vec::new() }
    }

    pub fn print(&mut self, text: &str) {
        let _ = write!(self.buffer, "{}", text);
    }

    pub fn println(&mut self, text: &str) {
        let _ = writeln!(self.buffer, "{}", text);
    }

    pub fn print_colored(&mut self, text: &str, color: &str) {
        let _ = write!(self.buffer, "{}{}{}", color, text, RESET);
    }

    pub fn println_colored(&mut self, text: &str, color: &str) {
        let _ = writeln!(self.buffer, "{}{}{}", color, text, RESET);
    }

    pub fn output(&self) -> String {
        String::from_utf8_lossy(&self.buffer).to_string()
    }
}

/// Render confirmation request to buffer
pub fn render_confirmation(buf: &mut RenderBuffer, content: &str) {
    buf.println("");
    buf.print_colored("Anna: ", YELLOW);
    buf.println(content);
    buf.println("");
}

/// Render missing info to buffer
pub fn render_missing_info(buf: &mut RenderBuffer, content: &str) {
    buf.print_colored("Anna: ", YELLOW);
    buf.println(content);
}

/// Render timeout to buffer
pub fn render_timeout(buf: &mut RenderBuffer, timeout_secs: u64) {
    buf.println("");
    buf.print_colored("Anna: ", YELLOW);
    buf.println(&format!("Request took longer than {}s. Try again shortly.", timeout_secs));
    buf.println("");
}

/// Render system alert to buffer
pub fn render_system_alert(buf: &mut RenderBuffer, content: &str) {
    buf.println("");
    buf.print_colored("Anna: ", YELLOW);
    buf.println(content);
    buf.println("");
}

/// Render investigation start to buffer
pub fn render_investigation_start(buf: &mut RenderBuffer, topic: &str) {
    buf.println("");
    buf.print_colored("Anna: ", CYAN);
    buf.println(&format!("Investigating: {}", topic));
}

/// Render investigation complete to buffer
pub fn render_investigation_complete(buf: &mut RenderBuffer, conclusion: &str) {
    buf.println("");
    buf.print_colored("Anna: ", GREEN);
    buf.println(conclusion);
}

/// Render experiment start to buffer
pub fn render_experiment_start(buf: &mut RenderBuffer, description: &str) {
    buf.println("");
    buf.print_colored("Anna: ", MAGENTA);
    buf.println(&format!("Trying: {}", description));
}

/// Render final answer to buffer
pub fn render_final_answer(buf: &mut RenderBuffer, answer: &str) {
    buf.println("");
    buf.print_colored("Anna: ", GREEN);
    buf.println(answer);
    buf.println("");
}

/// Render LLM error to buffer
pub fn render_llm_error(buf: &mut RenderBuffer, message: &str) {
    buf.print_colored("Anna: ", RED);
    buf.println(message);
    buf.println("");
}

#[cfg(test)]
mod tests {
    use super::*;

    fn load_snapshot(name: &str) -> String {
        let path = format!("../../tests/snapshots/{}.txt", name);
        std::fs::read_to_string(&path)
            .unwrap_or_else(|_| panic!("Snapshot file not found: {}", path))
    }

    #[test]
    fn test_confirmation_request_snapshot() {
        let mut buf = RenderBuffer::new();
        let content = "I will configure GDM resolution to 1920x1080.\n\nSteps:\n  1. Create /etc/gdm/custom.conf\n  2. Set WaylandEnable=false\n  3. Restart GDM service";
        render_confirmation(&mut buf, content);
        assert_eq!(buf.output(), load_snapshot("confirmation_request"));
    }

    #[test]
    fn test_missing_info_snapshot() {
        let mut buf = RenderBuffer::new();
        render_missing_info(&mut buf, "I need to know which resolution you want to set.");
        assert_eq!(buf.output(), load_snapshot("missing_info"));
    }

    #[test]
    fn test_timeout_snapshot() {
        let mut buf = RenderBuffer::new();
        render_timeout(&mut buf, 60);
        assert_eq!(buf.output(), load_snapshot("timeout"));
    }

    #[test]
    fn test_system_alert_snapshot() {
        let mut buf = RenderBuffer::new();
        render_system_alert(&mut buf, "Disk usage is at 95%. Consider cleaning up.");
        assert_eq!(buf.output(), load_snapshot("system_alert"));
    }

    #[test]
    fn test_investigation_start_snapshot() {
        let mut buf = RenderBuffer::new();
        render_investigation_start(&mut buf, "high CPU usage");
        assert_eq!(buf.output(), load_snapshot("investigation_start"));
    }

    #[test]
    fn test_investigation_complete_snapshot() {
        let mut buf = RenderBuffer::new();
        render_investigation_complete(&mut buf, "Found the cause: Chrome with 47 tabs.");
        assert_eq!(buf.output(), load_snapshot("investigation_complete"));
    }

    #[test]
    fn test_experiment_start_snapshot() {
        let mut buf = RenderBuffer::new();
        render_experiment_start(&mut buf, "restart nginx service");
        assert_eq!(buf.output(), load_snapshot("experiment_start"));
    }

    #[test]
    fn test_final_answer_snapshot() {
        let mut buf = RenderBuffer::new();
        render_final_answer(&mut buf, "Your disk has 234GB free out of 500GB (47% used).");
        assert_eq!(buf.output(), load_snapshot("final_answer"));
    }

    #[test]
    fn test_llm_error_snapshot() {
        let mut buf = RenderBuffer::new();
        render_llm_error(&mut buf, "Model not available. Try again in a moment.");
        assert_eq!(buf.output(), load_snapshot("llm_error"));
    }

    // Contract verification tests
    #[test]
    fn test_no_please_confirm_wrapper() {
        let mut buf = RenderBuffer::new();
        render_confirmation(&mut buf, "Test content");
        let output = buf.output();
        assert!(!output.contains("Please confirm"));
    }

    #[test]
    fn test_no_missing_information_wrapper() {
        let mut buf = RenderBuffer::new();
        render_missing_info(&mut buf, "Need more info");
        let output = buf.output();
        assert!(!output.contains("Missing information"));
    }

    #[test]
    fn test_no_system_alert_header() {
        let mut buf = RenderBuffer::new();
        render_system_alert(&mut buf, "Alert content");
        let output = buf.output();
        assert!(!output.contains("SYSTEM ALERT"));
    }

    #[test]
    fn test_timeout_concise() {
        let mut buf = RenderBuffer::new();
        render_timeout(&mut buf, 30);
        let output = buf.output();
        assert!(!output.contains("Possible causes"));
        assert!(!output.contains("Try:"));
        assert!(output.lines().count() <= 4);
    }

    #[test]
    fn test_anna_prefix_not_answer() {
        let mut buf = RenderBuffer::new();
        render_final_answer(&mut buf, "Test");
        let output = buf.output();
        assert!(output.contains("Anna:"));
        assert!(!output.contains("ANSWER:"));
    }
}
