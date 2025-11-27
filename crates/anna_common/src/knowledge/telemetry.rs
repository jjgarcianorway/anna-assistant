//! User-Centric Telemetry v0.11.0
//!
//! Tracks user interaction patterns to prioritize learning.
//! All data is local and privacy-respecting.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Topic categories for queries
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum QueryTopic {
    Cpu,
    Memory,
    Storage,
    Network,
    Dns,
    Packages,
    Services,
    Desktop,
    Games,
    Performance,
    Config,
    Users,
    Logs,
    Security,
    Updates,
    Anna, // Meta-questions about Anna herself
    Other,
}

impl QueryTopic {
    /// Detect topic from a query string
    pub fn from_query(query: &str) -> Self {
        let q = query.to_lowercase();

        // Order matters - more specific patterns first
        if q.contains("game") || q.contains("steam") || q.contains("lutris") || q.contains("wine") {
            return QueryTopic::Games;
        }
        if q.contains("dns") || q.contains("resolv") || q.contains("nameserver") {
            return QueryTopic::Dns;
        }
        if q.contains("network") || q.contains("wifi") || q.contains("ethernet")
            || q.contains("ip ") || q.contains("ip?") || q.contains("connected")
        {
            return QueryTopic::Network;
        }
        if q.contains("cpu") || q.contains("core") || q.contains("processor") || q.contains("thread") {
            return QueryTopic::Cpu;
        }
        if q.contains("memory") || q.contains("ram") || q.contains("swap") {
            return QueryTopic::Memory;
        }
        if q.contains("disk") || q.contains("storage") || q.contains("partition")
            || q.contains("mount") || q.contains("filesystem")
        {
            return QueryTopic::Storage;
        }
        if q.contains("package") || q.contains("pacman") || q.contains("yay")
            || q.contains("install") || q.contains("aur")
        {
            return QueryTopic::Packages;
        }
        if q.contains("service") || q.contains("systemd") || q.contains("daemon") {
            return QueryTopic::Services;
        }
        if q.contains("desktop") || q.contains("hyprland") || q.contains("kde")
            || q.contains("gnome") || q.contains("waybar") || q.contains("window manager")
        {
            return QueryTopic::Desktop;
        }
        if q.contains("performance") || q.contains("slow") || q.contains("fast")
            || q.contains("optimize")
        {
            return QueryTopic::Performance;
        }
        if q.contains("config") || q.contains("setting") || q.contains("preference") {
            return QueryTopic::Config;
        }
        if q.contains("user") || q.contains("account") || q.contains("permission") {
            return QueryTopic::Users;
        }
        if q.contains("log") || q.contains("journal") || q.contains("error") {
            return QueryTopic::Logs;
        }
        if q.contains("security") || q.contains("firewall") || q.contains("ssh") {
            return QueryTopic::Security;
        }
        if q.contains("update") || q.contains("upgrade") {
            return QueryTopic::Updates;
        }
        if q.contains("anna") || q.contains("version") || q.contains("help")
            || q.contains("status")
        {
            return QueryTopic::Anna;
        }

        QueryTopic::Other
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            QueryTopic::Cpu => "cpu",
            QueryTopic::Memory => "memory",
            QueryTopic::Storage => "storage",
            QueryTopic::Network => "network",
            QueryTopic::Dns => "dns",
            QueryTopic::Packages => "packages",
            QueryTopic::Services => "services",
            QueryTopic::Desktop => "desktop",
            QueryTopic::Games => "games",
            QueryTopic::Performance => "performance",
            QueryTopic::Config => "config",
            QueryTopic::Users => "users",
            QueryTopic::Logs => "logs",
            QueryTopic::Security => "security",
            QueryTopic::Updates => "updates",
            QueryTopic::Anna => "anna",
            QueryTopic::Other => "other",
        }
    }

    pub fn from_str(s: &str) -> Self {
        match s {
            "cpu" => QueryTopic::Cpu,
            "memory" => QueryTopic::Memory,
            "storage" => QueryTopic::Storage,
            "network" => QueryTopic::Network,
            "dns" => QueryTopic::Dns,
            "packages" => QueryTopic::Packages,
            "services" => QueryTopic::Services,
            "desktop" => QueryTopic::Desktop,
            "games" => QueryTopic::Games,
            "performance" => QueryTopic::Performance,
            "config" => QueryTopic::Config,
            "users" => QueryTopic::Users,
            "logs" => QueryTopic::Logs,
            "security" => QueryTopic::Security,
            "updates" => QueryTopic::Updates,
            "anna" => QueryTopic::Anna,
            _ => QueryTopic::Other,
        }
    }
}

/// Telemetry record for a topic
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TopicStats {
    pub topic: QueryTopic,
    pub count: u64,
    pub last_used: DateTime<Utc>,
    /// Weighted score based on recency and frequency
    pub weight: f64,
}

impl TopicStats {
    pub fn new(topic: QueryTopic) -> Self {
        Self {
            topic,
            count: 0,
            last_used: Utc::now(),
            weight: 0.0,
        }
    }

    /// Record a query for this topic
    pub fn record(&mut self) {
        self.count += 1;
        self.last_used = Utc::now();
        self.recalculate_weight();
    }

