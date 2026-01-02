//! Documentation source types

use serde::{Deserialize, Serialize};

/// Documentation source kind, in priority order
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DocSourceKind {
    /// Arch Wiki (preferred for conceptual/Arch-specific guidance)
    ArchWiki,
    /// Man pages (command semantics and flags)
    ManPage,
    /// Tool help output (--help, -h)
    ToolHelp,
    /// Local documentation files (/usr/share/doc, etc.)
    LocalDoc,
}

impl DocSourceKind {
    /// Priority for sorting (lower = higher priority)
    pub fn priority(&self) -> u8 {
        match self {
            Self::ArchWiki => 1,
            Self::ManPage => 2,
            Self::ToolHelp => 3,
            Self::LocalDoc => 4,
        }
    }

    /// Human-readable display name
    pub fn display_name(&self) -> &'static str {
        match self {
            Self::ArchWiki => "Arch Wiki",
            Self::ManPage => "Man page",
            Self::ToolHelp => "Help output",
            Self::LocalDoc => "Local doc",
        }
    }

    /// Citation format for evidence
    pub fn citation_format(&self, name: &str, section: Option<&str>) -> String {
        match self {
            Self::ArchWiki => {
                if let Some(sec) = section {
                    format!("Arch Wiki: {}#{}", name, sec)
                } else {
                    format!("Arch Wiki: {}", name)
                }
            }
            Self::ManPage => {
                if let Some(sec) = section {
                    format!("{}({})", name, sec)
                } else {
                    format!("{}", name)
                }
            }
            Self::ToolHelp => format!("{} --help", name),
            Self::LocalDoc => format!("/usr/share/doc/{}", name),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_doc_source_priority() {
        assert!(DocSourceKind::ArchWiki.priority() < DocSourceKind::ManPage.priority());
        assert!(DocSourceKind::ManPage.priority() < DocSourceKind::ToolHelp.priority());
    }
}
