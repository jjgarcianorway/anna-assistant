//! Knowledge Export - Export Anna's knowledge to portable format
//!
//! v6.55.1: Knowledge Backup, Export, Introspection and Pruning
//!
//! This module provides export capabilities:
//! - Export knowledge to JSON for backup
//! - Selective export by domain or category
//! - Portable format for migration between machines

use anyhow::Result;
use chrono::{DateTime, Utc};
use rusqlite::{params, Connection};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::Path;

use crate::knowledge_domain::{KnowledgeCategory, KnowledgeDomain, KnowledgeSummary};

/// Knowledge export container
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KnowledgeExport {
    /// Export metadata
    pub metadata: ExportMetadata,

    /// Exported data per domain
    pub domains: HashMap<String, DomainExport>,
}

/// Export metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExportMetadata {
    /// Anna version that created this export
    pub anna_version: String,

    /// Timestamp of export
    pub exported_at: DateTime<Utc>,

    /// Source machine hostname
    pub source_hostname: String,

    /// Source machine ID (from MachineFingerprint)
    pub source_machine_id: Option<String>,

    /// Export format version (for forward compatibility)
    pub format_version: u32,

    /// Domains included in this export
    pub included_domains: Vec<KnowledgeDomain>,

    /// Total record count
    pub total_records: u64,
}

/// Export data for a single domain
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DomainExport {
    /// Domain type
    pub domain: KnowledgeDomain,

    /// Records as JSON objects
    pub records: Vec<serde_json::Value>,

    /// Record count
    pub record_count: u64,

    /// Oldest record timestamp
    pub oldest_record: Option<DateTime<Utc>>,

    /// Newest record timestamp
    pub newest_record: Option<DateTime<Utc>>,
}

/// Export options
#[derive(Debug, Clone)]
pub struct ExportOptions {
    /// Domains to export (None = all)
    pub domains: Option<Vec<KnowledgeDomain>>,

    /// Categories to export (alternative to domains)
    pub categories: Option<Vec<KnowledgeCategory>>,

    /// Only export records newer than this
    pub since: Option<DateTime<Utc>>,

    /// Only export records older than this
    pub until: Option<DateTime<Utc>>,

    /// Include sensitive data
    pub include_sensitive: bool,

    /// Maximum records per domain (0 = unlimited)
    pub max_records_per_domain: usize,
}

impl Default for ExportOptions {
    fn default() -> Self {
        Self {
            domains: None,
            categories: None,
            since: None,
            until: None,
            include_sensitive: false,
            max_records_per_domain: 0,
        }
    }
}

/// Export knowledge to a KnowledgeExport struct
pub fn export_knowledge(conn: &Connection, options: &ExportOptions) -> Result<KnowledgeExport> {
    let hostname = get_hostname();
    let machine_id = get_machine_id();

    // Determine which domains to export
    let domains_to_export = determine_domains(options);

    let mut domain_exports = HashMap::new();
    let mut total_records = 0u64;

    for domain in &domains_to_export {
        if domain.is_sensitive() && !options.include_sensitive {
            continue;
        }

        let domain_export = export_domain(conn, *domain, options)?;
        total_records += domain_export.record_count;
        domain_exports.insert(domain.display_name().to_string(), domain_export);
    }

    Ok(KnowledgeExport {
        metadata: ExportMetadata {
            anna_version: env!("CARGO_PKG_VERSION").to_string(),
            exported_at: Utc::now(),
            source_hostname: hostname,
            source_machine_id: machine_id,
            format_version: 1,
            included_domains: domains_to_export,
            total_records,
        },
        domains: domain_exports,
    })
}

/// Determine which domains to export based on options
fn determine_domains(options: &ExportOptions) -> Vec<KnowledgeDomain> {
    if let Some(ref domains) = options.domains {
        return domains.clone();
    }

    if let Some(ref categories) = options.categories {
        let mut domains = Vec::new();
        for category in categories {
            domains.extend(category.domains());
        }
        return domains;
    }

    // Default: all domains
    KnowledgeDomain::all()
}

