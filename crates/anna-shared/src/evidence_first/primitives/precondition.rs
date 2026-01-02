//! Precondition checks for running probes.

/// Precondition for running a probe.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Precondition {
    /// Command must exist.
    CommandExists(&'static str),
    /// File must exist.
    FileExists(&'static str),
    /// Systemd must be running.
    SystemdRunning,
    /// Helper must be installed.
    HelperInstalled(&'static str),
}

impl Precondition {
    /// Check if precondition is met.
    pub fn check(&self) -> bool {
        match self {
            Self::CommandExists(cmd) => std::process::Command::new("which")
                .arg(cmd)
                .output()
                .map(|o| o.status.success())
                .unwrap_or(false),
            Self::FileExists(path) => std::path::Path::new(path).exists(),
            Self::SystemdRunning => std::process::Command::new("systemctl")
                .arg("--version")
                .output()
                .map(|o| o.status.success())
                .unwrap_or(false),
            Self::HelperInstalled(helper) => {
                // Check if helper command exists
                let cmd = match *helper {
                    "lm_sensors" => "sensors",
                    "smartmontools" => "smartctl",
                    "nvme_cli" => "nvme",
                    "ethtool" => "ethtool",
                    _ => helper,
                };
                std::process::Command::new("which")
                    .arg(cmd)
                    .output()
                    .map(|o| o.status.success())
                    .unwrap_or(false)
            }
        }
    }
}
