//! Anomaly Analysis - Provide root cause analysis for anomalies.
//!
//! Philosophy: Don't just say "memory is high" - explain WHY and what to do about it.

use anyhow::Result;
use tracing::{debug, info};

/// Root cause analysis for an anomaly.
#[derive(Debug, Clone)]
pub struct RootCauseAnalysis {
    /// Anomaly description
    pub anomaly: String,
    /// Root causes found
    pub causes: Vec<Cause>,
    /// Recommended actions
    pub recommendations: Vec<String>,
    /// Confidence in analysis (0.0-1.0)
    pub confidence: f32,
}

/// A potential cause of an anomaly.
#[derive(Debug, Clone)]
pub struct Cause {
    /// What caused it
    pub description: String,
    /// Supporting evidence
    pub evidence: String,
    /// Likelihood this is the root cause (0.0-1.0)
    pub likelihood: f32,
}

/// Analyze a memory anomaly.
pub async fn analyze_memory_anomaly(current_pct: f32, baseline_pct: f32) -> Result<RootCauseAnalysis> {
    info!("Analyzing memory anomaly: {:.1}% (baseline {:.1}%)", current_pct, baseline_pct);

    let mut causes = Vec::new();
    let mut recommendations = Vec::new();

    // Check top memory consumers
    if let Ok(output) = crate::core_loop::execute_command("ps aux --sort=-%mem | head -10") {
        let lines: Vec<&str> = output.lines().skip(1).take(5).collect();

        for line in lines {
            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.len() >= 11 {
                let mem_pct: f32 = parts[3].parse().unwrap_or(0.0);
                let process = parts[10];

                if mem_pct > 10.0 {
                    causes.push(Cause {
                        description: format!("{} using {:.1}% memory", process, mem_pct),
                        evidence: format!("Process consuming {} of total memory", format_percentage(mem_pct)),
                        likelihood: (mem_pct / current_pct).min(1.0),
                    });
                }
            }
        }
    }

    // Check for memory leaks (growing processes)
    if let Ok(output) = crate::core_loop::execute_command("ps -eo pid,comm,%mem --sort=-%mem | head -10") {
        debug!("Memory leak check: {}", output);
        // Track process memory growth over time (would need historical data)
    }

    // Generate recommendations
    if !causes.is_empty() {
        let top_cause = &causes[0];

        if top_cause.description.contains("chrome") || top_cause.description.contains("firefox") {
            recommendations.push("Close unused browser tabs or restart browser".to_string());
        } else if top_cause.description.contains("docker") {
            recommendations.push("Check for runaway Docker containers: docker stats".to_string());
        } else if top_cause.description.contains("java") {
            recommendations.push("Java process may have memory leak - consider restarting".to_string());
        } else {
            recommendations.push(format!("Investigate {} process", top_cause.description.split_whitespace().next().unwrap_or("unknown")));
        }
    }

    recommendations.push("Check 'htop' for detailed memory breakdown".to_string());

    let confidence = if causes.is_empty() { 0.3 } else { 0.8 };

    Ok(RootCauseAnalysis {
        anomaly: format!("Memory at {:.1}% ({:.0}% above baseline)", current_pct, current_pct - baseline_pct),
        causes,
        recommendations,
        confidence,
    })
}

/// Analyze a disk I/O anomaly.
pub async fn analyze_disk_io_anomaly() -> Result<RootCauseAnalysis> {
    info!("Analyzing disk I/O anomaly");

    let mut causes = Vec::new();
    let mut recommendations = Vec::new();

    // Check for processes doing heavy I/O
    if let Ok(output) = crate::core_loop::execute_command("iotop -b -n 1 2>/dev/null || pidstat -d 1 1 2>/dev/null") {
        let lines: Vec<&str> = output.lines().collect();

        for line in lines.iter().take(10) {
            if line.contains("kB/s") || line.contains("MB/s") {
                // Parse I/O activity
                debug!("I/O line: {}", line);
            }
        }
    }

    // Check for common I/O intensive operations
    if let Ok(output) = crate::core_loop::execute_command("ps aux | grep -E '(rsync|cp|dd|tar|updatedb|baloo|tracker)'") {
        let processes: Vec<&str> = output.lines().collect();

        for process in processes {
            if !process.contains("grep") {
                let parts: Vec<&str> = process.split_whitespace().collect();
                if parts.len() >= 11 {
                    causes.push(Cause {
                        description: format!("Background process: {}", parts[10]),
                        evidence: "Process performing file operations".to_string(),
                        likelihood: 0.7,
                    });
                }
            }
        }
    }

    if causes.is_empty() {
        causes.push(Cause {
            description: "Unknown process performing heavy disk I/O".to_string(),
            evidence: "System showing elevated I/O activity".to_string(),
            likelihood: 0.5,
        });
        recommendations.push("Run 'iotop' or 'pidstat -d' to identify I/O intensive process".to_string());
    } else {
        let top_cause = &causes[0];
        if top_cause.description.contains("baloo") || top_cause.description.contains("tracker") {
            recommendations.push("File indexing in progress - pause with 'balooctl suspend' or wait for completion".to_string());
        } else if top_cause.description.contains("updatedb") {
            recommendations.push("mlocate database update running - wait for completion or disable mlocate".to_string());
        }
    }

    let confidence = if causes.len() > 1 { 0.8 } else { 0.5 };

    Ok(RootCauseAnalysis {
        anomaly: "Disk I/O activity elevated".to_string(),
        causes,
        recommendations,
        confidence,
    })
}

