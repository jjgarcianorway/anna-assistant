//! Action handlers for package installation and service management.
//!
//! v0.0.99: Handles InstallPackage and ManageService query classes.
//! All actions require user confirmation via proposed_change.

use anna_shared::change::{ChangePlan, ChangeOperation, ChangeRisk};
use anna_shared::package_recipes::{self, PackageManager};
use anna_shared::service_recipes::{self, ServiceAction};
use anna_shared::rpc::{ServiceDeskResult, SpecialistDomain, EvidenceBlock, ReliabilitySignals};
use anna_shared::transcript::Transcript;
use std::path::PathBuf;

/// Extract package name from install query
pub fn extract_package_name(query: &str) -> Option<String> {
    let q = query.to_lowercase();

    // "install htop" -> "htop"
    if q.starts_with("install ") {
        return q.strip_prefix("install ").map(|s| s.trim().to_string());
    }

    // "add htop" -> "htop"
    if q.starts_with("add ") {
        return q.strip_prefix("add ").map(|s| s.trim().to_string());
    }

    // "install the htop package" -> "htop"
    if let Some(pos) = q.find("install the ") {
        let after = &q[pos + 12..];
        let pkg = after.split_whitespace().next()?;
        return Some(pkg.to_string());
    }

    // "can you install htop" -> "htop"
    if let Some(pos) = q.find("install ") {
        let after = &q[pos + 8..];
        let pkg = after.split_whitespace().next()?;
        return Some(pkg.to_string());
    }

    None
}

/// Extract service name and action from service query
pub fn extract_service_info(query: &str) -> Option<(String, ServiceAction)> {
    let q = query.to_lowercase();

    // Check for action verbs at start
    let actions = [
        ("start ", ServiceAction::Start),
        ("stop ", ServiceAction::Stop),
        ("restart ", ServiceAction::Restart),
        ("enable ", ServiceAction::Enable),
        ("disable ", ServiceAction::Disable),
        ("reload ", ServiceAction::Reload),
    ];

    for (prefix, action) in &actions {
        if q.starts_with(prefix) {
            let service = q.strip_prefix(prefix)?.trim().to_string();
            return Some((service, *action));
        }
    }

    // "can you restart docker" -> ("docker", Restart)
    for (verb, action) in &actions {
        let verb_trimmed = verb.trim();
        if q.contains(verb_trimmed) {
            if let Some(pos) = q.find(verb_trimmed) {
                let after = &q[pos + verb_trimmed.len()..];
                let service = after.trim().split_whitespace().next()?.to_string();
                return Some((service, *action));
            }
        }
    }

    None
}

/// Create a minimal ServiceDeskResult for action handlers
fn make_result(answer: String, score: u8, proposed_change: Option<ChangePlan>) -> ServiceDeskResult {
    ServiceDeskResult {
        request_id: uuid::Uuid::new_v4().to_string(),
        case_number: None,
        assigned_staff: None,
        staff_id: None,
        answer,
        reliability_score: score,
        reliability_signals: ReliabilitySignals::default(),
        reliability_explanation: None,
        domain: SpecialistDomain::System,
        evidence: EvidenceBlock::default(),
        needs_clarification: false,
        clarification_question: None,
        clarification_request: None,
        transcript: Transcript::default(),
        execution_trace: None,
        proposed_change,
        feedback_request: None,
    }
}

/// Handle package installation request
/// Returns a ServiceDeskResult with proposed_change for user confirmation
pub fn handle_install_package(query: &str) -> ServiceDeskResult {
    let package_name = match extract_package_name(query) {
        Some(name) => name,
        None => {
            return make_result(
                "I couldn't determine which package to install. Try: \"install htop\" or \"install vim\".".to_string(),
                60,
                None,
            );
        }
    };

    // Detect package manager
    let manager = PackageManager::detect().unwrap_or(PackageManager::Pacman);

    // Look up package recipe
    let recipe = package_recipes::find_recipe(&package_name);

    let (install_cmd, description) = match &recipe {
        Some(r) => {
            let cmd = r.install_command(&manager).unwrap_or_else(|| {
                format!("{} {}", manager.install_cmd(), package_name)
            });
            (cmd, r.description.clone())
        }
        None => {
            // Unknown package - use generic install command
            let cmd = format!("{} {}", manager.install_cmd(), package_name);
            (cmd, format!("Install {} using {}", package_name, manager))
        }
    };

    // Create a change plan for the installation
    let plan = ChangePlan {
        description: format!("Install package: {} (sudo {})", package_name, install_cmd),
        target_path: PathBuf::from(format!("/usr/bin/{}", package_name)),
        backup_path: PathBuf::new(), // No backup for package install
        operation: ChangeOperation::EnsureLine { line: install_cmd.clone() },
        risk: ChangeRisk::Medium,
        target_exists: false,
        is_noop: false,
    };

    let answer = format!(
        "**Install {}**\n\n{}\n\nCommand: `sudo {}`\n\nThis will install the package using {}.",
        package_name,
        description,
        install_cmd,
        manager.display_name()
    );

    make_result(answer, 90, Some(plan))
}

