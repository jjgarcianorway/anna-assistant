//! Opportunity Detection - Anna proposes solutions when data is missing.
//! 
//! Philosophy: Never just say "I don't have that" - always propose what to do next.

use anyhow::Result;
use chrono::{DateTime, Duration, Utc};
use serde::{Deserialize, Serialize};
use tracing::{debug, info};

/// An opportunity Anna detected to provide value.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Opportunity {
    /// Type of opportunity
    pub opportunity_type: OpportunityType,
    /// What data/capability is missing
    pub missing: String,
    /// What Anna can offer instead
    pub proposals: Vec<Proposal>,
    /// Confidence this is the right opportunity (0.0-1.0)
    pub confidence: f32,
}

/// Type of opportunity detected.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum OpportunityType {
    /// User wants historical data Anna doesn't have yet
    MissingHistoricalData,
    /// User wants analysis that requires data collection
    RequiresDataCollection,
    /// User wants optimization that needs experimentation
    OptimizationOpportunity,
    /// User wants comparison to baseline Anna doesn't have
    MissingBaseline,
    /// User wants prediction but needs more data
    InsufficientDataForPrediction,
}

/// A proposal Anna can offer.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Proposal {
    /// Proposal ID for tracking
    pub id: String,
    /// Description of what Anna will do
    pub action: String,
    /// Expected duration (seconds)
    pub duration_secs: u64,
    /// When results will be available
    pub available_at: Option<DateTime<Utc>>,
    /// What the deliverable will be
    pub deliverable: String,
    /// Risk level
    pub risk: RiskLevel,
    /// Whether this requires user confirmation
    pub requires_confirmation: bool,
}

/// Risk level for proposed actions.
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum RiskLevel {
    None,        // Just monitoring/reading
    Low,         // Minor changes, easily reversible
    Medium,      // Requires reboot or service restart
    High,        // System-wide changes
}

/// Detect opportunities in user questions.
pub async fn detect_opportunity(question: &str) -> Option<Opportunity> {
    let q = question.to_lowercase();
    
    // Pattern 1: Historical data requests
    if let Some(opp) = detect_historical_data_request(&q) {
        return Some(opp);
    }
    
    // Pattern 2: Optimization requests
    if let Some(opp) = detect_optimization_request(&q) {
        return Some(opp);
    }
    
    // Pattern 3: Comparison requests
    if let Some(opp) = detect_comparison_request(&q) {
        return Some(opp);
    }
    
    // Pattern 4: Prediction requests
    if let Some(opp) = detect_prediction_request(&q) {
        return Some(opp);
    }
    
    None
}

/// Detect requests for historical data Anna doesn't have.
fn detect_historical_data_request(question: &str) -> Option<Opportunity> {
    let patterns = [
        ("last 30 days", 30),
        ("last month", 30),
        ("past 30 days", 30),
        ("last 60 days", 60),
        ("last 90 days", 90),
        ("over the past month", 30),
    ];
    
    for (pattern, days_requested) in patterns {
        if question.contains(pattern) {
            // Check what type of data
            let data_type = if question.contains("boot") {
                "boot performance"
            } else if question.contains("memory") || question.contains("ram") {
                "memory usage"
            } else if question.contains("disk") {
                "disk usage"
            } else if question.contains("cpu") || question.contains("load") {
                "CPU load"
            } else if question.contains("network") || question.contains("bandwidth") {
                "network usage"
            } else {
                continue; // Unknown data type
            };
            
            // Check how much data we actually have
            let history = anna_shared::monitor::LongTermHistory::load();
            let days_available = history.daily_snapshots.len();
            
            if days_available < days_requested as usize {
                info!("Detected historical data opportunity: need {} days, have {}", days_requested, days_available);
                
                let mut proposals = Vec::new();
                
                // Proposal 1: Show what we have now
                if days_available > 0 {
                    proposals.push(Proposal {
                        id: format!("show-current-{}", data_type.replace(' ', "-")),
                        action: format!("Show {} chart with {} days of available data", data_type, days_available),
                        duration_secs: 5,
                        available_at: Some(Utc::now()),
                        deliverable: format!("{}-day {} chart", days_available, data_type),
                        risk: RiskLevel::None,
                        requires_confirmation: false,
                    });
                }
                
                // Proposal 2: Start collecting and deliver later
                let days_needed = days_requested - days_available as i64;
                let ready_date = Utc::now() + Duration::days(days_needed);
                
                proposals.push(Proposal {
                    id: format!("collect-{}-{}", data_type.replace(' ', "-"), days_requested),
                    action: format!("Continue daily {} monitoring for {} more days", data_type, days_needed),
                    duration_secs: 0, // Ongoing monitoring
                    available_at: Some(ready_date),
                    deliverable: format!("{}-day {} report on {}", days_requested, data_type, ready_date.format("%B %d")),
                    risk: RiskLevel::None,
                    requires_confirmation: true,
                });
                
                // Proposal 3: Set up automatic reports
                proposals.push(Proposal {
                    id: format!("auto-report-{}", data_type.replace(' ', "-")),
                    action: format!("Set up automatic weekly {} reports", data_type),
                    duration_secs: 0,
                    available_at: Some(Utc::now() + Duration::weeks(1)),
                    deliverable: format!("Weekly {} summary every Monday", data_type),
                    risk: RiskLevel::None,
                    requires_confirmation: true,
                });
                
                return Some(Opportunity {
                    opportunity_type: OpportunityType::MissingHistoricalData,
                    missing: format!("{} days of {} data", days_requested, data_type),
                    proposals,
                    confidence: 0.9,
                });
            }
        }
    }
    
    None
}

