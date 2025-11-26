//! Anna Brain v10.0.1 - Fallback Answer Extraction
//!
//! When the LLM fails to follow the protocol, these functions extract
//! answers directly from the gathered evidence using pattern matching.

use crate::brain_v10::contracts::{BrainSession, ReliabilityLabel};
use crate::brain_v10::orchestrator::BrainResult;

/// Try to extract an obvious answer from evidence when LLM fails
pub fn try_fallback_answer(query: &str, session: &BrainSession) -> Option<BrainResult> {
    let query_lower = query.to_lowercase();

    // Package queries
    if query_lower.contains("installed") {
        if let Some(result) = fallback_package_query(&query_lower, session) {
            return Some(result);
        }
    }

    // RAM queries
    if query_lower.contains("ram") || query_lower.contains("memory") {
        if let Some(result) = fallback_ram_query(session) {
            return Some(result);
        }
    }

    // CPU queries
    if query_lower.contains("cpu") || query_lower.contains("processor") {
        if let Some(result) = fallback_cpu_query(session) {
            return Some(result);
        }
    }

    // Disk space queries
    if query_lower.contains("disk") || query_lower.contains("space") || query_lower.contains("storage") {
        if let Some(result) = fallback_disk_query(session) {
            return Some(result);
        }
    }

    // GPU queries
    if query_lower.contains("gpu") || query_lower.contains("graphics") || query_lower.contains("video") {
        if let Some(result) = fallback_gpu_query(session) {
            return Some(result);
        }
    }

    // Desktop/WM queries
    if query_lower.contains("desktop") || query_lower.contains("window manager") {
        if let Some(result) = fallback_desktop_query(session) {
            return Some(result);
        }
    }

    // Games queries
    if query_lower.contains("game") {
        if let Some(result) = fallback_games_query(session) {
            return Some(result);
        }
    }

    // Orphan packages
    if query_lower.contains("orphan") {
        if let Some(result) = fallback_orphan_query(session) {
            return Some(result);
        }
    }

    // Network queries
    if query_lower.contains("network") || query_lower.contains("wifi") || query_lower.contains("ethernet") {
        if let Some(result) = fallback_network_query(session) {
            return Some(result);
        }
    }

    // Updates queries
    if query_lower.contains("update") {
        if let Some(result) = fallback_updates_query(session) {
            return Some(result);
        }
    }

    None
}

fn fallback_package_query(query_lower: &str, session: &BrainSession) -> Option<BrainResult> {
    let package_name = query_lower
        .split_whitespace()
        .find(|w| !["is", "installed", "installed?", "?", "do", "i", "have"].contains(w))?;

    for evidence in &session.evidence {
        if evidence.source == "run_shell" {
            let is_relevant = evidence.description.to_lowercase().contains(package_name)
                || evidence.content.to_lowercase().contains(package_name);
            if !is_relevant { continue; }

            if evidence.content.contains("local/") && evidence.is_success() {
                let pkg_line = evidence.content.lines().next()?;
                return Some(BrainResult::Answer {
                    text: format!("Yes, {} is installed [{}].\nVersion: {}", package_name, evidence.id, pkg_line.trim()),
                    reliability: 0.85,
                    label: ReliabilityLabel::Medium,
                });
            }
            if evidence.content.trim().is_empty() && evidence.is_success() {
                return Some(BrainResult::Answer {
                    text: format!("No, {} is not installed [{}].", package_name, evidence.id),
                    reliability: 0.85,
                    label: ReliabilityLabel::Medium,
                });
            }
            if !evidence.is_success() {
                return Some(BrainResult::Answer {
                    text: format!("No, {} is not installed [{}].", package_name, evidence.id),
                    reliability: 0.85,
                    label: ReliabilityLabel::Medium,
                });
            }
        }
    }
    None
}

fn fallback_ram_query(session: &BrainSession) -> Option<BrainResult> {
    for evidence in &session.evidence {
        if evidence.content.contains("Mem:") {
            for line in evidence.content.lines() {
                if line.starts_with("Mem:") {
                    let parts: Vec<&str> = line.split_whitespace().collect();
                    if parts.len() >= 2 {
                        let mb: i64 = parts[1].parse().unwrap_or(0);
                        let gb = mb as f64 / 1024.0;
                        return Some(BrainResult::Answer {
                            text: format!("You have {:.1} GB ({} MB) of RAM [{}].", gb, mb, evidence.id),
                            reliability: 0.9,
                            label: ReliabilityLabel::High,
                        });
                    }
                }
            }
        }
    }
    None
}

fn fallback_cpu_query(session: &BrainSession) -> Option<BrainResult> {
    for evidence in &session.evidence {
        for line in evidence.content.lines() {
            if line.contains("Model name:") {
                if let Some(name) = line.split(':').nth(1) {
                    return Some(BrainResult::Answer {
                        text: format!("Your CPU is: {} [{}].", name.trim(), evidence.id),
                        reliability: 0.9,
                        label: ReliabilityLabel::High,
                    });
                }
            }
        }
    }
    None
}

fn fallback_disk_query(session: &BrainSession) -> Option<BrainResult> {
    for evidence in &session.evidence {
        if evidence.content.contains("Filesystem") && evidence.content.contains("Use%") {
            let lines: Vec<&str> = evidence.content.lines()
                .filter(|l| !l.contains("tmpfs") && !l.contains("efivarfs"))
                .collect();
            if lines.len() > 1 {
                return Some(BrainResult::Answer {
                    text: format!("Disk usage [{}]:\n```\n{}\n```", evidence.id, lines.join("\n")),
                    reliability: 0.9,
                    label: ReliabilityLabel::High,
                });
            }
        }
    }
    None
}

