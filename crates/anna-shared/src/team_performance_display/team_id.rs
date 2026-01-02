//! Team identifier types

use serde::{Deserialize, Serialize};

/// Team identifier
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum TeamId {
    Desktop,
    Storage,
    Network,
    Performance,
    Services,
    Security,
    Hardware,
    General,
}

impl TeamId {
    /// Display name for the team
    pub fn display(&self) -> &'static str {
        match self {
            Self::Desktop => "Desktop",
            Self::Storage => "Storage",
            Self::Network => "Network",
            Self::Performance => "Performance",
            Self::Services => "Services",
            Self::Security => "Security",
            Self::Hardware => "Hardware",
            Self::General => "General",
        }
    }

    /// Get all teams
    pub fn all() -> Vec<TeamId> {
        vec![
            Self::Desktop,
            Self::Storage,
            Self::Network,
            Self::Performance,
            Self::Services,
            Self::Security,
            Self::Hardware,
            Self::General,
        ]
    }

    /// Parse from string
    pub fn parse(s: &str) -> Option<TeamId> {
        match s.to_lowercase().as_str() {
            "desktop" => Some(Self::Desktop),
            "storage" => Some(Self::Storage),
            "network" => Some(Self::Network),
            "performance" => Some(Self::Performance),
            "services" => Some(Self::Services),
            "security" => Some(Self::Security),
            "hardware" => Some(Self::Hardware),
            "general" => Some(Self::General),
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_team_id_display() {
        assert_eq!(TeamId::Desktop.display(), "Desktop");
        assert_eq!(TeamId::Network.display(), "Network");
    }

    #[test]
    fn test_team_id_parse() {
        assert_eq!(TeamId::parse("desktop"), Some(TeamId::Desktop));
        assert_eq!(TeamId::parse("NETWORK"), Some(TeamId::Network));
        assert_eq!(TeamId::parse("unknown"), None);
    }

    #[test]
    fn test_team_id_all() {
        let all = TeamId::all();
        assert_eq!(all.len(), 8);
    }
}
