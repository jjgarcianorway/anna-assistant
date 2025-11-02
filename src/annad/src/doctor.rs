// Anna v0.10.1 - Doctor System (--doctor-apply mode)
//
// System setup operations run by installer with sudo:
// - Create anna user/group
// - Create directories with correct permissions
// - Install systemd unit with resource limits
// - Install packages (Arch Linux only)
// - Reload systemd

use anyhow::{Context, Result};
use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::path::Path;
use std::process::Command;
use tracing::{info, warn};

pub struct DoctorApply {
    verbose: bool,
}

impl DoctorApply {
    pub fn new(verbose: bool) -> Self {
        Self { verbose }
    }

    /// Run all doctor apply operations
    pub fn apply(&self) -> Result<()> {
        info!("Doctor: Applying system configuration...");

        self.create_user_group()?;
        self.create_directories()?;
        self.install_systemd_unit()?;
        self.install_packages()?;
        self.reload_systemd()?;

        info!("Doctor: System configuration complete");
        Ok(())
    }

    fn create_user_group(&self) -> Result<()> {
        self.log("Creating anna system user and group...");

        // Check if user exists
        let check = Command::new("id")
            .arg("anna")
            .stdout(std::process::Stdio::null())
            .stderr(std::process::Stdio::null())
            .status();

        if check.is_ok() && check.unwrap().success() {
            self.log("  User 'anna' already exists");
            return Ok(());
        }

        // Create system user
        let status = Command::new("useradd")
            .args(&[
                "-r", // System user
                "-s",
                "/usr/bin/nologin", // No login shell
                "-d",
                "/var/lib/anna", // Home directory
                "-c",
                "Anna Assistant", // Comment
                "anna",
            ])
            .status()
            .context("Failed to create anna user")?;

        if !status.success() {
            anyhow::bail!("useradd failed with status {}", status);
        }

        self.log("  Created user 'anna'");
        Ok(())
    }

    fn create_directories(&self) -> Result<()> {
        self.log("Creating directories...");

        let dirs = vec![
            ("/var/lib/anna", 0o700, "State directory"),
            ("/var/log/anna", 0o750, "Log directory"),
            ("/run/anna", 0o750, "Runtime directory"),
            ("/etc/anna", 0o755, "Configuration directory"),
            ("/usr/lib/anna", 0o755, "System library directory"),
        ];

        for (path, mode, description) in dirs {
            if !Path::new(path).exists() {
                fs::create_dir_all(path).with_context(|| format!("Failed to create {}", path))?;
                self.log(&format!("  Created {}: {}", description, path));
            }

            // Set ownership to anna:anna
            let status = Command::new("chown")
                .args(&["-R", "anna:anna", path])
                .status()
                .with_context(|| format!("Failed to chown {}", path))?;

            if !status.success() {
                warn!("chown failed for {} (non-fatal)", path);
            }

            // Set permissions
            fs::set_permissions(path, fs::Permissions::from_mode(mode))
                .with_context(|| format!("Failed to set permissions on {}", path))?;
        }

        Ok(())
    }

    fn install_systemd_unit(&self) -> Result<()> {
        self.log("Installing systemd unit...");

        let unit_content = r#"[Unit]
Description=Anna Assistant Daemon
Documentation=https://github.com/yourusername/anna-assistant
After=network.target

[Service]
Type=simple
User=anna
Group=anna
RuntimeDirectory=anna
RuntimeDirectoryMode=0750

# Resource Limits
MemoryMax=80M
CPUQuota=5%
TasksMax=32

# Security
NoNewPrivileges=true
PrivateTmp=true
ProtectSystem=strict
ProtectHome=true
ReadWritePaths=/var/lib/anna /var/log/anna /run/anna

# Execution
ExecStart=/usr/local/bin/annad
Restart=on-failure
RestartSec=10
StandardOutput=journal
StandardError=journal

[Install]
WantedBy=multi-user.target
"#;

        let unit_path = "/etc/systemd/system/annad.service";
        fs::write(unit_path, unit_content).context("Failed to write systemd unit")?;

        self.log(&format!("  Installed {}", unit_path));
        Ok(())
    }

    fn install_packages(&self) -> Result<()> {
        self.log("Checking required packages...");

        // Detect package manager
        let is_arch = Path::new("/etc/arch-release").exists();

        if !is_arch {
            warn!("  Not Arch Linux - skipping auto package install");
            warn!("  Please manually install: lm_sensors iproute2 ethtool smartmontools sqlite");
            return Ok(());
        }

        // Check if pacman is available
        let has_pacman = Command::new("which")
            .arg("pacman")
            .stdout(std::process::Stdio::null())
            .status()
            .map(|s| s.success())
            .unwrap_or(false);

        if !has_pacman {
            warn!("  pacman not found - skipping package install");
            return Ok(());
        }

        let packages = vec![
            "lm_sensors",
            "iproute2",
            "ethtool",
            "smartmontools",
            "sqlite",
            "procps-ng",
            "util-linux",
        ];

        for pkg in packages {
            // Check if already installed
            let check = Command::new("pacman")
                .args(&["-Q", pkg])
                .stdout(std::process::Stdio::null())
                .stderr(std::process::Stdio::null())
                .status();

            if check.is_ok() && check.unwrap().success() {
                self.log(&format!("  {} already installed", pkg));
                continue;
            }

            // Install package
            self.log(&format!("  Installing {}...", pkg));
            let status = Command::new("pacman")
                .args(&["-S", "--noconfirm", "--needed", pkg])
                .status();

            match status {
                Ok(s) if s.success() => {
                    self.log(&format!("    {} installed", pkg));
                }
                _ => {
                    warn!("Failed to install {} (non-fatal)", pkg);
                }
            }
        }

        Ok(())
    }

    fn reload_systemd(&self) -> Result<()> {
        self.log("Reloading systemd...");

        let status = Command::new("systemctl")
            .arg("daemon-reload")
            .status()
            .context("Failed to reload systemd")?;

        if !status.success() {
            anyhow::bail!("systemctl daemon-reload failed");
        }

        self.log("  Systemd reloaded");
        Ok(())
    }

    fn log(&self, msg: &str) {
        if self.verbose {
            println!("{}", msg);
        }
        info!("{}", msg);
    }
}
