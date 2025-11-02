//! Anomaly command for Anna v0.14.0 "Orion III"
//!
//! Display detected anomalies

use anyhow::Result;
use anna_common::{header, section, TermCaps};
use std::fs;
use std::path::PathBuf;

use crate::anomaly::{Anomaly, AnomalyDetector};

/// Run anomaly detection command
pub fn run_anomalies(summary: bool, export: Option<String>, json: bool) -> Result<()> {
    let detector = AnomalyDetector::new()?;
    let anomalies = detector.detect_anomalies()?;

    if let Some(export_path) = export {
        let path = PathBuf::from(export_path);
        let json_str = serde_json::to_string_pretty(&anomalies)?;
        fs::write(&path, json_str)?;
        println!("✅ Anomalies exported to: {}", path.display());
        return Ok(());
    }

    if json {
        let json_str = serde_json::to_string_pretty(&anomalies)?;
        println!("{}", json_str);
        return Ok(());
    }

    if summary {
        print_summary(&anomalies);
    } else {
        print_detailed(&anomalies);
    }

    Ok(())
}

/// Print anomaly summary
fn print_summary(anomalies: &[Anomaly]) {
    let caps = TermCaps::detect();

    println!("{}", header(&caps, "Anomaly Detection Summary"));
    println!();

    if anomalies.is_empty() {
        println!("  ✅ No anomalies detected - system metrics are within normal range");
        println!();
        return;
    }

    let critical_count = anomalies.iter().filter(|a| a.is_critical()).count();
    let warning_count = anomalies.iter().filter(|a| a.severity == crate::anomaly::Severity::Warning).count();
    let info_count = anomalies.len() - critical_count - warning_count;

    println!("  Total Anomalies: {}", anomalies.len());
    println!();

    if critical_count > 0 {
        println!("  🚨 Critical: {}", critical_count);
    }
    if warning_count > 0 {
        println!("  ⚠️  Warning:  {}", warning_count);
    }
    if info_count > 0 {
        println!("  ℹ️  Info:     {}", info_count);
    }
    println!();

    // List critical anomalies
    if critical_count > 0 {
        println!("{}", section(&caps, "Critical Anomalies"));
        println!();

        for anomaly in anomalies.iter().filter(|a| a.is_critical()) {
            println!("  • {}", anomaly.description);
            println!("    → {}", anomaly.recommendation());
            println!();
        }
    }

    println!("  For full details: annactl anomalies");
}

/// Print detailed anomaly report
fn print_detailed(anomalies: &[Anomaly]) {
    let caps = TermCaps::detect();

    println!("{}", header(&caps, "Anomaly Detection Report"));
    println!();

    if anomalies.is_empty() {
        println!("  ✅ No anomalies detected");
        println!();
        println!("  All system metrics are within normal statistical range.");
        println!("  Continue monitoring with 'annactl report' for trend tracking.");
        println!();
        return;
    }

    println!("  {} anomal{} detected", anomalies.len(), if anomalies.len() == 1 { "y" } else { "ies" });
    println!();

    for anomaly in anomalies {
        let severity_color = anomaly.severity.color();
        let severity_emoji = anomaly.severity.emoji();

        println!("{}", section(&caps, &format!("{} {}", severity_emoji, anomaly.metric.replace("_", " "))));
        println!();

        println!("  Severity:     {}{:?}\x1b[0m", severity_color, anomaly.severity);
        println!("  Description:  {}", anomaly.description);
        println!();

        println!("  Current:      {:.1}", anomaly.current_value);
        println!("  Expected:     {:.1}", anomaly.expected_value);
        println!("  Deviation:    {:.1}", anomaly.deviation);
        println!("  Z-score:      {:.2}σ", anomaly.z_score);
        println!();

        if anomaly.consecutive_days > 0 {
            println!("  Persistence:  {} consecutive day(s)", anomaly.consecutive_days);
            println!();
        }

        println!("  Recommendation:");
        println!("    {}", anomaly.recommendation());
        println!();
    }

    // Overall advice
    println!("{}", section(&caps, "Next Steps"));
    println!();

    let has_critical = anomalies.iter().any(|a| a.is_critical());

    if has_critical {
        println!("  🚨 CRITICAL anomalies detected - immediate investigation recommended");
        println!();
        println!("  1. Run 'annactl report --verbose' for detailed system analysis");
        println!("  2. Check recent changes: updates, configuration, hardware");
        println!("  3. Review audit log: annactl audit --last 10");
    } else {
        println!("  ⚠️  Monitor these metrics closely");
        println!();
        println!("  • Run 'annactl report' daily to track trends");
        println!("  • Address warnings before they become critical");
        println!("  • Check 'annactl forecast' for predicted trajectory");
    }
    println!();
}
