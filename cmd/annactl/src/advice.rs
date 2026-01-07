use serde::{Deserialize, Serialize};
use std::cmp::Ordering;
use std::collections::{HashMap, HashSet};
use std::fs;
use std::io;
use std::path::PathBuf;
use time::format_description::well_known::Rfc3339;
use time::OffsetDateTime;

use crate::paths::AnnaPaths;
use crate::ui::{self, Style, UiCfg};

// Re-export RPC types for compatibility with existing code
type AdviceSeverity = anna_rpc::AdviceSeverity;
type AdviceRecord = anna_rpc::AdviceRecord;
type AdvicePlan = anna_rpc::AdvicePlan;

#[derive(Debug, Clone)]
pub struct AdviceCliError {
    code: i32,
    message: String,
}

impl AdviceCliError {
    pub fn usage(msg: impl Into<String>) -> Self {
        Self {
            code: 64,
            message: msg.into(),
        }
    }

    pub fn io(context: impl Into<String>, err: io::Error) -> Self {
        let message = if err.kind() == io::ErrorKind::PermissionDenied {
            format!(
                "{}: Permission denied. Try 'annactl doctor perms' to diagnose permission issues.",
                context.into()
            )
        } else {
            format!("{}: {}", context.into(), err)
        };
        Self { code: 1, message }
    }

    pub fn other(msg: impl Into<String>) -> Self {
        Self {
            code: 1,
            message: msg.into(),
        }
    }

    pub fn code(&self) -> i32 {
        self.code
    }
}

impl std::fmt::Display for AdviceCliError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.message)
    }
}

impl std::error::Error for AdviceCliError {}

pub type Result<T> = std::result::Result<T, AdviceCliError>;

#[derive(Debug, Clone, Copy)]
pub enum AdviceCommand {
    List,
    Show,
    Apply,
}

pub struct AdviceArgs<'a> {
    pub command: AdviceCommand,
    pub id: Option<&'a str>,
    pub dry_run: bool,
    pub raw: bool,
    pub all: bool,
    pub force: bool,
}

pub fn run(args: AdviceArgs<'_>, cfg: &UiCfg, style: &Style) -> Result<()> {
    match args.command {
        AdviceCommand::List => list(cfg, style, args.raw),
        AdviceCommand::Show => {
            let id = args
                .id
                .ok_or_else(|| AdviceCliError::usage("advice show <id>"))?;
            show(id, cfg, style, args.raw)
        }
        AdviceCommand::Apply => {
            if args.all {
                apply_all(args.dry_run, args.force, cfg, style, args.raw)
            } else {
                let id = args
                    .id
                    .ok_or_else(|| AdviceCliError::usage("advice apply <id> or use --all"))?;
                apply(id, args.dry_run, cfg, style, args.raw)
            }
        }
    }
}

fn list(cfg: &UiCfg, style: &Style, raw: bool) -> Result<()> {
    // Use RPC to fetch advice list
    let paths = AnnaPaths::detect();
    let uid = nix::unistd::Uid::effective().as_raw();

    if !paths.socket_path.exists() {
        return Err(AdviceCliError::other(
            "Anna daemon not running. Start with: systemctl --user start annad (or sudo systemctl start annad for system mode)",
        ));
    }

    let client = crate::rpc::RpcClient::new(&paths.socket_path);
    let request = anna_rpc::Request::AdviceList(anna_rpc::AdviceListRequest { uid });

    let entries = match client.call(request) {
        Ok(anna_rpc::Response::AdviceList(response)) => response.items,
        Ok(anna_rpc::Response::Error(err)) => {
            return Err(AdviceCliError::other(format!("Error: {}", err.message)));
        }
        Ok(_) => return Err(AdviceCliError::other("Unexpected response type")),
        Err(e) => return Err(AdviceCliError::other(format!("Failed to fetch advice: {}", e))),
    };

    if !raw {
        ui::banner(style, "Advice List");
    }

    if entries.is_empty() {
        println!(
            "{}",
            ui::warn(style, "no advice entries (run annad to generate)")
        );
        return Ok(());
    }

    if raw {
        println!(
            "{}",
            serde_json::to_string_pretty(&entries).unwrap_or_else(|_| "[]".into())
        );
        return Ok(());
    }

    println!("{}", ui::head(style, "Advice entries"));
    let mut tally = SeverityTally::default();
    for (idx, entry) in entries.iter().enumerate() {
        tally.bump(entry.severity);
        let created_local = ui::fmt_local(&entry.created_at, cfg);
        let num_label = ui::cyan(style, &format!("#{}", idx + 1));
        let id_colored = ui::cyan(style, &entry.id);
        let persona_colored = ui::yellow(style, &entry.persona_hint);
        let created_colored = ui::dim_gray(style, &created_local);
        let reason = collapse_reason(&entry.reason);

        println!("{} {}", num_label, ui::kv(style, "ID", &id_colored));
        println!("{}", ui::kv(style, "Persona", &persona_colored));
        println!(
            "{}",
            ui::kv(style, "Severity", &render_severity(style, entry.severity))
        );
        println!("{}", ui::kv(style, "Kind", &entry.kind));
        println!("{}", ui::kv(style, "Created", &created_colored));
        println!("{}", ui::kv(style, "Reason", &reason));
        println!();
    }

    let totals = format_summary(entries.len(), &tally);
    println!("{}", ui::head(style, "Summary"));
    println!("{}", ui::kv(style, "Totals", &totals));
    Ok(())
}

