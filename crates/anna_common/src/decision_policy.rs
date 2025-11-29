//! Decision Policy for v0.92.0
//!
//! Central routing logic for Anna's question handling.
//! Chooses between Brain-only, Brain+Senior, Junior+Senior paths.

use crate::xp_track::XpStore;
use serde::{Deserialize, Serialize};

// ============================================================================
// Brain Domain Classification
// ============================================================================

/// Domain categories for Brain-answerable questions
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum BrainDomain {
    /// CPU info: cores, threads, model
    CpuInfo,
    /// RAM info: total, free, usage
    RamInfo,
    /// Anna health: status, version, uptime
    AnnaHealth,
    /// Disk layout: partitions, mount points
    DiskLayout,
    /// Anna logs: journalctl, recent errors
    AnnaLogs,
    /// System updates: package status
    SystemUpdates,
    /// Unknown domain - can't route to Brain
    Unknown,
}

impl BrainDomain {
    /// Classify question into domain
    pub fn classify(question: &str) -> Self {
        let q = question.to_lowercase();

        // CPU patterns
        if q.contains("cpu") || q.contains("processor") || q.contains("core")
            || q.contains("thread") || q.contains("lscpu")
        {
            return BrainDomain::CpuInfo;
        }

        // RAM patterns
        if q.contains("ram") || q.contains("memory") || q.contains("free")
            || q.contains("swap") || q.contains("meminfo")
        {
            return BrainDomain::RamInfo;
        }

        // Anna health patterns
        if q.contains("anna status") || q.contains("anna health")
            || q.contains("anna version") || q.contains("anna running")
            || (q.contains("anna") && q.contains("ok"))
        {
            return BrainDomain::AnnaHealth;
        }

        // Disk patterns
        if q.contains("disk") || q.contains("partition") || q.contains("mount")
            || q.contains("storage") || q.contains("lsblk") || q.contains("df ")
            || q.ends_with("df")
        {
            return BrainDomain::DiskLayout;
        }

        // Log patterns - requires Brain + Senior
        if q.contains("log") || q.contains("journalctl") || q.contains("error")
            || q.contains("recent") || q.contains("annad")
        {
            return BrainDomain::AnnaLogs;
        }

        // Update patterns - requires Brain + Senior
        if q.contains("update") || q.contains("pacman") || q.contains("apt")
            || q.contains("upgrade") || q.contains("package")
        {
            return BrainDomain::SystemUpdates;
        }

        BrainDomain::Unknown
    }

    /// Whether this domain can be answered by Brain alone
    pub fn is_brain_only(&self) -> bool {
        matches!(
            self,
            BrainDomain::CpuInfo
                | BrainDomain::RamInfo
                | BrainDomain::AnnaHealth
                | BrainDomain::DiskLayout
        )
    }

    /// Whether this domain needs Brain + Senior summary
    pub fn needs_senior_summary(&self) -> bool {
        matches!(self, BrainDomain::AnnaLogs | BrainDomain::SystemUpdates)
    }
}

// ============================================================================
// Decision Plan
// ============================================================================

/// Routing decision for a question
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum DecisionPlan {
    /// Brain-only path - no LLM calls (CPU, RAM, Health, Disk)
    BrainOnly(BrainDomain),

    /// Brain probe + Senior summarization (Logs, Updates)
    BrainPlusSeniorSummary(BrainDomain),

    /// Full Junior planning + Senior audit (complex questions)
    JuniorAndSenior,

    /// Fast fail - dangerous or unsupported question
    FailFast(String),
}

impl DecisionPlan {
    /// Get human-readable description
    pub fn describe(&self) -> String {
        match self {
            DecisionPlan::BrainOnly(domain) => {
                format!("BrainOnly({:?}) - no LLM, instant answer", domain)
            }
            DecisionPlan::BrainPlusSeniorSummary(domain) => {
                format!("Brain+Senior({:?}) - probe + summarize", domain)
            }
            DecisionPlan::JuniorAndSenior => {
                "JuniorAndSenior - full LLM orchestration".to_string()
            }
            DecisionPlan::FailFast(reason) => {
                format!("FailFast - {}", reason)
            }
        }
    }

    /// Is this a fast path (Brain only)?
    pub fn is_fast_path(&self) -> bool {
        matches!(self, DecisionPlan::BrainOnly(_))
    }

