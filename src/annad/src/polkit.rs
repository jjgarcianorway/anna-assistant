use anyhow::Result;
use std::path::Path;

/// Polkit actions defined in com.anna.policy
pub const ACTION_CONFIG_WRITE: &str = "com.anna.config.write";
pub const ACTION_MAINTENANCE: &str = "com.anna.maintenance.execute";

/// Check if polkit policy is installed
pub fn is_policy_installed() -> bool {
    Path::new("/usr/share/polkit-1/actions/com.anna.policy").exists()
}

/// Check authorization for a given action
///
/// For Sprint 1, this is simplified: since annad runs as root via systemd,
/// we already have the necessary privileges. This function serves as a
/// placeholder for future per-action authorization checks.
pub fn check_authorization(_action: &str, _uid: u32) -> Result<bool> {
    // We're running as root, so we're authorized
    // Future: implement actual polkit D-Bus API calls
    Ok(true)
}

/// Write system configuration file
///
/// This is a privileged operation. annad runs as root, so it can
/// write to /etc/anna/ directly.
pub fn write_system_config(path: &str, content: &str) -> Result<()> {
    // Validate path is within /etc/anna/
    let path = Path::new(path);
    if !path.starts_with("/etc/anna") {
        anyhow::bail!("Invalid config path: must be under /etc/anna/");
    }

    // Ensure parent directory exists
    if let Some(parent) = path.parent() {
        if !parent.exists() {
            std::fs::create_dir_all(parent)?;
        }
    }

    // Write the file
    std::fs::write(path, content)?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_invalid_path_rejected() {
        let result = write_system_config("/tmp/evil.toml", "malicious");
        assert!(result.is_err());
    }
}