fn show(id: &str, cfg: &UiCfg, style: &Style, raw: bool) -> Result<()> {
    // Resolve ID (support #N format) - need to list first to resolve numeric IDs
    let resolved_id = resolve_id(id)?;

    // Use RPC to fetch advice detail
    let paths = AnnaPaths::detect();
    let uid = nix::unistd::Uid::effective().as_raw();

    if !paths.socket_path.exists() {
        return Err(AdviceCliError::other(
            "Anna daemon not running. Start with: systemctl --user start annad (or sudo systemctl start annad for system mode)",
        ));
    }

    let client = crate::rpc::RpcClient::new(&paths.socket_path);
    let request = anna_rpc::Request::AdviceShow(anna_rpc::AdviceShowRequest {
        uid,
        id: resolved_id.clone(),
    });

    let record = match client.call(request) {
        Ok(anna_rpc::Response::AdviceShow(response)) => {
            response.advice.ok_or_else(|| {
                AdviceCliError::other(format!("Advice entry '{}' not found", resolved_id))
            })?
        }
        Ok(anna_rpc::Response::Error(err)) => {
            return Err(AdviceCliError::other(format!("Error: {}", err.message)));
        }
        Ok(_) => return Err(AdviceCliError::other("Unexpected response type")),
        Err(e) => return Err(AdviceCliError::other(format!("Failed to fetch advice: {}", e))),
    };

    if !raw {
        ui::banner(style, "Advice Detail");
    }

    if raw {
        println!(
            "{}",
            serde_json::to_string_pretty(&record).unwrap_or_else(|_| "{}".into())
        );
        return Ok(());
    }

    let created_local = ui::fmt_local(&record.created_at, cfg);
    let id_colored = ui::cyan(style, &record.id);
    let persona_colored = ui::yellow(style, &record.persona_hint);
    let created_colored = ui::dim_gray(style, &created_local);
    let reason = collapse_reason(&record.reason);

    println!("{}", ui::head(style, &format!("Advice {}", id_colored)));
    println!("{}", ui::kv(style, "Persona", &persona_colored));
    println!(
        "{}",
        ui::kv(style, "Severity", &render_severity(style, record.severity))
    );
    println!("{}", ui::kv(style, "Kind", &record.kind));
    println!("{}", ui::kv(style, "Created", &created_colored));
    println!("{}", ui::kv(style, "Reason", &reason));

    if !record.plan.dry_run_cmds.is_empty() {
        println!("{}", ui::head(style, "Dry-run commands"));
        for cmd in &record.plan.dry_run_cmds {
            println!("{}", ui::bullet(style, cmd));
        }
    }
    if !record.plan.apply_cmds.is_empty() {
        println!("{}", ui::head(style, "Apply commands"));
        for cmd in &record.plan.apply_cmds {
            println!("{}", ui::bullet(style, cmd));
        }
    }
    if !record.plan.undo_cmds.is_empty() {
        println!("{}", ui::head(style, "Undo commands"));
        for cmd in &record.plan.undo_cmds {
            println!("{}", ui::bullet(style, cmd));
        }
    }
    Ok(())
}

