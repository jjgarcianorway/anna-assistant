//! Knowledge Import - Import Anna's knowledge from backup
//!
//! v6.55.1: Knowledge Backup, Export, Introspection and Pruning
//!
//! This module provides import capabilities:
//! - Import knowledge from JSON backup
//! - Merge or replace existing data
//! - Validate import compatibility

use anyhow::Result;
use chrono::{DateTime, Utc};
use rusqlite::{params, Connection};
use serde_json::Value;
use std::collections::HashMap;
use std::path::Path;

use crate::knowledge_domain::KnowledgeDomain;
use crate::knowledge_export::{DomainExport, ExportMetadata, KnowledgeExport};

/// Import mode
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ImportMode {
    /// Merge with existing data (skip duplicates)
    Merge,

    /// Replace all existing data in imported domains
    Replace,

    /// Only import if domain is empty
    IfEmpty,
}

/// Import options
#[derive(Debug, Clone)]
pub struct ImportOptions {
    /// Import mode
    pub mode: ImportMode,

    /// Domains to import (None = all in export)
    pub domains: Option<Vec<KnowledgeDomain>>,

    /// Skip validation checks
    pub skip_validation: bool,

    /// Dry run - report what would be imported
    pub dry_run: bool,
}

impl Default for ImportOptions {
    fn default() -> Self {
        Self {
            mode: ImportMode::Merge,
            domains: None,
            skip_validation: false,
            dry_run: false,
        }
    }
}

/// Import result
#[derive(Debug, Clone)]
pub struct ImportResult {
    /// Records imported per domain
    pub imported_per_domain: HashMap<KnowledgeDomain, u64>,

    /// Total records imported
    pub total_imported: u64,

    /// Records skipped (duplicates)
    pub total_skipped: u64,

    /// Was this a dry run?
    pub was_dry_run: bool,

    /// Warnings during import
    pub warnings: Vec<String>,

    /// Errors during import
    pub errors: Vec<String>,
}

impl ImportResult {
    fn new(dry_run: bool) -> Self {
        Self {
            imported_per_domain: HashMap::new(),
            total_imported: 0,
            total_skipped: 0,
            was_dry_run: dry_run,
            warnings: Vec::new(),
            errors: Vec::new(),
        }
    }
}

/// Validation result
#[derive(Debug)]
pub struct ValidationResult {
    pub is_valid: bool,
    pub warnings: Vec<String>,
    pub errors: Vec<String>,
}

/// Validate an export before importing
pub fn validate_export(export: &KnowledgeExport) -> ValidationResult {
    let mut result = ValidationResult {
        is_valid: true,
        warnings: Vec::new(),
        errors: Vec::new(),
    };

    // Check format version
    if export.metadata.format_version > 1 {
        result.warnings.push(format!(
            "Export format version {} is newer than supported (1). Some data may not import correctly.",
            export.metadata.format_version
        ));
    }

    // Check for empty export
    if export.metadata.total_records == 0 {
        result.warnings.push("Export contains no records".to_string());
    }

    // Check for unknown domains
    for (domain_name, domain_export) in &export.domains {
        let known = KnowledgeDomain::all()
            .iter()
            .any(|d| d.display_name() == domain_name);
        if !known {
            result.warnings.push(format!(
                "Unknown domain '{}' will be skipped",
                domain_name
            ));
        }
    }

    // Check timestamps
    if export.metadata.exported_at > Utc::now() {
        result.warnings.push("Export timestamp is in the future".to_string());
    }

    result
}

/// Import knowledge from a KnowledgeExport struct
pub fn import_knowledge(
    conn: &Connection,
    export: &KnowledgeExport,
    options: &ImportOptions,
) -> Result<ImportResult> {
    let mut result = ImportResult::new(options.dry_run);

    // Validate unless skipped
    if !options.skip_validation {
        let validation = validate_export(export);
        result.warnings.extend(validation.warnings);
        result.errors.extend(validation.errors);

        if !validation.is_valid {
            return Ok(result);
        }
    }

    // Determine which domains to import
    let domains_to_import: Vec<KnowledgeDomain> = if let Some(ref domains) = options.domains {
        domains.clone()
    } else {
        export.metadata.included_domains.clone()
    };

    for domain in domains_to_import {
        let domain_name = domain.display_name();

        // Find the export data for this domain
        let domain_export = match export.domains.get(domain_name) {
            Some(de) => de,
            None => {
                result.warnings.push(format!("No data for domain '{}'", domain_name));
                continue;
            }
        };

        // Import domain
        let (imported, skipped) = import_domain(conn, domain, domain_export, options)?;
        result.imported_per_domain.insert(domain, imported);
        result.total_imported += imported;
        result.total_skipped += skipped;
    }

    Ok(result)
}

