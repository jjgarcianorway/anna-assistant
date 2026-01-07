use crate::ui::{Style, UiCfg};
use anyhow::{Context, Result};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

pub enum UpdateCommand {
    Check,
    Apply,
    Rollback,
    Policy,
}

pub struct UpdateArgs<'a> {
    pub command: UpdateCommand,
    pub force: bool,
    pub mode: Option<&'a str>,
}

const CURRENT_VERSION: &str = env!("CARGO_PKG_VERSION");

pub fn run(args: UpdateArgs, _cfg: &UiCfg, style: &Style) -> Result<()> {
    match args.command {
        UpdateCommand::Check => check_for_updates(style),
        UpdateCommand::Apply => apply_update(args.force, style),
        UpdateCommand::Rollback => rollback_update(style),
        UpdateCommand::Policy => manage_policy(args.mode, style),
    }
}

fn check_for_updates(style: &Style) -> Result<()> {
    println!("{}", crate::ui::head(style, "Anna Update Check"));
    println!("{}", crate::ui::step(style, "Checking for updates…"));

    let local_version = CURRENT_VERSION;
    println!(
        "{}",
        crate::ui::info(style, &format!("Local version: v{}", local_version))
    );

    // Check if we're in a git repository
    let remote_version = get_remote_version()?;

    if let Some(remote) = remote_version {
        println!(
            "{}",
            crate::ui::info(style, &format!("Remote version: v{}", remote))
        );

        if remote != local_version {
            println!(
                "{}",
                crate::ui::warn(style, &format!("New version available: v{}", remote))
            );
            println!(
                "{}",
                crate::ui::info(style, "Run 'annactl update apply' to upgrade.")
            );
        } else {
            println!(
                "{}",
                crate::ui::ok(
                    style,
                    &format!("You're on the latest version (v{})", local_version)
                )
            );
        }
    } else {
        println!(
            "{}",
            crate::ui::ok(style, &format!("Current version: v{}", local_version))
        );
        println!(
            "{}",
            crate::ui::note(
                style,
                "Not in a git repository — cannot check for remote updates"
            )
        );
    }

    Ok(())
}

fn get_remote_version() -> Result<Option<String>> {
    // Check if we're in a git repository
    let git_check = Command::new("git")
        .args(["rev-parse", "--git-dir"])
        .output();

    if git_check.is_err() || !git_check.unwrap().status.success() {
        return Ok(None);
    }

    // Fetch latest tags
    let _ = Command::new("git")
        .args(["fetch", "--tags", "--quiet"])
        .output();

    // Try to read version from remote Cargo.toml
    let output = Command::new("git")
        .args(["show", "origin/main:cmd/annactl/Cargo.toml"])
        .output();

    if let Ok(output) = output {
        if output.status.success() {
            let content = String::from_utf8_lossy(&output.stdout);
            for line in content.lines() {
                if line.starts_with("version = ") {
                    let version = line.split('"').nth(1).map(|s| s.to_string());
                    return Ok(version);
                }
            }
        }
    }

    Ok(None)
}

fn apply_update(force: bool, style: &Style) -> Result<()> {
    println!("{}", crate::ui::head(style, "Anna Update Apply"));

    // Environment checks
    if !force && !check_environment(style)? {
        return Ok(());
    }

    // Detect install mode
    let is_root = unsafe { libc::geteuid() } == 0;
    if !is_root && !force {
        println!(
            "{}",
            crate::ui::warn(style, "Auto-update disabled (requires system privileges)")
        );
        println!(
            "{}",
            crate::ui::info(style, "Use --force to update user-space installation")
        );
        return Ok(());
    }

    println!("{}", crate::ui::step(style, "Fetching latest changes…"));
    let fetch = Command::new("git")
        .args(["pull", "--ff-only"])
        .output()
        .context("Failed to fetch updates")?;

    if !fetch.status.success() {
        println!(
            "{}",
            crate::ui::err(style, "Failed to fetch updates from git")
        );
        return Ok(());
    }

    println!("{}", crate::ui::step(style, "Compiling update…"));
    let build = Command::new("cargo")
        .args(["build", "--release", "--quiet"])
        .output()
        .context("Failed to build update")?;

    if !build.status.success() {
        println!("{}", crate::ui::err(style, "Build failed"));
        return Ok(());
    }

    // Backup current binary
    println!("{}", crate::ui::step(style, "Creating backup…"));
    backup_binary(style)?;

    // Install new binary
    println!("{}", crate::ui::step(style, "Installing update…"));
    install_binary(style)?;

    println!("{}", crate::ui::ok(style, "✅ Update applied successfully"));
    println!(
        "{}",
        crate::ui::info(
            style,
            "Restart annad service to complete: systemctl restart annad"
        )
    );

    Ok(())
}

