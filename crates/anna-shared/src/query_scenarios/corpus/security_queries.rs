//! Security team query scenarios (v0.0.268).

use super::{Difficulty, ExpectedPath, QueryScenario};
use crate::teams::Team;

pub(super) fn add_scenarios(scenarios: &mut Vec<QueryScenario>, id: &mut u32) {
    let mut next_id = || {
        *id += 1;
        *id
    };

    // ===== SECURITY TEAM (10 queries) =====
    scenarios.push(QueryScenario {
        id: next_id(),
        query: "check file permissions".into(),
        expected_team: Team::Security,
        difficulty: Difficulty::Simple,
        expected_path: ExpectedPath::JuniorOnly,
        similar_query: Some("show chmod".into()),
        tags: vec!["permissions".into(), "file".into()],
    });

    scenarios.push(QueryScenario {
        id: next_id(),
        query: "permission denied even as root".into(),
        expected_team: Team::Security,
        difficulty: Difficulty::Medium,
        expected_path: ExpectedPath::JuniorOnly,
        similar_query: None,
        tags: vec!["permissions".into(), "troubleshoot".into()],
    });

    scenarios.push(QueryScenario {
        id: next_id(),
        query: "setup ssh key authentication".into(),
        expected_team: Team::Security,
        difficulty: Difficulty::Medium,
        expected_path: ExpectedPath::LearnableRecipe,
        similar_query: Some("passwordless ssh".into()),
        tags: vec!["ssh".into(), "keys".into()],
    });

    scenarios.push(QueryScenario {
        id: next_id(),
        query: "configure ufw firewall".into(),
        expected_team: Team::Security,
        difficulty: Difficulty::Medium,
        expected_path: ExpectedPath::LearnableRecipe,
        similar_query: None,
        tags: vec!["firewall".into(), "ufw".into()],
    });

    scenarios.push(QueryScenario {
        id: next_id(),
        query: "harden ssh config".into(),
        expected_team: Team::Security,
        difficulty: Difficulty::Complex,
        expected_path: ExpectedPath::SeniorReview,
        similar_query: None,
        tags: vec!["ssh".into(), "security".into(), "harden".into()],
    });

    scenarios.push(QueryScenario {
        id: next_id(),
        query: "check for failed login attempts".into(),
        expected_team: Team::Security,
        difficulty: Difficulty::Simple,
        expected_path: ExpectedPath::JuniorOnly,
        similar_query: Some("who tried to login".into()),
        tags: vec!["login".into(), "audit".into()],
    });

    scenarios.push(QueryScenario {
        id: next_id(),
        query: "setup fail2ban".into(),
        expected_team: Team::Security,
        difficulty: Difficulty::Medium,
        expected_path: ExpectedPath::LearnableRecipe,
        similar_query: None,
        tags: vec!["fail2ban".into(), "security".into()],
    });

    scenarios.push(QueryScenario {
        id: next_id(),
        query: "encrypt a file with gpg".into(),
        expected_team: Team::Security,
        difficulty: Difficulty::Medium,
        expected_path: ExpectedPath::LearnableRecipe,
        similar_query: None,
        tags: vec!["gpg".into(), "encrypt".into()],
    });

    scenarios.push(QueryScenario {
        id: next_id(),
        query: "disable root login via ssh".into(),
        expected_team: Team::Security,
        difficulty: Difficulty::Medium,
        expected_path: ExpectedPath::LearnableRecipe,
        similar_query: None,
        tags: vec!["ssh".into(), "root".into()],
    });

    scenarios.push(QueryScenario {
        id: next_id(),
        query: "check open ports for security".into(),
        expected_team: Team::Security,
        difficulty: Difficulty::Simple,
        expected_path: ExpectedPath::JuniorOnly,
        similar_query: Some("scan local ports".into()),
        tags: vec!["ports".into(), "audit".into()],
    });
}
