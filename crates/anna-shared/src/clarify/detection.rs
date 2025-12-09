//! Clarification detection logic (v0.0.191).

use crate::facts::{FactKey, FactsStore};

use super::legacy::ClarifyKind;

/// Check if query needs clarification
pub fn needs_clarification(query: &str, facts: &FactsStore) -> Option<ClarifyKind> {
    let q = query.to_lowercase();
    if (q.contains("edit") || q.contains("editor"))
        && !q.contains("vim")
        && !q.contains("nano")
        && !q.contains("emacs")
        && !q.contains("code")
        && !facts.has_verified(&FactKey::PreferredEditor)
    {
        return Some(ClarifyKind::PreferredEditor);
    }
    if (q.contains("service") || q.contains("systemctl"))
        && !q.contains("--failed")
        && extract_service_name(&q).is_none()
    {
        return Some(ClarifyKind::ServiceName);
    }
    if (q.contains("mount") || q.contains("partition")) && !q.contains("/") {
        return Some(ClarifyKind::MountPoint);
    }
    None
}

/// Extract service name from query
pub fn extract_service_name(query: &str) -> Option<String> {
    let patterns = [
        "nginx",
        "docker",
        "sshd",
        "apache",
        "mysql",
        "postgresql",
        "redis",
    ];
    for p in patterns {
        if query.contains(p) {
            return Some(p.to_string());
        }
    }
    if let Some(idx) = query.find(".service") {
        let before = &query[..idx];
        if let Some(start) = before.rfind(' ') {
            return Some(before[start + 1..].to_string());
        }
    }
    None
}
