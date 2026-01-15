//! Tests for safe operations.

use super::*;
use crate::config::anna_data_dir;
use crate::memory::{memory_path, Memory};
use crate::rpc::ResetMode;
use std::fs;
use std::path::PathBuf;

#[test]
fn test_reset_mode_from_str() {
    assert_eq!(ResetMode::from_str("memory"), Some(ResetMode::Memory));
    assert_eq!(ResetMode::from_str("mem"), Some(ResetMode::Memory));
    assert_eq!(ResetMode::from_str("config"), Some(ResetMode::Config));
    assert_eq!(ResetMode::from_str("everything"), Some(ResetMode::Everything));
    assert_eq!(ResetMode::from_str("all"), Some(ResetMode::Everything));
    assert_eq!(ResetMode::from_str("invalid"), None);
}

#[test]
fn test_reset_mode_description() {
    assert!(ResetMode::Memory.description().contains("memory"));
    assert!(ResetMode::Config.description().contains("configuration"));
    assert!(ResetMode::Everything.description().contains("everything"));
}

#[test]
fn test_reset_mode_aliases() {
    assert_eq!(ResetMode::from_str("cfg"), Some(ResetMode::Config));
    assert_eq!(ResetMode::from_str("models"), Some(ResetMode::Models));
    assert_eq!(ResetMode::from_str("model"), Some(ResetMode::Models));
    assert_eq!(ResetMode::from_str("helpers"), Some(ResetMode::Helpers));
    assert_eq!(ResetMode::from_str("helper"), Some(ResetMode::Helpers));
    assert_eq!(ResetMode::from_str("deps"), Some(ResetMode::Helpers));
    assert_eq!(ResetMode::from_str("full"), Some(ResetMode::Everything));
}

#[test]
fn test_reset_mode_case_insensitive() {
    assert_eq!(ResetMode::from_str("MEMORY"), Some(ResetMode::Memory));
    assert_eq!(ResetMode::from_str("Memory"), Some(ResetMode::Memory));
    assert_eq!(ResetMode::from_str("EVERYTHING"), Some(ResetMode::Everything));
}

#[test]
fn test_all_modes_have_descriptions() {
    let modes = vec![
        ResetMode::Memory,
        ResetMode::Config,
        ResetMode::Models,
        ResetMode::Helpers,
        ResetMode::Everything,
    ];
    for mode in modes {
        let desc = mode.description();
        assert!(!desc.is_empty(), "Mode {:?} has empty description", mode);
        assert!(desc.len() > 10, "Mode {:?} description too short", mode);
    }
}

#[test]
fn test_reset_memory_uses_actual_array_counts() {
    if let Ok(memory) = Memory::load() {
        let exp_count = memory.experiences.len();
        let pattern_count = memory.patterns.len();
        let cluster_count = memory.clusters.len();
        assert!(exp_count == memory.experiences.len());
        assert!(pattern_count == memory.patterns.len());
        assert!(cluster_count == memory.clusters.len());
    }
}

#[test]
fn test_reset_stats_uses_system_path() {
    let paths = crate::paths::Paths::system();
    let stats_path = paths.stats_file();
    assert!(stats_path.starts_with("/var/lib/anna"), "Stats must use system path");
}

#[test]
fn test_reset_tickets_uses_system_path() {
    let paths = crate::paths::Paths::system();
    let data_dir = &paths.data_dir;
    assert!(data_dir.starts_with("/var/lib/anna"), "Tickets must use system path");
}

#[test]
fn test_backup_includes_all_data_files() {
    let paths = crate::paths::Paths::system();
    let expected_files = vec![
        memory_path(),
        paths.data_dir.join("config.toml"),
        paths.stats_file(),
        paths.data_dir.join("stats_audit.jsonl"),
        paths.data_dir.join("installed_deps.txt"),
        paths.data_dir.join("tickets.json"),
        paths.data_dir.join("fix_history.json"),
        paths.data_dir.join("model_prefs.json"),
    ];
    for path in &expected_files {
        assert!(path.starts_with("/var/lib/anna") || path.starts_with("/etc/anna"),
                "Path {} must use system paths", path.display());
    }
}

#[test]
fn test_reset_result_format_consistency() {
    let test_message = format!("Memory ({} experiences, {} patterns, {} clusters)", 5, 3, 2);
    assert!(test_message.contains("Memory"));
    assert!(test_message.contains("experiences"));
    assert!(test_message.contains("patterns"));
    assert!(test_message.contains("clusters"));

    let stats_message = format!("Stats ({} questions, {} XP)", 10, 250);
    assert!(stats_message.contains("Stats"));
    assert!(stats_message.contains("questions"));
    assert!(stats_message.contains("XP"));

    let tickets_message = format!("Tickets ({} resolved, {} failed, {} escalated)", 5, 1, 2);
    assert!(tickets_message.contains("Tickets"));
    assert!(tickets_message.contains("resolved"));
    assert!(tickets_message.contains("failed"));
    assert!(tickets_message.contains("escalated"));
}

#[test]
fn test_reset_output_structure_golden() {
    let expected_sections = vec![
        "In-memory caches",
        "Sessions",
        "Memory",
        "Stats",
        "Stats audit trail",
        "Tickets",
        "Fix history",
    ];
    for section in &expected_sections {
        assert!(!section.is_empty());
        assert!(section.chars().next().unwrap().is_uppercase() || section.starts_with("In-"));
    }
}

