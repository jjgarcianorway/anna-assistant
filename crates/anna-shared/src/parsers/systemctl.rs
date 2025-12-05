//! Parser for systemctl output.
//!
//! Parses service status information into typed structs.

use super::atoms::{normalize_service_name, ParseError, ParseErrorReason};
use serde::{Deserialize, Serialize};

/// Service state as reported by systemctl.
/// Fixed enum for deterministic status matching.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ServiceState {
    /// Service is running
    Running,
    /// Service is active (alias for running in some contexts)
    Active,
    /// Service failed to start or crashed
    Failed,
    /// Service is not running
    Inactive,
    /// Service is activating (starting up)
    Activating,
    /// Service is deactivating (shutting down)
    Deactivating,
    /// Service is reloading configuration
    Reloading,
    /// Service state is unknown (not recognized)
    Unknown,
}

impl ServiceState {
    /// Parse a state string into ServiceState.
    pub fn from_str(s: &str) -> Self {
        match s.trim().to_lowercase().as_str() {
            "running" => Self::Running,
            "active" => Self::Active,
            "failed" => Self::Failed,
            "inactive" => Self::Inactive,
            "activating" => Self::Activating,
            "deactivating" => Self::Deactivating,
            "reloading" => Self::Reloading,
            _ => Self::Unknown,
        }
    }

    /// Check if service is considered "up" (running or active).
    pub fn is_up(&self) -> bool {
        matches!(self, Self::Running | Self::Active)
    }

    /// Check if service is considered "down" (failed or inactive).
    pub fn is_down(&self) -> bool {
        matches!(self, Self::Failed | Self::Inactive)
    }
}

impl std::fmt::Display for ServiceState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Running => write!(f, "running"),
            Self::Active => write!(f, "active"),
            Self::Failed => write!(f, "failed"),
            Self::Inactive => write!(f, "inactive"),
            Self::Activating => write!(f, "activating"),
            Self::Deactivating => write!(f, "deactivating"),
            Self::Reloading => write!(f, "reloading"),
            Self::Unknown => write!(f, "unknown"),
        }
    }
}

/// Service status with canonical unit name.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ServiceStatus {
    /// Canonical unit name (e.g., "nginx.service")
    pub name: String,
    /// Current state
    pub state: ServiceState,
    /// Optional description from systemctl output
    pub description: Option<String>,
}

/// Parse `systemctl --failed` output.
///
/// Expected format:
/// ```text
///   UNIT                         LOAD   ACTIVE SUB    DESCRIPTION
/// ● nginx.service                loaded failed failed A high performance web server
/// ● postgresql.service           loaded failed failed PostgreSQL RDBMS
///
/// LOAD   = Reflects whether the unit definition was properly loaded.
/// ...
/// ```
pub fn parse_failed_units(probe_id: &str, output: &str) -> Result<Vec<ServiceStatus>, ParseError> {
    let mut units = Vec::new();
    let mut in_table = false;

    for (line_idx, line) in output.lines().enumerate() {
        let line_num = line_idx + 1;
        let trimmed = line.trim();

        // Skip empty lines
        if trimmed.is_empty() {
            continue;
        }

        // Detect header line
        if trimmed.starts_with("UNIT") && trimmed.contains("LOAD") {
            in_table = true;
            continue;
        }

        // Stop at footer (LOAD = ...)
        if trimmed.starts_with("LOAD") && trimmed.contains('=') {
            break;
        }

        // Skip lines that don't look like unit entries
        if !in_table {
            continue;
        }

        // Parse unit line (may start with ● or other status marker)
        if let Some(status) = parse_failed_unit_line(probe_id, trimmed, line_num)? {
            units.push(status);
        }
    }

    Ok(units)
}

/// Parse a single failed unit line.
fn parse_failed_unit_line(
    _probe_id: &str,
    line: &str,
    _line_num: usize,
) -> Result<Option<ServiceStatus>, ParseError> {
    // Remove leading status markers (●, ○, etc.)
    let line = line
        .trim_start_matches(|c: char| !c.is_ascii_alphanumeric());
    let line = line.trim();

    if line.is_empty() {
        return Ok(None);
    }

    let parts: Vec<&str> = line.split_whitespace().collect();
    if parts.len() < 4 {
        // Not enough columns, skip
        return Ok(None);
    }

    // Skip lines that don't look like unit entries
    // Valid unit names contain a '.' (e.g., nginx.service, foo.socket)
    // This filters out lines like "0 loaded units listed."
    let potential_unit = parts[0];
    if !potential_unit.contains('.') {
        return Ok(None);
    }

    let unit_name = normalize_service_name(potential_unit);
    // Column 2 is LOAD, Column 3 is ACTIVE, Column 4 is SUB
    // For failed units, ACTIVE is "failed"
    let state = ServiceState::from_str(parts[2]);

    // Description is the rest
    let description = if parts.len() > 4 {
        Some(parts[4..].join(" "))
    } else {
        None
    };

    Ok(Some(ServiceStatus {
        name: unit_name,
        state,
        description,
    }))
}