    /// Does this plan use Junior?
    pub fn uses_junior(&self) -> bool {
        matches!(self, DecisionPlan::JuniorAndSenior)
    }

    /// Does this plan use Senior?
    pub fn uses_senior(&self) -> bool {
        matches!(
            self,
            DecisionPlan::BrainPlusSeniorSummary(_) | DecisionPlan::JuniorAndSenior
        )
    }
}

// ============================================================================
// Agent Health (Circuit Breaker State)
// ============================================================================

/// Circuit breaker health state for an LLM agent
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentHealth {
    /// Agent name (Junior/Senior)
    pub name: String,
    /// Recent timeout count (rolling window)
    pub recent_timeouts: u32,
    /// Recent failure count (rolling window)
    pub recent_failures: u32,
    /// Recent success count (rolling window)
    pub recent_successes: u32,
    /// Rolling average latency in ms
    pub rolling_avg_latency_ms: u64,
    /// Is agent in degraded state?
    pub is_degraded: bool,
    /// Timestamp of last state update
    pub last_update: u64,
}

impl AgentHealth {
    pub fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
            recent_timeouts: 0,
            recent_failures: 0,
            recent_successes: 0,
            rolling_avg_latency_ms: 0,
            is_degraded: false,
            last_update: Self::now(),
        }
    }

    fn now() -> u64 {
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs()
    }

    /// Record a successful call
    pub fn record_success(&mut self, latency_ms: u64) {
        self.recent_successes += 1;
        self.update_latency(latency_ms);

        // Decay failures on success
        if self.recent_successes >= 3 && self.is_degraded {
            self.is_degraded = false;
        }
        self.recent_failures = self.recent_failures.saturating_sub(1);
        self.recent_timeouts = self.recent_timeouts.saturating_sub(1);
        self.last_update = Self::now();
    }

    /// Record a timeout
    pub fn record_timeout(&mut self) {
        self.recent_timeouts += 1;
        self.recent_successes = self.recent_successes.saturating_sub(2);
        self.check_degraded();
        self.last_update = Self::now();
    }

    /// Record a failure (parse error, invalid response)
    pub fn record_failure(&mut self) {
        self.recent_failures += 1;
        self.recent_successes = self.recent_successes.saturating_sub(1);
        self.check_degraded();
        self.last_update = Self::now();
    }

    /// Update rolling average latency
    fn update_latency(&mut self, latency_ms: u64) {
        if self.rolling_avg_latency_ms == 0 {
            self.rolling_avg_latency_ms = latency_ms;
        } else {
            // Exponential moving average (weight=0.2 for new value)
            self.rolling_avg_latency_ms =
                ((self.rolling_avg_latency_ms as f64 * 0.8) + (latency_ms as f64 * 0.2)) as u64;
        }
    }

    /// Check if agent should be degraded
    fn check_degraded(&mut self) {
        // Degrade if 3+ timeouts or 5+ failures in recent window
        if self.recent_timeouts >= 3 || self.recent_failures >= 5 {
            self.is_degraded = true;
        }
    }

    /// Reset health counters (e.g., after successful recovery)
    pub fn reset(&mut self) {
        self.recent_timeouts = 0;
        self.recent_failures = 0;
        self.recent_successes = 0;
        self.is_degraded = false;
        self.last_update = Self::now();
    }

    /// Format for status display
    pub fn format_status(&self) -> String {
        let status = if self.is_degraded { "DEGRADED" } else { "OK" };
        format!(
            "{}: {} (avg {}ms, timeouts={}, failures={}, successes={})",
            self.name, status, self.rolling_avg_latency_ms,
            self.recent_timeouts, self.recent_failures, self.recent_successes
        )
    }
}

// ============================================================================
// Path Metrics
// ============================================================================

/// Per-path latency and call tracking
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct PathMetrics {
    /// Brain path average latency (ms)
    pub brain_latency_avg_ms: u64,
    /// Brain path call count
    pub brain_calls: u64,

    /// Junior path average latency (ms)
    pub junior_latency_avg_ms: u64,
    /// Junior path call count
    pub junior_calls: u64,

    /// Senior path average latency (ms)
    pub senior_latency_avg_ms: u64,
    /// Senior path call count
    pub senior_calls: u64,

    /// Full orchestration average latency (ms)
    pub full_latency_avg_ms: u64,
    /// Full orchestration call count
    pub full_calls: u64,
}