fn rollback_update(style: &Style) -> Result<()> {
    println!("{}", crate::ui::head(style, "Anna Update Rollback"));

    let is_root = unsafe { libc::geteuid() } == 0;
    let backup_path = if is_root {
        PathBuf::from("/usr/local/bin/annactl.prev")
    } else {
        dirs::home_dir()
            .unwrap_or_default()
            .join(".local/bin/annactl.prev")
    };

    if !backup_path.exists() {
        println!(
            "{}",
            crate::ui::warn(style, "No backup found — cannot rollback")
        );
        return Ok(());
    }

    let target_path = if is_root {
        PathBuf::from("/usr/local/bin/annactl")
    } else {
        dirs::home_dir()
            .unwrap_or_default()
            .join(".local/bin/annactl")
    };

    println!("{}", crate::ui::step(style, "Restoring previous version…"));

    if is_root {
        Command::new("sudo")
            .args([
                "cp",
                backup_path.to_str().unwrap(),
                target_path.to_str().unwrap(),
            ])
            .status()
            .context("Failed to restore backup")?;
    } else {
        fs::copy(&backup_path, &target_path).context("Failed to restore backup")?;
    }

    println!("{}", crate::ui::ok(style, "Rollback complete"));

    Ok(())
}

fn manage_policy(mode: Option<&str>, style: &Style) -> Result<()> {
    println!("{}", crate::ui::head(style, "Anna Update Policy"));

    let config_path = if unsafe { libc::geteuid() } == 0 {
        PathBuf::from("/etc/anna/config.toml")
    } else {
        dirs::home_dir()
            .unwrap_or_default()
            .join(".anna/config.toml")
    };

    if let Some(new_mode) = mode {
        if !["auto", "manual", "notify"].contains(&new_mode) {
            println!(
                "{}",
                crate::ui::err(style, "Invalid mode. Use: auto, manual, or notify")
            );
            return Ok(());
        }

        println!(
            "{}",
            crate::ui::step(style, &format!("Setting update policy to '{}'", new_mode))
        );

        // Read existing config or create new
        let mut config_content = fs::read_to_string(&config_path).unwrap_or_default();
        if config_content.is_empty() {
            config_content = format!("[update]\npolicy = \"{}\"\n", new_mode);
        } else if config_content.contains("[update]") {
            // Update existing policy
            config_content = config_content
                .lines()
                .map(|line| {
                    if line.trim().starts_with("policy =") {
                        format!("policy = \"{}\"", new_mode)
                    } else {
                        line.to_string()
                    }
                })
                .collect::<Vec<_>>()
                .join("\n");
        } else {
            config_content.push_str(&format!("\n[update]\npolicy = \"{}\"\n", new_mode));
        }

        fs::create_dir_all(config_path.parent().unwrap())?;
        fs::write(&config_path, config_content)?;

        println!(
            "{}",
            crate::ui::ok(style, &format!("Update policy set to '{}'", new_mode))
        );
    } else {
        // Show current policy
        let policy = read_update_policy(&config_path);
        println!(
            "{}",
            crate::ui::info(style, &format!("Current policy: {}", policy))
        );
        println!("{}", crate::ui::note(style, "Available policies:"));
        println!(
            "{}",
            crate::ui::bullet(style, "auto - automatic background updates")
        );
        println!(
            "{}",
            crate::ui::bullet(style, "manual - check only, never auto-update")
        );
        println!(
            "{}",
            crate::ui::bullet(style, "notify - inform user but wait for confirmation")
        );
    }

    Ok(())
}

