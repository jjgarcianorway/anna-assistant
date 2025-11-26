//! Anna Learned Facts - v10.1.0
//!
//! Anna learns from her observations and caches interpreted facts.
//! Facts have freshness rules based on their nature - some rarely change,
//! others are volatile and need frequent refresh.
//!
//! This enables Anna to get smarter over time without hardcoding.

use chrono::{DateTime, Duration, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;

/// How often a fact type needs to be refreshed
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum FreshnessTier {
    /// Almost never changes (CPU model, GPU model) - refresh on boot only
    Static,
    /// Rarely changes (total RAM, kernel version) - refresh daily or on boot
    Stable,
    /// Changes occasionally (installed packages) - refresh on pacman events or hourly
    Moderate,
    /// Changes frequently (disk usage, memory usage) - refresh every 10 minutes
    Dynamic,
    /// Very volatile (network status, process list) - always fetch fresh
    Volatile,
}

impl FreshnessTier {
    /// Get the maximum age for this tier before refresh is needed
    pub fn max_age(&self) -> Duration {
        match self {
            FreshnessTier::Static => Duration::days(30),
            FreshnessTier::Stable => Duration::days(1),
            FreshnessTier::Moderate => Duration::hours(1),
            FreshnessTier::Dynamic => Duration::minutes(10),
            FreshnessTier::Volatile => Duration::seconds(0), // Always refresh
        }
    }
}

/// Categories of facts Anna can learn
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum FactCategory {
    // Hardware (mostly static)
    CpuModel,
    CpuCores,
    CpuThreads,
    CpuFeatures, // SSE, AVX, etc.
    GpuModel,
    TotalRam,

    // System (stable)
    KernelVersion,
    ArchVersion,
    Hostname,
    MachineType, // Laptop, Desktop, VM

    // Desktop (moderate)
    DesktopEnvironment,
    WindowManager,
    DisplayServer,

    // Packages (moderate - invalidate on pacman)
    InstalledPackage(String), // e.g., InstalledPackage("steam")
    OrphanPackages,
    PendingUpdates,

    // Storage (dynamic)
    DiskUsageRoot,
    DiskUsageHome,
    LargestFoldersHome,
    LargestFoldersVar,
    PacmanCacheSize,

    // Network (volatile)
    NetworkInterface,
    DnsServers,
    WifiStatus,

    // Services (moderate)
    FailedServices,

    // Custom learned facts
    Custom(String),
}

impl FactCategory {
    /// Get the freshness tier for this category
    pub fn freshness_tier(&self) -> FreshnessTier {
        match self {
            // Hardware - almost never changes
            FactCategory::CpuModel => FreshnessTier::Static,
            FactCategory::CpuCores => FreshnessTier::Static,
            FactCategory::CpuThreads => FreshnessTier::Static,
            FactCategory::CpuFeatures => FreshnessTier::Static,
            FactCategory::GpuModel => FreshnessTier::Static,
            FactCategory::TotalRam => FreshnessTier::Static,
            FactCategory::MachineType => FreshnessTier::Static,

            // System - rarely changes
            FactCategory::KernelVersion => FreshnessTier::Stable,
            FactCategory::ArchVersion => FreshnessTier::Stable,
            FactCategory::Hostname => FreshnessTier::Stable,

            // Desktop - changes on login/session
            FactCategory::DesktopEnvironment => FreshnessTier::Moderate,
            FactCategory::WindowManager => FreshnessTier::Moderate,
            FactCategory::DisplayServer => FreshnessTier::Moderate,

            // Packages - moderate, invalidate on pacman
            FactCategory::InstalledPackage(_) => FreshnessTier::Moderate,
            FactCategory::OrphanPackages => FreshnessTier::Moderate,
            FactCategory::PendingUpdates => FreshnessTier::Moderate,

            // Storage - dynamic
            FactCategory::DiskUsageRoot => FreshnessTier::Dynamic,
            FactCategory::DiskUsageHome => FreshnessTier::Dynamic,
            FactCategory::LargestFoldersHome => FreshnessTier::Dynamic,
            FactCategory::LargestFoldersVar => FreshnessTier::Dynamic,
            FactCategory::PacmanCacheSize => FreshnessTier::Dynamic,

            // Network - volatile
            FactCategory::NetworkInterface => FreshnessTier::Volatile,
            FactCategory::DnsServers => FreshnessTier::Moderate, // DNS config doesn't change often
            FactCategory::WifiStatus => FreshnessTier::Volatile,

            // Services
            FactCategory::FailedServices => FreshnessTier::Moderate,

            // Custom - default to moderate
            FactCategory::Custom(_) => FreshnessTier::Moderate,
        }
    }

    /// Get a human-readable name for this category
    pub fn display_name(&self) -> String {
        match self {
            FactCategory::CpuModel => "CPU Model".to_string(),
            FactCategory::CpuCores => "CPU Cores".to_string(),
            FactCategory::CpuThreads => "CPU Threads".to_string(),
            FactCategory::CpuFeatures => "CPU Features".to_string(),
            FactCategory::GpuModel => "GPU Model".to_string(),
            FactCategory::TotalRam => "Total RAM".to_string(),
            FactCategory::KernelVersion => "Kernel Version".to_string(),
            FactCategory::ArchVersion => "Arch Version".to_string(),
            FactCategory::Hostname => "Hostname".to_string(),
            FactCategory::MachineType => "Machine Type".to_string(),
            FactCategory::DesktopEnvironment => "Desktop Environment".to_string(),
            FactCategory::WindowManager => "Window Manager".to_string(),
            FactCategory::DisplayServer => "Display Server".to_string(),
            FactCategory::InstalledPackage(pkg) => format!("Package: {}", pkg),
            FactCategory::OrphanPackages => "Orphan Packages".to_string(),
            FactCategory::PendingUpdates => "Pending Updates".to_string(),
            FactCategory::DiskUsageRoot => "Root Disk Usage".to_string(),
            FactCategory::DiskUsageHome => "Home Disk Usage".to_string(),
            FactCategory::LargestFoldersHome => "Largest Folders (Home)".to_string(),
            FactCategory::LargestFoldersVar => "Largest Folders (Var)".to_string(),
            FactCategory::PacmanCacheSize => "Pacman Cache Size".to_string(),
            FactCategory::NetworkInterface => "Network Interface".to_string(),
            FactCategory::DnsServers => "DNS Servers".to_string(),
            FactCategory::WifiStatus => "WiFi Status".to_string(),
            FactCategory::FailedServices => "Failed Services".to_string(),
            FactCategory::Custom(name) => format!("Custom: {}", name),
        }
    }
}

/// A single learned fact
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LearnedFact {
    /// The category of this fact
    pub category: FactCategory,

    /// The interpreted value (human-readable)
    pub value: String,

    /// Raw evidence that led to this fact
    pub evidence: String,

    /// Command that produced the evidence
    pub source_command: String,

    /// When this fact was learned
    pub learned_at: DateTime<Utc>,

    /// How many times this fact has been used to answer questions
    pub usage_count: u32,

    /// Confidence score (0.0 - 1.0)
    pub confidence: f32,

    /// Boot ID when this was learned (for boot-sensitive facts)
    pub boot_id: Option<String>,
}

impl LearnedFact {
    /// Create a new learned fact
    pub fn new(
        category: FactCategory,
        value: String,
        evidence: String,
        source_command: String,
        confidence: f32,
    ) -> Self {
        Self {
            category,
            value,
            evidence,
            source_command,
            learned_at: Utc::now(),
            usage_count: 0,
            confidence,
            boot_id: get_current_boot_id(),
        }
    }

    /// Check if this fact is still fresh
    pub fn is_fresh(&self) -> bool {
        let max_age = self.category.freshness_tier().max_age();
        let age = Utc::now() - self.learned_at;
        age < max_age
    }

    /// Check if this fact needs refresh due to boot
    pub fn needs_boot_refresh(&self) -> bool {
        match self.category.freshness_tier() {
            FreshnessTier::Static | FreshnessTier::Stable => {
                // Check if boot ID changed
                if let (Some(fact_boot), Some(current_boot)) = (&self.boot_id, get_current_boot_id()) {
                    fact_boot != &current_boot
                } else {
                    false
                }
            }
            _ => false,
        }
    }

    /// Mark this fact as used
    pub fn mark_used(&mut self) {
        self.usage_count += 1;
    }
}

/// Get current boot ID
fn get_current_boot_id() -> Option<String> {
    fs::read_to_string("/proc/sys/kernel/random/boot_id")
        .ok()
        .map(|s| s.trim().to_string())
}

/// The learned facts database
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct LearnedFactsDb {
    /// All learned facts, keyed by category
    facts: HashMap<String, LearnedFact>,

    /// Last time packages were modified (for invalidation)
    pub last_pacman_operation: Option<DateTime<Utc>>,

    /// Statistics
    pub total_queries_answered: u64,
    pub total_facts_learned: u64,
    pub cache_hits: u64,
    pub cache_misses: u64,
}

impl LearnedFactsDb {
    /// Create a new empty database
    pub fn new() -> Self {
        Self::default()
    }

    /// Load from disk
    pub fn load() -> Self {
        let path = Self::db_path();
        if path.exists() {
            if let Ok(content) = fs::read_to_string(&path) {
                if let Ok(db) = serde_json::from_str(&content) {
                    return db;
                }
            }
        }
        Self::new()
    }

    /// Save to disk
    pub fn save(&self) -> anyhow::Result<()> {
        let path = Self::db_path();
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }
        let content = serde_json::to_string_pretty(self)?;
        fs::write(&path, content)?;
        Ok(())
    }

    /// Get the database path
    fn db_path() -> PathBuf {
        // Use XDG_DATA_HOME or ~/.local/share
        std::env::var("XDG_DATA_HOME")
            .map(PathBuf::from)
            .unwrap_or_else(|_| {
                std::env::var("HOME")
                    .map(|h| PathBuf::from(h).join(".local/share"))
                    .unwrap_or_else(|_| PathBuf::from("/tmp"))
            })
            .join("anna")
            .join("learned_facts.json")
    }

    /// Get a fact key for the HashMap
    fn fact_key(category: &FactCategory) -> String {
        match category {
            FactCategory::InstalledPackage(pkg) => format!("pkg:{}", pkg),
            FactCategory::Custom(name) => format!("custom:{}", name),
            other => format!("{:?}", other),
        }
    }

    /// Learn a new fact
    pub fn learn(&mut self, fact: LearnedFact) {
        let key = Self::fact_key(&fact.category);
        self.facts.insert(key, fact);
        self.total_facts_learned += 1;
        let _ = self.save();
    }

    /// Get a fact if it's fresh
    pub fn get_fresh(&mut self, category: &FactCategory) -> Option<&LearnedFact> {
        let key = Self::fact_key(category);
        if let Some(fact) = self.facts.get(&key) {
            if fact.is_fresh() && !fact.needs_boot_refresh() {
                self.cache_hits += 1;
                return Some(fact);
            }
        }
        self.cache_misses += 1;
        None
    }

    /// Get a fact and mark it as used
    pub fn use_fact(&mut self, category: &FactCategory) -> Option<LearnedFact> {
        let key = Self::fact_key(category);

        // Check if fact exists and is fresh
        let should_return = self.facts.get(&key)
            .map(|f| f.is_fresh() && !f.needs_boot_refresh())
            .unwrap_or(false);

        if should_return {
            // Now safely mutate and return
            if let Some(fact) = self.facts.get_mut(&key) {
                fact.mark_used();
                let result = fact.clone();
                self.cache_hits += 1;
                let _ = self.save();
                return Some(result);
            }
        }

        self.cache_misses += 1;
        None
    }

    /// Invalidate all package-related facts
    pub fn invalidate_packages(&mut self) {
        let keys_to_remove: Vec<String> = self.facts.keys()
            .filter(|k| k.starts_with("pkg:") ||
                       k.contains("Orphan") ||
                       k.contains("Update"))
            .cloned()
            .collect();

        for key in keys_to_remove {
            self.facts.remove(&key);
        }
        self.last_pacman_operation = Some(Utc::now());
        let _ = self.save();
    }

    /// Invalidate facts that depend on current session
    pub fn invalidate_session(&mut self) {
        let session_categories = [
            "DesktopEnvironment",
            "WindowManager",
            "DisplayServer",
        ];

        for cat in session_categories {
            self.facts.remove(cat);
        }
        let _ = self.save();
    }

    /// Get all facts for display
    pub fn all_facts(&self) -> Vec<&LearnedFact> {
        self.facts.values().collect()
    }

    /// Get statistics
    pub fn stats(&self) -> FactsStats {
        let fresh_count = self.facts.values().filter(|f| f.is_fresh()).count();
        let stale_count = self.facts.len() - fresh_count;

        FactsStats {
            total_facts: self.facts.len(),
            fresh_facts: fresh_count,
            stale_facts: stale_count,
            cache_hit_rate: if self.cache_hits + self.cache_misses > 0 {
                self.cache_hits as f32 / (self.cache_hits + self.cache_misses) as f32
            } else {
                0.0
            },
            total_queries: self.total_queries_answered,
        }
    }

    /// Prune very old or unused facts
    pub fn prune_old_facts(&mut self) {
        let cutoff = Utc::now() - Duration::days(7);

        let keys_to_remove: Vec<String> = self.facts.iter()
            .filter(|(_, fact)| {
                // Remove if older than 7 days AND never used
                fact.learned_at < cutoff && fact.usage_count == 0
            })
            .map(|(k, _)| k.clone())
            .collect();

        for key in keys_to_remove {
            self.facts.remove(&key);
        }
        let _ = self.save();
    }
}

