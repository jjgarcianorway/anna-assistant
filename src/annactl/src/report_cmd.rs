//! Report command for annactl v0.13.0 "Orion II"
//!
//! Generate natural language system health reports

use anyhow::{Context, Result};
use anna_common::beautiful::colors::*;
use anna_common::beautiful::boxes::*;
use serde::{Deserialize, Serialize};
use serde_json::Value as JsonValue;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::UnixStream;

use crate::advisor::{Advisor, Recommendation};
use crate::history::TrendSummary;
use crate::radar_cmd::{HardwareRadar, RadarSnapshot, SoftwareRadar, UserRadar};

const SOCKET_PATH: &str = "/run/anna/annad.sock";

/// Report mode
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ReportMode {
    Short,
    Verbose,
    Json,
}

/// System report with narrative and recommendations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemReport {
    pub narrative: String,
    pub overall_health: u8,
    pub radar_scores: RadarScores,
    pub recommendations: Vec<Recommendation>,
    pub trends: Option<TrendSummary>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RadarScores {
    pub hardware: u8,
    pub software: u8,
    pub user: u8,
}

/// Run report command
pub async fn run_report(mode: ReportMode) -> Result<()> {
    use crate::history::{HistoryEntry, HistoryManager};
    use std::time::{SystemTime, UNIX_EPOCH};

    let snapshot = fetch_radar_snapshot().await?;

    // Load history and compute trends
    let history_mgr = HistoryManager::new()?;
    let trends = history_mgr.compute_trends().ok().flatten();

    let mut report = generate_report(&snapshot);
    report.trends = trends;

    // Record this snapshot to history
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs();

    let entry = HistoryEntry {
        timestamp,
        hardware_score: snapshot.hardware.overall,
        software_score: snapshot.software.overall,
        user_score: snapshot.user.overall,
        overall_score: report.overall_health,
        top_recommendations: report.recommendations
            .iter()
            .take(3)
            .map(|r| r.title.clone())
            .collect(),
    };

    // Best-effort recording (don't fail if history write fails)
    let _ = history_mgr.record(entry);

    match mode {
        ReportMode::Json => {
            let json_str = serde_json::to_string_pretty(&report)?;
            println!("{}", json_str);
        }
        ReportMode::Short => {
            print_short_report(&report);
        }
        ReportMode::Verbose => {
            print_verbose_report(&report);
        }
    }

    Ok(())
}

