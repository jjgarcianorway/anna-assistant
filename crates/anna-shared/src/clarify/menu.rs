//! Menu-based clarification types (v0.0.191).
//!
//! v0.0.42: Menu-based prompts with numeric keys.

use serde::{Deserialize, Serialize};

use crate::clarify_v2::{KEY_CANCEL, KEY_OTHER};

/// Menu-based clarification prompt (v0.0.42)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClarifyPrompt {
    pub id: String,
    pub title: String,
    pub question: String,
    pub options: Vec<MenuOption>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub default_key: Option<u8>,
    #[serde(default)]
    pub reason: String,
}

/// A menu option with numeric key (v0.0.42)
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MenuOption {
    pub key: u8,
    pub label: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub fact_key: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub fact_value: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub verify_cmd: Option<String>,
}

impl MenuOption {
    pub fn new(key: u8, label: impl Into<String>) -> Self {
        Self {
            key,
            label: label.into(),
            fact_key: None,
            fact_value: None,
            verify_cmd: None,
        }
    }

    pub fn with_fact(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.fact_key = Some(key.into());
        self.fact_value = Some(value.into());
        self
    }

    pub fn with_verify(mut self, cmd: impl Into<String>) -> Self {
        self.verify_cmd = Some(cmd.into());
        self
    }

    pub fn cancel() -> Self {
        Self::new(KEY_CANCEL, "Cancel")
    }
    pub fn other() -> Self {
        Self::new(KEY_OTHER, "Other (specify)")
    }
}

impl ClarifyPrompt {
    pub fn new(
        id: impl Into<String>,
        title: impl Into<String>,
        question: impl Into<String>,
    ) -> Self {
        Self {
            id: id.into(),
            title: title.into(),
            question: question.into(),
            options: vec![MenuOption::cancel(), MenuOption::other()],
            default_key: None,
            reason: String::new(),
        }
    }

    pub fn add_option(mut self, opt: MenuOption) -> Self {
        self.options
            .retain(|o| o.key != KEY_CANCEL && o.key != KEY_OTHER);
        self.options.push(opt);
        self.options.sort_by_key(|o| o.key);
        self.options.push(MenuOption::cancel());
        self.options.push(MenuOption::other());
        self
    }

    pub fn with_options(mut self, opts: Vec<MenuOption>) -> Self {
        self.options = opts;
        self.ensure_escape_options();
        self
    }

    pub fn with_default(mut self, key: u8) -> Self {
        self.default_key = Some(key);
        self
    }
    pub fn with_reason(mut self, reason: impl Into<String>) -> Self {
        self.reason = reason.into();
        self
    }

    fn ensure_escape_options(&mut self) {
        if !self.options.iter().any(|o| o.key == KEY_CANCEL) {
            self.options.push(MenuOption::cancel());
        }
        if !self.options.iter().any(|o| o.key == KEY_OTHER) {
            self.options.push(MenuOption::other());
        }
        self.options.sort_by_key(|o| match o.key {
            KEY_CANCEL => 100,
            KEY_OTHER => 101,
            k => k,
        });
    }

    pub fn get_option(&self, key: u8) -> Option<&MenuOption> {
        self.options.iter().find(|o| o.key == key)
    }

    pub fn format_menu(&self) -> String {
        let mut lines = vec![
            format!("╭─ {} ─╮", self.title),
            self.question.clone(),
            String::new(),
        ];
        for opt in &self.options {
            let marker = if self.default_key == Some(opt.key) {
                " ←"
            } else {
                ""
            };
            lines.push(format!("  [{}] {}{}", opt.key, opt.label, marker));
        }
        if !self.reason.is_empty() {
            lines.push(String::new());
            lines.push(format!("  ({})", self.reason));
        }
        lines.join("\n")
    }
}

/// Outcome of menu interaction (v0.0.42)
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ClarifyOutcome {
    Answered {
        key: u8,
        label: String,
        prompt_id: String,
    },
    Cancelled,
    Other {
        text: String,
    },
    VerificationFailed {
        selected: String,
        reason: String,
        alternative: Option<String>,
    },
}

impl ClarifyOutcome {
    pub fn is_success(&self) -> bool {
        matches!(self, Self::Answered { .. } | Self::Other { .. })
    }
    pub fn selected_text(&self) -> Option<&str> {
        match self {
            Self::Answered { label, .. } => Some(label),
            Self::Other { text } => Some(text),
            _ => None,
        }
    }
}
