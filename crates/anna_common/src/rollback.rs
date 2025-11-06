//! Rollback command generation for Anna Assistant (Beta.89+)
//!
//! Generates undo commands for various types of operations, enabling users
//! to safely reverse actions they've applied.

/// Generate a rollback command for a given action command
///
/// This function analyzes the action command and generates an appropriate
/// undo command. If the action cannot be safely rolled back, returns None.
pub fn generate_rollback_command(command: &str) -> (Option<String>, bool, Option<String>) {
    // Returns: (rollback_command, can_rollback, reason_if_cannot)

    let trimmed = command.trim();

    // Pacman package installation
    if trimmed.contains("pacman -S") || trimmed.contains("pacman -Sy") {
        return match extract_packages_from_install(trimmed) {
            Some(packages) if !packages.is_empty() => {
                let rollback = format!("pacman -Rns --noconfirm {}", packages.join(" "));
                (Some(rollback), true, None)
            }
            _ => (None, false, Some("Could not extract package names".to_string())),
        };
    }

    // Pacman package removal - can be rolled back by reinstalling
    if trimmed.contains("pacman -R") {
        return match extract_packages_from_remove(trimmed) {
            Some(packages) if !packages.is_empty() => {
                let rollback = format!("pacman -S --noconfirm {}", packages.join(" "));
                (Some(rollback), true, None)
            }
            _ => (None, false, Some("Could not extract package names".to_string())),
        };
    }

    // File modification/creation - cannot safely rollback without backup
    if trimmed.contains("sed -i") || trimmed.contains("echo >") || trimmed.contains("tee") {
        return (
            None,
            false,
            Some("File modifications cannot be automatically rolled back without backup".to_string()),
        );
    }

    // Systemd service enable/disable
    if trimmed.contains("systemctl enable") {
        return match extract_service_name(trimmed) {
            Some(service) => {
                let rollback = format!("systemctl disable {}", service);
                (Some(rollback), true, None)
            }
            None => (None, false, Some("Could not extract service name".to_string())),
        };
    }

    if trimmed.contains("systemctl disable") {
        return match extract_service_name(trimmed) {
            Some(service) => {
                let rollback = format!("systemctl enable {}", service);
                (Some(rollback), true, None)
            }
            None => (None, false, Some("Could not extract service name".to_string())),
        };
    }

    // Systemd service start/stop
    if trimmed.contains("systemctl start") {
        return match extract_service_name(trimmed) {
            Some(service) => {
                let rollback = format!("systemctl stop {}", service);
                (Some(rollback), true, None)
            }
            None => (None, false, Some("Could not extract service name".to_string())),
        };
    }

    if trimmed.contains("systemctl stop") {
        return match extract_service_name(trimmed) {
            Some(service) => {
                let rollback = format!("systemctl start {}", service);
                (Some(rollback), true, None)
            }
            None => (None, false, Some("Could not extract service name".to_string())),
        };
    }

    // Git operations
    if trimmed.contains("git clone") {
        // Cannot easily rollback - would need to track the directory
        return (
            None,
            false,
            Some("Git clone operations require manual cleanup".to_string()),
        );
    }

    // Default: Cannot generate rollback
    (
        None,
        false,
        Some("Rollback not yet implemented for this command type".to_string()),
    )
}

/// Extract package names from a pacman install command
fn extract_packages_from_install(command: &str) -> Option<Vec<String>> {
    // Look for pattern: pacman -S [flags] package1 package2 ...

    // Find the position of -S or -Sy
    let start_pos = if let Some(pos) = command.find(" -S ") {
        pos + 4  // After " -S "
    } else if let Some(pos) = command.find(" -Sy ") {
        pos + 5  // After " -Sy "
    } else {
        return None;
    };

    // Extract everything after the -S flag
    let after_flag = &command[start_pos..];

    // Split by whitespace and filter out flags
    let packages: Vec<String> = after_flag
        .split_whitespace()
        .filter(|&word| !word.starts_with('-') && !word.is_empty())
        .filter(|&word| word != "&&" && word != "||" && word != "|")
        .map(String::from)
        .collect();

    if packages.is_empty() {
        None
    } else {
        Some(packages)
    }
}

/// Extract package names from a pacman remove command
fn extract_packages_from_remove(command: &str) -> Option<Vec<String>> {
    // Look for pattern: pacman -R [flags] package1 package2 ...

    let start_pos = command.find(" -R")? + 3;
    let after_flag = &command[start_pos..];

    // Skip any additional flags like 'ns'
    let clean_start = after_flag.trim_start_matches(|c: char| c != ' ');

    let packages: Vec<String> = clean_start
        .split_whitespace()
        .filter(|&word| !word.starts_with('-') && !word.is_empty())
        .filter(|&word| word != "&&" && word != "||" && word != "|")
        .map(String::from)
        .collect();

    if packages.is_empty() {
        None
    } else {
        Some(packages)
    }
}

/// Extract service name from a systemctl command
fn extract_service_name(command: &str) -> Option<String> {
    // Look for pattern: systemctl [command] servicename
    let parts: Vec<&str> = command.split_whitespace().collect();

    // Find 'systemctl' and get the next word after the command
    for (i, &word) in parts.iter().enumerate() {
        if word == "systemctl" && i + 2 < parts.len() {
            return Some(parts[i + 2].to_string());
        }
    }

    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pacman_install_rollback() {
        let (rollback, can_rollback, _) =
            generate_rollback_command("pacman -S --noconfirm vulkan-intel");

        assert!(can_rollback);
        assert_eq!(rollback, Some("pacman -Rns --noconfirm vulkan-intel".to_string()));
    }

    #[test]
    fn test_pacman_install_multiple_packages() {
        let (rollback, can_rollback, _) =
            generate_rollback_command("pacman -Sy --noconfirm mesa lib32-mesa");

        assert!(can_rollback);
        assert_eq!(rollback, Some("pacman -Rns --noconfirm mesa lib32-mesa".to_string()));
    }

    #[test]
    fn test_pacman_remove_rollback() {
        let (rollback, can_rollback, _) =
            generate_rollback_command("pacman -Rns --noconfirm orphan-pkg");

        assert!(can_rollback);
        assert_eq!(rollback, Some("pacman -S --noconfirm orphan-pkg".to_string()));
    }

    #[test]
    fn test_systemctl_enable_rollback() {
        let (rollback, can_rollback, _) =
            generate_rollback_command("systemctl enable bluetooth");

        assert!(can_rollback);
        assert_eq!(rollback, Some("systemctl disable bluetooth".to_string()));
    }

    #[test]
    fn test_file_modification_no_rollback() {
        let (rollback, can_rollback, reason) =
            generate_rollback_command("sed -i 's/foo/bar/g' /etc/config");

        assert!(!can_rollback);
        assert!(rollback.is_none());
        assert!(reason.is_some());
    }
}
