//! Storage team query scenarios (v0.0.268).

use super::{Difficulty, ExpectedPath, QueryScenario};
use crate::teams::Team;

pub(super) fn add_scenarios(scenarios: &mut Vec<QueryScenario>, id: &mut u32) {
    let mut next_id = || {
        *id += 1;
        *id
    };

    // ===== STORAGE TEAM (15 queries) =====
    scenarios.push(QueryScenario {
        id: next_id(),
        query: "how much disk space do I have".into(),
        expected_team: Team::Storage,
        difficulty: Difficulty::Simple,
        expected_path: ExpectedPath::FastPath,
        similar_query: Some("check free disk space".into()),
        tags: vec!["disk".into(), "fast_path".into()],
    });

    scenarios.push(QueryScenario {
        id: next_id(),
        query: "why is my disk full".into(),
        expected_team: Team::Storage,
        difficulty: Difficulty::Medium,
        expected_path: ExpectedPath::JuniorOnly,
        similar_query: Some("disk space running out".into()),
        tags: vec!["disk".into(), "troubleshoot".into()],
    });

    scenarios.push(QueryScenario {
        id: next_id(),
        query: "list all mounted filesystems".into(),
        expected_team: Team::Storage,
        difficulty: Difficulty::Simple,
        expected_path: ExpectedPath::JuniorOnly,
        similar_query: None,
        tags: vec!["mount".into(), "filesystem".into()],
    });

    scenarios.push(QueryScenario {
        id: next_id(),
        query: "what is eating my disk space in /var".into(),
        expected_team: Team::Storage,
        difficulty: Difficulty::Medium,
        expected_path: ExpectedPath::JuniorOnly,
        similar_query: Some("find large files in /var".into()),
        tags: vec!["disk".into(), "analysis".into()],
    });

    scenarios.push(QueryScenario {
        id: next_id(),
        query: "how to mount a USB drive".into(),
        expected_team: Team::Storage,
        difficulty: Difficulty::Medium,
        expected_path: ExpectedPath::LearnableRecipe,
        similar_query: Some("mount external drive".into()),
        tags: vec!["mount".into(), "usb".into(), "recipe".into()],
    });

    scenarios.push(QueryScenario {
        id: next_id(),
        query: "clean up old journal logs".into(),
        expected_team: Team::Logs, // v0.0.268: Fixed - journal logs go to Logs team
        difficulty: Difficulty::Medium,
        expected_path: ExpectedPath::LearnableRecipe,
        similar_query: Some("free disk space by removing old logs".into()),
        tags: vec!["cleanup".into(), "journal".into()],
    });

    scenarios.push(QueryScenario {
        id: next_id(),
        query: "how to resize a btrfs partition".into(),
        expected_team: Team::Storage,
        difficulty: Difficulty::Complex,
        expected_path: ExpectedPath::SeniorReview,
        similar_query: None,
        tags: vec!["btrfs".into(), "partition".into(), "risky".into()],
    });

    scenarios.push(QueryScenario {
        id: next_id(),
        query: "fix btrfs read-only filesystem".into(),
        expected_team: Team::Storage,
        difficulty: Difficulty::Complex,
        expected_path: ExpectedPath::SeniorReview,
        similar_query: None,
        tags: vec!["btrfs".into(), "error".into()],
    });

    scenarios.push(QueryScenario {
        id: next_id(),
        query: "show disk usage by directory".into(),
        expected_team: Team::Storage,
        difficulty: Difficulty::Simple,
        expected_path: ExpectedPath::JuniorOnly,
        similar_query: Some("du -sh".into()),
        tags: vec!["disk".into(), "usage".into()],
    });

    scenarios.push(QueryScenario {
        id: next_id(),
        query: "how to check disk health with SMART".into(),
        expected_team: Team::Storage,
        difficulty: Difficulty::Medium,
        expected_path: ExpectedPath::LearnableRecipe,
        similar_query: Some("check if disk is failing".into()),
        tags: vec!["smart".into(), "health".into()],
    });

    scenarios.push(QueryScenario {
        id: next_id(),
        query: "create a new ext4 partition".into(),
        expected_team: Team::Storage,
        difficulty: Difficulty::Complex,
        expected_path: ExpectedPath::SeniorReview,
        similar_query: None,
        tags: vec!["partition".into(), "format".into(), "risky".into()],
    });

    scenarios.push(QueryScenario {
        id: next_id(),
        query: "what partitions do I have".into(),
        expected_team: Team::Storage,
        difficulty: Difficulty::Simple,
        expected_path: ExpectedPath::JuniorOnly,
        similar_query: Some("show partition table".into()),
        tags: vec!["partition".into(), "list".into()],
    });

    scenarios.push(QueryScenario {
        id: next_id(),
        query: "how to check inode usage".into(),
        expected_team: Team::Storage,
        difficulty: Difficulty::Medium,
        expected_path: ExpectedPath::JuniorOnly,
        similar_query: Some("disk full but files not large".into()),
        tags: vec!["inode".into(), "disk".into()],
    });

    scenarios.push(QueryScenario {
        id: next_id(),
        query: "setup automatic snapshots with btrfs".into(),
        expected_team: Team::Storage,
        difficulty: Difficulty::Complex,
        expected_path: ExpectedPath::SeniorReview,
        similar_query: None,
        tags: vec!["btrfs".into(), "snapshot".into(), "config".into()],
    });

    scenarios.push(QueryScenario {
        id: next_id(),
        query: "find files larger than 1GB on disk".into(), // v0.0.268: Added "disk" for routing
        expected_team: Team::Storage,
        difficulty: Difficulty::Simple,
        expected_path: ExpectedPath::LearnableRecipe,
        similar_query: Some("locate big files on disk".into()),
        tags: vec!["find".into(), "large".into()],
    });
}