fn fallback_gpu_query(session: &BrainSession) -> Option<BrainResult> {
    for evidence in &session.evidence {
        if evidence.content.to_lowercase().contains("vga") {
            let gpu_lines: Vec<&str> = evidence.content.lines()
                .filter(|l| l.to_lowercase().contains("vga") || l.to_lowercase().contains("3d"))
                .collect();
            if !gpu_lines.is_empty() {
                return Some(BrainResult::Answer {
                    text: format!("Your GPU [{}]:\n{}", evidence.id, gpu_lines.join("\n")),
                    reliability: 0.9,
                    label: ReliabilityLabel::High,
                });
            }
        }
    }
    None
}

fn fallback_desktop_query(session: &BrainSession) -> Option<BrainResult> {
    for evidence in &session.evidence {
        let content = &evidence.content;
        if content.contains("XDG") || content.contains("Desktop=") || content.contains("GNOME")
           || content.contains("KDE") || content.contains("XFCE") || content.contains("Hyprland") {
            let lines: Vec<&str> = content.lines().filter(|l| !l.trim().is_empty()).collect();
            if !lines.is_empty() {
                return Some(BrainResult::Answer {
                    text: format!("Desktop environment [{}]:\n{}", evidence.id, lines.join("\n")),
                    reliability: 0.85,
                    label: ReliabilityLabel::Medium,
                });
            }
        }
    }
    None
}

fn fallback_games_query(session: &BrainSession) -> Option<BrainResult> {
    for evidence in &session.evidence {
        if evidence.content.contains("local/") {
            let games: Vec<&str> = evidence.content.lines()
                .filter(|l| l.contains("local/"))
                .collect();
            if !games.is_empty() {
                return Some(BrainResult::Answer {
                    text: format!("Gaming packages installed [{}]:\n{}", evidence.id, games.join("\n")),
                    reliability: 0.85,
                    label: ReliabilityLabel::Medium,
                });
            } else {
                return Some(BrainResult::Answer {
                    text: format!("No gaming packages found [{}].", evidence.id),
                    reliability: 0.85,
                    label: ReliabilityLabel::Medium,
                });
            }
        }
        if evidence.content.trim().is_empty() {
            return Some(BrainResult::Answer {
                text: format!("No gaming packages (Steam, Lutris, Wine, etc.) are installed [{}].", evidence.id),
                reliability: 0.85,
                label: ReliabilityLabel::Medium,
            });
        }
    }
    None
}

fn fallback_orphan_query(session: &BrainSession) -> Option<BrainResult> {
    for evidence in &session.evidence {
        if evidence.description.contains("orphan") || evidence.description.contains("Qdt") {
            if evidence.content.trim().is_empty() {
                return Some(BrainResult::Answer {
                    text: format!("No orphan packages found [{}]. Your system is clean!", evidence.id),
                    reliability: 0.9,
                    label: ReliabilityLabel::High,
                });
            } else {
                let count = evidence.content.lines().count();
                return Some(BrainResult::Answer {
                    text: format!("Found {} orphan packages [{}]:\n```\n{}\n```", count, evidence.id, evidence.content),
                    reliability: 0.9,
                    label: ReliabilityLabel::High,
                });
            }
        }
    }
    None
}

fn fallback_network_query(session: &BrainSession) -> Option<BrainResult> {
    for evidence in &session.evidence {
        if evidence.content.contains("DEVICE") || evidence.content.contains("state UP") {
            return Some(BrainResult::Answer {
                text: format!("Network status [{}]:\n```\n{}\n```", evidence.id, evidence.content),
                reliability: 0.85,
                label: ReliabilityLabel::Medium,
            });
        }
    }
    None
}

fn fallback_updates_query(session: &BrainSession) -> Option<BrainResult> {
    for evidence in &session.evidence {
        if evidence.description.contains("update") || evidence.description.contains("checkupdates") {
            if evidence.content.trim().is_empty() || evidence.content.contains("no updates") {
                return Some(BrainResult::Answer {
                    text: format!("Your system is up to date! No pending updates [{}].", evidence.id),
                    reliability: 0.9,
                    label: ReliabilityLabel::High,
                });
            } else {
                let count = evidence.content.lines().count();
                return Some(BrainResult::Answer {
                    text: format!("{} updates available [{}]:\n```\n{}\n```", count, evidence.id, evidence.content),
                    reliability: 0.9,
                    label: ReliabilityLabel::High,
                });
            }
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::brain_v10::contracts::EvidenceItem;

    fn create_test_session(evidence_content: &str, source: &str, desc: &str) -> BrainSession {
        let mut session = BrainSession::new(
            "test query",
            serde_json::json!({}),
            vec![],
        );
        session.evidence.push(EvidenceItem {
            id: "E1".to_string(),
            source: source.to_string(),
            description: desc.to_string(),
            content: evidence_content.to_string(),
            exit_code: 0,
        });
        session
    }

    #[test]
    fn test_ram_fallback() {
        let session = create_test_session(
            "              total        used        free      shared  buff/cache   available\nMem:          32000       12000       15000         500        5000       19000",
            "run_shell",
            "free -m"
        );
        let result = fallback_ram_query(&session);
        assert!(result.is_some());
        let answer = result.unwrap();
        if let BrainResult::Answer { text, .. } = answer {
            assert!(text.contains("32000"));
        }
    }

    #[test]
    fn test_cpu_fallback() {
        let session = create_test_session(
            "Model name:            AMD Ryzen 9 5900X 12-Core Processor",
            "run_shell",
            "lscpu"
        );
        let result = fallback_cpu_query(&session);
        assert!(result.is_some());
    }
}
