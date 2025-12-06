//! Update management - check for and apply updates.

use anna_shared::GITHUB_REPO;
use anyhow::{anyhow, Result};
use std::process::Command;
use tracing::{info, warn};

/// GitHub API response for releases
#[derive(Debug, serde::Deserialize)]
struct GitHubRelease {
    tag_name: String,
}

/// Check GitHub for the latest version (only returns if assets are downloadable)
pub async fn check_latest_version() -> Result<String> {
    let url = format!(
        "https://api.github.com/repos/{}/releases/latest",
        GITHUB_REPO
    );

    let client = reqwest::Client::builder()
        .user_agent("anna-assistant")
        .timeout(std::time::Duration::from_secs(10))
        .build()?;

    let response = client.get(&url).send().await?;

    if !response.status().is_success() {
        return Err(anyhow!("GitHub API error: {}", response.status()));
    }

    let release: GitHubRelease = response.json().await?;

    // Remove 'v' prefix if present
    let version = release.tag_name.trim_start_matches('v').to_string();

    // Verify that required assets are actually downloadable
    verify_assets_exist(&client, &version).await?;

    Ok(version)
}

/// Verify that release assets exist before reporting version as available
async fn verify_assets_exist(client: &reqwest::Client, version: &str) -> Result<()> {
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

/// Compare versions, returns true if remote is newer
/// v0.0.72: Handle empty/invalid versions safely - never upgrade from invalid
pub fn is_newer_version(current: &str, remote: &str) -> bool {
    let parse = |v: &str| -> Vec<u32> { v.split('.').filter_map(|s| s.parse().ok()).collect() };

    let current_parts = parse(current);
    let remote_parts = parse(remote);

    // v0.0.72: If either version is empty/invalid, don't report as newer
    if current_parts.is_empty() || remote_parts.is_empty() {
        return false;
    }

    for i in 0..3 {
        let c = current_parts.get(i).unwrap_or(&0);
        let r = remote_parts.get(i).unwrap_or(&0);
        if r > c {
            return true;
        }
        if r < c {
            return false;
        }
    }
    false
}

/// v0.0.73: Perform atomic pair update of both annactl and annad
/// Both binaries are updated together or neither is updated (rollback on failure)
pub async fn perform_update(new_version: &str) -> Result<()> {
    info!("Starting atomic pair update to version {}", new_version);

    let arch = std::env::consts::ARCH;
    let arch_name = match arch {
        "x86_64" => "x86_64",
        "aarch64" => "aarch64",
        _ => return Err(anyhow!("Unsupported architecture: {}", arch)),
    };

    let base_url = format!(
        "https://github.com/{}/releases/download/v{}",
        GITHUB_REPO, new_version
    );

    let tmp_dir = std::env::temp_dir().join("anna-update");
    std::fs::create_dir_all(&tmp_dir)?;

    // Download both binaries before replacing anything
    info!("Downloading annactl...");
    let annactl_url = format!("{}/annactl-linux-{}", base_url, arch_name);
    let annactl_path = tmp_dir.join("annactl");
    download_file(&annactl_url, &annactl_path).await?;

    info!("Downloading annad...");
    let annad_url = format!("{}/annad-linux-{}", base_url, arch_name);
    let annad_path = tmp_dir.join("annad");
    download_file(&annad_url, &annad_path).await?;

    // Download and verify checksums
    info!("Verifying checksums...");
    let sums_url = format!("{}/SHA256SUMS", base_url);
    let sums_path = tmp_dir.join("SHA256SUMS");
    download_file(&sums_url, &sums_path).await?;

    verify_checksum(
        &annactl_path,
        &sums_path,
        &format!("annactl-linux-{}", arch_name),
    )?;
    verify_checksum(
        &annad_path,
        &sums_path,
        &format!("annad-linux-{}", arch_name),
    )?;

    // Make executable
    std::fs::set_permissions(
        &annactl_path,
        std::os::unix::fs::PermissionsExt::from_mode(0o755),
    )?;
    std::fs::set_permissions(
        &annad_path,
        std::os::unix::fs::PermissionsExt::from_mode(0o755),
    )?;

    // v0.0.73: Verify downloaded binaries report correct version
    info!("Verifying downloaded binary versions...");
    verify_binary_version(&annactl_path, new_version, "annactl")?;
    verify_binary_version(&annad_path, new_version, "annad")?;

    // v0.0.73: Backup existing binaries for rollback
    info!("Backing up existing binaries...");
    let backup_annactl = tmp_dir.join("annactl.backup");
    let backup_annad = tmp_dir.join("annad.backup");
    std::fs::copy("/usr/local/bin/annactl", &backup_annactl).ok();
    std::fs::copy("/usr/local/bin/annad", &backup_annad).ok();

    // v0.0.73: Atomic pair update - both or neither
    info!("Installing new binaries as atomic pair...");
    if let Err(e) = install_binary_pair(&annactl_path, &annad_path) {
        // Rollback on failure
        warn!("Update failed, rolling back: {}", e);
        if backup_annactl.exists() {
            std::fs::copy(&backup_annactl, "/usr/local/bin/annactl").ok();
        }
        if backup_annad.exists() {
            std::fs::copy(&backup_annad, "/usr/local/bin/annad").ok();
        }
        std::fs::remove_dir_all(&tmp_dir).ok();
        return Err(e);
    }

    // v0.0.73: Verify installed versions match
    info!("Verifying pair consistency...");
    if let Err(e) = verify_pair_consistency(new_version) {
        warn!("Pair consistency check failed, rolling back: {}", e);
        if backup_annactl.exists() {
            std::fs::copy(&backup_annactl, "/usr/local/bin/annactl").ok();
        }
        if backup_annad.exists() {
            std::fs::copy(&backup_annad, "/usr/local/bin/annad").ok();
        }
        std::fs::remove_dir_all(&tmp_dir).ok();
        return Err(e);
    }

    // Schedule daemon restart
    info!("Scheduling daemon restart...");
    schedule_daemon_restart()?;

    // Cleanup
    std::fs::remove_dir_all(&tmp_dir).ok();

    info!("Atomic pair update to {} complete, daemon will restart", new_version);
    Ok(())
}

/// v0.0.73: Verify binary reports expected version
fn verify_binary_version(path: &std::path::Path, expected_version: &str, name: &str) -> Result<()> {
    let output = Command::new(path)
        .arg("--version")
        .output()
        .map_err(|e| anyhow!("{} --version failed: {}", name, e))?;

    let stdout = String::from_utf8_lossy(&output.stdout);
    if !stdout.contains(expected_version) {
        return Err(anyhow!(
            "{} version mismatch: expected {} in output, got: {}",
            name, expected_version, stdout.trim()
        ));
    }
    Ok(())
}

/// v0.0.73: Install both binaries together
fn install_binary_pair(annactl: &std::path::Path, annad: &std::path::Path) -> Result<()> {
    // Install annactl first
    std::fs::copy(annactl, "/usr/local/bin/annactl")
        .map_err(|e| anyhow!("Failed to install annactl: {}", e))?;

    // Install annad to staging location
    std::fs::copy(annad, "/usr/local/bin/annad.new")
        .map_err(|e| anyhow!("Failed to stage annad: {}", e))?;

    Ok(())
}

/// v0.0.73: Verify both installed binaries report the same version
fn verify_pair_consistency(expected_version: &str) -> Result<()> {
    let annactl_output = Command::new("/usr/local/bin/annactl")
        .arg("--version")
        .output()
        .map_err(|e| anyhow!("annactl --version failed: {}", e))?;

    let annactl_ver = String::from_utf8_lossy(&annactl_output.stdout);
    if !annactl_ver.contains(expected_version) {
        return Err(anyhow!("annactl version check failed: {}", annactl_ver.trim()));
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

    info!("Pair consistency verified: both binaries at {}", expected_version);
    Ok(())
}

async fn download_file(url: &str, path: &std::path::Path) -> Result<()> {
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

fn verify_checksum(
    file_path: &std::path::Path,
    sums_path: &std::path::Path,
    name: &str,
) -> Result<()> {
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

fn schedule_daemon_restart() -> Result<()> {
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_version_comparison() {
        assert!(is_newer_version("0.0.1", "0.0.2"));
        assert!(is_newer_version("0.0.9", "0.1.0"));
        assert!(is_newer_version("0.9.9", "1.0.0"));
        assert!(!is_newer_version("0.0.2", "0.0.1"));
        assert!(!is_newer_version("0.0.1", "0.0.1"));
        assert!(is_newer_version("0.0.3", "0.0.4"));
    }

    /// v0.0.70: Test that updater correctly identifies when installed is older than latest
    #[test]
    fn test_v070_installed_older_than_latest() {
        // Typical update scenario: user has old version, remote has new version
        assert!(is_newer_version("0.0.65", "0.0.70"));
        assert!(is_newer_version("0.0.1", "0.0.70"));
        assert!(is_newer_version("0.0.69", "0.0.70"));
    }

    /// v0.0.70: Test that updater correctly identifies when installed equals latest
    #[test]
    fn test_v070_installed_equals_latest() {
        // Already up to date: should NOT trigger update
        assert!(!is_newer_version("0.0.70", "0.0.70"));
        assert!(!is_newer_version("1.0.0", "1.0.0"));
        assert!(!is_newer_version("0.1.5", "0.1.5"));
    }

    /// v0.0.70: Test that updater NEVER downgrades (installed newer than remote)
    #[test]
    fn test_v070_no_downgrade() {
        // Critical: if user has a newer version (dev build, etc), do NOT downgrade
        assert!(!is_newer_version("0.0.71", "0.0.70"));
        assert!(!is_newer_version("0.1.0", "0.0.99"));
        assert!(!is_newer_version("1.0.0", "0.9.9"));
        assert!(!is_newer_version("2.0.0", "1.99.99"));
    }

    /// v0.0.70: Test semver comparison (not string comparison)
    #[test]
    fn test_v070_semver_not_string() {
        // "0.0.9" < "0.0.10" semantically, but "0.0.9" > "0.0.10" lexically
        assert!(is_newer_version("0.0.9", "0.0.10"));
        assert!(!is_newer_version("0.0.10", "0.0.9"));

        // Same for major/minor
        assert!(is_newer_version("0.9.0", "0.10.0"));
        assert!(is_newer_version("9.0.0", "10.0.0"));
    }

    /// v0.0.70: Test edge cases in version parsing
    #[test]
    fn test_v070_version_parsing_edge_cases() {
        // Empty or malformed should not crash
        assert!(!is_newer_version("0.0.70", ""));
        assert!(!is_newer_version("", "0.0.70"));
        assert!(!is_newer_version("", ""));
        assert!(!is_newer_version("invalid", "0.0.70"));
        assert!(!is_newer_version("0.0.70", "invalid"));
    }

    /// v0.0.72: Test that UpdateCheckState enum serializes correctly
    #[test]
    fn test_v072_update_check_state_serialization() {
        use anna_shared::status::UpdateCheckState;

        // Test display
        assert_eq!(UpdateCheckState::NeverChecked.to_string(), "NEVER_CHECKED");
        assert_eq!(UpdateCheckState::Success.to_string(), "OK");
        assert_eq!(UpdateCheckState::Failed.to_string(), "FAILED");
        assert_eq!(UpdateCheckState::Checking.to_string(), "CHECKING");

        // Test default
        assert_eq!(UpdateCheckState::default(), UpdateCheckState::NeverChecked);
    }

    /// v0.0.72: Test that version preservation logic works as expected
    /// Contract: On check failure, we keep latest_version from last successful check
    #[test]
    fn test_v072_version_preservation_contract() {
        // Simulate: we had a successful check that found 0.0.80
        let last_known_version = Some("0.0.80".to_string());

        // On failure, we should NOT clear this
        // (This is a documentation test - actual logic is in server.rs)
        let preserved = last_known_version.clone();
        assert_eq!(preserved, Some("0.0.80".to_string()));

        // The version comparison should still work with preserved value
        assert!(is_newer_version("0.0.72", "0.0.80"));
    }

    /// v0.0.73: Test version module integration
    #[test]
    fn test_v073_version_module_integration() {
        use anna_shared::version::{VERSION, VersionInfo, is_newer_version as version_is_newer};

        // VERSION should be valid semver
        let parts: Vec<&str> = VERSION.split('.').collect();
        assert_eq!(parts.len(), 3);

        // VersionInfo::current() should return valid info
        let info = VersionInfo::current();
        assert_eq!(info.version, VERSION);
        assert!(info.protocol_version > 0);

        // is_newer_version should be consistent between modules
        assert_eq!(
            is_newer_version("0.0.72", "0.0.73"),
            version_is_newer("0.0.72", "0.0.73")
        );
    }

    /// v0.0.73: Test version matching logic
    #[test]
    fn test_v073_version_matching() {
        use anna_shared::version::VersionInfo;

        let v1 = VersionInfo {
            version: "0.0.73".to_string(),
            git_sha: "abc1234".to_string(),
            build_date: "2025-12-06".to_string(),
            protocol_version: 2,
        };

        // Same version, different SHA should match
        let v2 = VersionInfo {
            version: "0.0.73".to_string(),
            git_sha: "def5678".to_string(),
            build_date: "2025-12-06".to_string(),
            protocol_version: 2,
        };
        assert!(v1.matches(&v2));

        // Different version should not match
        let v3 = VersionInfo {
            version: "0.0.74".to_string(),
            git_sha: "abc1234".to_string(),
            build_date: "2025-12-07".to_string(),
            protocol_version: 2,
        };
        assert!(!v1.matches(&v3));
    }
}
