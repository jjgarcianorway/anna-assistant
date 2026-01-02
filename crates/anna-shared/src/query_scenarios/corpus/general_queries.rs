//! General team query scenarios (v0.0.268).

use super::{Difficulty, ExpectedPath, QueryScenario};
use crate::teams::Team;

pub(super) fn add_scenarios(scenarios: &mut Vec<QueryScenario>, id: &mut u32) {
    let mut next_id = || {
        *id += 1;
        *id
    };

    // ===== GENERAL (5 queries) =====
    scenarios.push(QueryScenario {
        id: next_id(),
        query: "how is my computer".into(),
        expected_team: Team::General,
        difficulty: Difficulty::Simple,
        expected_path: ExpectedPath::FastPath,
        similar_query: Some("system status".into()),
        tags: vec!["health".into(), "fast_path".into()],
    });

    scenarios.push(QueryScenario {
        id: next_id(),
        query: "what changed since yesterday".into(),
        expected_team: Team::General,
        difficulty: Difficulty::Simple,
        expected_path: ExpectedPath::FastPath,
        similar_query: None,
        tags: vec!["changes".into(), "fast_path".into()],
    });

    scenarios.push(QueryScenario {
        id: next_id(),
        query: "how do I update my system".into(),
        expected_team: Team::General,
        difficulty: Difficulty::Simple,
        expected_path: ExpectedPath::LearnableRecipe,
        similar_query: Some("run system update".into()),
        tags: vec!["update".into(), "package".into()],
    });

    scenarios.push(QueryScenario {
        id: next_id(),
        query: "install htop".into(),
        expected_team: Team::General,
        difficulty: Difficulty::Simple,
        expected_path: ExpectedPath::LearnableRecipe,
        similar_query: Some("get htop package".into()),
        tags: vec!["install".into(), "package".into()],
    });

    scenarios.push(QueryScenario {
        id: next_id(),
        query: "backup my home directory".into(),
        expected_team: Team::General,
        difficulty: Difficulty::Medium,
        expected_path: ExpectedPath::LearnableRecipe,
        similar_query: None,
        tags: vec!["backup".into(), "files".into()],
    });
}
