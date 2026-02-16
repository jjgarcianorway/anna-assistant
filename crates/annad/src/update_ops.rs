//! Update operations - download, verify, and install binaries.
//!
//! Extracted from update.rs (v0.0.161) for modularization.
//! Contains the core operations for applying updates.

use anna_shared::GITHUB_REPO;
use anyhow::{anyhow, Result};
use std::path::Path;
use std::process::Command;
use tracing::{info, warn};

const ROLLBACK_DIR: &str = "/var/lib/anna/rollback";

/// Download a file from a URL to a local path
pub async fn download_file(url: &str, path: &Path) -> Result<()> {
    let client = reqwest::Client::builder()
        .user_agent("anna-assistant")
        .timeout(std::time::Duration::from_secs(300))
        .build()?;

    let response = client.get(url).send().await?;

    if !response.status().is_success() {
        return Err(anyhow!("Download failed: {} - {}", url, response.status()));
    }

    let bytes = response.bytes().await?;
    std::fs::write(path, &bytes)?;
    Ok(())
}

/// Verify a file's checksum against a SHA256SUMS file
pub fn verify_checksum(file_path: &Path, sums_path: &Path, name: &str) -> Result<()> {
    let sums_content = std::fs::read_to_string(sums_path)?;

    let expected = sums_content
        .lines()
        .find(|line| line.contains(name))
        .and_then(|line| line.split_whitespace().next())
        .ok_or_else(|| anyhow!("Checksum not found for {}", name))?;

    let output = Command::new("sha256sum").arg(file_path).output()?;

    let actual = String::from_utf8_lossy(&output.stdout);
    let actual = actual.split_whitespace().next().unwrap_or("");

    if actual != expected {
        return Err(anyhow!(
            "Checksum mismatch for {}: expected {}, got {}",
            name,
            expected,
            actual
        ));
    }

    Ok(())
}

/// Verify binary reports expected version
pub fn verify_binary_version(path: &Path, expected_version: &str, name: &str) -> Result<()> {
    let output = Command::new(path)
        .arg("--version")
        .output()
        .map_err(|e| anyhow!("{} --version failed: {}", name, e))?;

    let stdout = String::from_utf8_lossy(&output.stdout);
    if !stdout.contains(expected_version) {
        return Err(anyhow!(
            "{} version mismatch: expected {} in output, got: {}",
            name,
            expected_version,
            stdout.trim()
        ));
    }
    Ok(())
}

/// Get the binary directory from current executable
/// v0.3.11: Dynamic binary location instead of hardcoded /usr/local/bin
pub fn get_bin_dir() -> Result<std::path::PathBuf> {
    let exe = std::env::current_exe()
        .map_err(|e| anyhow!("Cannot determine binary location: {}", e))?;
    exe.parent()
        .map(|p| p.to_path_buf())
        .ok_or_else(|| anyhow!("Cannot determine binary directory"))
}

/// Install all binaries together (atomic triple update: annactl, annad, anna-executor)
pub fn install_binary_pair(annactl: &Path, annad: &Path, executor: &Path) -> Result<()> {
    let bin_dir = get_bin_dir()?;
    let annactl_dest = bin_dir.join("annactl");
    let annactl_new = bin_dir.join("annactl.new");
    let annad_new = bin_dir.join("annad.new");
    let executor_new = bin_dir.join("anna-executor.new");

    // Stage all binaries first
    std::fs::copy(annactl, &annactl_new)
        .map_err(|e| anyhow!("Failed to stage annactl: {}", e))?;
    std::fs::copy(annad, &annad_new)
        .map_err(|e| anyhow!("Failed to stage annad: {}", e))?;
    std::fs::copy(executor, &executor_new)
        .map_err(|e| anyhow!("Failed to stage anna-executor: {}", e))?;

    // Atomic rename for annactl — works even if binary is running
    std::fs::rename(&annactl_new, &annactl_dest)
        .map_err(|e| anyhow!("Failed to install annactl: {}", e))?;

    // annad.new and anna-executor.new stay staged — renamed during restart
    Ok(())
}