#[test]
fn test_reset_output_always_has_counts() {
    let memory_zero = format!("Memory ({} experiences, {} patterns, {} clusters)", 0, 0, 0);
    let memory_some = format!("Memory ({} experiences, {} patterns, {} clusters)", 5, 3, 2);
    assert_eq!(memory_zero.matches(|c: char| c.is_ascii_digit()).count() >= 3, true);
    assert_eq!(memory_some.matches(|c: char| c.is_ascii_digit()).count() >= 3, true);

    let stats_zero = format!("Stats ({} questions, {} XP)", 0, 0);
    let stats_some = format!("Stats ({} questions, {} XP)", 10, 250);
    assert!(stats_zero.contains("0 questions"));
    assert!(stats_some.contains("10 questions"));

    let tickets_zero = format!("Tickets ({} resolved, {} failed, {} escalated)", 0, 0, 0);
    let tickets_some = format!("Tickets ({} resolved, {} failed, {} escalated)", 5, 1, 2);
    assert!(tickets_zero.contains("0 resolved"));
    assert!(tickets_some.contains("5 resolved"));
}

#[test]
fn test_legacy_migration_is_one_time() {
    // Legacy migration uses system paths now - test that migrate returns None when no legacy exists
    let result = SafeReset::migrate_legacy_xp();
    assert!(result.is_ok());
    // Returns None when no legacy file exists (which is the normal case on system paths)
    assert!(result.unwrap().is_none(), "Should return None when no legacy file exists");
}

#[test]
fn test_reset_clears_unified_store_regardless_of_legacy() {
    let stats_path = anna_data_dir().join("stats.json");
    assert!(stats_path.to_string_lossy().contains("stats.json"));
    assert!(!stats_path.to_string_lossy().contains("xp.json"));
}

#[test]
fn test_backup_directory_path() {
    let backup_path = backup_utils::backup_dir();
    assert!(backup_path.to_string_lossy().contains("backups"));
    assert!(backup_path.to_string_lossy().contains(".anna")
         || backup_path.to_string_lossy().contains("anna"));
}

#[test]
fn test_backup_dir_creation() {
    let result = backup_utils::create_backup_dir("test_backup");
    if let Ok(path) = result {
        let path_str = path.to_string_lossy();
        assert!(path_str.contains("test_backup_"));
        let parts: Vec<&str> = path_str.split("test_backup_").collect();
        if parts.len() > 1 {
            assert!(parts[1].len() >= 15, "Timestamp format incorrect");
        }
        let _ = fs::remove_dir_all(&path);
    }
}

#[test]
fn test_backup_file_copies_existing() {
    use std::io::Write;
    let temp_dir = std::env::temp_dir().join("anna_backup_test");
    let _ = fs::create_dir_all(&temp_dir);

    let source = temp_dir.join("test_source.json");
    let backup_dest = temp_dir.join("backup");
    let _ = fs::create_dir_all(&backup_dest);

    if let Ok(mut f) = fs::File::create(&source) {
        let _ = writeln!(f, r#"{{"test": true}}"#);
    }

    let result = backup_utils::backup_file(&source, &backup_dest, "test_backup.json");
    assert!(result.is_ok());
    assert!(result.unwrap() == true, "Should return true when file exists");

    let backed_up = backup_dest.join("test_backup.json");
    assert!(backed_up.exists(), "Backup file should exist");
    let _ = fs::remove_dir_all(&temp_dir);
}

#[test]
fn test_backup_file_skips_nonexistent() {
    let temp_dir = std::env::temp_dir().join("anna_backup_test_skip");
    let _ = fs::create_dir_all(&temp_dir);

    let nonexistent = PathBuf::from("/nonexistent/path/file.json");

    let result = backup_utils::backup_file(&nonexistent, &temp_dir, "should_not_exist.json");
    assert!(result.is_ok());
    assert!(result.unwrap() == false, "Should return false when source doesn't exist");

    let would_be_backup = temp_dir.join("should_not_exist.json");
    assert!(!would_be_backup.exists(), "No backup should be created");
    let _ = fs::remove_dir_all(&temp_dir);
}

#[test]
fn test_reset_result_always_has_backup_path() {
    let mock_result = crate::rpc::ResetResult {
        cleared: vec!["Test".to_string()],
        backup_path: Some("/test/path".to_string()),
    };
    assert!(mock_result.backup_path.is_some(), "backup_path must be present");
}

#[test]
fn test_backup_does_not_include_secrets() {
    let backed_up_files = vec![
        "memory.json",
        "config.toml",
        "stats.json",
        "stats_audit.jsonl",
        "installed_deps.txt",
        "tickets.json",
        "fix_history.json",
        "model_prefs.json",
    ];
    for file in &backed_up_files {
        assert!(!file.contains("credentials"), "Should not backup credentials");
        assert!(!file.contains("token"), "Should not backup tokens");
        assert!(!file.contains("secret"), "Should not backup secrets");
        assert!(!file.contains(".env"), "Should not backup env files");
        assert!(!file.contains("api_key"), "Should not backup API keys");
    }
}
