//! System-specific knowledge accumulation.
//!
//! Design principle: Anna never hardcodes advice. Instead she:
//! 1. Detects system facts (hardware, filesystem, DE, governor, etc.)
//! 2. Queries Arch Wiki for topics relevant to those facts
//! 3. Asks LLM to extract specific, actionable advice for this system
//! 4. Stores the learned advice in /var/lib/anna/learned/
//! 5. On subsequent queries, retrieved from cache (refreshed weekly)
//!
//! This means advice gets richer and more accurate as Anna learns,
//! and is always grounded in official documentation.

use std::collections::HashMap;
use std::path::PathBuf;
use serde::{Deserialize, Serialize};
use tracing::{debug, info, warn};

use anna_shared::config::anna_data_dir;

/// A single learned insight about this system.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LearnedInsight {
    /// Topic that triggered research (e.g., "CPU frequency scaling")
    pub topic: String,
    /// System fact that prompted this (e.g., "cpu_governor=ondemand")
    pub trigger_fact: String,
    /// What Anna learned (LLM summary grounded in wiki)
    pub insight: String,
    /// Source wiki article titles
    pub wiki_sources: Vec<String>,
    /// Unix timestamp of when this was learned
    pub learned_at: u64,
    /// How many times this insight was retrieved/used
    pub use_count: u32,
}

/// The persistent knowledge store for this system.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct SystemKnowledge {
    /// key = trigger_fact, value = learned insight
    pub insights: HashMap<String, LearnedInsight>,
    /// Last time a full system scan was performed
    pub last_scan_ts: u64,
}

impl SystemKnowledge {
    fn storage_path() -> PathBuf {
        anna_data_dir().join("learned").join("system_knowledge.json")
    }

    pub fn load() -> Self {
        let path = Self::storage_path();
        if !path.exists() {
            return Self::default();
        }
        match std::fs::read_to_string(&path) {
            Ok(s) => serde_json::from_str(&s).unwrap_or_default(),
            Err(_) => Self::default(),
        }
    }

    pub fn save(&self) {
        let path = Self::storage_path();
        if let Some(parent) = path.parent() {
            let _ = std::fs::create_dir_all(parent);
        }
        match serde_json::to_string_pretty(self) {
            Ok(s) => { let _ = std::fs::write(&path, s); }
            Err(e) => warn!("Failed to save system knowledge: {}", e),
        }
    }

    /// Record a new insight learned from wiki research.
    pub fn record(&mut self, insight: LearnedInsight) {
        info!("Recording insight for: {}", insight.trigger_fact);
        self.insights.insert(insight.trigger_fact.clone(), insight);
        self.save();
    }

    /// Get all insights relevant to a question or system state.
    pub fn relevant_insights(&self, context: &[String]) -> Vec<&LearnedInsight> {
        context.iter()
            .filter_map(|fact| self.insights.get(fact))
            .collect()
    }

    /// Insights that are stale (older than 7 days) and need refresh.
    pub fn stale_topics(&self, facts: &[String]) -> Vec<String> {
        let now = unix_now();
        let week = 7 * 24 * 3600;
        facts.iter()
            .filter(|fact| {
                self.insights.get(*fact)
                    .map(|i| now - i.learned_at > week)
                    .unwrap_or(true) // Missing = stale
            })
            .cloned()
            .collect()
    }
}

/// Map system facts to Arch Wiki search topics.
/// This is deliberately minimal — wiki search fills in the specifics.
fn fact_to_wiki_topic(fact: &str) -> Option<&'static str> {
    let fact_lower = fact.to_lowercase();
    if fact_lower.contains("governor") || fact_lower.contains("cpufreq") {
        Some("CPU frequency scaling")
    } else if fact_lower.contains("thermal") || fact_lower.contains("temp") {
        Some("Fan speed control")
    } else if fact_lower.contains("ppd") || fact_lower.contains("power_profile") {
        Some("Power management")
    } else if fact_lower.contains("btrfs") {
        Some("Btrfs")
    } else if fact_lower.contains("ssd") || fact_lower.contains("nvme") {
        Some("Solid state drive")
    } else if fact_lower.contains("nvidia") {
        Some("NVIDIA")
    } else if fact_lower.contains("amd") || fact_lower.contains("radeon") {
        Some("AMDGPU")
    } else if fact_lower.contains("intel_gpu") {
        Some("Intel graphics")
    } else if fact_lower.contains("battery") || fact_lower.contains("bat") {
        Some("Laptop")
    } else if fact_lower.contains("wayland") {
        Some("Wayland")
    } else if fact_lower.contains("mirror") || fact_lower.contains("pacman") {
        Some("Mirrors")
    } else {
        None
    }
}