/// Fetch radar snapshot from daemon
async fn fetch_radar_snapshot() -> Result<RadarSnapshot> {
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

/// Generate system report with narrative and recommendations
fn generate_report(snapshot: &RadarSnapshot) -> SystemReport {
    let overall_health = (snapshot.hardware.overall as u16
                        + snapshot.software.overall as u16
                        + snapshot.user.overall as u16) / 3;

    let narrative = generate_narrative(snapshot, overall_health as u8);
    let recommendations = generate_recommendations(snapshot);

    SystemReport {
        narrative,
        overall_health: overall_health as u8,
        radar_scores: RadarScores {
            hardware: snapshot.hardware.overall,
            software: snapshot.software.overall,
            user: snapshot.user.overall,
        },
        recommendations,
        trends: None, // TODO: Historical tracking not yet implemented
    }
}

/// Generate natural language narrative
fn generate_narrative(snapshot: &RadarSnapshot, overall: u8) -> String {
    let mut parts = Vec::new();

    // Opening statement
    let opening = match overall {
        9..=10 => "Your system is in excellent health",
        7..=8 => "Your system is in good shape with minor areas for improvement",
        5..=6 => "Your system is functional but needs attention in several areas",
        3..=4 => "Your system has significant issues that need addressing",
        _ => "Your system requires immediate attention",
    };
    parts.push(opening.to_string());

    // Hardware assessment
    let hw_desc = describe_hardware(&snapshot.hardware);
    parts.push(hw_desc);

    // Software assessment
    let sw_desc = describe_software(&snapshot.software);
    parts.push(sw_desc);

    // User assessment
    let user_desc = describe_user(&snapshot.user);
    parts.push(user_desc);

    parts.join(". ") + "."
}

/// Describe hardware status
fn describe_hardware(hw: &HardwareRadar) -> String {
    let mut issues = Vec::new();
    let mut strengths = Vec::new();

    if hw.cpu_throughput >= 8 { strengths.push("excellent CPU performance"); }
    else if hw.cpu_throughput <= 4 { issues.push("limited CPU resources"); }

    if hw.cpu_thermal <= 5 { issues.push("high CPU temperatures"); }
    else if hw.cpu_thermal >= 8 { strengths.push("good thermal management"); }

    if hw.memory <= 4 { issues.push("memory pressure"); }
    else if hw.memory >= 8 { strengths.push("ample memory"); }

    if hw.disk_health <= 3 { issues.push("disk health concerns"); }
    if hw.disk_free <= 4 { issues.push("low disk space"); }

    if !issues.is_empty() {
        format!("Hardware shows {}", issues.join(" and "))
    } else if !strengths.is_empty() {
        format!("Hardware is solid with {}", strengths.join(" and "))
    } else {
        "Hardware is in acceptable condition".to_string()
    }
}

/// Describe software status
fn describe_software(sw: &SoftwareRadar) -> String {
    let mut issues = Vec::new();
    let mut strengths = Vec::new();

    if sw.os_freshness <= 4 { issues.push("outdated packages"); }
    else if sw.os_freshness >= 9 { strengths.push("up-to-date software"); }

    if sw.security <= 4 { issues.push("weak security posture"); }
    else if sw.security >= 8 { strengths.push("good security hardening"); }

    if sw.backups <= 3 { issues.push("inadequate backups"); }

    if sw.services <= 5 { issues.push("service failures"); }

    if !issues.is_empty() {
        format!("Software maintenance needs work: {}", issues.join(", "))
    } else if !strengths.is_empty() {
        format!("Software is well-maintained with {}", strengths.join(" and "))
    } else {
        "Software is in decent shape".to_string()
    }
}

/// Describe user habits
fn describe_user(user: &UserRadar) -> String {
    let mut issues = Vec::new();
    let mut strengths = Vec::new();

    if user.updates <= 4 { issues.push("delayed updates"); }
    else if user.updates >= 8 { strengths.push("prompt updates"); }

    if user.backups <= 3 { issues.push("inconsistent backups"); }

    if user.workspace <= 4 { issues.push("workspace clutter"); }

    if user.risk <= 4 { issues.push("elevated sudo usage"); }
    else if user.risk >= 8 { strengths.push("cautious security habits"); }

    if !issues.is_empty() {
        format!("User habits could improve: {}", issues.join(", "))
    } else if !strengths.is_empty() {
        format!("User habits are excellent with {}", strengths.join(" and "))
    } else {
        "User habits are reasonable".to_string()
    }
}

/// Generate actionable recommendations using centralized advisor
fn generate_recommendations(snapshot: &RadarSnapshot) -> Vec<Recommendation> {
    // Use centralized advisor to generate recommendations
    match Advisor::new() {
        Ok(advisor) => advisor.top_recommendations(snapshot, 5),
        Err(_) => {
            // Fallback to empty if advisor fails
            eprintln!("Warning: Failed to initialize advisor");
            Vec::new()
        }
    }
}

/// Print short report (one-page summary)
fn print_short_report(report: &SystemReport) {
    // Beautiful header
    println!("\n{DIM}{TOP_LEFT}{}{TOP_RIGHT}",
        HORIZONTAL.repeat(70));
    println!("{VERTICAL}{RESET}  {CYAN}{BOLD}📊  System Health Report{RESET}                                   {DIM}{VERTICAL}{RESET}");
    println!("{VERTICAL}{RESET}                                                                      {DIM}{VERTICAL}{RESET}");

    // Overall health with beautiful bar
    let (health_color, health_emoji) = match report.overall_health {
        9..=10 => (GREEN, "✨"),
        7..=8 => (CYAN, "✓"),
        5..=6 => (YELLOW, "⚠"),
        3..=4 => (MAGENTA, "⚡"),
        _ => (RED, "✗"),
    };

    let health_bar = "█".repeat(report.overall_health as usize);
    let empty_bar = "░".repeat(10 - report.overall_health as usize);

    println!("{VERTICAL}{RESET}  {BOLD}Overall Health:{RESET} {health_color}{health_emoji} {}/10{RESET}  [{health_color}{health_bar}{GRAY}{empty_bar}{RESET}]  {DIM}{VERTICAL}{RESET}",
        report.overall_health);
    println!("{VERTICAL}{RESET}                                                                      {DIM}{VERTICAL}{RESET}");

    // Narrative in a nice box
    println!("{VERTICAL}{RESET}  {BOLD}Summary{RESET}                                                           {DIM}{VERTICAL}{RESET}");

    // Word wrap narrative to 62 chars
    let words: Vec<&str> = report.narrative.split_whitespace().collect();
    let mut current_line = String::new();

    for word in words {
        if current_line.len() + word.len() + 1 > 62 {
            println!("{VERTICAL}{RESET}  {current_line:<62}  {DIM}{VERTICAL}{RESET}");
            current_line = word.to_string();
        } else {
            if !current_line.is_empty() {
                current_line.push(' ');
            }
            current_line.push_str(word);
        }
    }
    if !current_line.is_empty() {
        println!("{VERTICAL}{RESET}  {current_line:<62}  {DIM}{VERTICAL}{RESET}");
    }

    println!("{VERTICAL}{RESET}                                                                      {DIM}{VERTICAL}{RESET}");

    // Radar scores with color coding
    println!("{VERTICAL}{RESET}  {BOLD}Radar Scores{RESET}                                                      {DIM}{VERTICAL}{RESET}");

    let hw_color = if report.radar_scores.hardware >= 7 { GREEN } else if report.radar_scores.hardware >= 5 { YELLOW } else { RED };
    let sw_color = if report.radar_scores.software >= 7 { GREEN } else if report.radar_scores.software >= 5 { YELLOW } else { RED };
    let usr_color = if report.radar_scores.user >= 7 { GREEN } else if report.radar_scores.user >= 5 { YELLOW } else { RED };

    println!("{VERTICAL}{RESET}  {BOLD}Hardware:{RESET} {hw_color}{}/10{RESET}  {DIM}│{RESET}  {BOLD}Software:{RESET} {sw_color}{}/10{RESET}  {DIM}│{RESET}  {BOLD}User:{RESET} {usr_color}{}/10{RESET}    {DIM}{VERTICAL}{RESET}",
        report.radar_scores.hardware,
        report.radar_scores.software,
        report.radar_scores.user);
    println!("{VERTICAL}{RESET}                                                                      {DIM}{VERTICAL}{RESET}");

    // Trends (if available)
    if let Some(ref trends) = report.trends {
        let (trend_emoji, trend_color) = match trends.direction.as_str() {
            "improving" => ("↑", GREEN),
            "declining" => ("↓", RED),
            _ => ("→", CYAN),
        };

        println!("{VERTICAL}{RESET}  {BOLD}Trends{RESET}                                                            {DIM}{VERTICAL}{RESET}");
        println!("{VERTICAL}{RESET}  {trend_color}{trend_emoji}  {}{RESET}                                                   {DIM}{VERTICAL}{RESET}",
            trends.direction);

        if trends.change_7d != 0 {
            let change_color = if trends.change_7d > 0 { GREEN } else { RED };
            println!("{VERTICAL}{RESET}  7-day:  {change_color}{:+}/10{RESET}                                                {DIM}{VERTICAL}{RESET}",
                trends.change_7d);
        }
        if trends.change_30d != 0 {
            let change_color = if trends.change_30d > 0 { GREEN } else { RED };
            println!("{VERTICAL}{RESET}  30-day: {change_color}{:+}/10{RESET}                                               {DIM}{VERTICAL}{RESET}",
                trends.change_30d);
        }
        println!("{VERTICAL}{RESET}                                                                      {DIM}{VERTICAL}{RESET}");
    }

    // Top recommendations
    if !report.recommendations.is_empty() {
        println!("{VERTICAL}{RESET}  {BOLD}💡 Recommendations{RESET}                                                {DIM}{VERTICAL}{RESET}");
        println!("{VERTICAL}{RESET}                                                                      {DIM}{VERTICAL}{RESET}");

        for (i, rec) in report.recommendations.iter().take(3).enumerate() {
            let priority_color = match rec.priority.as_str() {
                "critical" => RED,
                "high" => YELLOW,
                "medium" => CYAN,
                _ => WHITE,
            };

            println!("{VERTICAL}{RESET}  {BOLD}{}. {}{RESET} {priority_color}[{}]{RESET} {}               {DIM}{VERTICAL}{RESET}",
                i + 1,
                rec.emoji,
                rec.priority.to_uppercase(),
                rec.title);
            println!("{VERTICAL}{RESET}     {DIM}→{RESET} {}                            {DIM}{VERTICAL}{RESET}",
                rec.action);
            println!("{VERTICAL}{RESET}     {DIM}Impact:{RESET} {}                       {DIM}{VERTICAL}{RESET}",
                rec.impact);

            if i < 2 && i < report.recommendations.len() - 1 {
                println!("{VERTICAL}{RESET}                                                                      {DIM}{VERTICAL}{RESET}");
            }
        }
    }

    // Beautiful footer
    println!("{DIM}{BOTTOM_LEFT}{}{BOTTOM_RIGHT}",
        HORIZONTAL.repeat(70));
    println!("{RESET}");
}

/// Print verbose report (detailed analysis)
fn print_verbose_report(report: &SystemReport) {
    print_short_report(report);

    // Additional details in verbose mode
    println!("\n{DIM}{TOP_LEFT}{}{TOP_RIGHT}",
        HORIZONTAL.repeat(70));
    println!("{VERTICAL}{RESET}  {CYAN}{BOLD}📋  Detailed Analysis{RESET}                                        {DIM}{VERTICAL}{RESET}");
    println!("{VERTICAL}{RESET}                                                                      {DIM}{VERTICAL}{RESET}");

    for rec in &report.recommendations {
        println!("{VERTICAL}{RESET}  {BOLD}{} {}{RESET}                                              {DIM}{VERTICAL}{RESET}",
            rec.emoji, rec.title);
        println!("{VERTICAL}{RESET}    {DIM}Category:{RESET} {}                                       {DIM}{VERTICAL}{RESET}",
            rec.category);
        println!("{VERTICAL}{RESET}    {DIM}Priority:{RESET} {}                                       {DIM}{VERTICAL}{RESET}",
            rec.priority);
        println!("{VERTICAL}{RESET}    {DIM}Reason:{RESET}   {}                                       {DIM}{VERTICAL}{RESET}",
            rec.reason);
        println!("{VERTICAL}{RESET}    {DIM}Action:{RESET}   {}                                       {DIM}{VERTICAL}{RESET}",
            rec.action);
        println!("{VERTICAL}{RESET}    {DIM}Impact:{RESET}   {}                                       {DIM}{VERTICAL}{RESET}",
            rec.impact);
        println!("{VERTICAL}{RESET}                                                                      {DIM}{VERTICAL}{RESET}");
    }

    println!("{DIM}{BOTTOM_LEFT}{}{BOTTOM_RIGHT}",
        HORIZONTAL.repeat(70));
    println!("{RESET}");
}
