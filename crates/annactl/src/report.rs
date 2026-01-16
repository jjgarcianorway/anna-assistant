//! Phase 29: Read-only outcome report for operator inspection.
//!
//! This module provides observational summaries of outcome records.
//! It does not modify system behavior, make decisions, or provide recommendations.

use anna_shared::paths::paths;
use std::collections::HashMap;
use std::fs::File;
use std::io::{BufRead, BufReader};

/// Outcome record for reporting (minimal fields needed).
#[derive(Debug)]
struct ReportRecord {
    outcome: String,
    intent: String,
    duration_ms: u64,
    abstention_reason: Option<String>,
    probes_used: Vec<String>,
}

/// Parse a single JSON line into a report record.
fn parse_record(line: &str) -> Option<ReportRecord> {
    let value: serde_json::Value = serde_json::from_str(line).ok()?;

    let outcome = value.get("outcome")?.as_str()?.to_string();
    let intent = value.get("intent")?.as_str()?.to_string();
    let duration_ms = value.get("duration_ms")?.as_u64()?;

    let abstention_reason = value
        .get("abstention_reason")
        .and_then(|v| {
            if v.is_object() {
                // Extract the variant name from the enum
                v.as_object()
                    .and_then(|obj| obj.keys().next().cloned())
            } else {
                None
            }
        });

    let probes_used = value
        .get("probes_used")
        .and_then(|v| v.as_array())
        .map(|arr| {
            arr.iter()
                .filter_map(|v| v.as_str().map(String::from))
                .collect()
        })
        .unwrap_or_default();

    Some(ReportRecord {
        outcome,
        intent,
        duration_ms,
        abstention_reason,
        probes_used,
    })
}

/// Generate and print the outcome report.
pub fn run_report(show_raw: bool, limit: Option<usize>) {
    println!();
    println!("OUTCOME REPORT (OBSERVATIONAL)");
    println!("==============================");
    println!();

    let path = std::env::var("ANNA_OUTCOMES_PATH")
        .map(std::path::PathBuf::from)
        .unwrap_or_else(|_| paths().outcomes_ledger_file());

    let file = match File::open(&path) {
        Ok(f) => f,
        Err(e) => {
            println!("Cannot read outcomes file: {}", e);
            println!("Path: {}", path.display());
            return;
        }
    };

    let reader = BufReader::new(file);
    let mut records: Vec<ReportRecord> = Vec::new();
    let mut parse_errors: usize = 0;

    for line in reader.lines() {
        let line = match line {
            Ok(l) => l,
            Err(_) => {
                parse_errors += 1;
                continue;
            }
        };

        if line.trim().is_empty() {
            continue;
        }

        match parse_record(&line) {
            Some(r) => records.push(r),
            None => parse_errors += 1,
        }
    }

    if records.is_empty() {
        println!("No records found.");
        if parse_errors > 0 {
            println!("Parse errors: {}", parse_errors);
        }
        return;
    }

    // Count by outcome type
    let mut outcome_counts: HashMap<String, usize> = HashMap::new();
    for r in &records {
        *outcome_counts.entry(r.outcome.clone()).or_insert(0) += 1;
    }

    println!("OUTCOMES BY TYPE");
    println!("----------------");
    let mut outcomes: Vec<_> = outcome_counts.iter().collect();
    outcomes.sort_by_key(|(k, _)| k.as_str());
    for (outcome, count) in outcomes {
        println!("  {:<12} {}", outcome, count);
    }
    println!();

    // Count by intent type
    let mut intent_counts: HashMap<String, usize> = HashMap::new();
    for r in &records {
        *intent_counts.entry(r.intent.clone()).or_insert(0) += 1;
    }

    println!("OUTCOMES BY INTENT");
    println!("------------------");
    let mut intents: Vec<_> = intent_counts.iter().collect();
    intents.sort_by_key(|(k, _)| k.as_str());
    for (intent, count) in intents {
        println!("  {:<12} {}", intent, count);
    }
    println!();

    // Duration summary (min/max/count only)
    let durations: Vec<u64> = records.iter().map(|r| r.duration_ms).collect();
    let min_dur = durations.iter().min().copied().unwrap_or(0);
    let max_dur = durations.iter().max().copied().unwrap_or(0);

    println!("DURATION (ms)");
    println!("-------------");
    println!("  count:  {}", durations.len());
    println!("  min:    {}", min_dur);
    println!("  max:    {}", max_dur);
    println!();

    // Abstention reasons
    let abstained: Vec<_> = records
        .iter()
        .filter(|r| r.outcome == "abstained")
        .collect();

    if !abstained.is_empty() {
        let mut reason_counts: HashMap<String, usize> = HashMap::new();
        for r in &abstained {
            let reason = r.abstention_reason.clone().unwrap_or_else(|| "unknown".to_string());
            *reason_counts.entry(reason).or_insert(0) += 1;
        }

        println!("ABSTENTION REASONS");
        println!("------------------");
        let mut reasons: Vec<_> = reason_counts.iter().collect();
        reasons.sort_by_key(|(k, _)| k.as_str());
        for (reason, count) in reasons {
            println!("  {:<20} {}", reason, count);
        }
        println!();
    }

    // Unique probes observed
    let mut probe_set: std::collections::HashSet<String> = std::collections::HashSet::new();
    for r in &records {
        for p in &r.probes_used {
            probe_set.insert(p.clone());
        }
    }

    if !probe_set.is_empty() {
        println!("UNIQUE PROBES OBSERVED");
        println!("----------------------");
        let mut probes: Vec<_> = probe_set.iter().collect();
        probes.sort();
        for probe in probes.iter().take(20) {
            println!("  {}", probe);
        }
        if probes.len() > 20 {
            println!("  ... and {} more", probes.len() - 20);
        }
        println!();
    }

    // Parse errors
    if parse_errors > 0 {
        println!("PARSE ERRORS: {}", parse_errors);
        println!();
    }

    // Raw excerpts if requested
    if show_raw {
        let excerpt_limit = limit.unwrap_or(5);
        println!("RAW RECORD EXCERPTS (last {})", excerpt_limit);
        println!("----------------------------");
        let start = if records.len() > excerpt_limit {
            records.len() - excerpt_limit
        } else {
            0
        };
        for (i, r) in records[start..].iter().enumerate() {
            println!(
                "  [{}] outcome={}, intent={}, duration={}ms",
                start + i + 1,
                r.outcome,
                r.intent,
                r.duration_ms
            );
        }
        println!();
    }

    println!("Total records: {}", records.len());
    println!();
    println!("This report is observational only.");
}
