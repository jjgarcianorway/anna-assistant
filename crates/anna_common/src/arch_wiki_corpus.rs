//! Arch Wiki Corpus - Local Embedded Wiki Snippets (6.13.0)
//!
//! Purpose: Provide offline, curated snippets from Arch Wiki for specific problems.
//! This is NOT a scraper or live fetcher - it's a small, hand-written corpus
//! of canonical Arch Wiki knowledge for concrete issues Anna can fix.
//!
//! Philosophy:
//! - One snippet per concrete problem (not entire wiki pages)
//! - Short summaries (1-3 sentences)
//! - Canonical commands from the wiki (not improvised)
//! - Always cite the source URL

/// Topics Anna has Arch Wiki knowledge for
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WikiTopic {
    /// TLP power management service
    TlpPowerSaving,
    /// Basic systemd service management
    SystemdServiceManagement,
}

/// A snippet of Arch Wiki knowledge for a specific topic
#[derive(Debug, Clone)]
pub struct WikiSnippet {
    /// The topic this snippet covers
    pub topic: WikiTopic,
    /// Canonical Arch Wiki URL
    pub url: &'static str,
    /// 1-3 sentence plain text summary
    pub summary: &'static str,
    /// Canonical commands from the wiki (in order)
    pub key_commands: &'static [&'static str],
}

/// Get Arch Wiki snippet for a topic
///
/// Returns canonical wiki knowledge including URL, summary, and commands.
/// This is the single source of truth for how Anna explains wiki-based fixes.
pub fn get_wiki_snippet(topic: WikiTopic) -> WikiSnippet {
    match topic {
        WikiTopic::TlpPowerSaving => WikiSnippet {
            topic: WikiTopic::TlpPowerSaving,
            url: "https://wiki.archlinux.org/title/TLP",
            summary: "TLP is a power management tool that applies laptop power saving settings. \
                      It runs as a systemd service and must be enabled to apply settings on boot. \
                      The service should be enabled with 'systemctl enable --now tlp.service'.",
            key_commands: &[
                "systemctl status tlp.service",
                "systemctl enable --now tlp.service",
                "systemctl disable --now tlp.service", // rollback
            ],
        },
        WikiTopic::SystemdServiceManagement => WikiSnippet {
            topic: WikiTopic::SystemdServiceManagement,
            url: "https://wiki.archlinux.org/title/Systemd#Using_units",
            summary: "Systemd services are controlled with systemctl. \
                      'enable' configures a service to start on boot, \
                      'start' runs it immediately, and '--now' does both. \
                      Always check status before and after changes.",
            key_commands: &[
                "systemctl status <unit>",
                "systemctl enable <unit>",
                "systemctl start <unit>",
                "systemctl enable --now <unit>",
                "systemctl disable --now <unit>",
            ],
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tlp_snippet_has_correct_url() {
        let snippet = get_wiki_snippet(WikiTopic::TlpPowerSaving);
        assert_eq!(snippet.url, "https://wiki.archlinux.org/title/TLP");
        assert_eq!(snippet.topic, WikiTopic::TlpPowerSaving);
    }

    #[test]
    fn test_tlp_snippet_has_canonical_commands() {
        let snippet = get_wiki_snippet(WikiTopic::TlpPowerSaving);

        // Must contain status check
        assert!(snippet.key_commands.contains(&"systemctl status tlp.service"));

        // Must contain enable command
        assert!(snippet.key_commands.contains(&"systemctl enable --now tlp.service"));

        // Must contain rollback command
        assert!(snippet.key_commands.contains(&"systemctl disable --now tlp.service"));
    }

    #[test]
    fn test_tlp_snippet_summary_not_empty() {
        let snippet = get_wiki_snippet(WikiTopic::TlpPowerSaving);
        assert!(!snippet.summary.is_empty());
        assert!(snippet.summary.len() > 50); // Should be substantial
    }

    #[test]
    fn test_systemd_snippet_has_correct_url() {
        let snippet = get_wiki_snippet(WikiTopic::SystemdServiceManagement);
        assert!(snippet.url.contains("wiki.archlinux.org"));
        assert!(snippet.url.contains("Systemd"));
    }

    #[test]
    fn test_systemd_snippet_has_generic_commands() {
        let snippet = get_wiki_snippet(WikiTopic::SystemdServiceManagement);

        // Should use <unit> placeholder for generic commands
        assert!(snippet.key_commands.iter().any(|cmd| cmd.contains("<unit>")));
    }
}