impl PathMetrics {
    pub fn new() -> Self {
        Self::default()
    }

    /// Record Brain path latency
    pub fn record_brain(&mut self, latency_ms: u64) {
        self.brain_calls += 1;
        self.brain_latency_avg_ms = Self::update_avg(
            self.brain_latency_avg_ms,
            self.brain_calls,
            latency_ms,
        );
    }

    /// Record Junior call latency
    pub fn record_junior(&mut self, latency_ms: u64) {
        self.junior_calls += 1;
        self.junior_latency_avg_ms = Self::update_avg(
            self.junior_latency_avg_ms,
            self.junior_calls,
            latency_ms,
        );
    }

    /// Record Senior call latency
    pub fn record_senior(&mut self, latency_ms: u64) {
        self.senior_calls += 1;
        self.senior_latency_avg_ms = Self::update_avg(
            self.senior_latency_avg_ms,
            self.senior_calls,
            latency_ms,
        );
    }

    /// Record full orchestration latency
    pub fn record_full(&mut self, latency_ms: u64) {
        self.full_calls += 1;
        self.full_latency_avg_ms = Self::update_avg(
            self.full_latency_avg_ms,
            self.full_calls,
            latency_ms,
        );
    }

    fn update_avg(current_avg: u64, count: u64, new_value: u64) -> u64 {
        if count <= 1 {
            new_value
        } else {
            // Running average
            ((current_avg as f64 * (count - 1) as f64 + new_value as f64) / count as f64) as u64
        }
    }

    /// Format for status display
    pub fn format_status(&self) -> String {
        format!(
            "Path Latencies:\n  Brain: {}ms avg ({} calls)\n  Junior: {}ms avg ({} calls)\n  Senior: {}ms avg ({} calls)\n  Full: {}ms avg ({} calls)",
            self.brain_latency_avg_ms, self.brain_calls,
            self.junior_latency_avg_ms, self.junior_calls,
            self.senior_latency_avg_ms, self.senior_calls,
            self.full_latency_avg_ms, self.full_calls
        )
    }
}

// ============================================================================
// Decision Policy Engine
// ============================================================================

/// Central decision policy engine
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DecisionPolicy {
    /// Junior agent health
    pub junior_health: AgentHealth,
    /// Senior agent health
    pub senior_health: AgentHealth,
    /// Path metrics
    pub path_metrics: PathMetrics,
}

impl Default for DecisionPolicy {
    fn default() -> Self {
        Self::new()
    }
}

impl DecisionPolicy {
    pub fn new() -> Self {
        Self {
            junior_health: AgentHealth::new("Junior"),
            senior_health: AgentHealth::new("Senior"),
            path_metrics: PathMetrics::new(),
        }
    }

    /// Storage path for decision policy state
    fn storage_path() -> std::path::PathBuf {
        let dir = std::path::PathBuf::from("/var/lib/anna/policy");
        dir.join("decision_policy.json")
    }

    /// Load from disk
    pub fn load() -> Self {
        let path = Self::storage_path();
        if let Ok(data) = std::fs::read_to_string(&path) {
            serde_json::from_str(&data).unwrap_or_default()
        } else {
            Self::new()
        }
    }

    /// Save to disk
    pub fn save(&self) -> std::io::Result<()> {
        let path = Self::storage_path();
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let data = serde_json::to_string_pretty(self)?;
        std::fs::write(path, data)
    }

