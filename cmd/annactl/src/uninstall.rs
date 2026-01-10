use crate::ui::{Style, UiCfg};
use anyhow::{Context, Result};
use std::fs;
use std::io::Write;
use std::path::PathBuf;
use std::process::Command;

pub struct UninstallArgs {
    pub complete: bool,
    pub keep: bool,
}

/// Detect install mode based on what's actually installed
fn detect_install_mode() -> &'static str {
    // Check for system mode installation
    if PathBuf::from("/etc/systemd/system/annad.service").exists()
        || PathBuf::from("/var/lib/anna").exists()
    {
        return "system";
    }

    // Check for user mode installation
    if let Some(home) = dirs::home_dir() {
        if home.join(".config/systemd/user/annad.service").exists() || home.join(".anna").exists() {
            return "user";
        }
    }

    "unknown"
}

pub fn run(args: UninstallArgs, _cfg: &UiCfg, style: &Style) -> Result<()> {
    println!("{}", crate::ui::head(style, "Anna Uninstaller"));

    // Detect install mode based on what's actually installed
    let install_mode = detect_install_mode();

    if install_mode == "unknown" {
        println!(
            "{}",
            crate::ui::warn(style, "No Anna installation detected")
        );
        println!(
            "{}",
            crate::ui::info(
                style,
                "Checked system paths (/etc/systemd/system, /var/lib/anna)"
            )
        );
        println!(
            "{}",
            crate::ui::info(
                style,
                "Checked user paths (~/.config/systemd/user, ~/.anna)"
            )
        );
        return Ok(());
    }

    println!(
        "{}",
        crate::ui::info(
            style,
            &format!("Detected installation mode: {}", install_mode)
        )
    );

    // Set paths based on detected install mode
    let (annad, annactl, unit, data, conf, socket_dir) = if install_mode == "system" {
        (
            PathBuf::from("/usr/local/sbin/annad"),
            PathBuf::from("/usr/local/bin/annactl"),
            PathBuf::from("/etc/systemd/system/annad.service"),
            PathBuf::from("/var/lib/anna"),
            PathBuf::from("/etc/anna"),
            PathBuf::from("/run/anna"),
        )
    } else {
        let home = dirs::home_dir().context("Unable to determine home directory")?;
        (
            home.join(".local/bin/annad"),
            home.join(".local/bin/annactl"),
            home.join(".config/systemd/user/annad.service"),
            home.join(".anna/data"),
            home.join(".anna/config"),
            home.join(".anna/run"),
        )
    };

    // Log file
    let log_path = if install_mode == "system" {
        PathBuf::from("/tmp/anna_uninstall.log")
    } else {
        dirs::home_dir()
            .unwrap_or_default()
            .join(".anna/uninstall.log")
    };

    let mut log_file = fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(&log_path)
        .ok();

    let mut log = |msg: &str| {
        if let Some(ref mut file) = log_file {
            let _ = writeln!(file, "{}", msg);
        }
    };

    log(&format!(
        "=== Anna Uninstall {} ===",
        time::OffsetDateTime::now_utc()
    ));

    // Validate flags
    if args.complete && args.keep {
        return Err(anyhow::anyhow!(
            "Cannot use both --complete and --keep flags"
        ));
    }

    // Create backup unless --complete or --keep
    if !args.complete && !args.keep && (data.exists() || conf.exists()) {
        println!("{}", crate::ui::step(style, "Creating backup…"));

        let backup_dir = if install_mode == "system" {
            PathBuf::from("/root")
        } else {
            dirs::home_dir().unwrap_or_default().join("Documents")
        };

        fs::create_dir_all(&backup_dir).ok();

        let timestamp = time::OffsetDateTime::now_utc()
            .format(&time::format_description::well_known::Iso8601::DEFAULT)
            .unwrap_or_else(|_| "unknown".to_string())
            .replace(":", "-");

        let backup_path = backup_dir.join(format!("anna_backup_{}.tar.gz", timestamp));

        let mut tar_cmd = Command::new("tar");
        tar_cmd.arg("czf").arg(&backup_path);

        if data.exists() {
            tar_cmd.arg(&data);
        }
        if conf.exists() {
            tar_cmd.arg(&conf);
        }

        match tar_cmd.output() {
            Ok(output) if output.status.success() => {
                println!(
                    "{}",
                    crate::ui::ok(style, &format!("Backup saved to {}", backup_path.display()))
                );
                log(&format!("Backup created: {}", backup_path.display()));
            }
            Ok(_) | Err(_) => {
                println!(
                    "{}",
                    crate::ui::warn(style, "Backup failed, continuing anyway")
                );
                log("Backup failed");
            }
        }
    }

    // Stop and disable service
    println!(
        "{}",
        crate::ui::step(style, "Stopping and disabling service…")
    );

    // Stop service
    let stop_result = if install_mode == "system" {
        Command::new("sudo")
            .args(["systemctl", "stop", "annad"])
            .output()
    } else {
        Command::new("systemctl")
            .args(["--user", "stop", "annad"])
            .output()
    };

    match stop_result {
        Ok(output) if output.status.success() => {
            println!("{}", crate::ui::ok(style, "Service stopped"));
            log("Service stopped");
        }
        _ => {
            println!(
                "{}",
                crate::ui::note(style, "Service already stopped or not found")
            );
            log("Service stop: not found");
        }
    }

    // Disable service
    let disable_result = if install_mode == "system" {
        Command::new("sudo")
            .args(["systemctl", "disable", "annad"])
            .output()
    } else {
        Command::new("systemctl")
            .args(["--user", "disable", "annad"])
            .output()
    };

    match disable_result {
        Ok(output) if output.status.success() => {
            println!("{}", crate::ui::ok(style, "Service disabled"));
            log("Service disabled");
        }
        _ => {
            println!(
                "{}",
                crate::ui::note(style, "Service already disabled or not found")
            );
            log("Service disable: not found");
        }
    }

    // Remove unit file
    if unit.exists() {
        println!("{}", crate::ui::step(style, "Removing systemd unit…"));
        let success = if install_mode == "system" {
            Command::new("sudo")
                .arg("rm")
                .arg(&unit)
                .status()
                .map(|s| s.success())
                .unwrap_or(false)
        } else {
            fs::remove_file(&unit).is_ok()
        };

        if success {
            println!("{}", crate::ui::ok(style, "Unit file removed"));
            log(&format!("Removed: {}", unit.display()));
        } else {
            println!(
                "{}",
                crate::ui::note(
                    style,
                    &format!("Unit file already removed: {}", unit.display())
                )
            );
            log(&format!("Already removed: {}", unit.display()));
        }

        // Reload daemon
        if install_mode == "system" {
            Command::new("sudo")
                .args(["systemctl", "daemon-reload"])
                .output()
                .ok();
        } else {
            Command::new("systemctl")
                .args(["--user", "daemon-reload"])
                .output()
                .ok();
        }
    }

    // Remove binaries
    println!("{}", crate::ui::step(style, "Removing binaries…"));
    for bin in &[&annad, &annactl] {
        if bin.exists() {
            let success = if install_mode == "system" {
                Command::new("sudo")
                    .arg("rm")
                    .arg(bin)
                    .status()
                    .map(|s| s.success())
                    .unwrap_or(false)
            } else {
                fs::remove_file(bin).is_ok()
            };

            if success {
                println!(
                    "{}",
                    crate::ui::ok(style, &format!("Removed {}", bin.display()))
                );
                log(&format!("Removed: {}", bin.display()));
            } else {
                println!(
                    "{}",
                    crate::ui::note(style, &format!("Already removed: {}", bin.display()))
                );
            }
        } else {
            println!(
                "{}",
                crate::ui::note(style, &format!("Already removed: {}", bin.display()))
            );
            log(&format!("Already removed: {}", bin.display()));
        }
    }

    // Remove socket directory
    if socket_dir.exists() {
        println!("{}", crate::ui::step(style, "Removing runtime directory…"));
        let success = if install_mode == "system" {
            Command::new("sudo")
                .arg("rm")
                .arg("-rf")
                .arg(&socket_dir)
                .status()
                .map(|s| s.success())
                .unwrap_or(false)
        } else {
            fs::remove_dir_all(&socket_dir).is_ok()
        };

        if success {
            println!(
                "{}",
                crate::ui::ok(style, &format!("Removed {}", socket_dir.display()))
            );
            log(&format!("Removed: {}", socket_dir.display()));
        } else {
            println!(
                "{}",
                crate::ui::warn(style, &format!("Failed to remove {}", socket_dir.display()))
            );
            log(&format!("Failed to remove: {}", socket_dir.display()));
        }
    }

    // v0.0.909: Remove packages Anna installed
    if !args.keep {
        let installed = anna_shared::deps::read_installed_packages().unwrap_or_default();
        if !installed.is_empty() {
            println!(
                "{}",
                crate::ui::step(style, "Removing Anna-installed packages…")
            );
            log(&format!("Found {} Anna-installed packages", installed.len()));

            match anna_shared::deps::remove_installed_packages() {
                Ok(removed) if !removed.is_empty() => {
                    println!(
                        "{}",
                        crate::ui::ok(style, &format!("Removed {} packages: {}", removed.len(), removed.join(", ")))
                    );
                    log(&format!("Removed packages: {:?}", removed));
                }
                Ok(_) => {
                    println!(
                        "{}",
                        crate::ui::note(style, "No packages to remove (may have been removed manually)")
                    );
                }
                Err(e) => {
                    println!(
                        "{}",
                        crate::ui::warn(style, &format!("Failed to remove packages: {}", e))
                    );
                    log(&format!("Package removal failed: {}", e));
                }
            }
        }
    }

    // Remove data/config unless --keep
    if !args.keep {
        println!(
            "{}",
            crate::ui::step(style, "Removing data and configuration…")
        );

        for dir in &[&data, &conf] {
            if dir.exists() {
                let success = if install_mode == "system" {
                    Command::new("sudo")
                        .arg("rm")
                        .arg("-rf")
                        .arg(dir)
                        .status()
                        .map(|s| s.success())
                        .unwrap_or(false)
                } else {
                    fs::remove_dir_all(dir).is_ok()
                };

                if success {
                    println!(
                        "{}",
                        crate::ui::ok(style, &format!("Removed {}", dir.display()))
                    );
                    log(&format!("Removed: {}", dir.display()));
                } else {
                    println!(
                        "{}",
                        crate::ui::warn(style, &format!("Failed to remove {}", dir.display()))
                    );
                    log(&format!("Failed to remove: {}", dir.display()));
                }
            } else {
                println!(
                    "{}",
                    crate::ui::note(style, &format!("Already removed: {}", dir.display()))
                );
                log(&format!("Already removed: {}", dir.display()));
            }
        }
    } else {
        println!(
            "{}",
            crate::ui::info(style, "Keeping data directories (--keep flag)")
        );
        log("Data kept (--keep flag)");
    }

    // Note about anna group (system mode only)
    if install_mode == "system" {
        let group_check = Command::new("getent")
            .args(["group", "anna"])
            .output()
            .ok()
            .filter(|o| o.status.success());

        if group_check.is_some() {
            println!();
            println!(
                "{}",
                crate::ui::info(style, "Note: Group 'anna' still exists (not removed)")
            );
            println!(
                "{}",
                crate::ui::info(style, "  To remove: sudo groupdel anna")
            );
        }
    }

    println!();
    println!("{}", crate::ui::head(style, "✅ Anna uninstalled"));
    println!(
        "{}",
        crate::ui::info(style, &format!("Log saved to: {}", log_path.display()))
    );

    Ok(())
}
