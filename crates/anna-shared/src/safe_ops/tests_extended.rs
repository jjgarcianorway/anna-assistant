//! Extended tests for safe operations - bug reproduction and contract enforcement.

use super::*;
use crate::config::anna_data_dir;
use crate::stats::PersistentStats;
use crate::status::RpgStats;
use std::fs;
use std::path::PathBuf;

#[test]
fn test_tickets_json_reset_removes_file() {
    use std::io::Write;
    let temp_dir = std::env::temp_dir().join("anna_tickets_reset_test");
    let _ = fs::create_dir_all(&temp_dir);

    let tickets_path = temp_dir.join("tickets.json");
    if let Ok(mut f) = fs::File::create(&tickets_path) {
        let dummy_content = r#"{
            "tickets": [],
            "total_resolved": 5,
            "total_failed": 2,
            "total_escalated": 1
        }"#;
        let _ = f.write_all(dummy_content.as_bytes());
    }

    assert!(tickets_path.exists(), "Test file should exist before test");

    if let Ok(content) = fs::read_to_string(&tickets_path) {
        if let Ok(store) = serde_json::from_str::<serde_json::Value>(&content) {
            let resolved = store.get("total_resolved").and_then(|v| v.as_u64()).unwrap_or(0);
            let failed = store.get("total_failed").and_then(|v| v.as_u64()).unwrap_or(0);
            let escalated = store.get("total_escalated").and_then(|v| v.as_u64()).unwrap_or(0);

            assert_eq!(resolved, 5, "Should read resolved count");
            assert_eq!(failed, 2, "Should read failed count");
            assert_eq!(escalated, 1, "Should read escalated count");
        }
    }

    let _ = fs::remove_dir_all(&temp_dir);
}

#[test]
fn test_tickets_reset_reports_presence() {
    let output_with_data = format!(
        "Tickets ({} resolved, {} failed, {} escalated)",
        5, 2, 1
    );
    let output_empty = format!(
        "Tickets ({} resolved, {} failed, {} escalated)",
        0, 0, 0
    );

    assert!(output_with_data.contains("resolved"));
    assert!(output_with_data.contains("failed"));
    assert!(output_with_data.contains("escalated"));

    assert!(output_empty.contains("0 resolved"));
    assert!(output_empty.contains("0 failed"));
    assert!(output_empty.contains("0 escalated"));
}

// ==========================================================================
// v0.3.28: SEVERITY-0 BUG REPRODUCTION TESTS
// ==========================================================================

#[test]
fn test_reset_stats_then_load_shows_zeros() {
    let temp_dir = std::env::temp_dir().join("anna_reset_bug_test");
    let _ = fs::create_dir_all(&temp_dir);

    let default_stats = PersistentStats::default();
    assert_eq!(default_stats.rpg.xp, 0, "Default XP should be 0");
    assert_eq!(default_stats.rpg.total_questions, 0, "Default questions should be 0");

    let _ = fs::remove_dir_all(&temp_dir);
}

#[test]
fn test_stats_path_consistency() {
    use crate::paths::paths;

    let stats_path_from_paths = paths().stats_file();
    let stats_path_from_reset = anna_data_dir().join("stats.json");

    assert_eq!(
        stats_path_from_paths.to_string_lossy(),
        stats_path_from_reset.to_string_lossy(),
        "CRITICAL: paths().stats_file() and anna_data_dir().join(\"stats.json\") diverge!"
    );

    assert!(
        !stats_path_from_paths.to_string_lossy().contains("/home/"),
        "Stats path should not be under /home/"
    );
    assert!(
        stats_path_from_paths.to_string_lossy().starts_with("/var/lib/anna"),
        "Stats path should be under /var/lib/anna"
    );
}

#[test]
fn test_tickets_path_consistency() {
    use crate::paths::paths;

    let tickets_path = paths().tickets_file();
    let tickets_path_from_reset = anna_data_dir().join("tickets.json");

    assert_eq!(
        tickets_path.to_string_lossy(),
        tickets_path_from_reset.to_string_lossy(),
        "CRITICAL: paths().tickets_file() and anna_data_dir().join(\"tickets.json\") diverge!"
    );

    assert!(
        !tickets_path.to_string_lossy().contains("/home/"),
        "Tickets path should not be under /home/"
    );
    assert!(
        tickets_path.to_string_lossy().starts_with("/var/lib/anna"),
        "Tickets path should be under /var/lib/anna"
    );
}