/// Statistics about learned facts
#[derive(Debug, Clone)]
pub struct FactsStats {
    pub total_facts: usize,
    pub fresh_facts: usize,
    pub stale_facts: usize,
    pub cache_hit_rate: f32,
    pub total_queries: u64,
}

/// Parse command output to learn facts
pub struct FactLearner;

impl FactLearner {
    /// Learn CPU model from lscpu output
    pub fn learn_cpu_from_lscpu(output: &str) -> Vec<LearnedFact> {
        let mut facts = Vec::new();

        for line in output.lines() {
            let parts: Vec<&str> = line.splitn(2, ':').collect();
            if parts.len() != 2 {
                continue;
            }

            let key = parts[0].trim();
            let value = parts[1].trim();

            match key {
                "Model name" => {
                    facts.push(LearnedFact::new(
                        FactCategory::CpuModel,
                        value.to_string(),
                        line.to_string(),
                        "lscpu".to_string(),
                        0.95,
                    ));
                }
                "CPU(s)" => {
                    if let Ok(threads) = value.parse::<u32>() {
                        facts.push(LearnedFact::new(
                            FactCategory::CpuThreads,
                            format!("{} threads", threads),
                            line.to_string(),
                            "lscpu".to_string(),
                            0.95,
                        ));
                    }
                }
                "Core(s) per socket" => {
                    // Need to combine with socket count for total cores
                    // Store temporarily
                }
                "Flags" => {
                    // Extract SSE and AVX features
                    let features: Vec<&str> = value.split_whitespace()
                        .filter(|f| f.starts_with("sse") || f.starts_with("avx"))
                        .collect();

                    if !features.is_empty() {
                        facts.push(LearnedFact::new(
                            FactCategory::CpuFeatures,
                            features.join(", "),
                            format!("Flags: {}", features.join(" ")),
                            "lscpu".to_string(),
                            0.95,
                        ));
                    }
                }
                _ => {}
            }
        }

        facts
    }

