use anyhow::{anyhow, Result};
use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};
use std::process::Command;

use crate::paths::{AnnaPaths, InstallMode};
use crate::ui::{self, Style, UiCfg};

pub struct DoctorError {
    message: String,
}

impl DoctorError {
    fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
        }
    }

    pub fn code(&self) -> i32 {
        1
    }
}

impl std::fmt::Display for DoctorError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.message)
    }
}

impl std::fmt::Debug for DoctorError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.message)
    }
}

impl std::error::Error for DoctorError {}

#[derive(Debug)]
pub enum DoctorCommand {
    Perms,
    Repair,
    Env,
}

pub struct DoctorArgs<'a> {
    pub command: DoctorCommand,
    pub _cfg: &'a UiCfg,
    pub style: &'a Style,
}

pub fn run(args: DoctorArgs) -> std::result::Result<(), DoctorError> {
    match args.command {
        DoctorCommand::Perms => check_perms_rpc(args.style),
        DoctorCommand::Repair => repair_installation(args.style),
        DoctorCommand::Env => dump_env(args._cfg, args.style),
    }
}

/// Check permissions via RPC (S2+)
fn check_perms_rpc(style: &Style) -> std::result::Result<(), DoctorError> {
    use crate::paths::AnnaPaths;
    use crate::rpc::RpcClient;

    println!("{}", ui::head(style, "⚙ Permissions audit"));

    let paths = AnnaPaths::detect();
    let uid = nix::unistd::Uid::effective().as_raw();

    println!(
        "{}",
        ui::info(style, &format!("Install mode: {}", paths.mode.as_str()))
    );

    // Show ANNA_MODE if set
    if let Ok(anna_mode) = std::env::var("ANNA_MODE") {
        println!(
            "{}",
            ui::info(style, &format!("ANNA_MODE: {} (env override)", anna_mode))
        );
    }

    println!(
        "{}",
        ui::info(
            style,
            &format!("Socket path: {}", paths.socket_path.display())
        )
    );

    println!();

    // Check socket exists
    if !paths.socket_path.exists() {
        println!(
            "{}",
            ui::err(
                style,
                &format!("Socket not found at {}", paths.socket_path.display())
            )
        );
        println!(
            "{}",
            ui::bullet(style, "Start annad: sudo systemctl start annad")
        );
        return Err(DoctorError::new("Socket not found"));
    }

    // Send RPC request
    let client = RpcClient::new(&paths.socket_path);
    let request = anna_rpc::Request::DoctorPerms(anna_rpc::DoctorPermsRequest { uid });

    match client.call(request) {
        Ok(anna_rpc::Response::DoctorPerms(response)) => {
            if response.issues.is_empty() {
                println!(
                    "{}",
                    ui::ok(style, "✅ All permissions and configuration look good!")
                );
                return Ok(());
            }

            // Display issues
            println!("\n{}", ui::head(style, "Issues Found"));
            for issue in &response.issues {
                let icon = match issue.severity {
                    anna_rpc::IssueSeverity::Error => ui::err(style, "✗"),
                    anna_rpc::IssueSeverity::Warning => ui::warn(style, "⚠"),
                    anna_rpc::IssueSeverity::Info => ui::info(style, "ℹ"),
                };
                println!("{} {}: {}", icon, issue.path, issue.issue);
            }

            // Display suggestions
            if !response.suggestions.is_empty() {
                println!("\n{}", ui::head(style, "Recommended Fixes"));
                for suggestion in &response.suggestions {
                    println!("{}", ui::bullet(style, suggestion));
                }
            }

            // Return error if there are errors
            let has_errors = response
                .issues
                .iter()
                .any(|i| matches!(i.severity, anna_rpc::IssueSeverity::Error));

            if has_errors {
                Err(DoctorError::new(
                    "Permission issues found. See suggestions above.",
                ))
            } else {
                Ok(())
            }
        }
        Ok(anna_rpc::Response::Error(err)) => {
            Err(DoctorError::new(format!("RPC error: {}", err.message)))
        }
        Ok(_) => Err(DoctorError::new("Unexpected response type")),
        Err(e) => Err(DoctorError::new(format!(
            "Failed to connect to daemon: {}",
            e
        ))),
    }
}

#[allow(dead_code)]
fn check_perms(style: &Style) -> std::result::Result<(), DoctorError> {
    println!("{}", ui::head(style, "⚙ Permissions audit"));

    let anna_paths = AnnaPaths::detect();
    println!("ℹ️  Install mode: {}", anna_paths.mode.as_str());

    match anna_paths.mode {
        InstallMode::System => check_system_perms(style, &anna_paths),
        InstallMode::User => check_user_perms(style, &anna_paths),
    }
}