/// Detect optimization requests.
fn detect_optimization_request(question: &str) -> Option<Opportunity> {
    let optimizations = [
        ("optimize boot", "boot time", "Boot Time Optimization", "systemd-analyze", 300),
        ("speed up boot", "boot time", "Boot Time Optimization", "systemd-analyze", 300),
        ("faster boot", "boot time", "Boot Time Optimization", "systemd-analyze", 300),
        ("optimize memory", "memory usage", "Memory Optimization", "memory profiling", 180),
        ("reduce memory", "memory usage", "Memory Optimization", "memory profiling", 180),
        ("optimize disk", "disk space", "Disk Optimization", "disk analysis", 240),
        ("free up disk", "disk space", "Disk Cleanup", "disk analysis", 120),
        ("reduce disk usage", "disk space", "Disk Optimization", "ncdu scan", 180),
    ];
    
    for (pattern, resource, name, tool, duration) in optimizations {
        if question.contains(pattern) {
            info!("Detected optimization opportunity: {}", name);
            
            let proposals = vec![
                Proposal {
                    id: format!("analyze-{}", resource.replace(' ', "-")),
                    action: format!("Run {} to identify bottlenecks", tool),
                    duration_secs: duration,
                    available_at: Some(Utc::now() + Duration::seconds(duration as i64)),
                    deliverable: format!("{} analysis with improvement suggestions", name),
                    risk: RiskLevel::None,
                    requires_confirmation: false,
                },
                Proposal {
                    id: format!("optimize-{}", resource.replace(' ', "-")),
                    action: format!("Run full {} experiment with before/after benchmarks", name.to_lowercase()),
                    duration_secs: duration * 3,
                    available_at: Some(Utc::now() + Duration::seconds((duration * 3) as i64)),
                    deliverable: format!("Applied optimizations with measured improvement report"),
                    risk: RiskLevel::Medium,
                    requires_confirmation: true,
                },
            ];
            
            return Some(Opportunity {
                opportunity_type: OpportunityType::OptimizationOpportunity,
                missing: format!("{} analysis and optimization plan", resource),
                proposals,
                confidence: 0.85,
            });
        }
    }
    
    None
}

