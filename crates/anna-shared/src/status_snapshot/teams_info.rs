//! Team availability info for status snapshot (v0.0.454).

use serde::{Deserialize, Serialize};

use crate::team_availability::{HardwareCapabilities, TeamAvailability};
use crate::teams::Team;

/// Team availability information for status display
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct TeamsInfo {
    /// Total number of available teams
    pub available_count: usize,
    /// Total number of hidden teams
    pub hidden_count: usize,
    /// List of available teams
    pub available: Vec<String>,
    /// List of hidden teams with reason
    pub hidden: Vec<HiddenTeam>,
    /// Hardware capabilities summary
    pub hardware: HardwareSummaryLite,
}

/// A team that is hidden due to missing hardware
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HiddenTeam {
    /// Team name
    pub name: String,
    /// Reason for hiding
    pub reason: String,
}

/// Lightweight hardware summary
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct HardwareSummaryLite {
    pub has_audio: bool,
    pub has_network: bool,
    pub has_wifi: bool,
    pub has_battery: bool,
    pub has_bluetooth: bool,
    pub has_gpu: bool,
}

impl TeamsInfo {
    /// Build from TeamAvailability
    pub fn from_availability(avail: &TeamAvailability) -> Self {
        let available: Vec<String> = avail
            .available_teams
            .iter()
            .map(|t| t.to_string())
            .collect();

        let hidden: Vec<HiddenTeam> = avail
            .hidden_teams
            .iter()
            .map(|t| HiddenTeam {
                name: t.to_string(),
                reason: reason_for_hidden(*t, &avail.capabilities),
            })
            .collect();

        Self {
            available_count: avail.available_count(),
            hidden_count: avail.hidden_count(),
            available,
            hidden,
            hardware: HardwareSummaryLite {
                has_audio: avail.capabilities.has_audio,
                has_network: avail.capabilities.has_network,
                has_wifi: avail.capabilities.has_wifi,
                has_battery: avail.capabilities.has_battery,
                has_bluetooth: avail.capabilities.has_bluetooth,
                has_gpu: avail.capabilities.has_gpu,
            },
        }
    }

    /// Detect and build info
    pub fn detect() -> Self {
        let avail = TeamAvailability::detect();
        Self::from_availability(&avail)
    }
}

/// Get reason why a team is hidden
fn reason_for_hidden(team: Team, caps: &HardwareCapabilities) -> String {
    match team {
        Team::Network if !caps.has_network => "no network interface detected".to_string(),
        Team::Desktop if !caps.has_display => "no display detected".to_string(),
        _ => "hardware not detected".to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_teams_info_default() {
        let info = TeamsInfo::default();
        assert_eq!(info.available_count, 0);
    }

    #[test]
    fn test_from_availability() {
        let avail = TeamAvailability::default();
        let info = TeamsInfo::from_availability(&avail);
        assert!(info.available_count > 0);
    }
}
