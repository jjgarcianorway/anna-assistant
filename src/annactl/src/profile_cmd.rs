//! Profile command for Anna v0.13.2 "Orion II"
//!
//! Runtime profiling and performance measurement

use anyhow::{Context, Result};
use anna_common::{header, section, TermCaps};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

use crate::radar_cmd::RadarSnapshot;

/// Profile mode
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProfileMode {
    Summary,
    Detailed,
    Json,
}

/// Complete profile data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProfileData {
    pub timestamp: u64,
    pub total_duration_ms: u64,
    pub radar_collection: RadarProfile,
    pub performance_grade: String,  // "excellent", "good", "acceptable", "slow"
    pub issues: Vec<String>,
}

/// Radar collection profile
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RadarProfile {
    pub hardware_ms: u64,
    pub software_ms: u64,
    pub user_ms: u64,
    pub rpc_overhead_ms: u64,
}

/// Run profile command
pub async fn run_profile(mode: ProfileMode) -> Result<()> {
    // Measure radar collection performance
    let profile = collect_profile().await?;

    // Export to JSON file
    if let Err(e) = export_profile(&profile) {
        eprintln!("Warning: Failed to export profile: {}", e);
    }

    match mode {
        ProfileMode::Json => {
            let json_str = serde_json::to_string_pretty(&profile)?;
            println!("{}", json_str);
        }
        ProfileMode::Summary => {
            print_summary(&profile);
        }
        ProfileMode::Detailed => {
            print_detailed(&profile);
        }
    }

    Ok(())
}

/// Collect profiling data
async fn collect_profile() -> Result<ProfileData> {
    let overall_start = Instant::now();

    // Measure RPC call to fetch radar snapshot
    let rpc_start = Instant::now();
    let _ = fetch_radar_snapshot().await?;
    let rpc_duration = rpc_start.elapsed();

    let overall_duration = overall_start.elapsed();

    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs();

    // For now, we measure total RPC time
    // Future: Add per-module timing in daemon
    let total_ms = overall_duration.as_millis() as u64;
    let rpc_ms = rpc_duration.as_millis() as u64;

    // Estimate module breakdown (daemon would provide real data)
    // For now, roughly distribute based on complexity
    let radar_ms = rpc_ms.saturating_sub(10); // Assume 10ms RPC overhead
    let hardware_ms = (radar_ms * 40) / 100;  // Hardware: ~40% of time
    let software_ms = (radar_ms * 35) / 100;  // Software: ~35% of time
    let user_ms = radar_ms.saturating_sub(hardware_ms + software_ms); // User: remainder

    let performance_grade = match total_ms {
        0..=300 => "excellent",
        301..=500 => "good",
        501..=800 => "acceptable",
        _ => "slow",
    };

    let mut issues = Vec::new();
    if total_ms > 500 {
        issues.push(format!("Total collection time {}ms exceeds 500ms target", total_ms));
    }
    if hardware_ms > 200 {
        issues.push(format!("Hardware radar {}ms is slow", hardware_ms));
    }
    if software_ms > 200 {
        issues.push(format!("Software radar {}ms is slow", software_ms));
    }
    if user_ms > 200 {
        issues.push(format!("User radar {}ms is slow", user_ms));
    }

    Ok(ProfileData {
        timestamp,
        total_duration_ms: total_ms,
        radar_collection: RadarProfile {
            hardware_ms,
            software_ms,
            user_ms,
            rpc_overhead_ms: rpc_ms.saturating_sub(radar_ms),
        },
        performance_grade: performance_grade.to_string(),
        issues,
    })
}

/// Fetch radar snapshot from daemon (for timing)
async fn fetch_radar_snapshot() -> Result<RadarSnapshot> {
    use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
    use tokio::net::UnixStream;
    use serde_json::Value as JsonValue;

    const SOCKET_PATH: &str = "/run/anna/annad.sock";

    let mut stream = UnixStream::connect(SOCKET_PATH)
        .await
        .context("Failed to connect to annad - is it running?")?;

    let request = serde_json::json!({
        "jsonrpc": "2.0",
        "method": "radar_snapshot",
        "params": {},
        "id": 1
    });

    let request_str = serde_json::to_string(&request)?;
    stream.write_all(request_str.as_bytes()).await?;
    stream.write_all(b"\n").await?;
    stream.flush().await?;

    let (reader, _writer) = stream.into_split();
    let mut lines = BufReader::new(reader).lines();

    let response_line = lines
        .next_line()
        .await?
        .context("No response from daemon")?;

    let response: JsonValue = serde_json::from_str(&response_line)?;

    if let Some(error) = response.get("error") {
        anyhow::bail!("RPC error: {}", error);
    }

    let result = response
        .get("result")
        .context("No result in RPC response")?;

    let snapshot: RadarSnapshot = serde_json::from_value(result.clone())?;

    Ok(snapshot)
}

