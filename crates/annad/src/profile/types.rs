// Phase 3.0.0-alpha.1: System Profile Types
// Data structures for adaptive intelligence

use serde::{Deserialize, Serialize};

/// Complete system profile collected by the daemon
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemProfile {
    /// Total system RAM in MB
    pub total_memory_mb: u64,

    /// Available RAM in MB
    pub available_memory_mb: u64,

    /// Number of CPU cores
    pub cpu_cores: usize,

    /// Total disk space in GB
    pub total_disk_gb: u64,

    /// Available disk space in GB
    pub available_disk_gb: u64,

    /// System uptime in seconds
    pub uptime_seconds: u64,

    /// Virtualization detection
    pub virtualization: VirtualizationInfo,

    /// Session type (desktop, headless, SSH)
    pub session_type: SessionType,

    /// GPU information
    pub gpu_info: GpuInfo,

    /// Recommended monitoring mode
    pub recommended_monitoring_mode: MonitoringMode,

    /// Timestamp when profile was collected
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

/// Virtualization and container detection
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum VirtualizationInfo {
    /// Running on bare metal
    None,

    /// Running in a VM (hypervisor type)
    VM(String),

    /// Running in a container (type)
    Container(String),

    /// Unknown/unable to detect
    Unknown,
}

/// Session type detection
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum SessionType {
    /// Desktop session with GUI (Wayland, X11, etc.)
    Desktop(String),

    /// Headless system (no display)
    Headless,

    /// SSH session (includes connection info)
    SSH {
        client_ip: Option<String>,
        display_forwarding: bool,
    },

    /// TTY console
    Console,

    /// Unknown session type
    Unknown,
}

/// GPU detection information
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct GpuInfo {
    /// Whether a GPU was detected
    pub present: bool,

    /// GPU vendor (NVIDIA, AMD, Intel, etc.)
    pub vendor: Option<String>,

    /// GPU model name
    pub model: Option<String>,
}

/// Monitoring mode selection
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum MonitoringMode {
    /// Full monitoring (Grafana + Prometheus)
    /// Requires: >4GB RAM, GUI available
    Full,

    /// Light monitoring (Prometheus text exporter only)
    /// Requires: 2-4GB RAM
    Light,

    /// Minimal monitoring (internal stats only)
    /// Used for: <2GB RAM or severely constrained systems
    Minimal,
}

impl MonitoringMode {
    /// Get human-readable description
    pub fn description(&self) -> &'static str {
        match self {
            Self::Full => "Grafana + Prometheus (full web UI)",
            Self::Light => "Prometheus text exporter (metrics only)",
            Self::Minimal => "Internal statistics (no external monitoring)",
        }
    }

    /// Get memory requirement
    pub fn memory_requirement_mb(&self) -> u64 {
        match self {
            Self::Full => 4096,
            Self::Light => 2048,
            Self::Minimal => 0,
        }
    }

    /// Check if Grafana should be enabled
    pub fn has_grafana(&self) -> bool {
        matches!(self, Self::Full)
    }

    /// Check if Prometheus should be enabled
    pub fn has_prometheus(&self) -> bool {
        matches!(self, Self::Full | Self::Light)
    }
}

impl SystemProfile {
    /// Calculate recommended monitoring mode based on system resources
    pub fn calculate_monitoring_mode(
        total_memory_mb: u64,
        session_type: &SessionType,
    ) -> MonitoringMode {
        // Rule 1: If RAM < 2 GB → minimal
        if total_memory_mb < 2048 {
            return MonitoringMode::Minimal;
        }

        // Rule 2: If 2-4 GB → light
        if total_memory_mb < 4096 {
            return MonitoringMode::Light;
        }

        // Rule 3: If > 4 GB and GUI available → full
        if matches!(session_type, SessionType::Desktop(_)) {
            return MonitoringMode::Full;
        }

        // Default: Light for headless/SSH with sufficient RAM
        MonitoringMode::Light
    }

    /// Get monitoring mode rationale for user display
    pub fn monitoring_rationale(&self) -> String {
        match self.recommended_monitoring_mode {
            MonitoringMode::Minimal => {
                format!(
                    "{} MB RAM detected → minimal mode (internal stats only)",
                    self.total_memory_mb
                )
            }
            MonitoringMode::Light => {
                format!(
                    "{} MB RAM detected → light mode (Prometheus only)",
                    self.total_memory_mb
                )
            }
            MonitoringMode::Full => {
                let session_desc = match &self.session_type {
                    SessionType::Desktop(s) => s.as_str(),
                    _ => "desktop",
                };
                format!(
                    "{} MB RAM + {} session detected → full mode (Grafana + Prometheus)",
                    self.total_memory_mb, session_desc
                )
            }
        }
    }

    /// Check if system is resource-constrained
    pub fn is_constrained(&self) -> bool {
        self.total_memory_mb < 4096 ||
        self.cpu_cores < 2 ||
        self.available_disk_gb < 10
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_monitoring_mode_minimal() {
        let mode = SystemProfile::calculate_monitoring_mode(1024, &SessionType::Headless);
        assert_eq!(mode, MonitoringMode::Minimal);
    }

    #[test]
    fn test_monitoring_mode_light() {
        let mode = SystemProfile::calculate_monitoring_mode(3072, &SessionType::Headless);
        assert_eq!(mode, MonitoringMode::Light);
    }

    #[test]
    fn test_monitoring_mode_full_desktop() {
        let mode = SystemProfile::calculate_monitoring_mode(
            8192,
            &SessionType::Desktop("wayland".to_string()),
        );
        assert_eq!(mode, MonitoringMode::Full);
    }

    #[test]
    fn test_monitoring_mode_light_headless_high_ram() {
        // Even with high RAM, headless defaults to light
        let mode = SystemProfile::calculate_monitoring_mode(8192, &SessionType::Headless);
        assert_eq!(mode, MonitoringMode::Light);
    }

    #[test]
    fn test_monitoring_mode_features() {
        assert!(MonitoringMode::Full.has_grafana());
        assert!(MonitoringMode::Full.has_prometheus());

        assert!(!MonitoringMode::Light.has_grafana());
        assert!(MonitoringMode::Light.has_prometheus());

        assert!(!MonitoringMode::Minimal.has_grafana());
        assert!(!MonitoringMode::Minimal.has_prometheus());
    }

    #[test]
    fn test_is_constrained() {
        let profile = SystemProfile {
            total_memory_mb: 2048,
            available_memory_mb: 1024,
            cpu_cores: 1,
            total_disk_gb: 50,
            available_disk_gb: 20,
            uptime_seconds: 3600,
            virtualization: VirtualizationInfo::None,
            session_type: SessionType::Headless,
            gpu_info: GpuInfo {
                present: false,
                vendor: None,
                model: None,
            },
            recommended_monitoring_mode: MonitoringMode::Light,
            timestamp: chrono::Utc::now(),
        };

        assert!(profile.is_constrained());
    }
}
