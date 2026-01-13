//! Stats command display for detailed RPG statistics.

use anna_shared::memory::memory_path;
use anna_shared::paths::Paths;
use anna_shared::stats::PersistentStats;
use anna_shared::status::RpgStats;

use super::colors::*;
use super::formatting::format_duration;

/// Print comprehensive stats (full RPG system per spec)
pub fn print_stats(detailed: bool) {
    let mem_path = memory_path();
    let p = anna_shared::paths::paths();

    println!();
    println_colored("ANNA STATISTICS", BOLD);
    println!();

    // PROGRESSION - RPG stats
    let stats = PersistentStats::load().unwrap_or_default();
    let rpg = stats.get_rpg_stats();

    println_colored("PROGRESSION", CYAN);
    print!("  title:         ");
    println_colored(&format!("\"{}\"", rpg.title), MAGENTA);

    print!("  xp:            ");
    println!("{}", rpg.xp_bar());
    println!();

    // REQUESTS
    println_colored("REQUESTS", CYAN);
    println!("  total:         {}", rpg.total_questions);

    // Solved alone (instant + memory, without LLM)
    let solved_alone = rpg.instant_answers + rpg.memory_answers;
    if rpg.total_questions > 0 {
        print!("  solved alone:  ");
        let alone_pct = solved_alone as f64 / rpg.total_questions as f64 * 100.0;
        print_colored(&format!("{}", solved_alone), if alone_pct > 50.0 { GREEN } else { DIM });
        println_colored(&format!(" ({:.0}%)", alone_pct), DIM);
    }

    // Breakdown
    if detailed && rpg.total_questions > 0 {
        print!("    instant:     ");
        let instant_pct = rpg.instant_answers as f64 / rpg.total_questions as f64 * 100.0;
        print_colored(&format!("{} ({:.0}%)", rpg.instant_answers, instant_pct), if instant_pct > 50.0 { GREEN } else { DIM });
        println!();
        print!("    memory:      ");
        let memory_pct = rpg.memory_answers as f64 / rpg.total_questions as f64 * 100.0;
        print_colored(&format!("{} ({:.0}%)", rpg.memory_answers, memory_pct), DIM);
        println!();
        print!("    llm:         ");
        let llm_pct = rpg.llm_answers as f64 / rpg.total_questions as f64 * 100.0;
        print_colored(&format!("{} ({:.0}%)", rpg.llm_answers, llm_pct), DIM);
        println!();
    }
    println!();

    // PERFORMANCE
    println_colored("PERFORMANCE", CYAN);

    print!("  reliability:   ");
    let rel_pct = rpg.reliability * 100.0;
    let rel_color = if rel_pct >= 95.0 { GREEN } else if rel_pct >= 80.0 { YELLOW } else { RED };
    println_colored(&format!("{:.1}%", rel_pct), rel_color);

    if rpg.avg_response_ms > 0 {
        print!("  avg response:  ");
        let avg_color = if rpg.avg_response_ms < 100 { GREEN } else if rpg.avg_response_ms < 1000 { YELLOW } else { DIM };
        println_colored(&format!("{}ms", rpg.avg_response_ms), avg_color);
    }

    if detailed {
        if rpg.fastest_response_ms > 0 {
            print!("  fastest:       ");
            println_colored(&format!("{}ms", rpg.fastest_response_ms), GREEN);
        }

        if rpg.slowest_response_ms > 0 {
            print!("  slowest:       ");
            println_colored(&format!("{}ms", rpg.slowest_response_ms), DIM);
        }
    }
    println!();

    // LEARNING
    println_colored("LEARNING", CYAN);

    let (exp_count, pattern_count, cluster_count, memory_hits, memory_misses) = load_memory_stats(&mem_path);

    println!("  recipes:       {} learned", rpg.recipes_learned);
    println!("  experiences:   {}", exp_count);
    println!("  patterns:      {}", pattern_count);

    if detailed {
        println!("  clusters:      {}", cluster_count);

        let total_queries = memory_hits + memory_misses;
        if total_queries > 0 {
            let hit_rate = memory_hits as f64 / total_queries as f64 * 100.0;
            print!("  memory hits:   ");
            let rate_color = if hit_rate >= 50.0 { GREEN } else if hit_rate >= 25.0 { YELLOW } else { DIM };
            print_colored(&format!("{:.1}%", hit_rate), rate_color);
            println_colored(&format!(" ({}/{})", memory_hits, total_queries), DIM);
        }
    }
    println!();

    // TICKET METRICS
    print_ticket_metrics(p);

    // ACTIVITY
    if detailed {
        print_activity(p, &rpg);
    }
}