/// Verify both installed binaries report the same version
/// v0.3.11: Use dynamic binary location
pub fn verify_pair_consistency(expected_version: &str) -> Result<()> {
    let bin_dir = get_bin_dir()?;
    let annactl_path = bin_dir.join("annactl");
    let annad_new_path = bin_dir.join("annad.new");

    let annactl_output = Command::new(&annactl_path)
        .arg("--version")
        .output()
        .map_err(|e| anyhow!("annactl --version failed: {}", e))?;

    let annactl_ver = String::from_utf8_lossy(&annactl_output.stdout);
    if !annactl_ver.contains(expected_version) {
        return Err(anyhow!(
            "annactl version check failed: {}",
            annactl_ver.trim()
        ));
    }

    // annad.new should also have correct version
    let annad_output = Command::new(&annad_new_path)
        .arg("--version")
        .output()
        .map_err(|e| anyhow!("annad.new --version failed: {}", e))?;

    let annad_ver = String::from_utf8_lossy(&annad_output.stdout);
    if !annad_ver.contains(expected_version) {
        return Err(anyhow!("annad version check failed: {}", annad_ver.trim()));
    }

    info!(
        "Pair consistency verified: both binaries at {}",
        expected_version
    );
    Ok(())
}

/// Save the current three binaries to the persistent rollback slot.
/// Called before any install so a crash mid-update can be recovered.
pub fn save_rollback_slot(bin_dir: &Path) {
    if let Err(e) = std::fs::create_dir_all(ROLLBACK_DIR) {
        warn!("Could not create rollback dir: {}", e);
        return;
    }
    for name in &["annactl", "annad", "anna-executor"] {
        let src = bin_dir.join(name);
        let dst = std::path::Path::new(ROLLBACK_DIR).join(name);
        if src.exists() {
            if let Err(e) = std::fs::copy(&src, &dst) {
                warn!("Could not save rollback copy of {}: {}", name, e);
            }
        }
    }
    info!("Rollback slot saved to {}", ROLLBACK_DIR);
}

/// Restore binaries from the persistent rollback slot.
/// Returns true if at least one binary was restored.
pub fn restore_rollback_slot(bin_dir: &Path) -> bool {
    let mut restored = false;
    for name in &["annactl", "annad", "anna-executor"] {
        let src = std::path::Path::new(ROLLBACK_DIR).join(name);
        let dst = bin_dir.join(name);
        if src.exists() {
            match std::fs::copy(&src, &dst) {
                Ok(_) => {
                    info!("Restored {} from rollback slot", name);
                    restored = true;
                }
                Err(e) => warn!("Could not restore {}: {}", name, e),
            }
        }
    }
    restored
}

/// Schedule daemon restart after update.
/// Moves both staged binaries (.new) into place, restarts services,
/// and polls annad --version to verify the new version is running.
/// On failure, restores the rollback slot and logs via syslog.
pub fn schedule_daemon_restart(new_version: &str) -> Result<()> {
    let bin_dir = get_bin_dir()?;

    let script = format!(
        r#"#!/bin/bash
BIN="{bin}"
VER="{ver}"
RB="{rb}"

# Move staged binaries into place
mv "$BIN/annad.new" "$BIN/annad" 2>/dev/null || true
mv "$BIN/anna-executor.new" "$BIN/anna-executor" 2>/dev/null || true

# Restart executor first, then annad
systemctl restart anna-executor 2>/dev/null || true
systemctl restart annad 2>/dev/null || true

# Poll annad --version up to 5 times (2s each)
for i in 1 2 3 4 5; do
    sleep 2
    if "$BIN/annad" --version 2>/dev/null | grep -qF "$VER"; then
        exit 0
    fi
done

# Self-check failed — restore rollback slot
logger -t anna-update "WARN: annad $VER self-check failed, restoring rollback"
for f in annactl annad anna-executor; do
    [ -f "$RB/$f" ] && cp "$RB/$f" "$BIN/$f"
done
systemctl restart anna-executor 2>/dev/null || true
systemctl restart annad 2>/dev/null || true
logger -t anna-update "WARN: rollback to previous version complete"
"#,
        bin = bin_dir.display(),
        ver = new_version,
        rb = ROLLBACK_DIR,
    );

    let script_path = format!("/tmp/anna-restart-{}.sh", std::process::id());
    let _ = std::fs::remove_file(&script_path);
    std::fs::write(&script_path, &script)?;
    std::fs::set_permissions(
        &script_path,
        std::os::unix::fs::PermissionsExt::from_mode(0o755),
    )?;

    Command::new("bash")
        .args(["-c", &format!("sleep 1 && {} && rm -f {} &", script_path, script_path)])
        .spawn()?;

    Ok(())
}

