//! Stats command display - Truthful telemetry from outcome ledger.
//!
//! Phase 23: All stats derived from /var/lib/anna/outcomes.jsonl.
//! Phase 26: Added abstention tracking.
//! Phase 27: Added probe effectiveness display.
//! No fake XP, no fake titles, no invented percentages.

use anna_shared::outcome_ledger::{read_all_outcomes, OutcomeStats};
use anna_shared::probe_stats::{aggregate_probe_effectiveness, top_probes_by_resolution};

use super::colors::*;

/// Print statistics from the outcome ledger.
pub fn print_stats(detailed: bool) {
    println!();
    println_colored("ANNA STATISTICS", BOLD);
    println!();

    // Load stats from outcome ledger
    let stats = match OutcomeStats::load() {
        Ok(s) => s,
        Err(_) => {
            println_colored("  No telemetry data available yet.", DIM);
            println!();
            return;
        }
    };

    if stats.total == 0 {
        println_colored("  No requests tracked yet.", DIM);
        println!();
        return;
    }

    // REQUESTS section
    println_colored("REQUESTS", CYAN);
    println!("  total:         {}", stats.total);
    print!("  read_only:     ");
    if stats.total > 0 {
        let pct = (stats.read_only as f64 / stats.total as f64) * 100.0;
        println!("{} ({:.0}%)", stats.read_only, pct);
    } else {
        println!("{}", stats.read_only);
    }
    print!("  mutating:      ");
    if stats.total > 0 {
        let pct = (stats.mutating as f64 / stats.total as f64) * 100.0;
        println!("{} ({:.0}%)", stats.mutating, pct);
    } else {
        println!("{}", stats.mutating);
    }
    println!();

    // TIMING section
    println_colored("TIMING", CYAN);
    if let Some(avg) = stats.avg_duration_ms() {
        print!("  avg:           ");
        let color = if avg < 1000 { GREEN } else if avg < 5000 { YELLOW } else { DIM };
        println_colored(&format!("{}ms", avg), color);
    } else {
        print!("  avg:           ");
        println_colored("[!]", DIM);
    }

    if detailed {
        if let Some(p50) = stats.percentile_duration_ms(50.0) {
            print!("  p50:           ");
            println_colored(&format!("{}ms", p50), DIM);
        }
        if let Some(p90) = stats.percentile_duration_ms(90.0) {
            print!("  p90:           ");
            let color = if p90 < 5000 { GREEN } else if p90 < 10000 { YELLOW } else { RED };
            println_colored(&format!("{}ms", p90), color);
        }
    }
    println!();

    // ESCALATION section
    println_colored("ESCALATION", CYAN);
    println!("  total:         {}", stats.escalated);
    if let Some(rate) = stats.escalation_rate() {
        print!("  rate:          ");
        let color = if rate < 10.0 { GREEN } else if rate < 30.0 { YELLOW } else { RED };
        println_colored(&format!("{:.1}%", rate), color);
    } else {
        print!("  rate:          ");
        println_colored("[!]", DIM);
    }
    println!();

    // SUCCESS RATE section
    println_colored("OUTCOMES", CYAN);
    print!("  resolved:      ");
    println_colored(&format!("{}", stats.resolved), GREEN);
    print!("  failed:        ");
    if stats.failed > 0 {
        println_colored(&format!("{}", stats.failed), RED);
    } else {
        println_colored(&format!("{}", stats.failed), DIM);
    }
    print!("  cancelled:     ");
    println_colored(&format!("{}", stats.cancelled), DIM);
    print!("  expired:       ");
    println_colored(&format!("{}", stats.expired), DIM);
    // Phase 26: Show abstained count
    print!("  abstained:     ");
    if stats.abstained > 0 {
        println_colored(&format!("{}", stats.abstained), YELLOW);
    } else {
        println_colored(&format!("{}", stats.abstained), DIM);
    }

    if let Some(rate) = stats.success_rate() {
        print!("  success rate:  ");
        let color = if rate >= 90.0 { GREEN } else if rate >= 70.0 { YELLOW } else { RED };
        println_colored(&format!("{:.1}%", rate), color);
    } else {
        print!("  success rate:  ");
        println_colored("[!] (no decisive outcomes)", DIM);
    }
    // Phase 26: Show abstention rate
    if let Some(rate) = stats.abstention_rate() {
        if rate > 0.0 {
            print!("  abstain rate:  ");
            let color = if rate < 5.0 { DIM } else if rate < 15.0 { YELLOW } else { RED };
            println_colored(&format!("{:.1}%", rate), color);
        }
    }
    println!();

    // Phase 27: Show probe effectiveness in detailed mode
    if detailed {
        if let Ok(records) = read_all_outcomes() {
            let probe_stats = aggregate_probe_effectiveness(&records);
            if !probe_stats.is_empty() {
                println_colored("PROBE EFFECTIVENESS", CYAN);
                let top = top_probes_by_resolution(&probe_stats, 5);
                for probe in top {
                    let rate_pct = (probe.resolution_rate * 100.0) as u32;
                    let color = if rate_pct >= 90 { GREEN } else if rate_pct >= 70 { YELLOW } else { RED };
                    print!("  ");
                    print_colored(&format!("{:<20}", truncate_probe(&probe.probe_pattern, 20)), DIM);
                    print_colored(&format!(" {:>3}%", rate_pct), color);
                    println_colored(&format!("  (n={})", probe.sample_count), DIM);
                }
                println!();
            }
        }
    }

    // Show ledger info in detailed mode
    if detailed {
        let path = anna_shared::paths::paths().outcomes_ledger_file();
        if path.exists() {
            if let Ok(metadata) = std::fs::metadata(&path) {
                println_colored("LEDGER", CYAN);
                println!("  path:          {}", path.display());
                println!("  size:          {} bytes", metadata.len());
                println!();
            }
        }
    }
}

/// Truncate probe pattern for display.
fn truncate_probe(s: &str, max_len: usize) -> String {
    if s.len() <= max_len {
        s.to_string()
    } else {
        format!("{}...", &s[..max_len - 3])
    }
}