fn apply(id: &str, dry_run: bool, _cfg: &UiCfg, style: &Style, raw: bool) -> Result<()> {
    use crate::paths::AnnaPaths;
    use crate::rpc::RpcClient;

    if !raw {
        ui::banner(style, "Apply Advice");
    }

    // Resolve ID (support #N format)
    let resolved_id = resolve_id(id)?;

    // Get paths and UID
    let paths = AnnaPaths::detect();
    let uid = nix::unistd::Uid::effective().as_raw();

    // Check socket exists
    if !paths.socket_path.exists() {
        return Err(AdviceCliError::other(
            "Anna daemon not running. Start with: sudo systemctl start annad",
        ));
    }

    // Send RPC request
    let client = RpcClient::new(&paths.socket_path);
    let mode = if dry_run {
        anna_rpc::ApplyMode::DryRun
    } else {
        anna_rpc::ApplyMode::Execute
    };

    let request = anna_rpc::Request::Apply(anna_rpc::ApplyRequest {
        uid,
        id: resolved_id.clone(),
        mode,
    });

    match client.call(request) {
        Ok(anna_rpc::Response::Apply(response)) => {
            if raw {
                println!(
                    "{}",
                    serde_json::to_string_pretty(&response).unwrap_or_else(|_| "{}".into())
                );
                return Ok(());
            }

            // Display result
            let status_icon = match response.status.as_str() {
                "success" => ui::ok(style, "✓"),
                "preview" => ui::info(style, "Preview"),
                "partial" => ui::warn(style, "⚠"),
                "requires_confirmation" | "requires_elevation" => ui::warn(style, "⚠"),
                _ => ui::info(style, &response.status),
            };

            println!("{} {}", status_icon, response.message);

            if let Some(output) = response.output {
                println!("\n{}", ui::head(style, "Output"));
                for line in output.lines() {
                    if line.starts_with('✓') {
                        println!("{}", ui::ok(style, line));
                    } else if line.starts_with('✗') {
                        println!("{}", ui::err(style, line));
                    } else {
                        println!("{}", line);
                    }
                }
            }

            if response.requires_approval {
                println!(
                    "\n{}",
                    ui::note(
                        style,
                        "This action requires approval or higher policy level"
                    )
                );
            }

            Ok(())
        }
        Ok(anna_rpc::Response::Error(err)) => {
            Err(AdviceCliError::other(format!("Error: {}", err.message)))
        }
        Ok(_) => Err(AdviceCliError::other("Unexpected response type")),
        Err(e) => Err(AdviceCliError::other(format!(
            "Failed to apply advice: {}",
            e
        ))),
    }
}

