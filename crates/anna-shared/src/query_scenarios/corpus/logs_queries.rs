//! Logs team query scenarios (v0.0.268).

use super::{Difficulty, ExpectedPath, QueryScenario};
use crate::teams::Team;

pub(super) fn add_scenarios(scenarios: &mut Vec<QueryScenario>, id: &mut u32) {
    let mut next_id = || {
        *id += 1;
        *id
    };

    // ===== LOGS TEAM (5 queries) =====
    scenarios.push(QueryScenario {
        id: next_id(),
        query: "show recent errors in system log".into(),
        expected_team: Team::Logs,
        difficulty: Difficulty::Simple,
        expected_path: ExpectedPath::JuniorOnly,
        similar_query: Some("check syslog errors".into()),
        tags: vec!["logs".into(), "errors".into()],
    });

    scenarios.push(QueryScenario {
        id: next_id(),
        query: "view kernel messages from last boot".into(),
        expected_team: Team::Logs,
        difficulty: Difficulty::Simple,
        expected_path: ExpectedPath::JuniorOnly,
        similar_query: Some("dmesg from previous boot".into()),
        tags: vec!["dmesg".into(), "kernel".into()],
    });

    scenarios.push(QueryScenario {
        id: next_id(),
        query: "follow live journalctl output".into(),
        expected_team: Team::Logs,
        difficulty: Difficulty::Simple,
        expected_path: ExpectedPath::LearnableRecipe,
        similar_query: Some("tail system logs".into()),
        tags: vec!["journal".into(), "live".into()],
    });

    scenarios.push(QueryScenario {
        id: next_id(),
        query: "analyze why system crashed".into(),
        expected_team: Team::Logs,
        difficulty: Difficulty::Complex,
        expected_path: ExpectedPath::SeniorReview,
        similar_query: None,
        tags: vec!["crash".into(), "analyze".into()],
    });

    scenarios.push(QueryScenario {
        id: next_id(),
        query: "setup centralized logging".into(),
        expected_team: Team::Logs,
        difficulty: Difficulty::Complex,
        expected_path: ExpectedPath::SeniorReview,
        similar_query: None,
        tags: vec!["logging".into(), "config".into()],
    });
}
