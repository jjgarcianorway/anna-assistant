//! Knowledge source types (v0.0.435).

use serde::{Deserialize, Serialize};

use super::help_text::HelpTextSource;
use super::local_docs::LocalDocsSource;
use super::man_page::ManPageSource;

/// A knowledge source that can provide evidence.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum KnowledgeSource {
    /// Man page documentation.
    ManPage(ManPageSource),
    /// Command help text.
    HelpText(HelpTextSource),
    /// Local documentation files.
    LocalDocs(LocalDocsSource),
    /// Cached Arch Wiki page.
    ArchWiki(String),
    /// System probe output.
    ProbeOutput(String),
}

impl KnowledgeSource {
    /// Human-readable label.
    pub fn label(&self) -> String {
        match self {
            Self::ManPage(m) => format!(
                "man {}{}",
                m.section.map(|s| format!("{} ", s)).unwrap_or_default(),
                m.command
            ),
            Self::HelpText(h) => format!("{} {}", h.command, h.variant.flag()),
            Self::LocalDocs(d) => format!("doc:{}", d.package.as_deref().unwrap_or(&d.path)),
            Self::ArchWiki(page) => format!("archwiki:{}", page),
            Self::ProbeOutput(id) => format!("probe:{}", id),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_knowledge_source_labels() {
        let man = KnowledgeSource::ManPage(ManPageSource::new("ls"));
        assert_eq!(man.label(), "man ls");

        let man_section = KnowledgeSource::ManPage(ManPageSource::new("ls").with_section(1));
        assert_eq!(man_section.label(), "man 1 ls");

        let help = KnowledgeSource::HelpText(HelpTextSource::new("git"));
        assert_eq!(help.label(), "git --help");

        let wiki = KnowledgeSource::ArchWiki("Systemd".to_string());
        assert_eq!(wiki.label(), "archwiki:Systemd");
    }
}
