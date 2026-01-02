//! Progress indicator component.

/// Progress indicator.
#[derive(Debug, Clone)]
pub struct ProgressIndicator {
    /// Current step.
    pub current: usize,
    /// Total steps.
    pub total: usize,
    /// Current step description.
    pub description: String,
}

impl ProgressIndicator {
    /// Create new indicator.
    pub fn new(total: usize) -> Self {
        Self {
            current: 0,
            total,
            description: String::new(),
        }
    }

    /// Advance to next step.
    pub fn advance(&mut self, description: &str) {
        self.current = (self.current + 1).min(self.total);
        self.description = description.to_string();
    }

    /// Format for display.
    pub fn display(&self) -> String {
        let pct = if self.total > 0 {
            (self.current * 100) / self.total
        } else {
            100
        };
        format!(
            "[{}/{}] {} ({}%)",
            self.current, self.total, self.description, pct
        )
    }

    /// Format for plain display.
    pub fn display_plain(&self) -> String {
        format!("{}/{}: {}", self.current, self.total, self.description)
    }
}