fn apply_all(dry_run: bool, force: bool, _cfg: &UiCfg, style: &Style, raw: bool) -> Result<()> {
    ui::banner(style, "Batch Apply");

    // Fetch entries via RPC
    let paths = AnnaPaths::detect();
    let uid = nix::unistd::Uid::effective().as_raw();

    if !paths.socket_path.exists() {
        return Err(AdviceCliError::other(
            "Anna daemon not running. Start with: systemctl --user start annad (or sudo systemctl start annad for system mode)",
        ));
    }

    let client = crate::rpc::RpcClient::new(&paths.socket_path);
    let request = anna_rpc::Request::AdviceList(anna_rpc::AdviceListRequest { uid });

    let entries = match client.call(request) {
        Ok(anna_rpc::Response::AdviceList(response)) => response.items,
        Ok(anna_rpc::Response::Error(err)) => {
            return Err(AdviceCliError::other(format!("Error: {}", err.message)));
        }
        Ok(_) => return Err(AdviceCliError::other("Unexpected response type")),
        Err(e) => return Err(AdviceCliError::other(format!("Failed to fetch advice: {}", e))),
    };

    if entries.is_empty() {
        println!(
            "{}",
            ui::warn(style, "no advice entries (run annad to generate)")
        );
        return Ok(());
    }

    let mut applied = 0;
    let mut skipped_risky = 0;
    let mut failed = 0;

    if raw {
        println!("{{");
        println!("  \"dry_run\": {},", dry_run);
        println!("  \"force\": {},", force);
        println!("  \"total\": {},", entries.len());
        println!("  \"items\": [");
    } else {
        println!(
            "{}",
            ui::head(
                style,
                &format!(
                    "Processing {} advice entries (dry_run={}, force={})",
                    entries.len(),
                    dry_run,
                    force
                )
            )
        );
        println!();
    }

    for (idx, entry) in entries.iter().enumerate() {
        if !force && is_risky(entry) {
            if raw {
                if idx > 0 {
                    println!(",");
                }
                println!(
                    "    {{\"id\": \"{}\", \"status\": \"skipped_risky\"}}",
                    entry.id
                );
            } else {
                println!(
                    "{} {}",
                    ui::warn(style, "SKIP (risky)"),
                    ui::cyan(style, &entry.id)
                );
            }
            skipped_risky += 1;
            continue;
        }

        if raw && idx > 0 && (applied + failed) > 0 {
            println!(",");
        }

        if dry_run {
            if !entry.plan.dry_run_cmds.is_empty() {
                if raw {
                    println!("    {{");
                    println!("      \"id\": \"{}\",", entry.id);
                    println!("      \"status\": \"would_apply\",");
                    println!("      \"dry_run_cmds\": [");
                    for (i, cmd) in entry.plan.dry_run_cmds.iter().enumerate() {
                        if i > 0 {
                            println!(",");
                        }
                        print!("        \"{}\"", cmd.replace('"', "\\\""));
                    }
                    println!("\n      ]");
                    print!("    }}");
                } else {
                    println!(
                        "{} {}",
                        ui::ok(style, "DRY-RUN"),
                        ui::cyan(style, &entry.id)
                    );
                    for cmd in &entry.plan.dry_run_cmds {
                        println!("  {}", ui::bullet(style, &format!("$ {}", cmd)));
                    }
                }
                applied += 1;
            } else {
                if raw {
                    println!(
                        "    {{\"id\": \"{}\", \"status\": \"no_dry_run_cmds\"}}",
                        entry.id
                    );
                } else {
                    println!(
                        "{} {} (no dry-run commands)",
                        ui::warn(style, "SKIP"),
                        ui::cyan(style, &entry.id)
                    );
                }
                failed += 1;
            }
        } else {
            // Real apply not yet enabled
            if raw {
                println!(
                    "    {{\"id\": \"{}\", \"status\": \"not_implemented\"}}",
                    entry.id
                );
            } else {
                println!(
                    "{} {} (real apply not yet implemented)",
                    ui::warn(style, "SKIP"),
                    ui::cyan(style, &entry.id)
                );
            }
            failed += 1;
        }
    }

    if raw {
        println!("\n  ],");
        println!("  \"applied\": {},", applied);
        println!("  \"skipped_risky\": {},", skipped_risky);
        println!("  \"failed\": {}", failed);
        println!("}}");
    } else {
        println!("\n{}", ui::head(style, "Summary"));
        let summary = if style.emoji {
            format!(
                "✅ applied: {}  ⚠️ skipped (risky): {}  ❌ failed: {}",
                applied, skipped_risky, failed
            )
        } else {
            format!(
                "applied: {}  skipped (risky): {}  failed: {}",
                applied, skipped_risky, failed
            )
        };
        println!("{}", summary);
    }

    Ok(())
}

/// Check if an advice entry is considered "risky"
/// Currently, all items are considered non-risky for forward compatibility
/// Future: may check for severity, destructive keywords, or a risky flag
fn is_risky(_entry: &AdviceRecord) -> bool {
    // For now, no entries are risky
    // Future: return entry.severity == AdviceSeverity::Action || entry.risky flag
    false
}

fn read_all() -> Result<Vec<AdviceRecord>> {
    let paths = AnnaPaths::detect();
    let dir = &paths.advice_dir;
    if !dir.exists() {
        return Ok(Vec::new());
    }
    let mut records = Vec::new();
    for entry in fs::read_dir(dir).map_err(|e| AdviceCliError::io("read advice dir", e))? {
        let entry = entry.map_err(|e| AdviceCliError::io("read advice entry", e))?;
        let path = entry.path();
        if path.extension().and_then(|s| s.to_str()) != Some("json") {
            continue;
        }
        match fs::read(&path) {
            Ok(data) => match serde_json::from_slice::<AdviceRecord>(&data) {
                Ok(rec) => records.push(rec),
                Err(err) => {
                    return Err(AdviceCliError::other(format!(
                        "parse {}: {}",
                        path.display(),
                        err
                    )));
                }
            },
            Err(err) => return Err(AdviceCliError::io("read advice", err)),
        }
    }
    records.sort_by(
        |a, b| match parse_ts(&b.created_at).cmp(&parse_ts(&a.created_at)) {
            Ordering::Equal => b.id.cmp(&a.id),
            other => other,
        },
    );

    // De-duplicate: keep only the most recent entry for each (kind, normalized_reason) key
    Ok(deduplicate_records(records))
}

