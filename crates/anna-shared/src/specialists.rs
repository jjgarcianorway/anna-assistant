//! Specialists registry for team-scoped review roles.
//!
//! Defines per-team specialist profiles with model identifiers and thresholds.
//! All defaults are deterministic and hardware-agnostic.

use crate::teams::Team;
use serde::{Deserialize, Serialize};

/// Role within a team
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SpecialistRole {
    /// Translates user query to structured request
    Translator,
    /// First-line reviewer (deterministic + optional LLM)
    Junior,
    /// Escalation reviewer (LLM-based)
    Senior,
}

impl std::fmt::Display for SpecialistRole {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Translator => write!(f, "translator"),
            Self::Junior => write!(f, "junior"),
            Self::Senior => write!(f, "senior"),
        }
    }
}

impl Default for SpecialistRole {
    fn default() -> Self {
        Self::Junior
    }
}

/// Profile for a specialist (team + role combination)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpecialistProfile {
    /// Team this specialist belongs to
    pub team: Team,
    /// Role within the team
    pub role: SpecialistRole,
    /// Model identifier (string, not actual model selection)
    pub model_id: String,
    /// Max review rounds for this role
    pub max_rounds: u8,
    /// Score threshold for escalation (junior -> senior when below this)
    pub escalation_threshold: u8,
    /// Style identifier for prompt templates
    pub style_id: String,
}

impl SpecialistProfile {
    /// Create a new specialist profile
    pub fn new(team: Team, role: SpecialistRole) -> Self {
        let (max_rounds, escalation_threshold) = match role {
            SpecialistRole::Translator => (1, 0),
            SpecialistRole::Junior => (3, 60),
            SpecialistRole::Senior => (2, 0),
        };

        Self {
            team,
            role,
            model_id: "local-default".to_string(),
            max_rounds,
            escalation_threshold,
            style_id: format!("{}-{}", team, role),
        }
    }

    /// Create profile with custom model ID
    pub fn with_model(mut self, model_id: impl Into<String>) -> Self {
        self.model_id = model_id.into();
        self
    }

    /// Create profile with custom max rounds
    pub fn with_max_rounds(mut self, rounds: u8) -> Self {
        self.max_rounds = rounds;
        self
    }

    /// Create profile with custom escalation threshold
    pub fn with_escalation_threshold(mut self, threshold: u8) -> Self {
        self.escalation_threshold = threshold;
        self
    }
}

/// Registry of specialist profiles (all teams × all roles)
#[derive(Debug, Clone, Default)]
pub struct SpecialistsRegistry {
    profiles: Vec<SpecialistProfile>,
}

impl SpecialistsRegistry {
    /// Create empty registry
    pub fn new() -> Self {
        Self {
            profiles: Vec::new(),
        }
    }

    /// Create registry with deterministic defaults for all teams
    pub fn with_defaults() -> Self {
        let teams = [
            Team::Desktop,
            Team::Storage,
            Team::Network,
            Team::Performance,
            Team::Services,
            Team::Security,
            Team::Hardware,
            Team::General,
        ];

        let roles = [
            SpecialistRole::Translator,
            SpecialistRole::Junior,
            SpecialistRole::Senior,
        ];

        let profiles: Vec<SpecialistProfile> = teams
            .iter()
            .flat_map(|&team| roles.iter().map(move |&role| SpecialistProfile::new(team, role)))
            .collect();

        Self { profiles }
    }

    /// Get profile for team + role
    pub fn get(&self, team: Team, role: SpecialistRole) -> Option<&SpecialistProfile> {
        self.profiles
            .iter()
            .find(|p| p.team == team && p.role == role)
    }

    /// Get mutable profile for team + role
    pub fn get_mut(&mut self, team: Team, role: SpecialistRole) -> Option<&mut SpecialistProfile> {
        self.profiles
            .iter_mut()
            .find(|p| p.team == team && p.role == role)
    }

    /// Check if all teams have junior and senior profiles
    pub fn is_complete(&self) -> bool {
        let teams = [
            Team::Desktop,
            Team::Storage,
            Team::Network,
            Team::Performance,
            Team::Services,
            Team::Security,
            Team::Hardware,
            Team::General,
        ];

        teams.iter().all(|&team| {
            self.get(team, SpecialistRole::Junior).is_some()
                && self.get(team, SpecialistRole::Senior).is_some()
        })
    }

    /// Get all profiles
    pub fn profiles(&self) -> &[SpecialistProfile] {
        &self.profiles
    }