    /// Recalculate weight based on recency and frequency
    fn recalculate_weight(&mut self) {
        let age_hours = Utc::now()
            .signed_duration_since(self.last_used)
            .num_hours() as f64;

        // Decay factor: halves every 24 hours
        let recency_factor = 0.5_f64.powf(age_hours / 24.0);

        // Frequency factor: log scale
        let frequency_factor = (self.count as f64 + 1.0).ln();

        self.weight = recency_factor * frequency_factor;
    }
}

/// User telemetry store
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserTelemetry {
    /// Stats by topic
    pub topics: HashMap<QueryTopic, TopicStats>,
    /// Mentioned tools/apps (e.g., "hyprland", "docker")
    pub mentioned_tools: HashMap<String, u64>,
    /// Total queries
    pub total_queries: u64,
    /// Last activity
    pub last_activity: DateTime<Utc>,
}

impl Default for UserTelemetry {
    fn default() -> Self {
        Self::new()
    }
}

impl UserTelemetry {
    pub fn new() -> Self {
        Self {
            topics: HashMap::new(),
            mentioned_tools: HashMap::new(),
            total_queries: 0,
            last_activity: Utc::now(),
        }
    }

    /// Record a query
    pub fn record_query(&mut self, query: &str) {
        self.total_queries += 1;
        self.last_activity = Utc::now();

        // Update topic stats
        let topic = QueryTopic::from_query(query);
        self.topics
            .entry(topic)
            .or_insert_with(|| TopicStats::new(topic))
            .record();

        // Extract mentioned tools
        self.extract_tools(query);
    }

    /// Extract tool mentions from a query
    fn extract_tools(&mut self, query: &str) {
        let q = query.to_lowercase();

        // Common tools to track
        let tools = [
            "hyprland", "waybar", "sway", "i3", "kde", "gnome", "xfce",
            "docker", "podman", "kubernetes", "vagrant",
            "vim", "neovim", "nvim", "emacs", "code", "vscode",
            "firefox", "chrome", "chromium", "brave",
            "steam", "lutris", "wine", "proton",
            "git", "ssh", "tmux", "zsh", "bash", "fish",
            "pacman", "yay", "paru",
            "nvidia", "amd", "intel",
        ];

        for tool in tools {
            if q.contains(tool) {
                *self.mentioned_tools.entry(tool.to_string()).or_insert(0) += 1;
            }
        }
    }

    /// Get top topics by weight
    pub fn top_topics(&self, n: usize) -> Vec<&TopicStats> {
        let mut topics: Vec<_> = self.topics.values().collect();
        topics.sort_by(|a, b| b.weight.partial_cmp(&a.weight).unwrap_or(std::cmp::Ordering::Equal));
        topics.truncate(n);
        topics
    }

    /// Get frequently mentioned tools
    pub fn frequent_tools(&self, min_count: u64) -> Vec<(&String, &u64)> {
        let mut tools: Vec<_> = self
            .mentioned_tools
            .iter()
            .filter(|(_, count)| **count >= min_count)
            .collect();
        tools.sort_by(|a, b| b.1.cmp(a.1));
        tools
    }

    /// Check if user cares about a topic
    pub fn cares_about(&self, topic: QueryTopic) -> bool {
        self.topics
            .get(&topic)
            .map(|s| s.weight > 0.5 || s.count >= 3)
            .unwrap_or(false)
    }

    /// Get priority score for learning about a topic
    pub fn learning_priority(&self, topic: QueryTopic) -> f64 {
        self.topics.get(&topic).map(|s| s.weight).unwrap_or(0.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_topic_detection() {
        assert_eq!(QueryTopic::from_query("How many CPU cores?"), QueryTopic::Cpu);
        assert_eq!(QueryTopic::from_query("Am I on wifi?"), QueryTopic::Network);
        assert_eq!(QueryTopic::from_query("DNS settings"), QueryTopic::Dns);
        assert_eq!(QueryTopic::from_query("Steam games"), QueryTopic::Games);
        assert_eq!(
            QueryTopic::from_query("pacman updates"),
            QueryTopic::Packages
        );
    }

    #[test]
    fn test_telemetry_recording() {
        let mut telem = UserTelemetry::new();

        telem.record_query("How many CPU cores do I have?");
        telem.record_query("What's my CPU model?");
        telem.record_query("Am I on wifi?");

        assert_eq!(telem.total_queries, 3);
        assert!(telem.topics.get(&QueryTopic::Cpu).unwrap().count >= 2);
        assert!(telem.topics.get(&QueryTopic::Network).unwrap().count >= 1);
    }

    #[test]
    fn test_tool_extraction() {
        let mut telem = UserTelemetry::new();

        telem.record_query("Configure hyprland");
        telem.record_query("Setup docker container");
        telem.record_query("hyprland waybar issue");

        assert_eq!(telem.mentioned_tools.get("hyprland"), Some(&2));
        assert_eq!(telem.mentioned_tools.get("docker"), Some(&1));
        assert_eq!(telem.mentioned_tools.get("waybar"), Some(&1));
    }

    #[test]
    fn test_top_topics() {
        let mut telem = UserTelemetry::new();

        for _ in 0..5 {
            telem.record_query("CPU info");
        }
        for _ in 0..3 {
            telem.record_query("network");
        }
        telem.record_query("memory");

        let top = telem.top_topics(2);
        assert_eq!(top.len(), 2);
        assert_eq!(top[0].topic, QueryTopic::Cpu);
    }
}