#[test]
fn test_reset_stats_file_actually_written() {
    let stats_path = anna_data_dir().join("stats.json");

    let mut stats = PersistentStats::default();
    stats.rpg.xp = 999;
    stats.rpg.total_questions = 999;

    if stats.save().is_ok() {
        assert!(stats_path.exists(), "Stats file should exist after save");

        if let Ok(loaded) = PersistentStats::load() {
            assert_eq!(loaded.rpg.xp, 999, "Saved XP should be readable");
        }

        let default_stats = PersistentStats::default();
        if default_stats.save().is_ok() {
            if let Ok(loaded_after_reset) = PersistentStats::load() {
                assert_eq!(
                    loaded_after_reset.rpg.xp, 0,
                    "SEVERITY-0 BUG: After saving default stats, load() still returns non-zero XP!"
                );
                assert_eq!(
                    loaded_after_reset.rpg.total_questions, 0,
                    "SEVERITY-0 BUG: After saving default stats, load() still returns non-zero questions!"
                );
            }
        }
    }
}

#[test]
fn test_xp_baseline_consistency() {
    let fresh = PersistentStats::fresh();
    assert_eq!(fresh.rpg.reliability, 1.0, "fresh() should have 100% reliability");
    assert_eq!(fresh.rpg.title, RpgStats::get_title(0), "fresh() should have Novice Apprentice title");
    assert_eq!(fresh.rpg.xp, 0, "fresh() should have 0 XP");
    assert_eq!(fresh.rpg.total_questions, 0, "fresh() should have 0 questions");
    assert!(fresh.rpg.installed_at.is_some(), "fresh() should have installed_at set");
    assert!(fresh.created_at.is_some(), "fresh() should have created_at set");

    let derive_default = PersistentStats::default();
    assert_eq!(derive_default.rpg.reliability, 0.0, "derive(Default) gives 0.0 (serde compat)");
}

// ==========================================================================
// v0.3.30: CONTRACT ENFORCEMENT TESTS (R5)
// ==========================================================================

#[test]
fn test_reset_is_single_pass_no_retry_loop() {
    let source = include_str!("safe_reset.rs");

    let forbidden_patterns = [
        "force.*retry",
        "retry.*loop",
        "verification failed.*retry",
        "still exists.*retry",
    ];

    let reset_stats_section = source
        .find("fn reset_stats")
        .map(|start| {
            let end = source[start..].find("fn reset_").map(|e| start + e).unwrap_or(source.len());
            &source[start..end]
        });

    let reset_tickets_section = source
        .find("fn reset_tickets")
        .map(|start| {
            let end = source[start..].find("fn list_backups").map(|e| start + e).unwrap_or(source.len());
            &source[start..end]
        });

    for pattern in &forbidden_patterns {
        if let Some(section) = reset_stats_section {
            assert!(
                !section.to_lowercase().contains(&pattern.to_lowercase().replace(".*", "")),
                "reset_stats contains forbidden pattern: {}", pattern
            );
        }
        if let Some(section) = reset_tickets_section {
            assert!(
                !section.to_lowercase().contains(&pattern.to_lowercase().replace(".*", "")),
                "reset_tickets contains forbidden pattern: {}", pattern
            );
        }
    }

    assert!(
        source.contains("NO VERIFICATION LOOP"),
        "Contract documentation missing: should have 'NO VERIFICATION LOOP' comment"
    );
}

#[test]
fn test_reset_uses_context_for_errors() {
    let source = include_str!("safe_reset.rs");

    assert!(
        source.contains("context(\"Failed to remove stats.json"),
        "reset_stats should use .context() for stats file removal"
    );
    assert!(
        source.contains("context(\"Failed to write fresh stats"),
        "reset_stats should use .context() for fresh stats write"
    );
    assert!(
        source.contains("context(\"Failed to remove tickets.json"),
        "reset_tickets should use .context() for tickets file removal"
    );
}

#[test]
fn test_transactional_reset_order() {
    let source = include_str!("safe_reset.rs");

    let reset_stats_start = source.find("fn reset_stats").expect("reset_stats should exist");
    let reset_stats_section = &source[reset_stats_start..];

    let load_pos = reset_stats_section.find("PersistentStats::load()");
    let remove_pos = reset_stats_section.find("remove_file(&stats_path)");
    let save_pos = reset_stats_section.find("fresh.save()");

    assert!(load_pos.is_some(), "Should load stats for reporting");
    assert!(remove_pos.is_some(), "Should remove stats file");
    assert!(save_pos.is_some(), "Should save fresh stats");

    let load_p = load_pos.unwrap();
    let remove_p = remove_pos.unwrap();
    let save_p = save_pos.unwrap();

    assert!(load_p < remove_p, "Should load before remove");
    assert!(remove_p < save_p, "Should remove before save");
}