fn read_update_policy(config_path: &Path) -> String {
    if let Ok(content) = fs::read_to_string(config_path) {
        for line in content.lines() {
            if line.trim().starts_with("policy =") {
                return line.split('"').nth(1).unwrap_or("manual").to_string();
            }
        }
    }
    "manual".to_string()
}

fn check_environment(style: &Style) -> Result<bool> {
    println!("{}", crate::ui::step(style, "Checking environment…"));

    // Check disk space
    let df_output = Command::new("df")
        .args(["-BM", "/"])
        .output()
        .context("Failed to check disk space")?;

    if df_output.status.success() {
        let output = String::from_utf8_lossy(&df_output.stdout);
        if let Some(line) = output.lines().nth(1) {
            if let Some(avail) = line.split_whitespace().nth(3) {
                let space_mb: u64 = avail.trim_end_matches('M').parse().unwrap_or(0);
                if space_mb < 500 {
                    println!(
                        "{}",
                        crate::ui::warn(
                            style,
                            &format!("Low disk space: {}MB (need at least 500MB)", space_mb)
                        )
                    );
                    return Ok(false);
                }
            }
        }
    }

    // Check load average
    if let Ok(load) = fs::read_to_string("/proc/loadavg") {
        if let Some(load_str) = load.split_whitespace().next() {
            if let Ok(load_val) = load_str.parse::<f32>() {
                if load_val > 1.5 {
                    println!(
                        "{}",
                        crate::ui::warn(style, &format!("High system load: {:.2}", load_val))
                    );
                    return Ok(false);
                }
            }
        }
    }

    println!("{}", crate::ui::ok(style, "Environment checks passed"));
    Ok(true)
}

fn backup_binary(style: &Style) -> Result<()> {
    let is_root = unsafe { libc::geteuid() } == 0;
    let source = if is_root {
        PathBuf::from("/usr/local/bin/annactl")
    } else {
        dirs::home_dir()
            .unwrap_or_default()
            .join(".local/bin/annactl")
    };

    let backup = if is_root {
        PathBuf::from("/usr/local/bin/annactl.prev")
    } else {
        dirs::home_dir()
            .unwrap_or_default()
            .join(".local/bin/annactl.prev")
    };

    if source.exists() {
        if is_root {
            Command::new("sudo")
                .args(["cp", source.to_str().unwrap(), backup.to_str().unwrap()])
                .status()
                .context("Failed to create backup")?;
        } else {
            fs::copy(&source, &backup).context("Failed to create backup")?;
        }
        println!(
            "{}",
            crate::ui::ok(style, &format!("Backup saved to {}", backup.display()))
        );
    }

    Ok(())
}

fn install_binary(style: &Style) -> Result<()> {
    let is_root = unsafe { libc::geteuid() } == 0;
    let source = PathBuf::from("target/release/annactl");
    let target = if is_root {
        PathBuf::from("/usr/local/bin/annactl")
    } else {
        dirs::home_dir()
            .unwrap_or_default()
            .join(".local/bin/annactl")
    };

    if is_root {
        Command::new("sudo")
            .args([
                "install",
                "-m755",
                source.to_str().unwrap(),
                target.to_str().unwrap(),
            ])
            .status()
            .context("Failed to install binary")?;
    } else {
        fs::create_dir_all(target.parent().unwrap())?;
        fs::copy(&source, &target).context("Failed to install binary")?;
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let mut perms = fs::metadata(&target)?.permissions();
            perms.set_mode(0o755);
            fs::set_permissions(&target, perms)?;
        }
    }

    println!(
        "{}",
        crate::ui::ok(style, &format!("Installed to {}", target.display()))
    );

    Ok(())
}