fn print_ticket_metrics(p: &Paths) {
    let tickets_path = p.tickets_file();
    if !tickets_path.exists() { return; }

    let content = match std::fs::read_to_string(&tickets_path) {
        Ok(c) => c,
        Err(_) => return,
    };
    let store = match serde_json::from_str::<serde_json::Value>(&content) {
        Ok(s) => s,
        Err(_) => return,
    };

    println_colored("TICKETS", CYAN);

    let total_resolved = store.get("total_resolved").and_then(|v| v.as_u64()).unwrap_or(0);
    let total_failed = store.get("total_failed").and_then(|v| v.as_u64()).unwrap_or(0);
    let total_escalated = store.get("total_escalated").and_then(|v| v.as_u64()).unwrap_or(0);

    println_colored("  by state:", DIM);
    print!("    resolved:    ");
    println_colored(&format!("{}", total_resolved), GREEN);
    print!("    failed:      ");
    println_colored(&format!("{}", total_failed), if total_failed > 0 { RED } else { DIM });
    print!("    escalated:   ");
    println_colored(&format!("{}", total_escalated), if total_escalated > 0 { YELLOW } else { DIM });

    if total_resolved > 0 || total_failed > 0 {
        let success_rate = total_resolved as f64 / (total_resolved + total_failed).max(1) as f64 * 100.0;
        print!("  success rate:  ");
        let rate_color = if success_rate >= 90.0 { GREEN } else if success_rate >= 70.0 { YELLOW } else { RED };
        println_colored(&format!("{:.1}%", success_rate), rate_color);
    }

    // Resolution time statistics
    if let Some(tickets_arr) = store.get("tickets").and_then(|v| v.as_array()) {
        let resolution_times: Vec<i64> = tickets_arr
            .iter()
            .filter_map(|t| {
                let created = t.get("created_at")?.as_str()?;
                let resolved = t.get("resolved_at")?.as_str()?;
                let created_dt = chrono::DateTime::parse_from_rfc3339(created).ok()?;
                let resolved_dt = chrono::DateTime::parse_from_rfc3339(resolved).ok()?;
                Some((resolved_dt - created_dt).num_seconds())
            })
            .filter(|&s| s >= 0)
            .collect();

        if !resolution_times.is_empty() {
            println!();
            println_colored("  resolution times:", DIM);

            let avg = resolution_times.iter().sum::<i64>() as f64 / resolution_times.len() as f64;
            print!("    average:     ");
            println_colored(&format_duration(avg as u64), if avg < 30.0 { GREEN } else if avg < 120.0 { YELLOW } else { DIM });

            if let Some(&min) = resolution_times.iter().min() {
                print!("    fastest:     ");
                println_colored(&format_duration(min as u64), GREEN);
            }

            if let Some(&max) = resolution_times.iter().max() {
                print!("    slowest:     ");
                println_colored(&format_duration(max as u64), DIM);
            }
        }
    }
    println!();
}

fn print_activity(p: &Paths, rpg: &RpgStats) {
    println_colored("ACTIVITY", CYAN);

    let fix_history_path = p.fix_history_file();
    let fixes_count = count_json_array(&fix_history_path, "fixes");
    println!("  fixes applied: {}", fixes_count);

    let deps_path = p.installed_deps_file();
    let helpers_count = if deps_path.exists() {
        std::fs::read_to_string(&deps_path).ok()
            .map(|c| c.lines().filter(|l| !l.is_empty()).count())
            .unwrap_or(0)
    } else { 0 };
    println!("  helpers:       {} installed", helpers_count);

    if rpg.total_uptime_secs > 0 {
        print!("  total uptime:  ");
        println_colored(&format_duration(rpg.total_uptime_secs), DIM);
    }
    println!();
}

/// Load memory statistics from file
fn load_memory_stats(path: &std::path::Path) -> (usize, usize, usize, u64, u64) {
    if !path.exists() { return (0, 0, 0, 0, 0); }

    let content = match std::fs::read_to_string(path) {
        Ok(c) => c,
        Err(_) => return (0, 0, 0, 0, 0),
    };
    let memory = match serde_json::from_str::<serde_json::Value>(&content) {
        Ok(m) => m,
        Err(_) => return (0, 0, 0, 0, 0),
    };

    let experiences = memory.get("experiences")
        .and_then(|v| v.as_array())
        .map(|a| a.len())
        .unwrap_or(0);
    let patterns = memory.get("patterns")
        .and_then(|v| v.as_array())
        .map(|a| a.len())
        .unwrap_or(0);
    let clusters = memory.get("clusters")
        .and_then(|v| v.as_array())
        .map(|a| a.len())
        .unwrap_or(0);
    let stats = memory.get("stats");
    let hits = stats
        .and_then(|s| s.get("memory_hits"))
        .and_then(|v| v.as_u64())
        .unwrap_or(0);
    let misses = stats
        .and_then(|s| s.get("memory_misses"))
        .and_then(|v| v.as_u64())
        .unwrap_or(0);

    (experiences, patterns, clusters, hits, misses)
}

/// Count items in a JSON array field
fn count_json_array(path: &std::path::Path, field: &str) -> usize {
    if !path.exists() { return 0; }

    std::fs::read_to_string(path).ok()
        .and_then(|c| serde_json::from_str::<serde_json::Value>(&c).ok())
        .and_then(|h| h.get(field).and_then(|f| f.as_array()).map(|a| a.len()))
        .unwrap_or(0)
}