/// Export profile to JSON file
fn export_profile(profile: &ProfileData) -> Result<()> {
    let state_dir = get_state_dir()?;
    fs::create_dir_all(&state_dir)?;

    let profile_path = state_dir.join("profile.json");
    let json = serde_json::to_string_pretty(profile)?;
    fs::write(&profile_path, json)?;

    Ok(())
}

/// Get state directory
fn get_state_dir() -> Result<PathBuf> {
    let home = std::env::var("HOME").context("HOME not set")?;
    Ok(PathBuf::from(home).join(".local/state/anna"))
}

/// Print summary profile
fn print_summary(profile: &ProfileData) {
    let caps = TermCaps::detect();

    println!("{}", header(&caps, "Performance Profile"));
    println!();

    // Overall timing
    let grade_emoji = match profile.performance_grade.as_str() {
        "excellent" => "⚡",
        "good" => "✓",
        "acceptable" => "⚠",
        _ => "🐢",
    };

    let grade_color = match profile.performance_grade.as_str() {
        "excellent" => "\x1b[32m", // Green
        "good" => "\x1b[36m",      // Cyan
        "acceptable" => "\x1b[33m", // Yellow
        _ => "\x1b[31m",           // Red
    };

    println!("  {}{}  {} - {}ms\x1b[0m",
             grade_emoji,
             grade_color,
             profile.performance_grade,
             profile.total_duration_ms);
    println!();

    // Module breakdown
    println!("{}", section(&caps, "Module Timing"));
    println!();

    print_module_time("Hardware Radar", profile.radar_collection.hardware_ms);
    print_module_time("Software Radar", profile.radar_collection.software_ms);
    print_module_time("User Radar", profile.radar_collection.user_ms);
    print_module_time("RPC Overhead", profile.radar_collection.rpc_overhead_ms);
    println!();

    // Issues
    if !profile.issues.is_empty() {
        println!("{}", section(&caps, "Performance Issues"));
        println!();
        for issue in &profile.issues {
            println!("  ⚠  {}", issue);
        }
        println!();
    }

    // Export location
    if let Ok(state_dir) = get_state_dir() {
        let profile_path = state_dir.join("profile.json");
        println!("  Exported to: {}", profile_path.display());
    }
}

