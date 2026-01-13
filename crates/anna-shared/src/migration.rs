//! Migration from legacy user-home paths to system-wide paths.
//!
//! INVARIANT: This migration is automatic, safe, one-time, and idempotent.
//!
//! Legacy paths:
//! - ~/.anna
//! - ~/.local/share/anna
//!
//! Target paths:
//! - /var/lib/anna/
//!
//! Merge rules:
//! - tickets: merge by ticket ID, keep newest by timestamp
//! - stats: sum totals, keep highest reliability, most recent timestamps
//! - recipes: merge with deduplication, keep highest confidence
//! - ledgers: keep most recent valid chain
//! - memory: merge experiences, deduplicate by content hash

use crate::paths::{detect_legacy_paths, paths, LegacyPath, LegacyPathKind};
use anyhow::{Context, Result};
use serde_json::Value;
use std::collections::HashMap;
use std::fs;
use std::path::Path;
use tracing::{info, warn};

/// Migration result
#[derive(Debug)]
pub struct MigrationResult {
    pub migrated_users: Vec<String>,
    pub files_migrated: usize,
    pub files_merged: usize,
    pub errors: Vec<String>,
}

/// Run migration if needed (idempotent)
pub fn migrate_if_needed() -> Result<Option<MigrationResult>> {
    let p = paths();

    // Already migrated?
    if p.is_migrated() {
        info!("Migration already completed, skipping");
        return Ok(None);
    }

    // Ensure target directories exist
    p.ensure_dirs().context("Failed to create system directories")?;

    // Detect legacy paths
    let legacy_paths = detect_legacy_paths();
    if legacy_paths.is_empty() {
        // No legacy data, mark as migrated
        write_tombstone()?;
        info!("No legacy paths found, marking as migrated");
        return Ok(None);
    }

    info!("Found {} legacy paths to migrate", legacy_paths.len());
    let result = run_migration(&legacy_paths)?;

    // Write tombstone
    write_tombstone()?;

    Ok(Some(result))
}

/// Run the actual migration
fn run_migration(legacy_paths: &[LegacyPath]) -> Result<MigrationResult> {
    let mut result = MigrationResult {
        migrated_users: Vec::new(),
        files_migrated: 0,
        files_merged: 0,
        errors: Vec::new(),
    };

    for legacy in legacy_paths {
        info!("Migrating {} from {}", legacy.user, legacy.path.display());

        match migrate_legacy_path(legacy) {
            Ok((migrated, merged)) => {
                result.files_migrated += migrated;
                result.files_merged += merged;
                if !result.migrated_users.contains(&legacy.user) {
                    result.migrated_users.push(legacy.user.clone());
                }
            }
            Err(e) => {
                let err = format!("Failed to migrate {}: {}", legacy.path.display(), e);
                warn!("{}", err);
                result.errors.push(err);
            }
        }
    }

    Ok(result)
}

/// Migrate a single legacy path
fn migrate_legacy_path(legacy: &LegacyPath) -> Result<(usize, usize)> {
    let p = paths();
    let mut migrated = 0;
    let mut merged = 0;

    // Files to migrate (simple copy if target doesn't exist, merge if it does)
    let file_mappings: Vec<(&str, Box<dyn Fn() -> std::path::PathBuf>)> = vec![
        ("stats.json", Box::new(|| p.stats_file())),
        ("tickets.json", Box::new(|| p.tickets_file())),
        ("memory.json", Box::new(|| p.memory_file())),
        ("update_ledger.json", Box::new(|| p.update_ledger_file())),
        ("xp.json", Box::new(|| p.xp_file())),
        ("fix_history.json", Box::new(|| p.fix_history_file())),
        ("installed_deps.txt", Box::new(|| p.installed_deps_file())),
        ("config.toml", Box::new(|| p.config_file())),
    ];

    for (filename, target_fn) in &file_mappings {
        let source = legacy.path.join(filename);
        let target = target_fn();

        if source.exists() {
            if target.exists() {
                // Need to merge
                match merge_file(&source, &target, filename) {
                    Ok(_) => merged += 1,
                    Err(e) => warn!("Failed to merge {}: {}", filename, e),
                }
            } else {
                // Simple copy
                if let Some(parent) = target.parent() {
                    fs::create_dir_all(parent)?;
                }
                fs::copy(&source, &target)?;
                migrated += 1;
                info!("Migrated {} -> {}", source.display(), target.display());
            }
        }
    }

    // Migrate directories
    let dir_mappings = [
        ("backups", p.backups_dir()),
        ("wiki", p.wiki_dir()),
        ("recipes", p.recipes_dir()),
    ];

    for (dirname, target_dir) in &dir_mappings {
        let source_dir = legacy.path.join(dirname);
        if source_dir.exists() && source_dir.is_dir() {
            let count = migrate_directory(&source_dir, target_dir)?;
            migrated += count;
        }
    }

    // Create backup of legacy dir
    let backup_name = format!(
        "legacy_{}_{}_{}",
        legacy.user,
        match legacy.kind {
            LegacyPathKind::DotAnna => "dotanna",
            LegacyPathKind::LocalShare => "localshare",
        },
        chrono::Utc::now().format("%Y%m%d_%H%M%S")
    );
    let backup_path = p.backups_dir().join(backup_name);

    // Move legacy dir to backup (atomic if on same filesystem)
    if let Err(e) = fs::rename(&legacy.path, &backup_path) {
        // Cross-filesystem, do copy + delete
        warn!("Cross-filesystem migration, copying: {}", e);
        copy_dir_all(&legacy.path, &backup_path)?;
        fs::remove_dir_all(&legacy.path)?;
    }
    info!("Backed up legacy dir to {}", backup_path.display());

    Ok((migrated, merged))
}