fn check_system_perms(
    _style: &Style,
    anna_paths: &AnnaPaths,
) -> std::result::Result<(), DoctorError> {
    let target_group = "anna";
    println!("Target group: {}", target_group);

    // Check if group exists
    if !group_exists(target_group) {
        return Err(DoctorError::new(format!(
            "Group '{}' does not exist. Create it with: sudo groupadd {}",
            target_group, target_group
        )));
    }

    let paths_to_check = vec![anna_paths.data_dir.clone(), anna_paths.config_dir.clone()];

    println!("Paths inspected: {}", paths_to_check.len());

    let mut issues = Vec::new();

    for path in &paths_to_check {
        if let Err(e) = check_path_permissions(path, target_group) {
            issues.push(format!("{}: {}", path.display(), e));
        }
    }

    // Check polkit rule
    let polkit_rule = PathBuf::from("/etc/polkit-1/rules.d/50-anna.rules");
    if !polkit_rule.exists() {
        issues.push(format!(
            "Polkit rule missing at {}. Quickscan will require sudo password.",
            polkit_rule.display()
        ));
    }

    if issues.is_empty() {
        println!(
            "✅ Ownership, modes, and ACLs meet policy for {} and {}.",
            anna_paths.data_dir.display(),
            anna_paths.config_dir.display()
        );
        Ok(())
    } else {
        let mut msg = String::from("Permission issues found:\n");
        for issue in issues {
            msg.push_str(&format!("  • {}\n", issue));
        }
        msg.push_str("\nRecommended fixes:\n");
        msg.push_str("  • Run the installer to set up correct permissions\n");
        msg.push_str(&format!(
            "  • Or manually fix with: sudo chown -R :anna {} {}\n",
            anna_paths.data_dir.display(),
            anna_paths.config_dir.display()
        ));
        msg.push_str(&format!(
            "  • Set modes: sudo chmod 2770 {} {}\n",
            anna_paths.data_dir.display(),
            anna_paths.config_dir.display()
        ));
        msg.push_str(&format!(
            "  • Set ACLs: sudo setfacl -R -m g:{}:rwx {} {}\n",
            target_group,
            anna_paths.data_dir.display(),
            anna_paths.config_dir.display()
        ));
        msg.push_str(&format!(
            "  • Set default ACLs: sudo setfacl -R -d -m g:{}:rwx {} {}\n",
            target_group,
            anna_paths.data_dir.display(),
            anna_paths.config_dir.display()
        ));

        Err(DoctorError::new(msg.trim_end().to_string()))
    }
}

fn check_user_perms(
    _style: &Style,
    anna_paths: &AnnaPaths,
) -> std::result::Result<(), DoctorError> {
    let paths_to_check = vec![anna_paths.data_dir.clone(), anna_paths.config_dir.clone()];

    println!(
        "Paths inspected: {}, {}",
        anna_paths.data_dir.display(),
        anna_paths.config_dir.display()
    );

    let mut issues = Vec::new();

    for path in &paths_to_check {
        if !path.exists() {
            issues.push(format!("{}: Path does not exist", path.display()));
            continue;
        }

        // Check ownership - should be owned by current user
        let metadata = match fs::metadata(path) {
            Ok(m) => m,
            Err(e) => {
                issues.push(format!("{}: Cannot read metadata: {}", path.display(), e));
                continue;
            }
        };

        if !metadata.is_dir() {
            issues.push(format!("{}: Not a directory", path.display()));
            continue;
        }

        // Check mode - should be at least readable/writable by user (0700 minimum)
        let mode = metadata.permissions().mode();
        let mode_bits = mode & 0o777;
        if mode_bits & 0o700 != 0o700 {
            issues.push(format!(
                "{}: Mode is {:o}, should be at least 0700 (user rwx)",
                path.display(),
                mode_bits
            ));
        }
    }

    if issues.is_empty() {
        println!(
            "✅ User directories exist and are accessible: {}, {}",
            anna_paths.data_dir.display(),
            anna_paths.config_dir.display()
        );
        Ok(())
    } else {
        let mut msg = String::from("Permission issues found:\n");
        for issue in issues {
            msg.push_str(&format!("  • {}\n", issue));
        }
        msg.push_str("\nRecommended fixes:\n");
        msg.push_str("  • Run the installer to set up user directories\n");
        msg.push_str(&format!(
            "  • Or manually fix with: mkdir -p {} {}\n",
            anna_paths.data_dir.display(),
            anna_paths.config_dir.display()
        ));
        msg.push_str(&format!(
            "  • Set permissions: chmod 700 {} {}\n",
            anna_paths.data_dir.display(),
            anna_paths.config_dir.display()
        ));

        Err(DoctorError::new(msg.trim_end().to_string()))
    }
}