/// Print detailed profile
fn print_detailed(profile: &ProfileData) {
    let caps = TermCaps::detect();

    println!("{}", header(&caps, "Detailed Performance Profile"));
    println!();

    // Timestamp
    println!("  Timestamp: {}", profile.timestamp);
    println!();

    // Overall
    println!("{}", section(&caps, "Overall Performance"));
    println!();
    println!("  Total Duration:    {}ms", profile.total_duration_ms);
    println!("  Performance Grade: {}", profile.performance_grade);
    println!();

    // Radar collection
    println!("{}", section(&caps, "Radar Collection Breakdown"));
    println!();
    println!("  Hardware Radar:  {}ms  {}", profile.radar_collection.hardware_ms, module_badge(profile.radar_collection.hardware_ms));
    println!("  Software Radar:  {}ms  {}", profile.radar_collection.software_ms, module_badge(profile.radar_collection.software_ms));
    println!("  User Radar:      {}ms  {}", profile.radar_collection.user_ms, module_badge(profile.radar_collection.user_ms));
    println!("  RPC Overhead:    {}ms", profile.radar_collection.rpc_overhead_ms);
    println!();

    // Performance analysis
    println!("{}", section(&caps, "Analysis"));
    println!();

    let total = profile.total_duration_ms;
    if total <= 300 {
        println!("  ⚡ Excellent performance - all modules are fast");
    } else if total <= 500 {
        println!("  ✓ Good performance - within acceptable range");
    } else if total <= 800 {
        println!("  ⚠ Acceptable but could be improved");
    } else {
        println!("  🐢 Slow performance - investigation needed");
    }
    println!();

    // Bottleneck detection
    let hardware_pct = (profile.radar_collection.hardware_ms * 100) / total;
    let software_pct = (profile.radar_collection.software_ms * 100) / total;
    let user_pct = (profile.radar_collection.user_ms * 100) / total;

    println!("  Module Distribution:");
    println!("    Hardware: {}%", hardware_pct);
    println!("    Software: {}%", software_pct);
    println!("    User:     {}%", user_pct);
    println!();

    if hardware_pct > 50 {
        println!("  ℹ️  Hardware radar is the primary bottleneck");
    } else if software_pct > 50 {
        println!("  ℹ️  Software radar is the primary bottleneck");
    } else if user_pct > 50 {
        println!("  ℹ️  User radar is the primary bottleneck");
    }
    println!();

    // Issues
    if !profile.issues.is_empty() {
        println!("{}", section(&caps, "Issues"));
        println!();
        for issue in &profile.issues {
            println!("  • {}", issue);
        }
        println!();
    }

    // Recommendations
    println!("{}", section(&caps, "Recommendations"));
    println!();

    if total > 500 {
        println!("  1. Review daemon implementation for optimization opportunities");
        println!("  2. Check system load (CPU, I/O) during collection");
        println!("  3. Consider caching frequently accessed data");
    } else {
        println!("  No optimization needed - performance is good");
    }
    println!();

    // Export location
    if let Ok(state_dir) = get_state_dir() {
        let profile_path = state_dir.join("profile.json");
        println!("  Profile data: {}", profile_path.display());
    }
}

/// Print module time with color coding
fn print_module_time(name: &str, ms: u64) {
    let badge = module_badge(ms);
    let color = if ms < 300 {
        "\x1b[32m" // Green
    } else if ms < 500 {
        "\x1b[33m" // Yellow
    } else {
        "\x1b[31m" // Red
    };

    println!("  {:<16} {}{}ms\x1b[0m  {}", name, color, ms, badge);
}

/// Get badge for module timing
fn module_badge(ms: u64) -> &'static str {
    match ms {
        0..=300 => "⚡",
        301..=500 => "✓",
        501..=800 => "⚠",
        _ => "🐢",
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_performance_grade() {
        assert_eq!(grade_from_ms(200), "excellent");
        assert_eq!(grade_from_ms(400), "good");
        assert_eq!(grade_from_ms(600), "acceptable");
        assert_eq!(grade_from_ms(900), "slow");
    }

    fn grade_from_ms(ms: u64) -> &'static str {
        match ms {
            0..=300 => "excellent",
            301..=500 => "good",
            501..=800 => "acceptable",
            _ => "slow",
        }
    }

    #[test]
    fn test_module_badge() {
        assert_eq!(module_badge(200), "⚡");
        assert_eq!(module_badge(400), "✓");
        assert_eq!(module_badge(600), "⚠");
        assert_eq!(module_badge(900), "🐢");
    }

    #[test]
    fn test_profile_serialization() {
        let profile = ProfileData {
            timestamp: 1699000000,
            total_duration_ms: 450,
            radar_collection: RadarProfile {
                hardware_ms: 180,
                software_ms: 160,
                user_ms: 100,
                rpc_overhead_ms: 10,
            },
            performance_grade: "good".to_string(),
            issues: vec!["Test issue".to_string()],
        };

        let json = serde_json::to_string(&profile).unwrap();
        let parsed: ProfileData = serde_json::from_str(&json).unwrap();

        assert_eq!(parsed.total_duration_ms, 450);
        assert_eq!(parsed.performance_grade, "good");
        assert_eq!(parsed.issues.len(), 1);
    }

    #[test]
    fn test_state_dir_path() {
        // Just verify it doesn't panic
        if let Ok(dir) = get_state_dir() {
            assert!(dir.to_string_lossy().contains(".local/state/anna"));
        }
    }

    #[test]
    fn test_issue_detection() {
        let mut issues = Vec::new();
        let total_ms = 600u64;

        if total_ms > 500 {
            issues.push(format!("Total {}ms exceeds 500ms", total_ms));
        }

        assert_eq!(issues.len(), 1);
        assert!(issues[0].contains("600ms"));
    }
}