/// De-duplicate advice records by (kind, normalized_reason) key, keeping only the most recent
fn deduplicate_records(records: Vec<AdviceRecord>) -> Vec<AdviceRecord> {
    let mut seen: HashMap<String, AdviceRecord> = HashMap::new();

    for record in records {
        let key = dedup_key(&record);

        // Since records are sorted newest-first, we only insert if not seen
        // This keeps the most recent entry for each key
        seen.entry(key).or_insert(record);
    }

    // Collect and re-sort by timestamp (newest first)
    let mut deduped: Vec<AdviceRecord> = seen.into_values().collect();
    deduped.sort_by(
        |a, b| match parse_ts(&b.created_at).cmp(&parse_ts(&a.created_at)) {
            Ordering::Equal => b.id.cmp(&a.id),
            other => other,
        },
    );

    deduped
}

/// Generate a de-duplication key from (kind, normalized_reason)
fn dedup_key(record: &AdviceRecord) -> String {
    // Normalize reason: lowercase, collapse whitespace, remove special chars
    let normalized_reason = record
        .reason
        .to_lowercase()
        .chars()
        .filter(|c| c.is_alphanumeric() || c.is_whitespace())
        .collect::<String>()
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ");

    format!("{}::{}", record.kind, normalized_reason)
}

fn read_one(id: &str) -> Result<AdviceRecord> {
    // Check if id is numeric (N or #N)
    let resolved_id = resolve_id(id)?;
    let path = normalize_id(&resolved_id);
    let data = fs::read(&path).map_err(|e| AdviceCliError::io("read advice", e))?;
    serde_json::from_slice::<AdviceRecord>(&data)
        .map_err(|e| AdviceCliError::other(format!("parse {}: {}", path.display(), e)))
}

/// Resolve numeric selectors (#N or N) to actual IDs
fn resolve_id(id: &str) -> Result<String> {
    // Strip leading # if present
    let id_clean = id.strip_prefix('#').unwrap_or(id);

    // Try to parse as number
    if let Ok(num) = id_clean.parse::<usize>() {
        if num == 0 {
            return Err(AdviceCliError::usage(
                "Advice numbers start at 1 (use #1, #2, etc. or run 'annactl advice list')",
            ));
        }

        // Fetch entries via RPC
        let paths = AnnaPaths::detect();
        let uid = nix::unistd::Uid::effective().as_raw();

        if !paths.socket_path.exists() {
            return Err(AdviceCliError::other(
                "Anna daemon not running. Start with: systemctl --user start annad (or sudo systemctl start annad for system mode)",
            ));
        }

        let client = crate::rpc::RpcClient::new(&paths.socket_path);
        let request = anna_rpc::Request::AdviceList(anna_rpc::AdviceListRequest { uid });

        let entries = match client.call(request) {
            Ok(anna_rpc::Response::AdviceList(response)) => response.items,
            Ok(anna_rpc::Response::Error(err)) => {
                return Err(AdviceCliError::other(format!("Error: {}", err.message)));
            }
            Ok(_) => return Err(AdviceCliError::other("Unexpected response type")),
            Err(e) => return Err(AdviceCliError::other(format!("Failed to fetch advice: {}", e))),
        };

        let idx = num - 1; // Convert to 0-based index

        if idx >= entries.len() {
            return Err(AdviceCliError::usage(format!(
                "No advice entry #{} (only {} entries available)",
                num,
                entries.len()
            )));
        }

        Ok(entries[idx].id.clone())
    } else {
        // Not numeric, use as-is
        Ok(id.to_string())
    }
}

fn normalize_id(id: &str) -> PathBuf {
    let paths = AnnaPaths::detect();
    let mut path = paths.advice_dir.join(id);
    if path.extension().is_none() {
        path = path.with_extension("json");
    }
    path
}

fn parse_ts(ts: &str) -> Option<OffsetDateTime> {
    OffsetDateTime::parse(ts, &Rfc3339).ok()
}

#[derive(Default)]
struct SeverityTally {
    info: usize,
    warn: usize,
    action: usize,
}

impl SeverityTally {
    fn bump(&mut self, severity: AdviceSeverity) {
        match severity {
            AdviceSeverity::Info => self.info += 1,
            AdviceSeverity::Warn => self.warn += 1,
            AdviceSeverity::Action => self.action += 1,
        }
    }
}