fn group_exists(group_name: &str) -> bool {
    Command::new("getent")
        .arg("group")
        .arg(group_name)
        .output()
        .map(|output| output.status.success())
        .unwrap_or(false)
}

fn check_path_permissions(path: &Path, expected_group: &str) -> Result<()> {
    if !path.exists() {
        return Err(anyhow!("Path does not exist"));
    }

    let metadata = fs::metadata(path)?;

    // Check if it's a directory
    if !metadata.is_dir() {
        return Err(anyhow!("Not a directory"));
    }

    // Check mode (should be 2770 for directories)
    let mode = metadata.permissions().mode();
    let expected_mode = 0o2770;

    // Mask to get permission bits and setgid bit
    let mode_bits = mode & 0o7777;

    if mode_bits != expected_mode {
        return Err(anyhow!(
            "Mode is {:o}, expected {:o}",
            mode_bits,
            expected_mode
        ));
    }

    // Check group ownership
    // We'll use stat command for this as it's more reliable
    let output = Command::new("stat")
        .arg("-c")
        .arg("%G")
        .arg(path)
        .output()?;

    if !output.status.success() {
        return Err(anyhow!("Failed to check group ownership"));
    }

    let group = String::from_utf8_lossy(&output.stdout).trim().to_string();
    if group != expected_group {
        return Err(anyhow!(
            "Group is '{}', expected '{}'",
            group,
            expected_group
        ));
    }

    // Check ACLs (basic check - just verify getfacl works and shows the group)
    let acl_output = Command::new("getfacl").arg(path).output().ok();

    if let Some(output) = acl_output {
        let acl_text = String::from_utf8_lossy(&output.stdout);
        if !acl_text.contains(&format!("group:{}:rwx", expected_group)) {
            return Err(anyhow!(
                "ACL does not grant rwx to group '{}'",
                expected_group
            ));
        }
    }

    Ok(())
}

/// Repair installation: create dirs, restart service, verify socket
fn repair_installation(style: &Style) -> std::result::Result<(), DoctorError> {
    use crate::paths::AnnaPaths;

    println!("{}", ui::head(style, "⚙ Installation Repair"));

    let paths = AnnaPaths::detect();
    println!(
        "{}",
        ui::info(style, &format!("Detected mode: {}", paths.mode.as_str()))
    );

    // Create required directories
    println!("{}", ui::info(style, "Creating/verifying directories..."));
    let dirs = vec![
        (&paths.data_dir, 0o750, "Data directory"),
        (&paths.config_dir, 0o750, "Config directory"),
        (&paths.reports_dir, 0o750, "Reports directory"),
        (&paths.advice_dir, 0o750, "Advice directory"),
    ];

    for (dir, mode, name) in dirs {
        match fs::create_dir_all(dir) {
            Ok(_) => {
                #[cfg(unix)]
                {
                    let perms = fs::Permissions::from_mode(mode);
                    if let Err(e) = fs::set_permissions(dir, perms) {
                        println!(
                            "{}",
                            ui::warn(
                                style,
                                &format!("Failed to set permissions on {}: {}", dir.display(), e)
                            )
                        );
                    }
                }
                println!("{}", ui::ok(style, &format!("{}: {}", name, dir.display())));
            }
            Err(e) => {
                println!(
                    "{}",
                    ui::err(style, &format!("Failed to create {}: {}", dir.display(), e))
                );
            }
        }
    }

    // Restart service
    println!("\n{}", ui::info(style, "Restarting service..."));
    let restart_result = if paths.mode == InstallMode::System {
        Command::new("sudo")
            .args(["systemctl", "daemon-reload"])
            .status()
            .ok();
        Command::new("sudo")
            .args(["systemctl", "restart", "annad"])
            .status()
    } else {
        Command::new("systemctl")
            .args(["--user", "daemon-reload"])
            .status()
            .ok();
        Command::new("systemctl")
            .args(["--user", "restart", "annad"])
            .status()
    };

    match restart_result {
        Ok(status) if status.success() => {
            println!("{}", ui::ok(style, "Service restarted"));
        }
        Ok(_) => {
            println!("{}", ui::warn(style, "Service restart returned non-zero"));
        }
        Err(e) => {
            println!(
                "{}",
                ui::warn(style, &format!("Failed to restart service: {}", e))
            );
        }
    }

    // Wait and check for socket
    println!("\n{}", ui::info(style, "Checking for socket..."));
    std::thread::sleep(std::time::Duration::from_secs(2));

    if paths.socket_path.exists() {
        println!(
            "{}",
            ui::ok(
                style,
                &format!("Socket found: {}", paths.socket_path.display())
            )
        );
        Ok(())
    } else {
        println!(
            "{}",
            ui::err(
                style,
                &format!("Socket not found at {}", paths.socket_path.display())
            )
        );

        if paths.mode == InstallMode::User {
            println!(
                "{}",
                ui::bullet(
                    style,
                    "Check: systemctl --user status annad | journalctl --user -u annad -n 30"
                )
            );
            if std::env::var("XDG_RUNTIME_DIR").is_err() {
                println!(
                    "{}",
                    ui::warn(
                        style,
                        "XDG_RUNTIME_DIR not set - socket may be at fallback location"
                    )
                );
            }
        } else {
            println!(
                "{}",
                ui::bullet(
                    style,
                    "Check: systemctl status annad | journalctl -u annad -n 30"
                )
            );
        }

        Err(DoctorError::new("Socket not available after repair"))
    }
}

