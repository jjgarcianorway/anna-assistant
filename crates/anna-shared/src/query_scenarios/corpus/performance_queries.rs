//! Performance team query scenarios (v0.0.268).

use super::{Difficulty, ExpectedPath, QueryScenario};
use crate::teams::Team;

pub(super) fn add_scenarios(scenarios: &mut Vec<QueryScenario>, id: &mut u32) {
    let mut next_id = || {
        *id += 1;
        *id
    };

    // ===== PERFORMANCE TEAM (10 queries) =====
    scenarios.push(QueryScenario {
        id: next_id(),
        query: "why is my computer slow".into(),
        expected_team: Team::Performance,
        difficulty: Difficulty::Medium,
        expected_path: ExpectedPath::JuniorOnly,
        similar_query: Some("system running slowly".into()),
        tags: vec!["slow".into(), "performance".into()],
    });

    scenarios.push(QueryScenario {
        id: next_id(),
        query: "what is using all my RAM".into(),
        expected_team: Team::Performance,
        difficulty: Difficulty::Simple,
        expected_path: ExpectedPath::JuniorOnly,
        similar_query: Some("high memory usage".into()),
        tags: vec!["ram".into(), "memory".into()],
    });

    scenarios.push(QueryScenario {
        id: next_id(),
        query: "check system load".into(),
        expected_team: Team::Performance,
        difficulty: Difficulty::Simple,
        expected_path: ExpectedPath::FastPath,
        similar_query: Some("show load average".into()),
        tags: vec!["load".into(), "fast_path".into()],
    });

    scenarios.push(QueryScenario {
        id: next_id(),
        query: "top cpu consuming processes".into(),
        expected_team: Team::Performance,
        difficulty: Difficulty::Simple,
        expected_path: ExpectedPath::JuniorOnly,
        similar_query: Some("which process uses most cpu".into()),
        tags: vec!["cpu".into(), "process".into()],
    });

    scenarios.push(QueryScenario {
        id: next_id(),
        query: "enable swap on a file".into(),
        expected_team: Team::Performance,
        difficulty: Difficulty::Medium,
        expected_path: ExpectedPath::LearnableRecipe,
        similar_query: None,
        tags: vec!["swap".into(), "config".into()],
    });

    scenarios.push(QueryScenario {
        id: next_id(),
        query: "tune kernel for better performance".into(),
        expected_team: Team::Performance,
        difficulty: Difficulty::Complex,
        expected_path: ExpectedPath::SeniorReview,
        similar_query: None,
        tags: vec!["kernel".into(), "tune".into(), "risky".into()],
    });

    scenarios.push(QueryScenario {
        id: next_id(),
        query: "reduce power consumption".into(),
        expected_team: Team::Performance,
        difficulty: Difficulty::Medium,
        expected_path: ExpectedPath::LearnableRecipe,
        similar_query: Some("improve battery life".into()),
        tags: vec!["power".into(), "laptop".into()],
    });

    scenarios.push(QueryScenario {
        id: next_id(),
        query: "show cpu frequency".into(),
        expected_team: Team::Performance,
        difficulty: Difficulty::Simple,
        expected_path: ExpectedPath::JuniorOnly,
        similar_query: None,
        tags: vec!["cpu".into(), "frequency".into()],
    });

    scenarios.push(QueryScenario {
        id: next_id(),
        query: "benchmark disk io".into(),
        expected_team: Team::Performance,
        difficulty: Difficulty::Medium,
        expected_path: ExpectedPath::LearnableRecipe,
        similar_query: None,
        tags: vec!["disk".into(), "benchmark".into()],
    });

    scenarios.push(QueryScenario {
        id: next_id(),
        query: "high iowait what does it mean".into(),
        expected_team: Team::Performance,
        difficulty: Difficulty::Medium,
        expected_path: ExpectedPath::JuniorOnly,
        similar_query: None,
        tags: vec!["io".into(), "performance".into()],
    });
}