/// Collect current system facts from all data collectors.
pub fn collect_system_facts() -> Vec<String> {
    let mut facts = Vec::new();

    // Power/CPU facts
    let power = crate::power_profile::PowerState::capture();
    facts.extend(power.facts_for_context());

    // Battery facts
    for bat in crate::battery::Battery::detect_all() {
        facts.push(format!("battery_{}={}", bat.name, bat.status.to_lowercase()));
        if let Some(h) = bat.health_pct {
            facts.push(format!("battery_{}_health={:.0}", bat.name, h));
        }
    }

    // GPU facts
    for gpu in crate::gpu_monitor::GpuInfo::detect_all() {
        facts.push(format!("gpu_vendor={}", gpu.vendor));
        if let Some(temp) = gpu.temp_celsius {
            if temp >= 85.0 {
                facts.push(format!("gpu_temp_hot={:.0}C", temp));
            }
        }
    }

    // Filesystem fact
    if let Ok(out) = std::process::Command::new("findmnt")
        .args(["-n", "-o", "FSTYPE", "/"])
        .output()
    {
        let fs = String::from_utf8_lossy(&out.stdout).trim().to_string();
        if !fs.is_empty() {
            facts.push(format!("root_fs={}", fs));
        }
    }

    facts
}

/// Learn about stale topics in background — query wiki, ask LLM for insight.
/// Called from scheduler or on-demand; results stored in SystemKnowledge.
pub async fn learn_from_system_facts(model: &str) {
    let facts = collect_system_facts();
    let mut knowledge = SystemKnowledge::load();
    let stale = knowledge.stale_topics(&facts);

    if stale.is_empty() {
        debug!("All system knowledge is fresh, no learning needed");
        return;
    }

    info!("Learning about {} stale system facts", stale.len());

    for fact in stale.iter().take(5) { // Cap at 5 per run
        let wiki_topic = match fact_to_wiki_topic(fact) {
            Some(t) => t,
            None => continue,
        };

        // Search wiki for this topic
        let wiki_content = match anna_shared::wiki::search::keyword_search_text(wiki_topic, 1500) {
            Some(c) => c,
            None => {
                debug!("No wiki content for topic: {}", wiki_topic);
                continue;
            }
        };

        // Ask LLM to extract insight specific to this system fact
        let prompt = format!(
            "You are analyzing an Arch Linux system. System fact: {}\n\n\
            Arch Wiki documentation on '{}':\n{}\n\n\
            Based on this specific system fact and the Arch Wiki, provide ONE concise, \
            actionable insight for this system. Focus on what is specifically relevant \
            to the detected fact. Do not give generic advice. Be specific and brief (2-4 sentences).",
            fact, wiki_topic, wiki_content
        );

        match crate::ollama::chat_with_timeout(model, &prompt, 30).await {
            Ok(insight) if !insight.trim().is_empty() => {
                knowledge.record(LearnedInsight {
                    topic: wiki_topic.to_string(),
                    trigger_fact: fact.clone(),
                    insight: insight.trim().to_string(),
                    wiki_sources: vec![wiki_topic.to_string()],
                    learned_at: unix_now(),
                    use_count: 0,
                });
            }
            Ok(_) => debug!("LLM returned empty insight for {}", fact),
            Err(e) => warn!("LLM error while learning about {}: {}", fact, e),
        }
    }

    knowledge.last_scan_ts = unix_now();
    knowledge.save();
}

/// Get learned insights relevant to current system state, for briefing injection.
pub fn insights_for_briefing() -> String {
    let facts = collect_system_facts();
    let knowledge = SystemKnowledge::load();
    let relevant = knowledge.relevant_insights(&facts);

    if relevant.is_empty() {
        return String::new();
    }

    let mut out = "## Anna's Learned Insights (from Arch Wiki research)\n".to_string();
    for insight in relevant.iter().take(5) {
        out.push_str(&format!("- [{}] {}\n", insight.topic, insight.insight));
    }
    out
}

fn unix_now() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fact_to_wiki_topic() {
        assert_eq!(fact_to_wiki_topic("cpu_governor=ondemand"), Some("CPU frequency scaling"));
        assert_eq!(fact_to_wiki_topic("root_fs=btrfs"), Some("Btrfs"));
        assert_eq!(fact_to_wiki_topic("gpu_vendor=nvidia"), Some("NVIDIA"));
        assert_eq!(fact_to_wiki_topic("battery_BAT0=discharging"), Some("Laptop"));
    }

    #[test]
    fn test_system_knowledge_roundtrip() {
        let mut kb = SystemKnowledge::default();
        kb.insights.insert("test_fact".to_string(), LearnedInsight {
            topic: "Test".to_string(),
            trigger_fact: "test_fact".to_string(),
            insight: "Test insight".to_string(),
            wiki_sources: vec!["TestPage".to_string()],
            learned_at: 0,
            use_count: 0,
        });
        let facts = vec!["test_fact".to_string()];
        let relevant = kb.relevant_insights(&facts);
        assert_eq!(relevant.len(), 1);
        assert_eq!(relevant[0].insight, "Test insight");
    }

    #[test]
    fn test_collect_facts_no_panic() {
        let facts = collect_system_facts();
        assert!(facts.iter().all(|f| f.contains('=')));
    }
}
