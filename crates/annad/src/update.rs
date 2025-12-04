//! Update management - check for and apply updates.

use anna_shared::GITHUB_REPO;
use anyhow::{anyhow, Result};
use std::process::Command;
use tracing::info;

/// GitHub API response for releases
#[derive(Debug, serde::Deserialize)]
struct GitHubRelease {
    tag_name: String,
}

/// Check GitHub for the latest version
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
    Ok(version)
}

/// Compare versions, returns true if remote is newer
pub fn is_newer_version(current: &str, remote: &str) -> bool {
    let parse = |v: &str| -> Vec<u32> {
        v.split('.')
            .filter_map(|s| s.parse().ok())
            .collect()
    };

    let current_parts = parse(current);
    let remote_parts = parse(remote);

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

/// Perform the update by downloading and replacing binaries
pub async fn perform_update(new_version: &str) -> Result<()> {
    info!("Starting update to version {}", new_version);

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

    // Download new binaries
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

    verify_checksum(&annactl_path, &sums_path, &format!("annactl-linux-{}", arch_name))?;
    verify_checksum(&annad_path, &sums_path, &format!("annad-linux-{}", arch_name))?;

    // Make executable
    std::fs::set_permissions(&annactl_path, std::os::unix::fs::PermissionsExt::from_mode(0o755))?;
    std::fs::set_permissions(&annad_path, std::os::unix::fs::PermissionsExt::from_mode(0o755))?;

    // Replace binaries (annactl first, then schedule annad restart)
    info!("Installing new annactl...");
    std::fs::copy(&annactl_path, "/usr/local/bin/annactl")?;

    info!("Installing new annad...");
    // Copy to temp location, then use systemd to restart with new binary
    std::fs::copy(&annad_path, "/usr/local/bin/annad.new")?;

    // Atomic replace and restart via systemd
    info!("Scheduling daemon restart...");
    schedule_daemon_restart()?;

    // Cleanup
    std::fs::remove_dir_all(&tmp_dir).ok();

    info!("Update to {} complete, daemon will restart", new_version);
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

fn verify_checksum(file_path: &std::path::Path, sums_path: &std::path::Path, name: &str) -> Result<()> {
    let sums_content = std::fs::read_to_string(sums_path)?;

    let expected = sums_content
        .lines()
        .find(|line| line.contains(name))
        .and_then(|line| line.split_whitespace().next())
        .ok_or_else(|| anyhow!("Checksum not found for {}", name))?;

    let output = Command::new("sha256sum")
        .arg(file_path)
        .output()?;

    let actual = String::from_utf8_lossy(&output.stdout);
    let actual = actual.split_whitespace().next().unwrap_or("");

    if actual != expected {
        return Err(anyhow!(
            "Checksum mismatch for {}: expected {}, got {}",
            name, expected, actual
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
    std::fs::set_permissions(script_path, std::os::unix::fs::PermissionsExt::from_mode(0o755))?;

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
}
