//! Knowledge adapter - Arch Wiki consultation
//!
//! Provides procedural knowledge from Arch Wiki

/// Knowledge source type
#[derive(Debug, Clone, PartialEq)]
pub enum KnowledgeSourceKind {
    ArchWiki,
    OfficialProjectDoc,
}

/// Reference to a knowledge source
#[derive(Debug, Clone)]
pub struct KnowledgeSourceRef {
    pub url: String,
    pub kind: KnowledgeSourceKind,
}

/// Summary of wiki guidance for a topic
#[derive(Debug, Clone)]
pub struct WikiSummary {
    pub topic: String,
    pub sources: Vec<KnowledgeSourceRef>,
    pub recommended_commands: Vec<String>,
    pub warnings: Vec<String>,
}

/// Get Arch Wiki help for DNS troubleshooting
///
/// Based on: https://wiki.archlinux.org/title/Systemd-resolved
pub fn get_arch_help_dns() -> WikiSummary {
    WikiSummary {
        topic: "DNS resolution troubleshooting (systemd-resolved)".to_string(),
        sources: vec![
            KnowledgeSourceRef {
                url: "https://wiki.archlinux.org/title/Systemd-resolved".to_string(),
                kind: KnowledgeSourceKind::ArchWiki,
            },
            KnowledgeSourceRef {
                url: "https://wiki.archlinux.org/title/Domain_name_resolution".to_string(),
                kind: KnowledgeSourceKind::ArchWiki,
            },
        ],
        recommended_commands: vec![
            "systemctl status systemd-resolved.service".to_string(),
            "journalctl -u systemd-resolved.service -n 50".to_string(),
            "resolvectl status".to_string(),
            "resolvectl query archlinux.org".to_string(),
            "cat /etc/resolv.conf".to_string(),
            "sudo systemctl restart systemd-resolved.service".to_string(),
        ],
        warnings: vec![
            "Restarting systemd-resolved may briefly interrupt DNS resolution".to_string(),
            "Check /etc/resolv.conf is symlinked to systemd-resolved stub".to_string(),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_dns_wiki_has_arch_sources() {
        let wiki = get_arch_help_dns();

        assert!(!wiki.sources.is_empty(), "Should have at least one source");

        for source in &wiki.sources {
            assert!(
                source.url.starts_with("https://wiki.archlinux.org/"),
                "All sources should be from Arch Wiki"
            );
            assert_eq!(source.kind, KnowledgeSourceKind::ArchWiki);
        }
    }

    #[test]
    fn test_dns_wiki_has_commands() {
        let wiki = get_arch_help_dns();

        assert!(!wiki.recommended_commands.is_empty(), "Should have commands");

        // Verify it includes systemd-resolved commands
        let has_status = wiki.recommended_commands.iter()
            .any(|cmd| cmd.contains("systemctl status systemd-resolved"));
        let has_restart = wiki.recommended_commands.iter()
            .any(|cmd| cmd.contains("systemctl restart systemd-resolved"));

        assert!(has_status, "Should include status check");
        assert!(has_restart, "Should include restart command");
    }
}
