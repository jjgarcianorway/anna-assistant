//! Dialog components - questions, choices, confirmations.

/// A clean dialog question.
#[derive(Debug, Clone)]
pub struct DialogQuestion {
    /// Question text.
    pub question: String,
    /// Choices.
    pub choices: Vec<DialogChoice>,
    /// Allow cancel?
    pub allow_cancel: bool,
    /// Allow other input?
    pub allow_other: bool,
}

/// A dialog choice.
#[derive(Debug, Clone)]
pub struct DialogChoice {
    /// Choice key (for selection).
    pub key: String,
    /// Display label.
    pub label: String,
    /// Value to return if selected.
    pub value: String,
}

impl DialogChoice {
    /// Create new choice.
    pub fn new(key: &str, label: &str, value: &str) -> Self {
        Self {
            key: key.to_string(),
            label: label.to_string(),
            value: value.to_string(),
        }
    }

    /// Create numbered choice.
    pub fn numbered(num: usize, label: &str, value: &str) -> Self {
        Self::new(&num.to_string(), label, value)
    }
}

impl DialogQuestion {
    /// Create new question.
    pub fn new(question: &str) -> Self {
        Self {
            question: question.to_string(),
            choices: Vec::new(),
            allow_cancel: true,
            allow_other: false,
        }
    }

    /// Add choice.
    pub fn choice(mut self, label: &str, value: &str) -> Self {
        let num = self.choices.len() + 1;
        self.choices.push(DialogChoice::numbered(num, label, value));
        self
    }

    /// Allow other input.
    pub fn with_other(mut self) -> Self {
        self.allow_other = true;
        self
    }

    /// Disallow cancel.
    pub fn no_cancel(mut self) -> Self {
        self.allow_cancel = false;
        self
    }

    /// Format for display (normal).
    pub fn display(&self) -> String {
        let mut output = format!("{}\n", self.question);

        for choice in &self.choices {
            output.push_str(&format!("  {}) {}\n", choice.key, choice.label));
        }

        if self.allow_other {
            output.push_str("  9) Something else (type it)\n");
        }

        if self.allow_cancel {
            output.push_str("  0) Cancel\n");
        }

        output
    }

    /// Format for plain display.
    pub fn display_plain(&self) -> String {
        let choices: Vec<_> = self.choices.iter().map(|c| c.label.as_str()).collect();
        format!("{} [{}]", self.question, choices.join("/"))
    }

    /// Format for JSON.
    pub fn to_json(&self) -> Result<String, String> {
        let obj = serde_json::json!({
            "question": self.question,
            "choices": self.choices.iter().map(|c| {
                serde_json::json!({
                    "key": c.key,
                    "label": c.label,
                    "value": c.value
                })
            }).collect::<Vec<_>>(),
            "allow_cancel": self.allow_cancel,
            "allow_other": self.allow_other
        });
        serde_json::to_string(&obj).map_err(|e| e.to_string())
    }

    /// Parse user input.
    pub fn parse_input(&self, input: &str) -> DialogResult {
        let input = input.trim();

        // Cancel
        if input == "0" && self.allow_cancel {
            return DialogResult::Cancelled;
        }

        // Other
        if input == "9" && self.allow_other {
            return DialogResult::Other;
        }

        // Match choice
        for choice in &self.choices {
            if input == choice.key {
                return DialogResult::Selected(choice.value.clone());
            }
        }

        // Invalid
        DialogResult::Invalid(input.to_string())
    }
}

/// Dialog result.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DialogResult {
    /// User selected a choice.
    Selected(String),
    /// User cancelled.
    Cancelled,
    /// User chose "other".
    Other,
    /// Invalid input.
    Invalid(String),
}

/// Confirmation dialog.
pub struct ConfirmDialog {
    /// Question text.
    pub question: String,
    /// Default answer.
    pub default: bool,
}

impl ConfirmDialog {
    /// Create new confirmation.
    pub fn new(question: &str) -> Self {
        Self {
            question: question.to_string(),
            default: false,
        }
    }

    /// Set default to yes.
    pub fn default_yes(mut self) -> Self {
        self.default = true;
        self
    }

    /// Format for display.
    pub fn display(&self) -> String {
        let hint = if self.default { "[Y/n]" } else { "[y/N]" };
        format!("{} {}", self.question, hint)
    }

    /// Parse user input.
    pub fn parse_input(&self, input: &str) -> bool {
        let input = input.trim().to_lowercase();
        if input.is_empty() {
            return self.default;
        }
        input == "y" || input == "yes"
    }
}
