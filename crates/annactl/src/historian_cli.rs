use anna_common::context::db::{ContextDb, DbLocation};
use anyhow::Result;
use rusqlite::Row;

fn read_samples(conn: &rusqlite::Connection, query: &str, limit: usize) -> Vec<Vec<String>> {
    let mut stmt = match conn.prepare(query) {
        Ok(s) => s,
        Err(_) => return Vec::new(),
    };
    let rows = stmt
        .query_map([], |row: &Row| {
            let mut cols = Vec::new();
            for idx in 0..row.column_count() {
                let val: Result<String, _> = row.get(idx);
                cols.push(val.unwrap_or_default());
            }
            Ok(cols)
        })
        .unwrap_or_else(|_| rusqlite::MappedRows::empty());

    rows.flatten().take(limit).collect()
}

pub async fn run_historian_inspect() -> Result<()> {
    let db = ContextDb::open(DbLocation::auto_detect()).await?;

    let (boots, cpu_windows, log_sigs, llm_windows) = db
        .execute(|conn| {
            let boots = read_samples(
                conn,
                "SELECT boot_id, ts_start, boot_health_score FROM boot_sessions ORDER BY ts_start DESC LIMIT 3",
                3,
            );
            let cpu_windows = read_samples(
                conn,
                "SELECT window_start, avg_util_per_core, peak_util_per_core FROM cpu_windows ORDER BY window_start DESC LIMIT 3",
                3,
            );
            let log_sigs = read_samples(
                conn,
                "SELECT signature_hash, last_seen, count FROM log_signatures ORDER BY last_seen DESC LIMIT 3",
                3,
            );
            let llm_windows = read_samples(
                conn,
                "SELECT window_start, model_name, total_calls, failed_calls FROM llm_usage_windows ORDER BY window_start DESC LIMIT 3",
                3,
            );
            Ok((boots, cpu_windows, log_sigs, llm_windows))
        })
        .await?;

    println!("Historian sanity check\n");
    println!("Recent boots:");
    for row in boots {
        println!("  boot_id={} at {} score={}", row.get(0).unwrap_or(&String::new()), row.get(1).unwrap_or(&String::new()), row.get(2).unwrap_or(&String::new()));
    }

    println!("\nCPU windows:");
    for row in cpu_windows {
        println!("  {} avg={} peak={}", row.get(0).unwrap_or(&String::new()), row.get(1).unwrap_or(&String::new()), row.get(2).unwrap_or(&String::new()));
    }

    println!("\nLog signatures:");
    for row in log_sigs {
        println!("  {} last={} count={}", row.get(0).unwrap_or(&String::new()), row.get(1).unwrap_or(&String::new()), row.get(2).unwrap_or(&String::new()));
    }

    println!("\nLLM usage windows:");
    for row in llm_windows {
        println!("  {} model={} calls={} failed={}", row.get(0).unwrap_or(&String::new()), row.get(1).unwrap_or(&String::new()), row.get(2).unwrap_or(&String::new()), row.get(3).unwrap_or(&String::new()));
    }

    Ok(())
}