    /// Choose execution path for a question
    ///
    /// Decision logic:
    /// 1. Classify question domain
    /// 2. Check for dangerous patterns → FailFast
    /// 3. If Brain-only domain → BrainOnly
    /// 4. If Logs/Updates domain → BrainPlusSeniorSummary
    /// 5. If Junior is degraded → BrainOnly fallback or FailFast
    /// 6. Otherwise → JuniorAndSenior
    pub fn choose_path(&self, question: &str, xp_store: Option<&XpStore>) -> DecisionPlan {
        // Step 1: Classify domain
        let domain = BrainDomain::classify(question);

        // Step 2: Check for dangerous patterns
        if Self::is_dangerous(question) {
            return DecisionPlan::FailFast("Dangerous command detected".to_string());
        }

        // Step 3: Check if Junior is degraded
        if self.junior_health.is_degraded {
            // Fall back to Brain-only if possible
            if domain.is_brain_only() {
                return DecisionPlan::BrainOnly(domain);
            }
            // For logs/updates, still try Senior path
            if domain.needs_senior_summary() && !self.senior_health.is_degraded {
                return DecisionPlan::BrainPlusSeniorSummary(domain);
            }
            // Otherwise fail fast
            return DecisionPlan::FailFast("Junior agent degraded".to_string());
        }

        // Step 4: Trust-based adjustments
        if let Some(xp) = xp_store {
            // High Anna trust + Brain domain → prefer Brain
            if xp.should_anna_use_brain() && domain.is_brain_only() {
                return DecisionPlan::BrainOnly(domain);
            }
        }

        // Step 5: Route based on domain
        match domain {
            BrainDomain::CpuInfo
            | BrainDomain::RamInfo
            | BrainDomain::AnnaHealth
            | BrainDomain::DiskLayout => DecisionPlan::BrainOnly(domain),

            BrainDomain::AnnaLogs | BrainDomain::SystemUpdates => {
                if self.senior_health.is_degraded {
                    // Senior is degraded, try Junior+Senior anyway
                    DecisionPlan::JuniorAndSenior
                } else {
                    DecisionPlan::BrainPlusSeniorSummary(domain)
                }
            }

            BrainDomain::Unknown => DecisionPlan::JuniorAndSenior,
        }
    }

    /// Check for dangerous patterns
    fn is_dangerous(question: &str) -> bool {
        let q = question.to_lowercase();
        let dangerous = [
            "rm -rf",
            "rm -f",
            "dd if=",
            "mkfs",
            "fdisk",
            ":(){:|:&};:",
            "chmod 777",
            "chmod -r",
            "> /dev/",
            "sudo rm",
            "format c:",
        ];
        dangerous.iter().any(|d| q.contains(d))
    }

    /// Record Junior success
    pub fn junior_success(&mut self, latency_ms: u64) {
        self.junior_health.record_success(latency_ms);
        self.path_metrics.record_junior(latency_ms);
        let _ = self.save();
    }

    /// Record Junior timeout
    pub fn junior_timeout(&mut self) {
        self.junior_health.record_timeout();
        let _ = self.save();
    }

    /// Record Junior failure
    pub fn junior_failure(&mut self) {
        self.junior_health.record_failure();
        let _ = self.save();
    }

    /// Record Senior success
    pub fn senior_success(&mut self, latency_ms: u64) {
        self.senior_health.record_success(latency_ms);
        self.path_metrics.record_senior(latency_ms);
        let _ = self.save();
    }

    /// Record Senior timeout
    pub fn senior_timeout(&mut self) {
        self.senior_health.record_timeout();
        let _ = self.save();
    }

    /// Record Senior failure
    pub fn senior_failure(&mut self) {
        self.senior_health.record_failure();
        let _ = self.save();
    }

    /// Record Brain path execution
    pub fn brain_executed(&mut self, latency_ms: u64) {
        self.path_metrics.record_brain(latency_ms);
        let _ = self.save();
    }

    /// Record full orchestration execution
    pub fn full_executed(&mut self, latency_ms: u64) {
        self.path_metrics.record_full(latency_ms);
        let _ = self.save();
    }

