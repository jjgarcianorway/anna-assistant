//! Domain and safety inference (v0.0.427).

use crate::learning_engine::{RecipeSafety, RiskLevel};

/// Infer domain from specialist department or question
pub fn infer_domain(department: &str, question: &str) -> String {
    // First try department
    let domain = match department.to_lowercase().as_str() {
        "desktop" => "desktop",
        "server" => "server",
        "network" => "network",
        "security" => "security",
        _ => "",
    };

    if !domain.is_empty() {
        // Add sub-domain from question
        let sub = infer_subdomain(question);
        if sub.is_empty() {
            domain.to_string()
        } else {
            format!("{}.{}", domain, sub)
        }
    } else {
        // Infer from question only
        let sub = infer_subdomain(question);
        if sub.is_empty() {
            "general".to_string()
        } else {
            sub
        }
    }
}

/// Infer sub-domain from question content
pub fn infer_subdomain(question: &str) -> String {
    let q = question.to_lowercase();

    if q.contains("systemd") || q.contains("service") || q.contains("systemctl") {
        "services.systemd"
    } else if q.contains("memory") || q.contains("ram") || q.contains("swap") {
        "performance.memory"
    } else if q.contains("disk") || q.contains("storage") || q.contains("mount") {
        "storage.disk"
    } else if q.contains("network") || q.contains("wifi") || q.contains("ethernet") {
        "network"
    } else if q.contains("pacman") || q.contains("package") || q.contains("install") {
        "packages"
    } else if q.contains("boot") || q.contains("startup") {
        "boot"
    } else if q.contains("process") || q.contains("cpu") {
        "performance.cpu"
    } else {
        ""
    }
    .to_string()
}

/// Infer safety level from actions
pub fn infer_safety(actions: &[crate::specialist_v3::Action]) -> RecipeSafety {
    let mut max_risk = RiskLevel::Low;
    let mut requires_sudo = false;

    for action in actions {
        if action.run_as == crate::specialist_v3::RunAs::Root {
            requires_sudo = true;
        }

        let action_risk = match action.risk_level {
            crate::specialist_v3::RiskLevel::High => RiskLevel::High,
            crate::specialist_v3::RiskLevel::Medium => RiskLevel::Medium,
            crate::specialist_v3::RiskLevel::Low => RiskLevel::Low,
        };

        if action_risk > max_risk {
            max_risk = action_risk;
        }
    }

    RecipeSafety {
        risk: max_risk,
        needs_backup: max_risk >= RiskLevel::Medium,
        requires_sudo,
        warning: if max_risk >= RiskLevel::High {
            Some("This recipe may make significant system changes".to_string())
        } else {
            None
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_domain_inference() {
        assert!(infer_domain("desktop", "check memory").contains("memory"));
        assert!(infer_domain("server", "systemd service").contains("systemd"));
        assert!(infer_domain("", "check disk space").contains("disk"));
    }

    #[test]
    fn test_safety_inference() {
        let actions = vec![crate::specialist_v3::Action {
            id: "act-1".to_string(),
            title: "Restart".to_string(),
            command: "sudo systemctl restart nginx".to_string(),
            run_as: crate::specialist_v3::RunAs::Root,
            risk_level: crate::specialist_v3::RiskLevel::Medium,
            auto_run: false,
        }];

        let safety = infer_safety(&actions);
        assert!(safety.requires_sudo);
        assert_eq!(safety.risk, RiskLevel::Medium);
    }
}
