//! Services team query scenarios (v0.0.268).

use super::{Difficulty, ExpectedPath, QueryScenario};
use crate::teams::Team;

pub(super) fn add_scenarios(scenarios: &mut Vec<QueryScenario>, id: &mut u32) {
    let mut next_id = || {
        *id += 1;
        *id
    };

    // ===== SERVICES TEAM (15 queries) =====
    scenarios.push(QueryScenario {
        id: next_id(),
        query: "restart nginx".into(),
        expected_team: Team::Services,
        difficulty: Difficulty::Simple,
        expected_path: ExpectedPath::LearnableRecipe,
        similar_query: Some("reload nginx config".into()),
        tags: vec!["nginx".into(), "restart".into()],
    });

    scenarios.push(QueryScenario {
        id: next_id(),
        query: "why is docker not starting".into(),
        expected_team: Team::Services,
        difficulty: Difficulty::Medium,
        expected_path: ExpectedPath::JuniorOnly,
        similar_query: Some("docker daemon won't start".into()),
        tags: vec!["docker".into(), "troubleshoot".into()],
    });

    scenarios.push(QueryScenario {
        id: next_id(),
        query: "list all failed systemd services".into(),
        expected_team: Team::Services,
        difficulty: Difficulty::Simple,
        expected_path: ExpectedPath::JuniorOnly,
        similar_query: Some("show failed units".into()),
        tags: vec!["systemd".into(), "failed".into()],
    });

    scenarios.push(QueryScenario {
        id: next_id(),
        query: "create a systemd timer".into(),
        expected_team: Team::Services,
        difficulty: Difficulty::Complex,
        expected_path: ExpectedPath::SeniorReview,
        similar_query: None,
        tags: vec!["systemd".into(), "timer".into(), "config".into()],
    });

    scenarios.push(QueryScenario {
        id: next_id(),
        query: "enable service to start on boot".into(),
        expected_team: Team::Services,
        difficulty: Difficulty::Simple,
        expected_path: ExpectedPath::LearnableRecipe,
        similar_query: Some("autostart service".into()),
        tags: vec!["systemd".into(), "enable".into()],
    });

    scenarios.push(QueryScenario {
        id: next_id(),
        query: "setup docker compose".into(),
        expected_team: Team::Services,
        difficulty: Difficulty::Medium,
        expected_path: ExpectedPath::LearnableRecipe,
        similar_query: None,
        tags: vec!["docker".into(), "compose".into()],
    });

    scenarios.push(QueryScenario {
        id: next_id(),
        query: "view logs for sshd".into(),
        expected_team: Team::Services,
        difficulty: Difficulty::Simple,
        expected_path: ExpectedPath::JuniorOnly,
        similar_query: Some("show ssh service logs".into()),
        tags: vec!["sshd".into(), "logs".into()],
    });

    scenarios.push(QueryScenario {
        id: next_id(),
        query: "configure cron job".into(),
        expected_team: Team::Services,
        difficulty: Difficulty::Medium,
        expected_path: ExpectedPath::LearnableRecipe,
        similar_query: Some("schedule recurring task".into()),
        tags: vec!["cron".into(), "schedule".into()],
    });

    scenarios.push(QueryScenario {
        id: next_id(),
        query: "postgresql won't accept connections".into(),
        expected_team: Team::Services,
        difficulty: Difficulty::Complex,
        expected_path: ExpectedPath::SeniorReview,
        similar_query: None,
        tags: vec!["postgresql".into(), "troubleshoot".into()],
    });

    scenarios.push(QueryScenario {
        id: next_id(),
        query: "check if apache is running".into(),
        expected_team: Team::Services,
        difficulty: Difficulty::Simple,
        expected_path: ExpectedPath::JuniorOnly,
        similar_query: Some("is httpd active".into()),
        tags: vec!["apache".into(), "status".into()],
    });

    scenarios.push(QueryScenario {
        id: next_id(),
        query: "run docker container in background".into(),
        expected_team: Team::Services,
        difficulty: Difficulty::Simple,
        expected_path: ExpectedPath::LearnableRecipe,
        similar_query: Some("docker detached mode".into()),
        tags: vec!["docker".into(), "run".into()],
    });

    scenarios.push(QueryScenario {
        id: next_id(),
        query: "write a custom systemd service".into(),
        expected_team: Team::Services,
        difficulty: Difficulty::Complex,
        expected_path: ExpectedPath::SeniorReview,
        similar_query: None,
        tags: vec!["systemd".into(), "unit".into(), "create".into()],
    });

    scenarios.push(QueryScenario {
        id: next_id(),
        query: "limit docker container memory".into(),
        expected_team: Team::Services,
        difficulty: Difficulty::Medium,
        expected_path: ExpectedPath::LearnableRecipe,
        similar_query: None,
        tags: vec!["docker".into(), "resources".into()],
    });

    scenarios.push(QueryScenario {
        id: next_id(),
        query: "nginx 502 bad gateway error".into(),
        expected_team: Team::Services,
        difficulty: Difficulty::Medium,
        expected_path: ExpectedPath::JuniorOnly,
        similar_query: None,
        tags: vec!["nginx".into(), "error".into()],
    });

    scenarios.push(QueryScenario {
        id: next_id(),
        query: "reload systemd daemon after changes".into(),
        expected_team: Team::Services,
        difficulty: Difficulty::Simple,
        expected_path: ExpectedPath::LearnableRecipe,
        similar_query: Some("systemctl daemon-reload".into()),
        tags: vec!["systemd".into(), "reload".into()],
    });
}