/// Export a single domain
fn export_domain(
    conn: &Connection,
    domain: KnowledgeDomain,
    options: &ExportOptions,
) -> Result<DomainExport> {
    let mut records = Vec::new();
    let mut oldest: Option<DateTime<Utc>> = None;
    let mut newest: Option<DateTime<Utc>> = None;

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

        // Get column names
        let columns = get_table_columns(conn, table)?;
        if columns.is_empty() {
            continue;
        }

        // Build query with optional time filtering
        let timestamp_col = find_timestamp_column(conn, table)?;
        let mut query = format!("SELECT {} FROM {}", columns.join(", "), table);

        let mut conditions = Vec::new();
        if let (Some(ts_col), Some(since)) = (&timestamp_col, options.since) {
            conditions.push(format!("{} >= '{}'", ts_col, since.format("%Y-%m-%d %H:%M:%S")));
        }
        if let (Some(ts_col), Some(until)) = (&timestamp_col, options.until) {
            conditions.push(format!("{} <= '{}'", ts_col, until.format("%Y-%m-%d %H:%M:%S")));
        }

        if !conditions.is_empty() {
            query.push_str(" WHERE ");
            query.push_str(&conditions.join(" AND "));
        }

        if let Some(ref ts_col) = timestamp_col {
            query.push_str(&format!(" ORDER BY {} DESC", ts_col));
        }

        if options.max_records_per_domain > 0 {
            query.push_str(&format!(" LIMIT {}", options.max_records_per_domain));
        }

        // Execute query and collect records
        let mut stmt = conn.prepare(&query)?;
        let mut rows = stmt.query([])?;

        while let Some(row) = rows.next()? {
            let mut record = serde_json::Map::new();

            for (i, col) in columns.iter().enumerate() {
                let value = row_to_json_value(row, i)?;
                record.insert(col.clone(), value);
            }

            // Track timestamps for metadata
            if let Some(ref ts_col) = timestamp_col {
                if let Some(serde_json::Value::String(ts_str)) = record.get(ts_col) {
                    if let Ok(ts) = parse_timestamp(ts_str) {
                        if oldest.is_none() || Some(ts) < oldest {
                            oldest = Some(ts);
                        }
                        if newest.is_none() || Some(ts) > newest {
                            newest = Some(ts);
                        }
                    }
                }
            }

            records.push(serde_json::Value::Object(record));
        }
    }

    Ok(DomainExport {
        domain,
        record_count: records.len() as u64,
        records,
        oldest_record: oldest,
        newest_record: newest,
    })
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

/// Find the timestamp column in a table
fn find_timestamp_column(conn: &Connection, table: &str) -> Result<Option<String>> {
    let timestamp_cols = ["ts", "timestamp", "window_start", "ts_start", "created_at", "set_at"];

    for col in timestamp_cols {
        let exists: bool = conn
            .query_row(
                &format!(
                    "SELECT COUNT(*) FROM pragma_table_info('{}') WHERE name=?1",
                    table
                ),
                [col],
                |row| {
                    let count: i64 = row.get(0)?;
                    Ok(count > 0)
                },
            )
            .unwrap_or(false);

        if exists {
            return Ok(Some(col.to_string()));
        }
    }

    Ok(None)
}

/// Convert a row value to JSON
fn row_to_json_value(row: &rusqlite::Row<'_>, idx: usize) -> Result<serde_json::Value> {
    // Try different types
    if let Ok(v) = row.get::<_, i64>(idx) {
        return Ok(serde_json::Value::Number(v.into()));
    }
    if let Ok(v) = row.get::<_, f64>(idx) {
        if let Some(n) = serde_json::Number::from_f64(v) {
            return Ok(serde_json::Value::Number(n));
        }
    }
    if let Ok(v) = row.get::<_, String>(idx) {
        // Check if it's JSON
        if v.starts_with('{') || v.starts_with('[') {
            if let Ok(json) = serde_json::from_str(&v) {
                return Ok(json);
            }
        }
        return Ok(serde_json::Value::String(v));
    }
    if let Ok(v) = row.get::<_, bool>(idx) {
        return Ok(serde_json::Value::Bool(v));
    }
    if let Ok(None) = row.get::<_, Option<String>>(idx) {
        return Ok(serde_json::Value::Null);
    }

    Ok(serde_json::Value::Null)
}

