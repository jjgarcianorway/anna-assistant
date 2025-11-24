//! Selftest command - Built-in capability verification (6.3.1)
//!
//! Runs offline self-tests proving what Anna can actually do.
//! No LLM, no network - completely deterministic.

use anna_common::terminal_format as fmt;
use anna_common::selftest::run_selftests;
use anyhow::Result;
use std::time::Instant;

use crate::logging::LogEntry;

/// Execute 'annactl selftest' command
pub async fn execute_selftest_command(
    _req_id: &str,
    _start_time: Instant,
) -> Result<()> {
    println!("{}", fmt::bold("Anna self-test (offline planner capability check)"));
    println!();

    // Run all self-tests
    let results = run_selftests();

    // Display results
    for result in &results {
        let status_icon = if result.passed { "✓" } else { "✗" };
        let status_label = if result.passed {
            fmt::dimmed("OK")
        } else {
            fmt::bold("FAIL")
        };

        println!(
            "[{}]: {} ({})",
            fmt::bold(&result.scenario),
            status_label,
            fmt::dimmed(&result.details)
        );
    }

    println!();

    // Summary
    let passed_count = results.iter().filter(|r| r.passed).count();
    let total_count = results.len();

    if passed_count == total_count {
        println!(
            "{} All {} scenarios passed. Planner behavior matches safety and Arch Wiki-only constraints.",
            fmt::bold("Result:"),
            total_count
        );
        Ok(())
    } else {
        let failed_count = total_count - passed_count;
        eprintln!(
            "{} {} of {} scenarios failed.",
            fmt::bold("Result:"),
            failed_count,
            total_count
        );
        std::process::exit(1);
    }
}