    /// Learn RAM from free -h output
    pub fn learn_ram_from_free(output: &str) -> Option<LearnedFact> {
        for line in output.lines() {
            if line.starts_with("Mem:") {
                let parts: Vec<&str> = line.split_whitespace().collect();
                if parts.len() >= 2 {
                    return Some(LearnedFact::new(
                        FactCategory::TotalRam,
                        parts[1].to_string(),
                        line.to_string(),
                        "free -h".to_string(),
                        0.95,
                    ));
                }
            }
        }
        None
    }

    /// Learn GPU from lspci output
    pub fn learn_gpu_from_lspci(output: &str) -> Option<LearnedFact> {
        for line in output.lines() {
            let lower = line.to_lowercase();
            if lower.contains("vga") || lower.contains("3d") || lower.contains("display") {
                // Extract the GPU name after the colon
                if let Some(colon_pos) = line.find(':') {
                    let gpu_info = line[colon_pos + 1..].trim();
                    return Some(LearnedFact::new(
                        FactCategory::GpuModel,
                        gpu_info.to_string(),
                        line.to_string(),
                        "lspci | grep -iE 'vga|3d|display'".to_string(),
                        0.90,
                    ));
                }
            }
        }
        None
    }

    /// Learn package installation status
    pub fn learn_package_from_pacman(package: &str, output: &str) -> LearnedFact {
        let is_installed = !output.trim().is_empty();
        LearnedFact::new(
            FactCategory::InstalledPackage(package.to_string()),
            if is_installed {
                format!("{} is installed", package)
            } else {
                format!("{} is not installed", package)
            },
            output.to_string(),
            format!("pacman -Qs {}", package),
            0.99,
        )
    }