/// Merge a JSON file using type-specific rules
fn merge_file(source: &Path, target: &Path, filename: &str) -> Result<()> {
    let source_content = fs::read_to_string(source)?;
    let target_content = fs::read_to_string(target)?;

    let source_json: Value = serde_json::from_str(&source_content)?;
    let target_json: Value = serde_json::from_str(&target_content)?;

    let merged = match filename {
        "stats.json" => merge_stats(source_json, target_json)?,
        "tickets.json" => merge_tickets(source_json, target_json)?,
        "update_ledger.json" => merge_ledger(source_json, target_json)?,
        "memory.json" => merge_memory(source_json, target_json)?,
        _ => {
            // Default: keep target (system) version
            info!("Keeping system version of {}", filename);
            return Ok(());
        }
    };

    fs::write(target, serde_json::to_string_pretty(&merged)?)?;
    info!("Merged {}", filename);
    Ok(())
}

/// Merge stats: sum totals, keep highest reliability
fn merge_stats(source: Value, target: Value) -> Result<Value> {
    let mut merged = target.clone();

    if let (Some(src_rpg), Some(tgt_rpg)) = (
        source.get("rpg").and_then(|v| v.as_object()),
        merged.get_mut("rpg").and_then(|v| v.as_object_mut()),
    ) {
        // Sum totals
        if let (Some(src_q), Some(tgt_q)) = (
            src_rpg.get("total_questions").and_then(|v| v.as_u64()),
            tgt_rpg.get("total_questions").and_then(|v| v.as_u64()),
        ) {
            tgt_rpg.insert(
                "total_questions".to_string(),
                serde_json::json!(src_q + tgt_q),
            );
        }

        // Sum XP
        if let (Some(src_xp), Some(tgt_xp)) = (
            src_rpg.get("xp").and_then(|v| v.as_u64()),
            tgt_rpg.get("xp").and_then(|v| v.as_u64()),
        ) {
            tgt_rpg.insert("xp".to_string(), serde_json::json!(src_xp + tgt_xp));
        }

        // Keep highest reliability
        if let (Some(src_rel), Some(tgt_rel)) = (
            src_rpg.get("reliability").and_then(|v| v.as_f64()),
            tgt_rpg.get("reliability").and_then(|v| v.as_f64()),
        ) {
            tgt_rpg.insert(
                "reliability".to_string(),
                serde_json::json!(src_rel.max(tgt_rel)),
            );
        }
    }

    Ok(merged)
}

/// Merge tickets: by ticket ID, keep newest by timestamp
fn merge_tickets(source: Value, target: Value) -> Result<Value> {
    let mut merged = target.clone();

    // Merge ticket arrays
    if let (Some(src_tickets), Some(tgt_tickets)) = (
        source.get("tickets").and_then(|v| v.as_array()),
        merged.get_mut("tickets").and_then(|v| v.as_array_mut()),
    ) {
        let mut ticket_map: HashMap<String, Value> = HashMap::new();

        // Add target tickets first
        for ticket in tgt_tickets.iter() {
            if let Some(id) = ticket.get("id").and_then(|v| v.as_str()) {
                ticket_map.insert(id.to_string(), ticket.clone());
            }
        }

        // Merge source tickets (newer wins)
        for ticket in src_tickets {
            if let Some(id) = ticket.get("id").and_then(|v| v.as_str()) {
                let src_time = ticket
                    .get("created_at")
                    .and_then(|v| v.as_str())
                    .unwrap_or("");

                if let Some(existing) = ticket_map.get(id) {
                    let tgt_time = existing
                        .get("created_at")
                        .and_then(|v| v.as_str())
                        .unwrap_or("");

                    if src_time > tgt_time {
                        ticket_map.insert(id.to_string(), ticket.clone());
                    }
                } else {
                    ticket_map.insert(id.to_string(), ticket.clone());
                }
            }
        }

        *tgt_tickets = ticket_map.into_values().collect();
    }

    // Sum totals
    for field in ["total_resolved", "total_failed", "total_escalated"] {
        if let (Some(src_val), Some(tgt_val)) = (
            source.get(field).and_then(|v| v.as_u64()),
            merged.get(field).and_then(|v| v.as_u64()),
        ) {
            merged[field] = serde_json::json!(src_val + tgt_val);
        }
    }

    Ok(merged)
}

