//! Specialists registry for team-scoped review roles.
//!
//! Defines per-team specialist profiles with model identifiers, thresholds,
//! and prompts. All defaults are deterministic and hardware-agnostic.
//!
//! v0.0.28: Added explicit prompt accessors for team-specialized review execution.

use crate::review_prompts::{junior_prompt, senior_prompt};
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

    /// Get the prompt for this specialist profile (v0.0.28)
    pub fn prompt(&self) -> &'static str {
        match self.role {
            SpecialistRole::Junior => junior_prompt(self.team),
            SpecialistRole::Senior => senior_prompt(self.team),
            SpecialistRole::Translator => "", // Translator uses separate prompt
        }
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

    /// Get junior prompt for a team (v0.0.28)
    pub fn junior_prompt(&self, team: Team) -> &'static str {
        junior_prompt(team)
    }

    /// Get senior prompt for a team (v0.0.28)
    pub fn senior_prompt(&self, team: Team) -> &'static str {
        senior_prompt(team)
    }

    /// Get junior model ID for a team (v0.0.28)
    pub fn junior_model(&self, team: Team) -> Option<&str> {
        self.get(team, SpecialistRole::Junior)
            .map(|p| p.model_id.as_str())
    }

    /// Get senior model ID for a team (v0.0.28)
    pub fn senior_model(&self, team: Team) -> Option<&str> {
        self.get(team, SpecialistRole::Senior)
            .map(|p| p.model_id.as_str())
    }

    /// Get escalation threshold for a team's junior reviewer (v0.0.28)
    pub fn escalation_threshold(&self, team: Team) -> u8 {
        self.get(team, SpecialistRole::Junior)
            .map(|p| p.escalation_threshold)
            .unwrap_or(60)
    }
}

// Tests: tests/specialists_tests.rs
