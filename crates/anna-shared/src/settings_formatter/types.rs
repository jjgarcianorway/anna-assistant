// v0.0.644: Settings Formatter Types (Phase 220)
// Type definitions for settings formatter

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::unified_settings::SettingsCategory;

/// Format type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub enum FormatType {
    /// Plain text
    #[default]
    Plain,
    /// JSON format
    Json,
    /// TOML format
    Toml,
    /// YAML format
    Yaml,
    /// Table format
    Table,
}

impl std::fmt::Display for FormatType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Plain => write!(f, "plain"),
            Self::Json => write!(f, "json"),
            Self::Toml => write!(f, "toml"),
            Self::Yaml => write!(f, "yaml"),
            Self::Table => write!(f, "table"),
        }
    }
}

/// Format style
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum FormatStyle {
    /// Compact style
    #[default]
    Compact,
    /// Pretty style
    Pretty,
    /// Minimal style
    Minimal,
    /// Verbose style
    Verbose,
}

impl std::fmt::Display for FormatStyle {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Compact => write!(f, "compact"),
            Self::Pretty => write!(f, "pretty"),
            Self::Minimal => write!(f, "minimal"),
            Self::Verbose => write!(f, "verbose"),
        }
    }
}

/// Formatter config
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FormatterConfig {
    /// Format type
    pub format_type: FormatType,
    /// Format style
    pub format_style: FormatStyle,
    /// Category filter
    pub category: Option<SettingsCategory>,
    /// Include metadata
    pub include_metadata: bool,
    /// Indent size
    pub indent_size: usize,
}

impl FormatterConfig {
    /// Create new config
    pub fn new(format_type: FormatType) -> Self {
        Self {
            format_type,
            format_style: FormatStyle::Compact,
            category: None,
            include_metadata: false,
            indent_size: 2,
        }
    }

    /// Set format style
    pub fn format_style(mut self, style: FormatStyle) -> Self {
        self.format_style = style;
        self
    }

    /// Set category
    pub fn category(mut self, category: SettingsCategory) -> Self {
        self.category = Some(category);
        self
    }

    /// Set include metadata
    pub fn include_metadata(mut self, include: bool) -> Self {
        self.include_metadata = include;
        self
    }

    /// Set indent size
    pub fn indent_size(mut self, size: usize) -> Self {
        self.indent_size = size;
        self
    }
}

impl Default for FormatterConfig {
    fn default() -> Self {
        Self::new(FormatType::Plain)
    }
}

/// Format result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FormatResult {
    /// Original value
    pub original: String,
    /// Formatted value
    pub formatted: String,
    /// Format type used
    pub format_type: FormatType,
    /// Format style used
    pub format_style: FormatStyle,
}

impl FormatResult {
    /// Create new result
    pub fn new(
        original: impl Into<String>,
        formatted: impl Into<String>,
        format_type: FormatType,
        format_style: FormatStyle,
    ) -> Self {
        Self {
            original: original.into(),
            formatted: formatted.into(),
            format_type,
            format_style,
        }
    }

    /// Get output length
    pub fn output_length(&self) -> usize {
        self.formatted.len()
    }

    /// Was transformed
    pub fn was_transformed(&self) -> bool {
        self.original != self.formatted
    }
}

/// Formatter stats
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct FormatterStats {
    /// Total formatted
    pub total_formatted: usize,
    /// By format type
    pub by_type: HashMap<String, usize>,
    /// By format style
    pub by_style: HashMap<String, usize>,
    /// Total output bytes
    pub total_output_bytes: usize,
}

impl FormatterStats {
    /// Record formatting
    pub fn record(&mut self, format_type: FormatType, format_style: FormatStyle, output_len: usize) {
        self.total_formatted += 1;
        *self.by_type.entry(format_type.to_string()).or_insert(0) += 1;
        *self.by_style.entry(format_style.to_string()).or_insert(0) += 1;
        self.total_output_bytes += output_len;
    }

    /// Average output size
    pub fn average_output_size(&self) -> f64 {
        if self.total_formatted == 0 {
            0.0
        } else {
            self.total_output_bytes as f64 / self.total_formatted as f64
        }
    }
}
