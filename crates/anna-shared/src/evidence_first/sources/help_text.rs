//! Help text source (v0.0.435).

use serde::{Deserialize, Serialize};
use std::process::Command;

use super::error::SourceError;

/// Help text source.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HelpTextSource {
    /// Command name.
    pub command: String,
    /// Which variant worked.
    pub variant: HelpVariant,
    /// Retrieved content.
    pub content: Option<String>,
}

/// Help text retrieval variant.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum HelpVariant {
    /// --help flag.
    LongHelp,
    /// -h flag.
    ShortHelp,
    /// help subcommand.
    HelpSubcommand,
}

impl HelpVariant {
    /// Get the flag/argument.
    pub fn flag(&self) -> &'static str {
        match self {
            Self::LongHelp => "--help",
            Self::ShortHelp => "-h",
            Self::HelpSubcommand => "help",
        }
    }
}

impl HelpTextSource {
    /// Create a new help text source.
    pub fn new(command: &str) -> Self {
        Self {
            command: command.to_string(),
            variant: HelpVariant::LongHelp,
            content: None,
        }
    }

    /// Retrieve help text, trying multiple variants.
    pub fn retrieve(&mut self) -> Result<(), SourceError> {
        // Try variants in order
        for variant in [
            HelpVariant::LongHelp,
            HelpVariant::ShortHelp,
            HelpVariant::HelpSubcommand,
        ] {
            if let Ok(content) = self.try_variant(variant) {
                self.variant = variant;
                self.content = Some(content);
                return Ok(());
            }
        }

        Err(SourceError::NotFound(format!("help for {}", self.command)))
    }

    /// Try a specific help variant.
    fn try_variant(&self, variant: HelpVariant) -> Result<String, SourceError> {
        let output = match variant {
            HelpVariant::LongHelp => Command::new(&self.command).arg("--help").output(),
            HelpVariant::ShortHelp => Command::new(&self.command).arg("-h").output(),
            HelpVariant::HelpSubcommand => Command::new(&self.command).arg("help").output(),
        };

        let output = output.map_err(|e| SourceError::CommandFailed(e.to_string()))?;

        // Help might be on stdout or stderr
        let stdout = String::from_utf8_lossy(&output.stdout);
        let stderr = String::from_utf8_lossy(&output.stderr);

        let content = if stdout.len() > stderr.len() {
            stdout.to_string()
        } else {
            stderr.to_string()
        };

        if content.len() > 50 {
            Ok(content)
        } else {
            Err(SourceError::NotFound("empty help".to_string()))
        }
    }

    /// Search for text in help content.
    pub fn search(&self, query: &str) -> Vec<String> {
        let Some(content) = &self.content else {
            return Vec::new();
        };

        let mut results = Vec::new();
        let query_lower = query.to_lowercase();

        for line in content.lines() {
            if line.to_lowercase().contains(&query_lower) {
                let snippet = line.trim().to_string();
                if !snippet.is_empty() {
                    if snippet.len() <= super::super::MAX_CITATION_EXCERPT_LEN {
                        results.push(snippet);
                    } else {
                        results.push(format!(
                            "{}...",
                            &snippet[..super::super::MAX_CITATION_EXCERPT_LEN]
                        ));
                    }
                }
            }
        }

        results.truncate(5);
        results
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_help_variant_flags() {
        assert_eq!(HelpVariant::LongHelp.flag(), "--help");
        assert_eq!(HelpVariant::ShortHelp.flag(), "-h");
        assert_eq!(HelpVariant::HelpSubcommand.flag(), "help");
    }
}