/// Parse `systemctl is-active <service>` output.
///
/// Returns the state of a single service.
/// Output is typically just "active", "inactive", or "failed".
pub fn parse_is_active(
    probe_id: &str,
    service_name: &str,
    output: &str,
) -> Result<ServiceStatus, ParseError> {
    let state_str = output.trim().lines().next().unwrap_or("").trim();

    if state_str.is_empty() {
        return Err(ParseError::new(
            probe_id,
            ParseErrorReason::EmptyNumber, // Reusing error type
            output,
        ));
    }

    let state = ServiceState::from_str(state_str);
    let canonical_name = normalize_service_name(service_name);

    Ok(ServiceStatus {
        name: canonical_name,
        state,
        description: None,
    })
}

/// Parse `systemctl status <service>` output for state.
///
/// Looks for the "Active:" line in verbose status output.
pub fn parse_status_verbose(
    _probe_id: &str,
    service_name: &str,
    output: &str,
) -> Result<ServiceStatus, ParseError> {
    let canonical_name = normalize_service_name(service_name);
    let mut state = ServiceState::Unknown;
    let mut description = None;

    for line in output.lines() {
        let trimmed = line.trim();

        // Look for "Active: active (running)" or "Active: failed"
        if trimmed.starts_with("Active:") {
            let rest = trimmed.trim_start_matches("Active:").trim();
            // First word is the state
            if let Some(state_word) = rest.split_whitespace().next() {
                state = ServiceState::from_str(state_word);
            }
        }

        // Look for description line (usually first line with the unit name)
        if trimmed.starts_with('●') || trimmed.starts_with('○') {
            let rest = trimmed
                .trim_start_matches(|c: char| !c.is_ascii_alphanumeric())
                .trim();
            // Format: "unit.service - Description here"
            if let Some(desc_start) = rest.find(" - ") {
                description = Some(rest[desc_start + 3..].to_string());
            }
        }
    }

    Ok(ServiceStatus {
        name: canonical_name,
        state,
        description,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    const FAILED_OUTPUT: &str = r#"  UNIT                         LOAD   ACTIVE SUB    DESCRIPTION
● nginx.service                loaded failed failed A high performance web server
● postgresql.service           loaded failed failed PostgreSQL RDBMS

LOAD   = Reflects whether the unit definition was properly loaded.
ACTIVE = The high-level unit activation state, i.e. generalization of SUB.
SUB    = The low-level unit activation state, values depend on unit type.
"#;

    const FAILED_OUTPUT_EMPTY: &str = r#"  UNIT LOAD ACTIVE SUB DESCRIPTION

0 loaded units listed.
"#;

    #[test]
    fn golden_parse_failed_units() {
        let units = parse_failed_units("systemctl", FAILED_OUTPUT).unwrap();
        assert_eq!(units.len(), 2);

        assert_eq!(units[0].name, "nginx.service");
        assert_eq!(units[0].state, ServiceState::Failed);
        assert!(units[0].description.as_ref().unwrap().contains("web server"));

        assert_eq!(units[1].name, "postgresql.service");
        assert_eq!(units[1].state, ServiceState::Failed);
    }

    #[test]
    fn golden_parse_failed_units_empty() {
        let units = parse_failed_units("systemctl", FAILED_OUTPUT_EMPTY).unwrap();
        assert!(units.is_empty());
    }

    #[test]
    fn golden_parse_is_active() {
        let status = parse_is_active("systemctl", "nginx", "active\n").unwrap();
        assert_eq!(status.name, "nginx.service");
        assert_eq!(status.state, ServiceState::Active);

        let status = parse_is_active("systemctl", "nginx.service", "inactive\n").unwrap();
        assert_eq!(status.name, "nginx.service");
        assert_eq!(status.state, ServiceState::Inactive);

        let status = parse_is_active("systemctl", "sshd@foo", "failed\n").unwrap();
        assert_eq!(status.name, "sshd@foo.service");
        assert_eq!(status.state, ServiceState::Failed);
    }

    #[test]
    fn golden_service_state_from_str() {
        assert_eq!(ServiceState::from_str("running"), ServiceState::Running);
        assert_eq!(ServiceState::from_str("RUNNING"), ServiceState::Running);
        assert_eq!(ServiceState::from_str("active"), ServiceState::Active);
        assert_eq!(ServiceState::from_str("failed"), ServiceState::Failed);
        assert_eq!(ServiceState::from_str("inactive"), ServiceState::Inactive);
        assert_eq!(ServiceState::from_str("garbage"), ServiceState::Unknown);
    }

    #[test]
    fn golden_service_state_is_up_down() {
        assert!(ServiceState::Running.is_up());
        assert!(ServiceState::Active.is_up());
        assert!(!ServiceState::Running.is_down());

        assert!(ServiceState::Failed.is_down());
        assert!(ServiceState::Inactive.is_down());
        assert!(!ServiceState::Failed.is_up());
    }

    const STATUS_VERBOSE: &str = r#"● nginx.service - A high performance web server and a reverse proxy server
     Loaded: loaded (/lib/systemd/system/nginx.service; enabled; vendor preset: enabled)
     Active: active (running) since Mon 2024-01-15 10:30:00 UTC; 2 days ago
       Docs: man:nginx(8)
   Main PID: 1234 (nginx)
"#;

    #[test]
    fn golden_parse_status_verbose() {
        let status = parse_status_verbose("systemctl", "nginx", STATUS_VERBOSE).unwrap();
        assert_eq!(status.name, "nginx.service");
        assert_eq!(status.state, ServiceState::Active);
        assert!(status.description.is_some());
        assert!(status
            .description
            .as_ref()
            .unwrap()
            .contains("high performance"));
    }
}