/// Import a single domain
fn import_domain(
    conn: &Connection,
    domain: KnowledgeDomain,
    export: &DomainExport,
    options: &ImportOptions,
) -> Result<(u64, u64)> {
    let mut imported = 0u64;
    let mut skipped = 0u64;

    for table in domain.tables() {
        // Check if table exists
        let exists: bool = conn
            .query_row(
                "SELECT COUNT(*) FROM sqlite_master WHERE type='table' AND name=?1",
                [table],
                |row| {
                    let count: i64 = row.get(0)?;
                    Ok(count > 0)
                },
            )
            .unwrap_or(false);

        if !exists {
            continue;
        }

        // Handle Replace mode - clear existing data first
        if options.mode == ImportMode::Replace && !options.dry_run {
            conn.execute(&format!("DELETE FROM {}", table), [])?;
        }

        // Handle IfEmpty mode - check if table has data
        if options.mode == ImportMode::IfEmpty {
            let count: i64 = conn
                .query_row(&format!("SELECT COUNT(*) FROM {}", table), [], |row| {
                    row.get(0)
                })
                .unwrap_or(0);

            if count > 0 {
                skipped += export.record_count;
                continue;
            }
        }

        // Get column names
        let columns = get_table_columns(conn, table)?;
        if columns.is_empty() {
            continue;
        }

        // Import records for this table
        for record in &export.records {
            if let Value::Object(map) = record {
                // Check if all required columns are present
                let record_cols: Vec<_> = map.keys().cloned().collect();
                let matching_cols: Vec<_> = columns
                    .iter()
                    .filter(|c| record_cols.contains(c))
                    .cloned()
                    .collect();

                if matching_cols.is_empty() {
                    continue;
                }

                if options.dry_run {
                    imported += 1;
                    continue;
                }

                // Build INSERT statement
                let placeholders: Vec<_> = (1..=matching_cols.len())
                    .map(|i| format!("?{}", i))
                    .collect();
                let sql = format!(
                    "INSERT OR IGNORE INTO {} ({}) VALUES ({})",
                    table,
                    matching_cols.join(", "),
                    placeholders.join(", ")
                );

                // Prepare values
                let values: Vec<_> = matching_cols
                    .iter()
                    .map(|col| json_to_sql_value(map.get(col)))
                    .collect();

                // Execute insert
                match execute_insert(conn, &sql, &values) {
                    Ok(true) => imported += 1,
                    Ok(false) => skipped += 1,
                    Err(e) => {
                        // Log error but continue
                        tracing::warn!("Failed to import record to {}: {}", table, e);
                    }
                }
            }
        }
    }

    Ok((imported, skipped))
}

/// Get column names for a table
fn get_table_columns(conn: &Connection, table: &str) -> Result<Vec<String>> {
    let mut columns = Vec::new();
    let mut stmt = conn.prepare(&format!("PRAGMA table_info({})", table))?;
    let mut rows = stmt.query([])?;

    while let Some(row) = rows.next()? {
        let name: String = row.get(1)?;
        columns.push(name);
    }

    Ok(columns)
}

/// Convert JSON value to SQL-compatible value
fn json_to_sql_value(value: Option<&Value>) -> SqlValue {
    match value {
        None | Some(Value::Null) => SqlValue::Null,
        Some(Value::Bool(b)) => SqlValue::Integer(if *b { 1 } else { 0 }),
        Some(Value::Number(n)) => {
            if let Some(i) = n.as_i64() {
                SqlValue::Integer(i)
            } else if let Some(f) = n.as_f64() {
                SqlValue::Real(f)
            } else {
                SqlValue::Null
            }
        }
        Some(Value::String(s)) => SqlValue::Text(s.clone()),
        Some(Value::Array(a)) => SqlValue::Text(serde_json::to_string(a).unwrap_or_default()),
        Some(Value::Object(o)) => SqlValue::Text(serde_json::to_string(o).unwrap_or_default()),
    }
}

/// SQL value wrapper
#[derive(Debug)]
enum SqlValue {
    Null,
    Integer(i64),
    Real(f64),
    Text(String),
}

/// Execute an INSERT statement
fn execute_insert(conn: &Connection, sql: &str, values: &[SqlValue]) -> Result<bool> {
    let mut stmt = conn.prepare(sql)?;

    for (i, value) in values.iter().enumerate() {
        match value {
            SqlValue::Null => stmt.raw_bind_parameter(i + 1, rusqlite::types::Null)?,
            SqlValue::Integer(v) => stmt.raw_bind_parameter(i + 1, *v)?,
            SqlValue::Real(v) => stmt.raw_bind_parameter(i + 1, *v)?,
            SqlValue::Text(v) => stmt.raw_bind_parameter(i + 1, v.as_str())?,
        }
    }

    let rows = stmt.raw_execute()?;
    Ok(rows > 0)
}

/// Import from file
pub fn import_from_file(
    conn: &Connection,
    path: &Path,
    options: &ImportOptions,
) -> Result<ImportResult> {
    let content = std::fs::read_to_string(path)?;
    let export: KnowledgeExport = serde_json::from_str(&content)?;
    import_knowledge(conn, &export, options)
}

/// Import from compressed file
pub fn import_from_compressed_file(
    conn: &Connection,
    path: &Path,
    options: &ImportOptions,
) -> Result<ImportResult> {
    use std::io::Read;

    let file = std::fs::File::open(path)?;
    let mut decoder = flate2::read::GzDecoder::new(file);
    let mut content = String::new();
    decoder.read_to_string(&mut content)?;

    let export: KnowledgeExport = serde_json::from_str(&content)?;
    import_knowledge(conn, &export, options)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_import_mode_default() {
        let options = ImportOptions::default();
        assert_eq!(options.mode, ImportMode::Merge);
        assert!(!options.dry_run);
    }

    #[test]
    fn test_validate_empty_export() {
        let export = KnowledgeExport {
            metadata: ExportMetadata {
                anna_version: "6.55.1".to_string(),
                exported_at: Utc::now(),
                source_hostname: "test".to_string(),
                source_machine_id: None,
                format_version: 1,
                included_domains: vec![],
                total_records: 0,
            },
            domains: HashMap::new(),
        };

        let result = validate_export(&export);
        assert!(result.is_valid);
        assert!(!result.warnings.is_empty()); // Should warn about empty export
    }

    #[test]
    fn test_json_to_sql_value() {
        assert!(matches!(json_to_sql_value(None), SqlValue::Null));
        assert!(matches!(
            json_to_sql_value(Some(&Value::Bool(true))),
            SqlValue::Integer(1)
        ));
        assert!(matches!(
            json_to_sql_value(Some(&Value::String("test".to_string()))),
            SqlValue::Text(_)
        ));
    }
}