    /// Count profiles
    pub fn len(&self) -> usize {
        self.profiles.len()
    }

    /// Check if empty
    pub fn is_empty(&self) -> bool {
        self.profiles.is_empty()
    }

    /// Add or update a profile
    pub fn set(&mut self, profile: SpecialistProfile) {
        if let Some(existing) = self.get_mut(profile.team, profile.role) {
            *existing = profile;
        } else {
            self.profiles.push(profile);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_specialist_role_display() {
        assert_eq!(SpecialistRole::Translator.to_string(), "translator");
        assert_eq!(SpecialistRole::Junior.to_string(), "junior");
        assert_eq!(SpecialistRole::Senior.to_string(), "senior");
    }

    #[test]
    fn test_specialist_profile_new() {
        let profile = SpecialistProfile::new(Team::Storage, SpecialistRole::Junior);

        assert_eq!(profile.team, Team::Storage);
        assert_eq!(profile.role, SpecialistRole::Junior);
        assert_eq!(profile.model_id, "local-default");
        assert_eq!(profile.max_rounds, 3);
        assert_eq!(profile.escalation_threshold, 60);
        assert_eq!(profile.style_id, "storage-junior");
    }

    #[test]
    fn test_specialist_profile_builders() {
        let profile = SpecialistProfile::new(Team::Network, SpecialistRole::Senior)
            .with_model("llama3.2")
            .with_max_rounds(5)
            .with_escalation_threshold(70);

        assert_eq!(profile.model_id, "llama3.2");
        assert_eq!(profile.max_rounds, 5);
        assert_eq!(profile.escalation_threshold, 70);
    }

    #[test]
    fn test_registry_with_defaults() {
        let registry = SpecialistsRegistry::with_defaults();

        // 8 teams × 3 roles = 24 profiles
        assert_eq!(registry.len(), 24);
        assert!(registry.is_complete());
    }

    #[test]
    fn test_registry_lookup() {
        let registry = SpecialistsRegistry::with_defaults();

        let junior = registry.get(Team::Storage, SpecialistRole::Junior);
        assert!(junior.is_some());
        assert_eq!(junior.unwrap().team, Team::Storage);
        assert_eq!(junior.unwrap().role, SpecialistRole::Junior);

        let senior = registry.get(Team::Performance, SpecialistRole::Senior);
        assert!(senior.is_some());
        assert_eq!(senior.unwrap().team, Team::Performance);
    }

    #[test]
    fn test_registry_is_complete() {
        let registry = SpecialistsRegistry::with_defaults();
        assert!(registry.is_complete());

        let empty = SpecialistsRegistry::new();
        assert!(!empty.is_complete());
    }

    #[test]
    fn test_deterministic_defaults_stable() {
        let registry1 = SpecialistsRegistry::with_defaults();
        let registry2 = SpecialistsRegistry::with_defaults();

        // Same number of profiles
        assert_eq!(registry1.len(), registry2.len());

        // Same profiles in same order
        for (p1, p2) in registry1.profiles().iter().zip(registry2.profiles().iter()) {
            assert_eq!(p1.team, p2.team);
            assert_eq!(p1.role, p2.role);
            assert_eq!(p1.model_id, p2.model_id);
            assert_eq!(p1.max_rounds, p2.max_rounds);
            assert_eq!(p1.escalation_threshold, p2.escalation_threshold);
        }
    }

    #[test]
    fn test_all_teams_have_junior_and_senior() {
        let registry = SpecialistsRegistry::with_defaults();

        let teams = [
            Team::Desktop,
            Team::Storage,
            Team::Network,
            Team::Performance,
            Team::Services,
            Team::Security,
            Team::Hardware,
            Team::General,
        ];

        for team in teams {
            assert!(
                registry.get(team, SpecialistRole::Junior).is_some(),
                "Missing junior for {:?}",
                team
            );
            assert!(
                registry.get(team, SpecialistRole::Senior).is_some(),
                "Missing senior for {:?}",
                team
            );
        }
    }

    #[test]
    fn test_registry_set() {
        let mut registry = SpecialistsRegistry::with_defaults();

        let custom = SpecialistProfile::new(Team::Security, SpecialistRole::Junior)
            .with_model("security-model");

        registry.set(custom);

        let profile = registry.get(Team::Security, SpecialistRole::Junior).unwrap();
        assert_eq!(profile.model_id, "security-model");

        // Count should not change (updated existing)
        assert_eq!(registry.len(), 24);
    }
}