/// Analyze a CPU load anomaly.
pub async fn analyze_cpu_anomaly(current_load: f32, baseline_load: f32) -> Result<RootCauseAnalysis> {
    info!("Analyzing CPU anomaly: {:.2} (baseline {:.2})", current_load, baseline_load);

    let mut causes = Vec::new();
    let mut recommendations = Vec::new();

    // Check top CPU consumers
    if let Ok(output) = crate::core_loop::execute_command("ps aux --sort=-%cpu | head -10") {
        let lines: Vec<&str> = output.lines().skip(1).take(5).collect();

        for line in lines {
            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.len() >= 11 {
                let cpu_pct: f32 = parts[2].parse().unwrap_or(0.0);
                let process = parts[10];

                if cpu_pct > 50.0 {
                    causes.push(Cause {
                        description: format!("{} using {:.1}% CPU", process, cpu_pct),
                        evidence: format!("Process consuming {} of single core", format_percentage(cpu_pct)),
                        likelihood: 0.9,
                    });
                }
            }
        }
    }

    if !causes.is_empty() {
        let top_cause = &causes[0];

        if top_cause.description.contains("compile") || top_cause.description.contains("gcc") || top_cause.description.contains("cargo") {
            recommendations.push("Compilation in progress - normal high CPU usage".to_string());
        } else if top_cause.description.contains("ffmpeg") || top_cause.description.contains("handbrake") {
            recommendations.push("Video encoding in progress - normal high CPU usage".to_string());
        } else {
            recommendations.push(format!("Investigate high CPU usage by {}", top_cause.description.split_whitespace().next().unwrap_or("process")));
        }
    } else {
        recommendations.push("Check 'htop' for CPU breakdown by process".to_string());
    }

    let confidence = if causes.is_empty() { 0.4 } else { 0.85 };

    Ok(RootCauseAnalysis {
        anomaly: format!("CPU load at {:.2} ({:.0}% above baseline)", current_load, ((current_load - baseline_load) / baseline_load) * 100.0),
        causes,
        recommendations,
        confidence,
    })
}

/// Format root cause analysis for display.
pub fn format_analysis(analysis: &RootCauseAnalysis) -> String {
    let mut response = format!("Anomaly Detected: {}\n\n", analysis.anomaly);

    if !analysis.causes.is_empty() {
        response.push_str("Root Causes Found:\n");
        for (i, cause) in analysis.causes.iter().enumerate().take(3) {
            response.push_str(&format!(
                "{}. {} ({:.0}% likely)\n   Evidence: {}\n\n",
                i + 1,
                cause.description,
                cause.likelihood * 100.0,
                cause.evidence
            ));
        }
    }

    if !analysis.recommendations.is_empty() {
        response.push_str("Recommended Actions:\n");
        for (i, rec) in analysis.recommendations.iter().enumerate() {
            response.push_str(&format!("{}. {}\n", i + 1, rec));
        }
    }

    response.push_str(&format!("\nConfidence: {:.0}%", analysis.confidence * 100.0));
    response
}

fn format_percentage(pct: f32) -> String {
    if pct > 90.0 {
        "most".to_string()
    } else if pct > 50.0 {
        "over half".to_string()
    } else if pct > 25.0 {
        "a significant portion".to_string()
    } else {
        "some".to_string()
    }
}
