//! Update operations - download, verify, and install binaries.
//!
//! Extracted from update.rs (v0.0.161) for modularization.
//! Contains the core operations for applying updates.

use anna_shared::GITHUB_REPO;
use anyhow::{anyhow, Result};
use std::path::Path;
use std::process::Command;
use tracing::info;

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

/// Install both binaries together (atomic pair update)
/// v0.0.318: Better error messages when binary is in use
/// v0.0.387: Use staging + atomic rename to update even while binaries are running
pub fn install_binary_pair(annactl: &Path, annad: &Path) -> Result<()> {
    // Stage both binaries first (copy to .new files)
    std::fs::copy(annactl, "/usr/local/bin/annactl.new")
        .map_err(|e| anyhow!("Failed to stage annactl: {}", e))?;

    std::fs::copy(annad, "/usr/local/bin/annad.new")
        .map_err(|e| anyhow!("Failed to stage annad: {}", e))?;

    // Atomic rename for annactl - works even if binary is running
    // rename() is atomic and works on busy executables (unlike copy)
    std::fs::rename("/usr/local/bin/annactl.new", "/usr/local/bin/annactl")
        .map_err(|e| anyhow!("Failed to install annactl: {}", e))?;

    // annad.new stays staged - will be renamed during restart
    Ok(())
}

/// Verify both installed binaries report the same version
pub fn verify_pair_consistency(expected_version: &str) -> Result<()> {
    let annactl_output = Command::new("/usr/local/bin/annactl")
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
    let annad_output = Command::new("/usr/local/bin/annad.new")
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

/// Schedule daemon restart after update
pub fn schedule_daemon_restart() -> Result<()> {
    // Move new binary into place and restart
    // This is done via a short shell script to ensure atomic replacement
    let script = r#"
#!/bin/bash
mv /usr/local/bin/annad.new /usr/local/bin/annad
systemctl restart annad
"#;

    let script_path = "/tmp/anna-restart.sh";
    std::fs::write(script_path, script)?;
    std::fs::set_permissions(
        script_path,
        std::os::unix::fs::PermissionsExt::from_mode(0o755),
    )?;

    // Run in background so current process can exit cleanly
    Command::new("bash")
        .args(["-c", &format!("sleep 1 && {} &", script_path)])
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
        format!("{}/SHA256SUMS", base_url),
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
pub fn rollback_binaries(backup_annactl: &Path, backup_annad: &Path) {
    if backup_annactl.exists() {
        std::fs::copy(backup_annactl, "/usr/local/bin/annactl").ok();
    }
    if backup_annad.exists() {
        std::fs::copy(backup_annad, "/usr/local/bin/annad").ok();
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