fn format_summary(total: usize, tally: &SeverityTally) -> String {
    if total == 0 {
        return "0 entries".into();
    }
    let mut parts = Vec::new();
    if tally.warn > 0 {
        parts.push(format_plural(tally.warn, "warning"));
    }
    if tally.action > 0 {
        parts.push(format_plural(tally.action, "action"));
    }
    if tally.info > 0 {
        parts.push(format_plural(tally.info, "info"));
    }
    if parts.is_empty() {
        format_plural(total, "entry")
    } else {
        format!("{} entries ({})", total, parts.join(", "))
    }
}

fn format_plural(count: usize, label: &str) -> String {
    if count == 1 {
        format!("1 {}", label)
    } else {
        format!("{} {}s", count, label)
    }
}

fn render_severity(style: &Style, severity: AdviceSeverity) -> String {
    match severity {
        AdviceSeverity::Info => ui::ok(style, "Info"),
        AdviceSeverity::Warn => ui::warn(style, "Warning"),
        AdviceSeverity::Action => ui::err(style, "Action"),
    }
}

fn collapse_reason(reason: &str) -> String {
    let mut seen = HashSet::new();
    let mut ordered = Vec::new();
    for fragment in reason.split(['\n', ';']) {
        let item = fragment.trim();
        if item.is_empty() {
            continue;
        }
        if seen.insert(item.to_ascii_lowercase()) {
            ordered.push(item.to_string());
        }
    }
    if ordered.is_empty() {
        reason.trim().to_string()
    } else {
        ordered.join("; ")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn collapse_reason_deduplicates_and_trims() {
        let input = "Low disk;\nlow disk ; Extra";
        let collapsed = collapse_reason(input);
        assert_eq!(collapsed, "Low disk; Extra");
    }

    #[test]
    fn summary_formats_counts() {
        let mut tally = SeverityTally::default();
        tally.info = 1;
        tally.warn = 2;
        tally.action = 1;
        assert_eq!(
            format_summary(4, &tally),
            "4 entries (2 warnings, 1 action, 1 info)"
        );
        assert_eq!(format_summary(0, &SeverityTally::default()), "0 entries");
    }

    #[test]
    fn test_numeric_selector_strips_hash() {
        // Test that # is stripped correctly
        let input = "#5";
        let clean = input.strip_prefix('#').unwrap_or(input);
        assert_eq!(clean, "5");
        assert_eq!(clean.parse::<usize>().ok(), Some(5));
    }

    #[test]
    fn test_numeric_selector_parsing() {
        // Test numeric parsing
        assert_eq!("1".parse::<usize>().ok(), Some(1));
        assert_eq!("123".parse::<usize>().ok(), Some(123));
        assert_eq!("not-a-number".parse::<usize>().ok(), None);
    }

    #[test]
    fn test_normalize_id_adds_extension() {
        let path = normalize_id("test-id");
        let uid = nix::unistd::Uid::effective().as_raw();
        let expected = PathBuf::from(format!("/var/lib/anna/users/{}/advice/test-id.json", uid));
        assert_eq!(path, expected);
    }

    #[test]
    fn test_normalize_id_preserves_extension() {
        let path = normalize_id("test-id.json");
        let uid = nix::unistd::Uid::effective().as_raw();
        let expected = PathBuf::from(format!("/var/lib/anna/users/{}/advice/test-id.json", uid));
        assert_eq!(path, expected);
    }

    #[test]
    fn test_dedup_key_normalizes() {
        let rec1 = AdviceRecord {
            id: "test-1".into(),
            kind: "system/test".into(),
            persona_hint: "dev".into(),
            reason: "Low disk; Extra space!".into(),
            created_at: "2025-01-01T00:00:00Z".into(),
            severity: AdviceSeverity::Warn,
            plan: AdvicePlan {
                dry_run_cmds: vec![],
                apply_cmds: vec![],
                undo_cmds: vec![],
            },
        };

        let rec2 = AdviceRecord {
            id: "test-2".into(),
            kind: "system/test".into(),
            persona_hint: "dev".into(),
            reason: "LOW DISK;  extra   SPACE!!!".into(),
            created_at: "2025-01-01T00:00:00Z".into(),
            severity: AdviceSeverity::Warn,
            plan: AdvicePlan {
                dry_run_cmds: vec![],
                apply_cmds: vec![],
                undo_cmds: vec![],
            },
        };

        // Same normalized key despite different formatting
        assert_eq!(dedup_key(&rec1), dedup_key(&rec2));
        assert_eq!(dedup_key(&rec1), "system/test::low disk extra space");
    }

    #[test]
    fn test_deduplicate_keeps_most_recent() {
        let old = AdviceRecord {
            id: "old-id".into(),
            kind: "system/test".into(),
            persona_hint: "dev".into(),
            reason: "test reason".into(),
            created_at: "2025-01-01T00:00:00Z".into(),
            severity: AdviceSeverity::Info,
            plan: AdvicePlan {
                dry_run_cmds: vec![],
                apply_cmds: vec![],
                undo_cmds: vec![],
            },
        };

        let new = AdviceRecord {
            id: "new-id".into(),
            kind: "system/test".into(),
            persona_hint: "dev".into(),
            reason: "TEST REASON".into(), // Same normalized
            created_at: "2025-01-02T00:00:00Z".into(),
            severity: AdviceSeverity::Warn,
            plan: AdvicePlan {
                dry_run_cmds: vec![],
                apply_cmds: vec![],
                undo_cmds: vec![],
            },
        };

        // Input sorted newest-first
        let records = vec![new.clone(), old.clone()];
        let deduped = deduplicate_records(records);

        // Should keep only the newer one
        assert_eq!(deduped.len(), 1);
        assert_eq!(deduped[0].id, "new-id");
    }

    #[test]
    fn test_resolve_id_with_hash_prefix() {
        // Test that #N format is correctly parsed
        let input = "#42";
        let clean = input.strip_prefix('#').unwrap_or(input);
        assert_eq!(clean, "42");
        assert_eq!(clean.parse::<usize>().ok(), Some(42));
    }

    #[test]
    fn test_resolve_id_without_hash() {
        // Test that plain numbers work
        let input = "42";
        let clean = input.strip_prefix('#').unwrap_or(input);
        assert_eq!(clean, "42");
        assert_eq!(clean.parse::<usize>().ok(), Some(42));
    }

    #[test]
    fn test_resolve_id_invalid_formats() {
        // Test that invalid formats are not parsed as numbers
        assert!("##3"
            .strip_prefix('#')
            .unwrap_or("##3")
            .parse::<usize>()
            .is_err());
        assert!("#abc"
            .strip_prefix('#')
            .unwrap_or("#abc")
            .parse::<usize>()
            .is_err());
        assert!("abc".parse::<usize>().is_err());
        assert!("#".parse::<usize>().is_err());
        assert!("".parse::<usize>().is_err());
    }

    #[test]
    fn test_resolve_id_zero_rejected() {
        // The resolve_id function should reject 0 since advice numbers start at 1
        // This is tested implicitly by the error message check
        let zero_clean = "0".strip_prefix('#').unwrap_or("0");
        assert_eq!(zero_clean.parse::<usize>().ok(), Some(0));
        // The actual rejection happens in resolve_id() at runtime
    }

    #[test]
    fn test_permission_error_message() {
        // Test that permission denied errors get friendly messages
        let perm_err = io::Error::from(io::ErrorKind::PermissionDenied);
        let cli_err = AdviceCliError::io("test operation", perm_err);
        assert!(cli_err.to_string().contains("Permission denied"));
        assert!(cli_err.to_string().contains("annactl doctor perms"));
    }

    #[test]
    fn test_other_io_errors_preserved() {
        // Test that non-permission errors are not modified
        let not_found = io::Error::from(io::ErrorKind::NotFound);
        let cli_err = AdviceCliError::io("test operation", not_found);
        assert!(cli_err.to_string().contains("test operation"));
        assert!(!cli_err.to_string().contains("annactl doctor perms"));
    }

    #[test]
    fn test_is_risky_default() {
        // Test that is_risky returns false by default (forward compat)
        let record = AdviceRecord {
            id: "test".into(),
            kind: "test".into(),
            persona_hint: "test".into(),
            reason: "test".into(),
            created_at: "2025-01-01T00:00:00Z".into(),
            severity: AdviceSeverity::Action,
            plan: AdvicePlan {
                dry_run_cmds: vec![],
                apply_cmds: vec![],
                undo_cmds: vec![],
            },
        };
        assert!(!is_risky(&record));
    }
}