    /// Learn disk usage from df output
    pub fn learn_disk_from_df(output: &str) -> Vec<LearnedFact> {
        let mut facts = Vec::new();

        for line in output.lines() {
            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.len() >= 6 {
                let mount = parts[5];
                let available = parts[3];

                if mount == "/" {
                    facts.push(LearnedFact::new(
                        FactCategory::DiskUsageRoot,
                        format!("{} free", available),
                        line.to_string(),
                        "df -h".to_string(),
                        0.95,
                    ));
                } else if mount == "/home" {
                    facts.push(LearnedFact::new(
                        FactCategory::DiskUsageHome,
                        format!("{} free", available),
                        line.to_string(),
                        "df -h".to_string(),
                        0.95,
                    ));
                }
            }
        }

        facts
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_freshness_tiers() {
        assert!(FreshnessTier::Static.max_age() > FreshnessTier::Stable.max_age());
        assert!(FreshnessTier::Stable.max_age() > FreshnessTier::Moderate.max_age());
        assert!(FreshnessTier::Volatile.max_age() == Duration::seconds(0));
    }

    #[test]
    fn test_fact_freshness() {
        let fact = LearnedFact::new(
            FactCategory::CpuModel,
            "Intel i9".to_string(),
            "Model name: Intel i9".to_string(),
            "lscpu".to_string(),
            0.95,
        );

        assert!(fact.is_fresh()); // Just created, should be fresh
    }

    #[test]
    fn test_learn_ram() {
        let output = "              total        used        free      shared  buff/cache   available
Mem:            31Gi        20Gi       1.2Gi       1.1Gi        10Gi       9.1Gi
Swap:             0B          0B          0B";

        let fact = FactLearner::learn_ram_from_free(output).unwrap();
        assert_eq!(fact.value, "31Gi");
    }
}
