//! Network team query scenarios (v0.0.268).

use super::{Difficulty, ExpectedPath, QueryScenario};
use crate::teams::Team;

pub(super) fn add_scenarios(scenarios: &mut Vec<QueryScenario>, id: &mut u32) {
    let mut next_id = || {
        *id += 1;
        *id
    };

    // ===== NETWORK TEAM (15 queries) =====
    scenarios.push(QueryScenario {
        id: next_id(),
        query: "am I connected to the internet".into(),
        expected_team: Team::Network,
        difficulty: Difficulty::Simple,
        expected_path: ExpectedPath::FastPath,
        similar_query: Some("check network connection".into()),
        tags: vec!["network".into(), "fast_path".into()],
    });

    scenarios.push(QueryScenario {
        id: next_id(),
        query: "what is my IP address".into(),
        expected_team: Team::Network,
        difficulty: Difficulty::Simple,
        expected_path: ExpectedPath::FastPath,
        similar_query: Some("show IP".into()),
        tags: vec!["ip".into(), "fast_path".into()],
    });

    scenarios.push(QueryScenario {
        id: next_id(),
        query: "wifi not working after update".into(),
        expected_team: Team::Network,
        difficulty: Difficulty::Complex,
        expected_path: ExpectedPath::SeniorReview,
        similar_query: None,
        tags: vec!["wifi".into(), "troubleshoot".into()],
    });

    scenarios.push(QueryScenario {
        id: next_id(),
        query: "how to connect to hidden wifi network".into(),
        expected_team: Team::Network,
        difficulty: Difficulty::Medium,
        expected_path: ExpectedPath::LearnableRecipe,
        similar_query: Some("connect to ssid not broadcasting".into()),
        tags: vec!["wifi".into(), "hidden".into()],
    });

    scenarios.push(QueryScenario {
        id: next_id(),
        query: "configure static IP address".into(),
        expected_team: Team::Network,
        difficulty: Difficulty::Medium,
        expected_path: ExpectedPath::LearnableRecipe,
        similar_query: Some("set fixed IP".into()),
        tags: vec!["ip".into(), "config".into()],
    });

    scenarios.push(QueryScenario {
        id: next_id(),
        query: "DNS not resolving".into(),
        expected_team: Team::Network,
        difficulty: Difficulty::Medium,
        expected_path: ExpectedPath::JuniorOnly,
        similar_query: Some("can ping IP but not hostname".into()),
        tags: vec!["dns".into(), "troubleshoot".into()],
    });

    scenarios.push(QueryScenario {
        id: next_id(),
        query: "setup wireguard VPN".into(),
        expected_team: Team::Network,
        difficulty: Difficulty::Complex,
        expected_path: ExpectedPath::SeniorReview,
        similar_query: None,
        tags: vec!["vpn".into(), "wireguard".into(), "config".into()],
    });

    scenarios.push(QueryScenario {
        id: next_id(),
        query: "port 8080 already in use".into(),
        expected_team: Team::Network,
        difficulty: Difficulty::Medium,
        expected_path: ExpectedPath::JuniorOnly,
        similar_query: Some("find process using port".into()),
        tags: vec!["port".into(), "troubleshoot".into()],
    });

    scenarios.push(QueryScenario {
        id: next_id(),
        query: "how to open port in firewall".into(),
        expected_team: Team::Network,
        difficulty: Difficulty::Medium,
        expected_path: ExpectedPath::LearnableRecipe,
        similar_query: Some("allow incoming connections".into()),
        tags: vec!["firewall".into(), "port".into()],
    });

    scenarios.push(QueryScenario {
        id: next_id(),
        query: "network speed test from terminal".into(),
        expected_team: Team::Network,
        difficulty: Difficulty::Simple,
        expected_path: ExpectedPath::LearnableRecipe,
        similar_query: Some("check bandwidth".into()),
        tags: vec!["speed".into(), "test".into()],
    });

    scenarios.push(QueryScenario {
        id: next_id(),
        query: "show all open network connections".into(),
        expected_team: Team::Network,
        difficulty: Difficulty::Simple,
        expected_path: ExpectedPath::JuniorOnly,
        similar_query: Some("list network sockets".into()),
        tags: vec!["netstat".into(), "connections".into()],
    });

    scenarios.push(QueryScenario {
        id: next_id(),
        query: "bridge two network interfaces".into(),
        expected_team: Team::Network,
        difficulty: Difficulty::Complex,
        expected_path: ExpectedPath::SeniorReview,
        similar_query: None,
        tags: vec!["bridge".into(), "advanced".into()],
    });

    scenarios.push(QueryScenario {
        id: next_id(),
        query: "why is my network slow".into(),
        expected_team: Team::Network,
        difficulty: Difficulty::Medium,
        expected_path: ExpectedPath::JuniorOnly,
        similar_query: Some("diagnose slow connection".into()),
        tags: vec!["slow".into(), "diagnose".into()],
    });

    scenarios.push(QueryScenario {
        id: next_id(),
        query: "setup network bonding".into(),
        expected_team: Team::Network,
        difficulty: Difficulty::Complex,
        expected_path: ExpectedPath::SeniorReview,
        similar_query: None,
        tags: vec!["bonding".into(), "advanced".into()],
    });

    scenarios.push(QueryScenario {
        id: next_id(),
        query: "restart network manager".into(),
        expected_team: Team::Network,
        difficulty: Difficulty::Simple,
        expected_path: ExpectedPath::LearnableRecipe,
        similar_query: Some("reload network config".into()),
        tags: vec!["restart".into(), "service".into()],
    });
}