/// Detect comparison/baseline requests.
fn detect_comparison_request(question: &str) -> Option<Opportunity> {
    let patterns = [
        ("compared to", "comparative baseline"),
        ("vs ", "comparison data"),
        ("better than", "performance baseline"),
        ("worse than", "performance baseline"),
        ("average", "average baseline"),
    ];
    
    for (pattern, missing_data) in patterns {
        if question.contains(pattern) && question.contains("other") || question.contains("similar") {
            info!("Detected comparison opportunity");
            
            let proposals = vec![
                Proposal {
                    id: "establish-baseline".to_string(),
                    action: "Establish performance baseline by collecting metrics over 7 days".to_string(),
                    duration_secs: 0,
                    available_at: Some(Utc::now() + Duration::days(7)),
                    deliverable: "System performance baseline with comparison to typical systems".to_string(),
                    risk: RiskLevel::None,
                    requires_confirmation: true,
                },
                Proposal {
                    id: "compare-now".to_string(),
                    action: "Compare current metrics to theoretical optimal values".to_string(),
                    duration_secs: 30,
                    available_at: Some(Utc::now()),
                    deliverable: "Current performance vs optimal theoretical performance".to_string(),
                    risk: RiskLevel::None,
                    requires_confirmation: false,
                },
            ];
            
            return Some(Opportunity {
                opportunity_type: OpportunityType::MissingBaseline,
                missing: missing_data.to_string(),
                proposals,
                confidence: 0.75,
            });
        }
    }
    
    None
}

/// Detect prediction requests without sufficient data.
fn detect_prediction_request(question: &str) -> Option<Opportunity> {
    let patterns = [
        "will ", "going to", "predict", "forecast", "trend", "estimate"
    ];
    
    if !patterns.iter().any(|p| question.contains(p)) {
        return None;
    }
    
    // Check if we have enough data for predictions
    let history = anna_shared::monitor::LongTermHistory::load();
    let days_available = history.daily_snapshots.len();
    
    if days_available < 7 {
        info!("Detected prediction request with insufficient data");
        
        let days_needed = 7 - days_available;
        let ready_date = Utc::now() + Duration::days(days_needed as i64);
        
        let proposals = vec![
            Proposal {
                id: "collect-for-prediction".to_string(),
                action: format!("Collect {} more days of data for accurate predictions", days_needed),
                duration_secs: 0,
                available_at: Some(ready_date),
                deliverable: format!("Predictive analysis on {}", ready_date.format("%B %d")),
                risk: RiskLevel::None,
                requires_confirmation: true,
            },
            Proposal {
                id: "rough-estimate".to_string(),
                action: "Provide rough estimate based on current limited data".to_string(),
                duration_secs: 10,
                available_at: Some(Utc::now()),
                deliverable: format!("Rough prediction (low confidence with {} days data)", days_available),
                risk: RiskLevel::None,
                requires_confirmation: false,
            },
        ];
        
        return Some(Opportunity {
            opportunity_type: OpportunityType::InsufficientDataForPrediction,
            missing: format!("{} more days of data for accurate predictions", days_needed),
            proposals,
            confidence: 0.8,
        });
    }
    
    None
}

/// Format opportunity as user-friendly message.
pub fn format_opportunity(opp: &Opportunity) -> String {
    let mut response = format!("I don't have {} yet, but I can help:\n\n", opp.missing);
    
    for (i, proposal) in opp.proposals.iter().enumerate() {
        response.push_str(&format!("{}. {}\n", i + 1, proposal.action));
        
        if let Some(available_at) = proposal.available_at {
            if available_at > Utc::now() + Duration::hours(1) {
                response.push_str(&format!("   → Deliverable: {} (ready {})\n", 
                    proposal.deliverable,
                    available_at.format("%B %d at %H:%M")));
            } else {
                response.push_str(&format!("   → Deliverable: {}\n", proposal.deliverable));
            }
        }
        
        match proposal.risk {
            RiskLevel::None => {},
            RiskLevel::Low => response.push_str("   → Risk: Low (easily reversible)\n"),
            RiskLevel::Medium => response.push_str("   → Risk: Medium (may require reboot)\n"),
            RiskLevel::High => response.push_str("   → Risk: High (system-wide changes)\n"),
        }
        
        if proposal.duration_secs > 60 {
            response.push_str(&format!("   → Duration: ~{} minutes\n", proposal.duration_secs / 60));
        } else if proposal.duration_secs > 0 {
            response.push_str(&format!("   → Duration: ~{} seconds\n", proposal.duration_secs));
        }
        
        response.push('\n');
    }
    
    response.push_str("Which option would you like?");
    response
}
