//! Rendering logic for REPL greetings.

use crate::ui::colors;

use super::types::ReplGreeting;

impl ReplGreeting {
    /// Render the greeting for display
    pub fn render(&self) -> String {
        let mut output = String::new();

        // Header
        output.push_str(&format!(
            "\n{}Hello {},{}\n\n",
            colors::CYAN,
            self.user_name,
            colors::RESET
        ));

        if self.first_time {
            output.push_str("First time here? I'm Anna, your local IT department.\n");
            output.push_str(
                "Ask me anything about your system - disk, memory, services, config.\n\n",
            );
        } else {
            // Last session stats
            output.push_str(&format!(
                "Last session: {} tickets handled",
                self.tickets_handled
            ));
            if !self.top_topics.is_empty() {
                output.push_str(&format!(", top topics: {}", self.top_topics.join(", ")));
            }
            output.push_str("\n");

            // System status
            output.push_str(&format!(
                "System status: {}{}{} - {}\n",
                self.system_status.color(),
                self.system_status,
                colors::RESET,
                self.status_summary
            ));
        }

        // Active staff
        output.push_str(&format!(
            "Active staff: {} across {} departments\n",
            self.active_staff,
            self.departments.len()
        ));

        // Error announcements (v0.0.463)
        if !self.errors.is_empty() {
            output.push_str(&format!("\n{}Issues detected:{}\n", colors::WARN, colors::RESET));
            for error in &self.errors {
                output.push_str(&format!(
                    "  {}[{}]{} {}\n",
                    colors::ERR,
                    error.category,
                    colors::RESET,
                    error.message
                ));
                if let Some(ref hint) = error.fix_hint {
                    output.push_str(&format!(
                        "    {}Fix: {}{}\n",
                        colors::DIM,
                        hint,
                        colors::RESET
                    ));
                }
            }
        }

        // Prompt
        output.push_str(&format!(
            "\n{}What can I help with?{}\n",
            colors::DIM,
            colors::RESET
        ));

        output
    }

    /// Render compact greeting (one line)
    pub fn render_compact(&self) -> String {
        if self.first_time {
            format!(
                "{}Anna{} - Your local IT department. Ask anything!",
                colors::CYAN,
                colors::RESET
            )
        } else {
            format!(
                "{}Anna{} - {} tickets, status: {}{}{}",
                colors::CYAN,
                colors::RESET,
                self.tickets_handled,
                self.system_status.color(),
                self.system_status,
                colors::RESET
            )
        }
    }
}