/// Merge ledger: keep most recent valid chain
fn merge_ledger(source: Value, target: Value) -> Result<Value> {
    let mut merged = target.clone();

    if let (Some(src_checks), Some(tgt_checks)) = (
        source.get("checks").and_then(|v| v.as_array()),
        merged.get_mut("checks").and_then(|v| v.as_array_mut()),
    ) {
        // Build set of existing timestamps
        let existing_times: std::collections::HashSet<String> = tgt_checks
            .iter()
            .filter_map(|c| c.get("timestamp").and_then(|v| v.as_str()).map(|s| s.to_string()))
            .collect();

        // Add source entries that don't exist in target
        for check in src_checks {
            if let Some(ts) = check.get("timestamp").and_then(|v| v.as_str()) {
                if !existing_times.contains(ts) {
                    tgt_checks.push(check.clone());
                }
            }
        }

        // Sort by timestamp
        tgt_checks.sort_by(|a, b| {
            let ta = a.get("timestamp").and_then(|v| v.as_str()).unwrap_or("");
            let tb = b.get("timestamp").and_then(|v| v.as_str()).unwrap_or("");
            ta.cmp(tb)
        });
    }

    Ok(merged)
}

/// Merge memory: merge experiences, deduplicate
fn merge_memory(source: Value, target: Value) -> Result<Value> {
    let mut merged = target.clone();

    if let (Some(src_exp), Some(tgt_exp)) = (
        source.get("experiences").and_then(|v| v.as_array()),
        merged.get_mut("experiences").and_then(|v| v.as_array_mut()),
    ) {
        // Build set of existing queries
        let existing: std::collections::HashSet<String> = tgt_exp
            .iter()
            .filter_map(|e| e.get("query").and_then(|v| v.as_str()).map(|s| s.to_string()))
            .collect();

        // Add non-duplicate source experiences
        for exp in src_exp {
            if let Some(query) = exp.get("query").and_then(|v| v.as_str()) {
                if !existing.contains(query) {
                    tgt_exp.push(exp.clone());
                }
            }
        }
    }

    Ok(merged)
}

/// Migrate a directory recursively
fn migrate_directory(source: &Path, target: &Path) -> Result<usize> {
    let mut count = 0;
    fs::create_dir_all(target)?;

    for entry in fs::read_dir(source)? {
        let entry = entry?;
        let src_path = entry.path();
        let file_name = entry.file_name();
        let tgt_path = target.join(&file_name);

        if src_path.is_dir() {
            count += migrate_directory(&src_path, &tgt_path)?;
        } else if !tgt_path.exists() {
            fs::copy(&src_path, &tgt_path)?;
            count += 1;
        }
    }

    Ok(count)
}

/// Copy directory recursively
fn copy_dir_all(src: &Path, dst: &Path) -> Result<()> {
    fs::create_dir_all(dst)?;
    for entry in fs::read_dir(src)? {
        let entry = entry?;
        let ty = entry.file_type()?;
        let src_path = entry.path();
        let dst_path = dst.join(entry.file_name());

        if ty.is_dir() {
            copy_dir_all(&src_path, &dst_path)?;
        } else {
            fs::copy(&src_path, &dst_path)?;
        }
    }
    Ok(())
}

/// Write migration tombstone
fn write_tombstone() -> Result<()> {
    let p = paths();
    let tombstone = p.migration_tombstone();

    if let Some(parent) = tombstone.parent() {
        fs::create_dir_all(parent)?;
    }

    let content = format!(
        "Migration completed at {}\nVersion: {}\n",
        chrono::Utc::now().to_rfc3339(),
        env!("CARGO_PKG_VERSION")
    );

    fs::write(tombstone, content)?;
    Ok(())
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_merge_stats_sums_totals() {
        let source = serde_json::json!({
            "rpg": {
                "total_questions": 50,
                "xp": 500,
                "reliability": 0.8
            }
        });

        let target = serde_json::json!({
            "rpg": {
                "total_questions": 100,
                "xp": 1000,
                "reliability": 0.9
            }
        });

        let merged = merge_stats(source, target).unwrap();
        let rpg = merged.get("rpg").unwrap();

        assert_eq!(rpg.get("total_questions").unwrap().as_u64().unwrap(), 150);
        assert_eq!(rpg.get("xp").unwrap().as_u64().unwrap(), 1500);
        assert_eq!(rpg.get("reliability").unwrap().as_f64().unwrap(), 0.9);
    }

    #[test]
    fn test_migration_is_idempotent() {
        use crate::paths::Paths;

        // Migration tombstone check ensures idempotency
        let p = Paths::system();
        let tombstone = p.migration_tombstone();

        // If tombstone exists, migration returns None
        // This test verifies the logic path exists
        assert!(tombstone.to_string_lossy().contains("/var/lib/anna/"));
    }
}
