//! Hardware team query scenarios (v0.0.268).

use super::{Difficulty, ExpectedPath, QueryScenario};
use crate::teams::Team;

pub(super) fn add_scenarios(scenarios: &mut Vec<QueryScenario>, id: &mut u32) {
    let mut next_id = || {
        *id += 1;
        *id
    };

    // ===== HARDWARE TEAM (10 queries) =====
    scenarios.push(QueryScenario {
        id: next_id(),
        query: "how many CPU cores do I have".into(),
        expected_team: Team::Hardware,
        difficulty: Difficulty::Simple,
        expected_path: ExpectedPath::FastPath,
        similar_query: Some("show cpu info".into()),
        tags: vec!["cpu".into(), "info".into()],
    });

    scenarios.push(QueryScenario {
        id: next_id(),
        query: "which GPU is installed".into(),
        expected_team: Team::Hardware,
        difficulty: Difficulty::Simple,
        expected_path: ExpectedPath::JuniorOnly,
        similar_query: Some("show graphics card".into()),
        tags: vec!["gpu".into(), "detect".into()],
    });

    scenarios.push(QueryScenario {
        id: next_id(),
        query: "install nvidia drivers".into(),
        expected_team: Team::Hardware,
        difficulty: Difficulty::Complex,
        expected_path: ExpectedPath::SeniorReview,
        similar_query: None,
        tags: vec!["nvidia".into(), "driver".into(), "risky".into()],
    });

    scenarios.push(QueryScenario {
        id: next_id(),
        query: "bluetooth not working".into(),
        expected_team: Team::Hardware,
        difficulty: Difficulty::Medium,
        expected_path: ExpectedPath::JuniorOnly,
        similar_query: None,
        tags: vec!["bluetooth".into(), "troubleshoot".into()],
    });

    scenarios.push(QueryScenario {
        id: next_id(),
        query: "check RAM speed and type".into(),
        expected_team: Team::Hardware,
        difficulty: Difficulty::Simple,
        expected_path: ExpectedPath::JuniorOnly,
        similar_query: Some("show memory info".into()),
        tags: vec!["ram".into(), "info".into()],
    });

    scenarios.push(QueryScenario {
        id: next_id(),
        query: "no sound on my laptop".into(),
        expected_team: Team::Hardware,
        difficulty: Difficulty::Medium,
        expected_path: ExpectedPath::JuniorOnly,
        similar_query: Some("audio not working".into()),
        tags: vec!["audio".into(), "troubleshoot".into()],
    });

    scenarios.push(QueryScenario {
        id: next_id(),
        query: "setup dual monitor".into(),
        expected_team: Team::Hardware,
        difficulty: Difficulty::Medium,
        expected_path: ExpectedPath::LearnableRecipe,
        similar_query: Some("configure multiple displays".into()),
        tags: vec!["monitor".into(), "config".into()],
    });

    scenarios.push(QueryScenario {
        id: next_id(),
        query: "check CPU temperature".into(),
        expected_team: Team::Hardware,
        difficulty: Difficulty::Simple,
        expected_path: ExpectedPath::JuniorOnly,
        similar_query: Some("is my cpu overheating".into()),
        tags: vec!["cpu".into(), "temp".into()],
    });

    scenarios.push(QueryScenario {
        id: next_id(),
        query: "keyboard backlight not working".into(),
        expected_team: Team::Hardware,
        difficulty: Difficulty::Medium,
        expected_path: ExpectedPath::JuniorOnly,
        similar_query: None,
        tags: vec!["keyboard".into(), "hardware".into()],
    });

    scenarios.push(QueryScenario {
        id: next_id(),
        query: "webcam not detected".into(),
        expected_team: Team::Hardware,
        difficulty: Difficulty::Medium,
        expected_path: ExpectedPath::JuniorOnly,
        similar_query: None,
        tags: vec!["webcam".into(), "hardware".into()],
    });
}