/// Handle service management request
/// Returns a ServiceDeskResult with proposed_change for user confirmation
pub fn handle_manage_service(query: &str) -> ServiceDeskResult {
    let (service_name, action) = match extract_service_info(query) {
        Some(info) => info,
        None => {
            return make_result(
                "I couldn't determine which service to manage. Try: \"restart docker\" or \"start sshd\".".to_string(),
                60,
                None,
            );
        }
    };

    // Look up service recipe
    let recipe = service_recipes::find_service(&service_name);

    let (cmd, description, risk, service_unit) = match &recipe {
        Some(r) => {
            // Check if service is protected
            if !r.risk.allows_modification() {
                return make_result(
                    format!(
                        "**Cannot modify {}**\n\n{} is a protected system service that cannot be modified.\n\nReason: {}",
                        r.display_name, r.name, r.description
                    ),
                    100,
                    None,
                );
            }

            let risk = match r.risk {
                service_recipes::ServiceRisk::Low => ChangeRisk::Low,
                service_recipes::ServiceRisk::Medium => ChangeRisk::Medium,
                service_recipes::ServiceRisk::High => ChangeRisk::High,
                service_recipes::ServiceRisk::Protected => ChangeRisk::High,
            };

            (r.command_for(action), r.description.clone(), risk, r.name.clone())
        }
        None => {
            // Unknown service - use generic systemctl command
            let unit = if service_name.ends_with(".service") {
                service_name.clone()
            } else {
                format!("{}.service", service_name)
            };
            let cmd = action.systemctl_cmd(&unit);
            (cmd, format!("Manage {} service", service_name), ChangeRisk::Medium, unit)
        }
    };

    // Create a change plan for the service action
    let plan = ChangePlan {
        description: format!("{} service: {} (sudo {})", capitalize_first(action.display_name()), service_unit, cmd),
        target_path: PathBuf::from(format!("/etc/systemd/system/{}", service_unit)),
        backup_path: PathBuf::new(), // No backup for service action
        operation: ChangeOperation::EnsureLine { line: cmd.clone() },
        risk,
        target_exists: true,
        is_noop: false,
    };

    let mut answer = format!(
        "**{} {}**\n\n{}\n\nCommand: `sudo {}`",
        capitalize_first(action.display_name()),
        service_unit,
        description,
        cmd
    );

    // Add rollback info if available
    if let Some(opposite) = action.opposite() {
        answer.push_str(&format!("\n\nTo undo: `sudo {}`", opposite.systemctl_cmd(&service_unit)));
    }

    make_result(answer, 90, Some(plan))
}

/// Capitalize first letter of a string
fn capitalize_first(s: &str) -> String {
    let mut chars = s.chars();
    match chars.next() {
        None => String::new(),
        Some(c) => c.to_uppercase().collect::<String>() + chars.as_str(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_package_name() {
        assert_eq!(extract_package_name("install htop"), Some("htop".to_string()));
        assert_eq!(extract_package_name("install vim"), Some("vim".to_string()));
        assert_eq!(extract_package_name("add nano"), Some("nano".to_string()));
        assert_eq!(extract_package_name("can you install git"), Some("git".to_string()));
        assert_eq!(extract_package_name("random query"), None);
    }

    #[test]
    fn test_extract_service_info() {
        assert_eq!(
            extract_service_info("restart docker"),
            Some(("docker".to_string(), ServiceAction::Restart))
        );
        assert_eq!(
            extract_service_info("start sshd"),
            Some(("sshd".to_string(), ServiceAction::Start))
        );
        assert_eq!(
            extract_service_info("stop nginx"),
            Some(("nginx".to_string(), ServiceAction::Stop))
        );
        assert_eq!(
            extract_service_info("enable bluetooth"),
            Some(("bluetooth".to_string(), ServiceAction::Enable))
        );
    }

    #[test]
    fn test_handle_install_package() {
        let result = handle_install_package("install htop");
        assert!(result.answer.contains("htop"));
        assert!(result.proposed_change.is_some());
    }

    #[test]
    fn test_handle_manage_service() {
        let result = handle_manage_service("restart docker");
        assert!(result.answer.contains("docker"));
        assert!(result.proposed_change.is_some());
    }

    #[test]
    fn test_protected_service() {
        let result = handle_manage_service("stop dbus");
        assert!(result.answer.contains("protected"));
        assert!(result.proposed_change.is_none());
    }
}