/// Parse a timestamp string
fn parse_timestamp(ts_str: &str) -> Result<DateTime<Utc>> {
    if let Ok(ts) = DateTime::parse_from_rfc3339(ts_str) {
        return Ok(ts.with_timezone(&Utc));
    }
    if let Ok(ts) = chrono::NaiveDateTime::parse_from_str(ts_str, "%Y-%m-%d %H:%M:%S") {
        return Ok(DateTime::<Utc>::from_naive_utc_and_offset(ts, Utc));
    }
    if let Ok(date) = chrono::NaiveDate::parse_from_str(ts_str, "%Y-%m-%d") {
        let datetime = date.and_hms_opt(0, 0, 0).unwrap();
        return Ok(DateTime::<Utc>::from_naive_utc_and_offset(datetime, Utc));
    }
    anyhow::bail!("Could not parse timestamp: {}", ts_str)
}

/// Get current hostname
fn get_hostname() -> String {
    std::fs::read_to_string("/etc/hostname")
        .ok()
        .map(|s| s.trim().to_string())
        .unwrap_or_else(|| "unknown".to_string())
}

/// Get machine ID from MachineFingerprint
fn get_machine_id() -> Option<String> {
    let fp = crate::machine_identity::MachineFingerprint::collect();
    Some(fp.id)
}

/// Export to file
pub fn export_to_file(conn: &Connection, path: &Path, options: &ExportOptions) -> Result<u64> {
    let export = export_knowledge(conn, options)?;
    let json = serde_json::to_string_pretty(&export)?;
    std::fs::write(path, &json)?;
    Ok(export.metadata.total_records)
}

/// Export to compressed file
pub fn export_to_compressed_file(
    conn: &Connection,
    path: &Path,
    options: &ExportOptions,
) -> Result<u64> {
    use std::io::Write;

    let export = export_knowledge(conn, options)?;
    let json = serde_json::to_string(&export)?;

    let file = std::fs::File::create(path)?;
    let mut encoder = flate2::write::GzEncoder::new(file, flate2::Compression::default());
    encoder.write_all(json.as_bytes())?;
    encoder.finish()?;

    Ok(export.metadata.total_records)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_determine_domains_all() {
        let options = ExportOptions::default();
        let domains = determine_domains(&options);
        assert_eq!(domains.len(), 16);
    }

    #[test]
    fn test_determine_domains_specific() {
        let options = ExportOptions {
            domains: Some(vec![KnowledgeDomain::Telemetry]),
            ..Default::default()
        };
        let domains = determine_domains(&options);
        assert_eq!(domains.len(), 1);
        assert_eq!(domains[0], KnowledgeDomain::Telemetry);
    }

    #[test]
    fn test_determine_domains_by_category() {
        let options = ExportOptions {
            categories: Some(vec![KnowledgeCategory::SystemMetrics]),
            ..Default::default()
        };
        let domains = determine_domains(&options);
        assert!(domains.contains(&KnowledgeDomain::Telemetry));
    }

    #[test]
    fn test_export_metadata() {
        let metadata = ExportMetadata {
            anna_version: "6.55.1".to_string(),
            exported_at: Utc::now(),
            source_hostname: "test".to_string(),
            source_machine_id: None,
            format_version: 1,
            included_domains: vec![KnowledgeDomain::Telemetry],
            total_records: 100,
        };
        assert_eq!(metadata.format_version, 1);
    }
}