/// Verify that release assets exist before reporting version as available
pub async fn verify_assets_exist(client: &reqwest::Client, version: &str) -> Result<()> {
    let arch = std::env::consts::ARCH;
    let arch_name = match arch {
        "x86_64" => "x86_64",
        "aarch64" => "aarch64",
        _ => return Err(anyhow!("Unsupported architecture: {}", arch)),
    };

    let base_url = format!(
        "https://github.com/{}/releases/download/v{}",
        GITHUB_REPO, version
    );

    // Check that all required assets exist via HEAD requests
    let assets = [
        format!("{}/annactl-linux-{}", base_url, arch_name),
        format!("{}/annad-linux-{}", base_url, arch_name),
        format!("{}/anna-executor-linux-{}", base_url, arch_name),
        format!("{}/SHA256SUMS", base_url),
        format!("{}/SHA256SUMS.asc", base_url),
    ];

    for asset_url in &assets {
        let response = client.head(asset_url).send().await?;
        if !response.status().is_success() {
            return Err(anyhow!(
                "Release {} missing asset: {} ({})",
                version,
                asset_url,
                response.status()
            ));
        }
    }

    Ok(())
}

/// Rollback installed binaries from backups
pub fn rollback_binaries(backup_annactl: &Path, backup_annad: &Path, backup_executor: &Path) {
    let bin_dir = match get_bin_dir() {
        Ok(dir) => dir,
        Err(_) => return,
    };

    if backup_annactl.exists() {
        std::fs::copy(backup_annactl, bin_dir.join("annactl")).ok();
    }
    if backup_annad.exists() {
        std::fs::copy(backup_annad, bin_dir.join("annad")).ok();
    }
    if backup_executor.exists() {
        std::fs::copy(backup_executor, bin_dir.join("anna-executor")).ok();
    }
}

/// Ensure the annad.service unit has PATH= set.
/// Auto-update replaces binaries but not the service file, so older installs
/// may be missing PATH= and fail to find system commands like pacman/ollama.
pub fn patch_service_unit_path() {
    const SERVICE: &str = "/etc/systemd/system/annad.service";
    const PATH_LINE: &str =
        "Environment=PATH=/usr/local/sbin:/usr/local/bin:/usr/sbin:/usr/bin:/sbin:/bin";

    let content = match std::fs::read_to_string(SERVICE) {
        Ok(c) => c,
        Err(_) => return,
    };

    if content.contains(PATH_LINE) {
        return; // already patched
    }

    // Insert PATH= after [Service] line
    let patched = content.replace(
        "[Service]\n",
        &format!("[Service]\n{}\n", PATH_LINE),
    );

    if std::fs::write(SERVICE, &patched).is_ok() {
        // Reload systemd so the change takes effect on next restart
        let _ = Command::new("/usr/bin/systemctl")
            .args(["daemon-reload"])
            .output();
        info!("Patched annad.service to add PATH=");
    }
}

/// Get the architecture-specific binary name suffix
pub fn get_arch_name() -> Result<&'static str> {
    let arch = std::env::consts::ARCH;
    match arch {
        "x86_64" => Ok("x86_64"),
        "aarch64" => Ok("aarch64"),
        _ => Err(anyhow!("Unsupported architecture: {}", arch)),
    }
}