    /// Format status for display
    pub fn format_status(&self) -> String {
        let mut lines = Vec::new();
        lines.push("DECISION POLICY".to_string());
        lines.push("-".repeat(60));
        lines.push(format!("  {}", self.junior_health.format_status()));
        lines.push(format!("  {}", self.senior_health.format_status()));
        lines.push("-".repeat(60));
        lines.push(self.path_metrics.format_status());
        lines.join("\n")
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_brain_domain_classify() {
        assert_eq!(BrainDomain::classify("How many CPU cores?"), BrainDomain::CpuInfo);
        assert_eq!(BrainDomain::classify("What's my RAM usage?"), BrainDomain::RamInfo);
        assert_eq!(BrainDomain::classify("Is Anna running?"), BrainDomain::AnnaHealth);
        assert_eq!(BrainDomain::classify("Show disk partitions"), BrainDomain::DiskLayout);
        assert_eq!(BrainDomain::classify("Show Anna logs"), BrainDomain::AnnaLogs);
        assert_eq!(BrainDomain::classify("Check for updates"), BrainDomain::SystemUpdates);
        assert_eq!(BrainDomain::classify("What is the weather?"), BrainDomain::Unknown);
    }

    #[test]
    fn test_brain_domain_is_brain_only() {
        assert!(BrainDomain::CpuInfo.is_brain_only());
        assert!(BrainDomain::RamInfo.is_brain_only());
        assert!(BrainDomain::AnnaHealth.is_brain_only());
        assert!(BrainDomain::DiskLayout.is_brain_only());
        assert!(!BrainDomain::AnnaLogs.is_brain_only());
        assert!(!BrainDomain::SystemUpdates.is_brain_only());
        assert!(!BrainDomain::Unknown.is_brain_only());
    }

    #[test]
    fn test_decision_plan_properties() {
        let brain = DecisionPlan::BrainOnly(BrainDomain::CpuInfo);
        assert!(brain.is_fast_path());
        assert!(!brain.uses_junior());
        assert!(!brain.uses_senior());

        let summary = DecisionPlan::BrainPlusSeniorSummary(BrainDomain::AnnaLogs);
        assert!(!summary.is_fast_path());
        assert!(!summary.uses_junior());
        assert!(summary.uses_senior());

        let full = DecisionPlan::JuniorAndSenior;
        assert!(!full.is_fast_path());
        assert!(full.uses_junior());
        assert!(full.uses_senior());
    }

    #[test]
    fn test_agent_health_success() {
        let mut health = AgentHealth::new("Junior");
        health.record_success(100);
        assert_eq!(health.recent_successes, 1);
        assert_eq!(health.rolling_avg_latency_ms, 100);
        assert!(!health.is_degraded);
    }

    #[test]
    fn test_agent_health_degraded() {
        let mut health = AgentHealth::new("Junior");
        health.record_timeout();
        health.record_timeout();
        health.record_timeout();
        assert!(health.is_degraded);
    }

    #[test]
    fn test_agent_health_recovery() {
        let mut health = AgentHealth::new("Junior");
        health.record_timeout();
        health.record_timeout();
        health.record_timeout();
        assert!(health.is_degraded);

        // 3 successes should recover
        health.record_success(100);
        health.record_success(100);
        health.record_success(100);
        assert!(!health.is_degraded);
    }

    #[test]
    fn test_path_metrics() {
        let mut metrics = PathMetrics::new();
        metrics.record_brain(50);
        metrics.record_brain(100);
        assert_eq!(metrics.brain_calls, 2);
        assert_eq!(metrics.brain_latency_avg_ms, 75);
    }

    #[test]
    fn test_decision_policy_choose_path() {
        let policy = DecisionPolicy::new();

        // Brain-only questions
        let plan = policy.choose_path("How many CPU cores?", None);
        assert!(matches!(plan, DecisionPlan::BrainOnly(BrainDomain::CpuInfo)));

        // Logs need Senior
        let plan = policy.choose_path("Show Anna logs", None);
        assert!(matches!(plan, DecisionPlan::BrainPlusSeniorSummary(BrainDomain::AnnaLogs)));

        // Unknown goes to full orchestration
        let plan = policy.choose_path("What is quantum computing?", None);
        assert!(matches!(plan, DecisionPlan::JuniorAndSenior));
    }

    #[test]
    fn test_decision_policy_dangerous() {
        let policy = DecisionPolicy::new();

        let plan = policy.choose_path("rm -rf /", None);
        assert!(matches!(plan, DecisionPlan::FailFast(_)));

        let plan = policy.choose_path("dd if=/dev/zero of=/dev/sda", None);
        assert!(matches!(plan, DecisionPlan::FailFast(_)));
    }

    #[test]
    fn test_decision_policy_degraded_fallback() {
        let mut policy = DecisionPolicy::new();
        policy.junior_health.is_degraded = true;

        // Brain-only still works when Junior is degraded
        let plan = policy.choose_path("How many CPU cores?", None);
        assert!(matches!(plan, DecisionPlan::BrainOnly(BrainDomain::CpuInfo)));

        // Unknown fails fast when Junior is degraded
        let plan = policy.choose_path("What is quantum computing?", None);
        assert!(matches!(plan, DecisionPlan::FailFast(_)));
    }
}
