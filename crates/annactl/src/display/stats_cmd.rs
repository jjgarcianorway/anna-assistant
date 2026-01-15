//! Stats command display - Truthful telemetry from outcome ledger.
//!
//! Phase 23: All stats derived from /var/lib/anna/outcomes.jsonl.
//! No fake XP, no fake titles, no invented percentages.

use anna_shared::outcome_ledger::OutcomeStats;

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

    if let Some(rate) = stats.success_rate() {
        print!("  success rate:  ");
        let color = if rate >= 90.0 { GREEN } else if rate >= 70.0 { YELLOW } else { RED };
        println_colored(&format!("{:.1}%", rate), color);
    } else {
        print!("  success rate:  ");
        println_colored("[!] (no decisive outcomes)", DIM);
    }
    println!();

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
