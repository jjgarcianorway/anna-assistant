//! Smart recommendations engine for personalized suggestions.
//!
//! Generates recommendations based on:
//! - User profile and interests
//! - System optimizations
//! - Arch Wiki best practices
//! - Usage patterns

use super::profile::UserProfile;

/// Priority level for recommendations.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Priority {
    High,
    Medium,
    Low,
}

/// A personalized recommendation.
#[derive(Debug)]
pub struct SmartRecommendation {
    pub category: String,
    pub title: String,
    pub description: String,
    pub relevance: String,
    pub arch_wiki_ref: Option<String>,
    pub priority: Priority,
}

/// Generate smart recommendations based on user profile and system state.
pub fn generate_smart_recommendations(profile: &UserProfile) -> Vec<SmartRecommendation> {
    let mut recs = Vec::new();

    // 1. System optimizations (existing)
    add_system_optimizations(&mut recs);

    // 2. Profile-based recommendations
    add_profile_based_recommendations(&mut recs, profile);

    // 3. Service suggestions based on installed packages
    add_service_suggestions(&mut recs, profile);

    // 4. Tool suggestions based on topics
    add_tool_suggestions(&mut recs, profile);

    // Sort by priority
    recs.sort_by(|a, b| a.priority.cmp(&b.priority));

    // Return top 5
    recs.into_iter().take(5).collect()
}

fn add_system_optimizations(recs: &mut Vec<SmartRecommendation>) {
    let suggestions = crate::anomaly::check_optimizations();

    for s in suggestions.iter().take(2) {
        recs.push(SmartRecommendation {
            category: s.category.clone(),
            title: "System Optimization".to_string(),
            description: s.description.clone(),
            relevance: s.potential_savings.clone().unwrap_or_else(|| "Improve system health".to_string()),
            arch_wiki_ref: None,
            priority: Priority::High,
        });
    }
}

fn add_profile_based_recommendations(recs: &mut Vec<SmartRecommendation>, profile: &UserProfile) {
    // Check if user frequently asks about a topic but might benefit from related tools
    for (topic, count) in &profile.top_topics {
        if *count >= 3 {
            match topic.as_str() {
                "Docker" => {
                    if !is_package_installed("docker-compose") {
                        recs.push(SmartRecommendation {
                            category: "Tools".to_string(),
                            title: "Docker Compose".to_string(),
                            description: "Consider installing docker-compose for multi-container workflows".to_string(),
                            relevance: format!("You've asked about Docker {} times", count),
                            arch_wiki_ref: Some("https://wiki.archlinux.org/title/Docker".to_string()),
                            priority: Priority::Medium,
                        });
                    }
                }
                "Performance" => {
                    if !is_service_active("irqbalance") {
                        recs.push(SmartRecommendation {
                            category: "Performance".to_string(),
                            title: "Enable irqbalance".to_string(),
                            description: "Distribute hardware interrupts across CPUs for better performance".to_string(),
                            relevance: "Based on your interest in system performance".to_string(),
                            arch_wiki_ref: Some("https://wiki.archlinux.org/title/Improving_performance".to_string()),
                            priority: Priority::Medium,
                        });
                    }
                }
                "Memory" => {
                    if !is_service_active("earlyoom") {
                        recs.push(SmartRecommendation {
                            category: "Memory".to_string(),
                            title: "Consider earlyoom".to_string(),
                            description: "Prevents system freeze under memory pressure by killing heavy processes".to_string(),
                            relevance: "Based on your memory-related questions".to_string(),
                            arch_wiki_ref: Some("https://wiki.archlinux.org/title/Earlyoom".to_string()),
                            priority: Priority::Low,
                        });
                    }
                }
                "Networking" | "WiFi" => {
                    recs.push(SmartRecommendation {
                        category: "Network".to_string(),
                        title: "Network diagnostics".to_string(),
                        description: "Consider installing nethogs for per-process bandwidth monitoring".to_string(),
                        relevance: "Based on your networking questions".to_string(),
                        arch_wiki_ref: Some("https://wiki.archlinux.org/title/Network_Debugging".to_string()),
                        priority: Priority::Low,
                    });
                }
                _ => {}
            }
        }
    }
}

fn add_service_suggestions(recs: &mut Vec<SmartRecommendation>, profile: &UserProfile) {
    // Check for commonly problematic services the user monitors
    for service in &profile.watched_services {
        match service.as_str() {
            "bluetooth" => {
                recs.push(SmartRecommendation {
                    category: "Bluetooth".to_string(),
                    title: "Bluetooth tips".to_string(),
                    description: "Enable bluetooth.service at boot for faster connection".to_string(),
                    relevance: "You frequently check Bluetooth status".to_string(),
                    arch_wiki_ref: Some("https://wiki.archlinux.org/title/Bluetooth".to_string()),
                    priority: Priority::Low,
                });
            }
            "docker" => {
                recs.push(SmartRecommendation {
                    category: "Docker".to_string(),
                    title: "Docker cleanup".to_string(),
                    description: "Schedule regular 'docker system prune' to reclaim disk space".to_string(),
                    relevance: "You frequently interact with Docker".to_string(),
                    arch_wiki_ref: Some("https://wiki.archlinux.org/title/Docker".to_string()),
                    priority: Priority::Medium,
                });
            }
            _ => {}
        }
    }
}

fn add_tool_suggestions(recs: &mut Vec<SmartRecommendation>, profile: &UserProfile) {
    // Suggest tools based on package usage
    for pkg in &profile.frequent_packages {
        match pkg.as_str() {
            "python" | "python3" => {
                if !is_package_installed("python-pip") {
                    recs.push(SmartRecommendation {
                        category: "Development".to_string(),
                        title: "Python tools".to_string(),
                        description: "Install python-pip for package management".to_string(),
                        relevance: "Based on your Python usage".to_string(),
                        arch_wiki_ref: Some("https://wiki.archlinux.org/title/Python".to_string()),
                        priority: Priority::Low,
                    });
                }
            }
            "rust" | "cargo" => {
                if !is_package_installed("rust-analyzer") {
                    recs.push(SmartRecommendation {
                        category: "Development".to_string(),
                        title: "Rust tools".to_string(),
                        description: "Install rust-analyzer for IDE support".to_string(),
                        relevance: "Based on your Rust usage".to_string(),
                        arch_wiki_ref: Some("https://wiki.archlinux.org/title/Rust".to_string()),
                        priority: Priority::Low,
                    });
                }
            }
            _ => {}
        }
    }
}

/// Check if a package is installed.
fn is_package_installed(pkg: &str) -> bool {
    std::process::Command::new("pacman")
        .args(["-Q", pkg])
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
}

/// Check if a service is active.
fn is_service_active(service: &str) -> bool {
    std::process::Command::new("systemctl")
        .args(["is-active", "--quiet", service])
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
}

/// Format recommendations for PDF.
pub fn format_recommendations_for_pdf(recs: &[SmartRecommendation]) -> Vec<String> {
    recs.iter()
        .map(|r| {
            let mut line = format!("{}: {}", r.category, r.description);
            if !r.relevance.is_empty() {
                line.push_str(&format!(" ({})", r.relevance));
            }
            line
        })
        .collect()
}
