//! Query scenario corpus (v0.0.268).
//!
//! 100+ real-world queries for comprehensive testing.

use crate::teams::Team;
use serde::{Deserialize, Serialize};

mod storage_queries;
mod network_queries;
mod desktop_queries;
mod services_queries;
mod performance_queries;
mod hardware_queries;
mod security_queries;
mod logs_queries;
mod general_queries;

/// Query difficulty level
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Difficulty {
    /// Simple lookup, fast path eligible
    Simple,
    /// Needs probe + single specialist
    Medium,
    /// May need escalation or multiple probes
    Complex,
}

/// Expected resolution path
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ExpectedPath {
    /// Should use fast path (no LLM)
    FastPath,
    /// Junior only (no escalation)
    JuniorOnly,
    /// Senior review likely
    SeniorReview,
    /// May need clarification
    Clarification,
    /// Recipe should be learned after success
    LearnableRecipe,
}

/// Single test scenario
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QueryScenario {
    pub id: u32,
    pub query: String,
    pub expected_team: Team,
    pub difficulty: Difficulty,
    pub expected_path: ExpectedPath,
    /// Similar query to test recipe recall
    pub similar_query: Option<String>,
    pub tags: Vec<String>,
}

/// Full corpus of test scenarios
pub struct ScenarioCorpus {
    pub scenarios: Vec<QueryScenario>,
}

impl ScenarioCorpus {
    pub fn load() -> Self {
        Self {
            scenarios: build_corpus(),
        }
    }

    pub fn by_team(&self, team: Team) -> Vec<&QueryScenario> {
        self.scenarios
            .iter()
            .filter(|s| s.expected_team == team)
            .collect()
    }

    pub fn by_difficulty(&self, difficulty: Difficulty) -> Vec<&QueryScenario> {
        self.scenarios
            .iter()
            .filter(|s| s.difficulty == difficulty)
            .collect()
    }

    pub fn fast_path_scenarios(&self) -> Vec<&QueryScenario> {
        self.scenarios
            .iter()
            .filter(|s| s.expected_path == ExpectedPath::FastPath)
            .collect()
    }

    pub fn learnable_scenarios(&self) -> Vec<&QueryScenario> {
        self.scenarios
            .iter()
            .filter(|s| s.expected_path == ExpectedPath::LearnableRecipe)
            .collect()
    }
}

fn build_corpus() -> Vec<QueryScenario> {
    let mut scenarios = Vec::new();
    let mut id = 0;

    storage_queries::add_scenarios(&mut scenarios, &mut id);
    network_queries::add_scenarios(&mut scenarios, &mut id);
    desktop_queries::add_scenarios(&mut scenarios, &mut id);
    services_queries::add_scenarios(&mut scenarios, &mut id);
    performance_queries::add_scenarios(&mut scenarios, &mut id);
    hardware_queries::add_scenarios(&mut scenarios, &mut id);
    security_queries::add_scenarios(&mut scenarios, &mut id);
    logs_queries::add_scenarios(&mut scenarios, &mut id);
    general_queries::add_scenarios(&mut scenarios, &mut id);

    scenarios
}
