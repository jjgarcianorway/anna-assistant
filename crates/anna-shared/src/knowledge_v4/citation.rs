//! Citation formatting and handling (v0.0.424).

use serde::{Deserialize, Serialize};

use super::query::KnowledgeSource;

/// A structured citation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Citation {
    /// Citation ID (e.g., "man:vim")
    pub id: String,
    /// Source type
    pub source: KnowledgeSource,
    /// Display text (e.g., "man vim")
    pub display: String,
    /// Optional URL or path
    pub location: Option<String>,
}

impl Citation {
    /// Create a new citation
    pub fn new(source: KnowledgeSource, name: &str) -> Self {
        let id = format!("{}:{}", source.citation_prefix(), name);
        let display = format_display(source, name);

        Self {
            id,
            source,
            display,
            location: None,
        }
    }

    /// Create from man page
    pub fn man(command: &str) -> Self {
        Self {
            id: format!("man:{}", command),
            source: KnowledgeSource::ManPage,
            display: format!("man {}", command),
            location: None,
        }
    }

    /// Create from man page with section
    pub fn man_section(command: &str, section: u8) -> Self {
        Self {
            id: format!("man:{}({})", command, section),
            source: KnowledgeSource::ManPage,
            display: format!("{}({})", command, section),
            location: None,
        }
    }

    /// Create from help output
    pub fn help(command: &str) -> Self {
        Self {
            id: format!("help:{}", command),
            source: KnowledgeSource::CommandHelp,
            display: format!("{} --help", command),
            location: None,
        }
    }

    /// Create from local doc
    pub fn doc(name: &str, path: Option<&str>) -> Self {
        Self {
            id: format!("doc:{}", name),
            source: KnowledgeSource::LocalDocs,
            display: format!("doc: {}", name),
            location: path.map(String::from),
        }
    }

    /// Create from Arch Wiki
    pub fn wiki(topic: &str) -> Self {
        Self {
            id: format!("wiki:{}", topic.to_lowercase().replace(' ', "_")),
            source: KnowledgeSource::ArchWiki,
            display: format!("Arch Wiki: {}", topic),
            location: Some(format!(
                "https://wiki.archlinux.org/title/{}",
                topic.replace(' ', "_")
            )),
        }
    }

    /// Set location
    pub fn with_location(mut self, location: &str) -> Self {
        self.location = Some(location.to_string());
        self
    }

    /// Format for inline reference
    pub fn inline(&self) -> String {
        format!("[{}]", self.id)
    }

    /// Format for footnote
    pub fn footnote(&self) -> String {
        if let Some(ref loc) = self.location {
            format!("{}: {}", self.id, loc)
        } else {
            format!("{}: {}", self.id, self.display)
        }
    }
}

/// Format display text for a source and name
fn format_display(source: KnowledgeSource, name: &str) -> String {
    match source {
        KnowledgeSource::ManPage => format!("man {}", name),
        KnowledgeSource::CommandHelp => format!("{} --help", name),
        KnowledgeSource::LocalDocs => format!("doc: {}", name),
        KnowledgeSource::ArchWiki => format!("Arch Wiki: {}", name),
    }
}

/// Format a citation ID for display
pub fn format_citation(citation_id: &str) -> String {
    if let Some((prefix, name)) = citation_id.split_once(':') {
        match prefix {
            "man" => format!("man {}", name),
            "help" => format!("{} --help", name),
            "doc" => format!("doc: {}", name),
            "wiki" => format!("Arch Wiki: {}", name),
            _ => citation_id.to_string(),
        }
    } else {
        citation_id.to_string()
    }
}

/// Parse a citation ID into source and name
pub fn parse_citation(citation_id: &str) -> Option<(KnowledgeSource, String)> {
    let (prefix, name) = citation_id.split_once(':')?;
    let source = match prefix {
        "man" => KnowledgeSource::ManPage,
        "help" => KnowledgeSource::CommandHelp,
        "doc" => KnowledgeSource::LocalDocs,
        "wiki" => KnowledgeSource::ArchWiki,
        _ => return None,
    };
    Some((source, name.to_string()))
}

/// Format citations for user-facing output
pub fn format_sources(citations: &[&str]) -> String {
    if citations.is_empty() {
        return String::new();
    }

    let formatted: Vec<String> = citations.iter().map(|c| format_citation(c)).collect();

    if formatted.len() == 1 {
        format!("Source: {}", formatted[0])
    } else {
        format!(
            "Sources:\n{}",
            formatted
                .iter()
                .map(|s| format!("  - {}", s))
                .collect::<Vec<_>>()
                .join("\n")
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_citation_man() {
        let cite = Citation::man("vim");
        assert_eq!(cite.id, "man:vim");
        assert_eq!(cite.display, "man vim");
    }

    #[test]
    fn test_citation_man_section() {
        let cite = Citation::man_section("systemctl", 1);
        assert_eq!(cite.id, "man:systemctl(1)");
        assert_eq!(cite.display, "systemctl(1)");
    }

    #[test]
    fn test_citation_wiki() {
        let cite = Citation::wiki("Systemd");
        assert_eq!(cite.id, "wiki:systemd");
        assert!(cite.location.unwrap().contains("wiki.archlinux.org"));
    }

    #[test]
    fn test_format_citation() {
        assert_eq!(format_citation("man:vim"), "man vim");
        assert_eq!(format_citation("help:pacman"), "pacman --help");
        assert_eq!(format_citation("wiki:systemd"), "Arch Wiki: systemd");
    }

    #[test]
    fn test_parse_citation() {
        let (source, name) = parse_citation("man:vim").unwrap();
        assert_eq!(source, KnowledgeSource::ManPage);
        assert_eq!(name, "vim");

        assert!(parse_citation("invalid").is_none());
    }

    #[test]
    fn test_format_sources() {
        let result = format_sources(&["man:vim"]);
        assert!(result.contains("Source: man vim"));

        let result2 = format_sources(&["man:vim", "help:pacman"]);
        assert!(result2.contains("Sources:"));
    }
}
