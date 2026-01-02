//! User-facing error types (v0.0.407).

/// User-facing error response
#[derive(Debug, Clone)]
pub struct ErrorResponse {
    /// Main message (1-2 sentences)
    pub message: String,
    /// Optional hint for next steps
    pub hint: Option<String>,
    /// Whether debug mode would help
    pub debug_helpful: bool,
}

impl ErrorResponse {
    /// Format for display
    pub fn format(&self, show_debug_hint: bool) -> String {
        let mut output = self.message.clone();

        if let Some(ref hint) = self.hint {
            output.push_str("\n\n");
            output.push_str(hint);
        }

        if show_debug_hint && self.debug_helpful {
            output.push_str("\n\n(Enable debug mode for details.)");
        }

        output
    }
}