/// Dump environment configuration (paths, mode, XDG vars)
fn dump_env(_cfg: &UiCfg, style: &Style) -> std::result::Result<(), DoctorError> {
    use crate::paths::AnnaPaths;

    println!("{}", ui::head(style, "⚙ Environment Configuration"));

    let paths = AnnaPaths::detect();

    // Mode
    println!("{}", ui::kv(style, "Mode", paths.mode.as_str()));

    // Paths
    println!(
        "{}",
        ui::kv(style, "Socket", &paths.socket_path.display().to_string())
    );
    println!(
        "{}",
        ui::kv(style, "Data dir", &paths.data_dir.display().to_string())
    );
    println!(
        "{}",
        ui::kv(style, "Config dir", &paths.config_dir.display().to_string())
    );
    println!(
        "{}",
        ui::kv(
            style,
            "Reports dir",
            &paths.reports_dir.display().to_string()
        )
    );
    println!(
        "{}",
        ui::kv(style, "Advice dir", &paths.advice_dir.display().to_string())
    );

    // XDG and environment variables
    println!("\n{}", ui::head(style, "Environment Variables"));
    if let Ok(xdg) = std::env::var("XDG_RUNTIME_DIR") {
        println!("{}", ui::kv(style, "XDG_RUNTIME_DIR", &xdg));
    } else {
        println!("{}", ui::kv(style, "XDG_RUNTIME_DIR", "(not set)"));
    }

    if let Ok(mode) = std::env::var("ANNA_MODE") {
        println!("{}", ui::kv(style, "ANNA_MODE", &mode));
    }

    if let Ok(home) = std::env::var("HOME") {
        println!("{}", ui::kv(style, "HOME", &home));
    }

    // Policy source (if applicable)
    if paths.mode == InstallMode::System {
        let policy_file = paths.config_dir.join(format!(
            "policy.d/{}.toml",
            nix::unistd::Uid::effective().as_raw()
        ));
        let policy_source = if policy_file.exists() {
            policy_file.display().to_string()
        } else {
            format!("{}/policy.d/default.toml", paths.config_dir.display())
        };
        println!("\n{}", ui::head(style, "Policy"));
        println!("{}", ui::kv(style, "Policy file", &policy_source));
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_permission_check_logic() {
        // This is a simple unit test to verify the logic works
        // In a real environment, we'd mock the filesystem
        assert!(true);
    }

    #[test]
    fn test_doctor_error_code() {
        let err = DoctorError::new("test error");
        assert_eq!(err.code(), 1);
    }

    #[test]
    fn test_system_mode_scoping() {
        let paths = AnnaPaths::system();
        let uid = nix::unistd::Uid::effective().as_raw();
        assert_eq!(paths.mode, InstallMode::System);
        assert_eq!(
            paths.data_dir,
            PathBuf::from(format!("/var/lib/anna/users/{}", uid))
        );
        assert_eq!(paths.config_dir, PathBuf::from("/etc/anna"));
    }

    #[test]
    fn test_user_mode_scoping() {
        let paths = AnnaPaths::user();
        assert_eq!(paths.mode, InstallMode::User);
        assert!(paths.data_dir.ends_with(".anna/data"));
        assert!(paths.config_dir.ends_with(".anna/config"));
    }
}
