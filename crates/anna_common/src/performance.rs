//! Anna Performance v0.0.21 - Performance and Latency Sprint
//!
//! This module provides:
//! - Token budgets and response caps per role
//! - Read-only tool result caching
//! - LLM response caching (safe, redacted)
//! - Performance statistics tracking
//!
//! Cache Storage:
//! - Tool results: /var/lib/anna/internal/cache/tools/
//! - LLM responses: /var/lib/anna/internal/cache/llm/
//! - Performance stats: /var/lib/anna/internal/perf_stats.json

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::time::{Duration, SystemTime, UNIX_EPOCH};

// =============================================================================
// Constants and Paths
// =============================================================================

/// Cache base directory
pub const CACHE_DIR: &str = "/var/lib/anna/internal/cache";

/// Tool result cache directory
pub const TOOL_CACHE_DIR: &str = "/var/lib/anna/internal/cache/tools";

/// LLM response cache directory
pub const LLM_CACHE_DIR: &str = "/var/lib/anna/internal/cache/llm";

/// Performance stats file
pub const PERF_STATS_FILE: &str = "/var/lib/anna/internal/perf_stats.json";

/// Default TTL for tool cache (5 minutes)
pub const TOOL_CACHE_TTL_SECS: u64 = 300;

/// Default TTL for LLM cache (10 minutes)
pub const LLM_CACHE_TTL_SECS: u64 = 600;

/// Max cache entries per category (to prevent unbounded growth)
pub const MAX_CACHE_ENTRIES: usize = 1000;

// =============================================================================
// Token Budgets
// =============================================================================

/// Token budget configuration per role
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenBudget {
    /// Maximum tokens for response
    pub max_tokens: u32,
    /// Maximum milliseconds for generation
    pub max_ms: u64,
}

impl TokenBudget {
    /// Default budget for Translator role
    pub fn translator_default() -> Self {
        Self {
            max_tokens: 256,
            max_ms: 1500,
        }
    }

    /// Default budget for Junior role
    pub fn junior_default() -> Self {
        Self {
            max_tokens: 384,
            max_ms: 2500,
        }
    }

    /// Check if response exceeds budget
    pub fn is_exceeded(&self, tokens: u32, elapsed_ms: u64) -> bool {
        tokens > self.max_tokens || elapsed_ms > self.max_ms
    }

    /// Get budget status string
    pub fn status_string(&self, tokens: u32, elapsed_ms: u64) -> String {
        let token_status = if tokens > self.max_tokens {
            format!("OVER by {} tokens", tokens - self.max_tokens)
        } else {
            format!("{}/{} tokens", tokens, self.max_tokens)
        };

        let time_status = if elapsed_ms > self.max_ms {
            format!("OVER by {}ms", elapsed_ms - self.max_ms)
        } else {
            format!("{}ms/{}ms", elapsed_ms, self.max_ms)
        };

        format!("{}, {}", token_status, time_status)
    }
}

/// Budget settings configuration (for config.toml)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BudgetSettings {
    /// Translator token budget
    #[serde(default = "default_translator_budget")]
    pub translator: TokenBudget,

    /// Junior token budget
    #[serde(default = "default_junior_budget")]
    pub junior: TokenBudget,

    /// Whether to enforce budgets strictly (fail if exceeded)
    #[serde(default)]
    pub strict_enforcement: bool,

    /// Whether to log budget overruns
    #[serde(default = "default_true")]
    pub log_overruns: bool,
}

fn default_translator_budget() -> TokenBudget {
    TokenBudget::translator_default()
}

fn default_junior_budget() -> TokenBudget {
    TokenBudget::junior_default()
}

fn default_true() -> bool {
    true
}

impl Default for BudgetSettings {
    fn default() -> Self {
        Self {
            translator: TokenBudget::translator_default(),
            junior: TokenBudget::junior_default(),
            strict_enforcement: false,
            log_overruns: true,
        }
    }
}

/// Budget violation record
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BudgetViolation {
    /// Role that violated budget
    pub role: String,
    /// Actual tokens used
    pub tokens_used: u32,
    /// Token limit
    pub token_limit: u32,
    /// Actual time taken (ms)
    pub time_ms: u64,
    /// Time limit (ms)
    pub time_limit: u64,
    /// Timestamp (epoch seconds)
    pub timestamp: u64,
}

// =============================================================================
// Tool Result Cache
// =============================================================================

/// Cache key for tool results
#[derive(Debug, Clone, Hash, PartialEq, Eq, Serialize, Deserialize)]
pub struct ToolCacheKey {
    /// Tool name
    pub tool_name: String,
    /// Tool arguments (sorted for determinism)
    pub args: Vec<(String, String)>,
    /// Snapshot version hash (for cache invalidation)
    pub snapshot_hash: String,
}

impl ToolCacheKey {
    /// Create a new cache key
    pub fn new(tool_name: &str, args: &[(String, String)], snapshot_hash: &str) -> Self {
        let mut sorted_args = args.to_vec();
        sorted_args.sort_by(|a, b| a.0.cmp(&b.0));
        Self {
            tool_name: tool_name.to_string(),
            args: sorted_args,
            snapshot_hash: snapshot_hash.to_string(),
        }
    }

    /// Generate deterministic hash for filename
    pub fn to_hash(&self) -> String {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};

        let mut hasher = DefaultHasher::new();
        self.hash(&mut hasher);
        format!("{:016x}", hasher.finish())
    }
}

/// Cached tool result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolCacheEntry {
    /// The cache key (for validation)
    pub key: ToolCacheKey,
    /// The cached result (JSON)
    pub result: String,
    /// Creation timestamp (epoch seconds)
    pub created_at: u64,
    /// TTL in seconds
    pub ttl_secs: u64,
    /// Execution time of original call (ms)
    pub original_exec_ms: u64,
}

impl ToolCacheEntry {
    /// Check if cache entry is expired
    pub fn is_expired(&self) -> bool {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
        now > self.created_at + self.ttl_secs
    }

    /// Time saved by using cache (ms)
    pub fn time_saved(&self) -> u64 {
        self.original_exec_ms
    }
}

/// Tool result cache manager
pub struct ToolCache {
    cache_dir: PathBuf,
}

impl Default for ToolCache {
    fn default() -> Self {
        Self::new()
    }
}

impl ToolCache {
    /// Create new cache manager
    pub fn new() -> Self {
        Self {
            cache_dir: PathBuf::from(TOOL_CACHE_DIR),
        }
    }

    /// Ensure cache directory exists
    pub fn ensure_dir(&self) -> std::io::Result<()> {
        fs::create_dir_all(&self.cache_dir)
    }

    /// Get cache file path for a key
    fn cache_path(&self, key: &ToolCacheKey) -> PathBuf {
        self.cache_dir.join(format!("{}.json", key.to_hash()))
    }

    /// Get cached result if valid
    pub fn get(&self, key: &ToolCacheKey) -> Option<ToolCacheEntry> {
        let path = self.cache_path(key);
        if !path.exists() {
            return None;
        }

        let content = fs::read_to_string(&path).ok()?;
        let entry: ToolCacheEntry = serde_json::from_str(&content).ok()?;

        // Validate key matches
        if entry.key != *key {
            return None;
        }

        // Check expiry
        if entry.is_expired() {
            // Clean up expired entry
            let _ = fs::remove_file(&path);
            return None;
        }

        Some(entry)
    }

    /// Store result in cache
    pub fn put(
        &self,
        key: &ToolCacheKey,
        result: &str,
        exec_ms: u64,
        ttl_secs: u64,
    ) -> std::io::Result<()> {
        self.ensure_dir()?;

        let entry = ToolCacheEntry {
            key: key.clone(),
            result: result.to_string(),
            created_at: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs(),
            ttl_secs,
            original_exec_ms: exec_ms,
        };

        let content = serde_json::to_string_pretty(&entry)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;

        let path = self.cache_path(key);
        crate::atomic_write(path.to_str().unwrap_or(""), &content)
    }

    /// Clear expired entries
    pub fn prune(&self) -> std::io::Result<usize> {
        let mut pruned = 0;

        if let Ok(entries) = fs::read_dir(&self.cache_dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.extension().and_then(|e| e.to_str()) == Some("json") {
                    if let Ok(content) = fs::read_to_string(&path) {
                        if let Ok(cached) = serde_json::from_str::<ToolCacheEntry>(&content) {
                            if cached.is_expired() {
                                let _ = fs::remove_file(&path);
                                pruned += 1;
                            }
                        }
                    }
                }
            }
        }

        Ok(pruned)
    }

    /// Get cache statistics
    pub fn stats(&self) -> CacheStats {
        let mut stats = CacheStats::default();

        if let Ok(entries) = fs::read_dir(&self.cache_dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.extension().and_then(|e| e.to_str()) == Some("json") {
                    stats.total_entries += 1;
                    if let Ok(metadata) = entry.metadata() {
                        stats.total_size_bytes += metadata.len();
                    }
                    if let Ok(content) = fs::read_to_string(&path) {
                        if let Ok(cached) = serde_json::from_str::<ToolCacheEntry>(&content) {
                            if cached.is_expired() {
                                stats.expired_entries += 1;
                            }
                            *stats
                                .entries_by_tool
                                .entry(cached.key.tool_name.clone())
                                .or_insert(0) += 1;
                        }
                    }
                }
            }
        }

        stats
    }

    /// Clear all cached entries
    pub fn clear(&self) -> std::io::Result<usize> {
        let mut cleared = 0;

        if let Ok(entries) = fs::read_dir(&self.cache_dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.extension().and_then(|e| e.to_str()) == Some("json") {
                    let _ = fs::remove_file(&path);
                    cleared += 1;
                }
            }
        }

        Ok(cleared)
    }
}

// =============================================================================
// LLM Response Cache
// =============================================================================

/// Cache key for LLM responses (safe caching with redaction)
#[derive(Debug, Clone, Hash, PartialEq, Eq, Serialize, Deserialize)]
pub struct LlmCacheKey {
    /// Role: "translator" or "junior"
    pub role: String,
    /// Request text (normalized, redacted)
    pub request_hash: String,
    /// Evidence hashes (sorted for determinism)
    pub evidence_hashes: Vec<String>,
    /// Policy version
    pub policy_version: String,
    /// Model version/name
    pub model_version: String,
}

impl LlmCacheKey {
    /// Create a new cache key
    pub fn new(
        role: &str,
        request: &str,
        evidence_hashes: &[String],
        policy_version: &str,
        model_version: &str,
    ) -> Self {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};

        // Hash the request text (already redacted before calling this)
        let mut hasher = DefaultHasher::new();
        request.hash(&mut hasher);
        let request_hash = format!("{:016x}", hasher.finish());

        let mut sorted_hashes = evidence_hashes.to_vec();
        sorted_hashes.sort();

        Self {
            role: role.to_string(),
            request_hash,
            evidence_hashes: sorted_hashes,
            policy_version: policy_version.to_string(),
            model_version: model_version.to_string(),
        }
    }

    /// Generate deterministic hash for filename
    pub fn to_hash(&self) -> String {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};

        let mut hasher = DefaultHasher::new();
        self.hash(&mut hasher);
        format!("{:016x}", hasher.finish())
    }
}

/// Cached LLM response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LlmCacheEntry {
    /// The cache key (for validation)
    pub key: LlmCacheKey,
    /// The cached response (already redacted)
    pub response: String,
    /// Creation timestamp (epoch seconds)
    pub created_at: u64,
    /// TTL in seconds
    pub ttl_secs: u64,
    /// Original generation time (ms)
    pub original_gen_ms: u64,
    /// Tokens in response
    pub tokens: u32,
}

impl LlmCacheEntry {
    /// Check if cache entry is expired
    pub fn is_expired(&self) -> bool {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
        now > self.created_at + self.ttl_secs
    }
}

/// LLM response cache manager
pub struct LlmCache {
    cache_dir: PathBuf,
}

impl Default for LlmCache {
    fn default() -> Self {
        Self::new()
    }
}

impl LlmCache {
    /// Create new cache manager
    pub fn new() -> Self {
        Self {
            cache_dir: PathBuf::from(LLM_CACHE_DIR),
        }
    }

    /// Ensure cache directory exists
    pub fn ensure_dir(&self) -> std::io::Result<()> {
        fs::create_dir_all(&self.cache_dir)
    }

    /// Get cache file path for a key
    fn cache_path(&self, key: &LlmCacheKey) -> PathBuf {
        self.cache_dir
            .join(format!("{}_{}.json", key.role, key.to_hash()))
    }

    /// Get cached response if valid
    pub fn get(&self, key: &LlmCacheKey) -> Option<LlmCacheEntry> {
        let path = self.cache_path(key);
        if !path.exists() {
            return None;
        }

        let content = fs::read_to_string(&path).ok()?;
        let entry: LlmCacheEntry = serde_json::from_str(&content).ok()?;

        // Validate key matches
        if entry.key != *key {
            return None;
        }

        // Check expiry
        if entry.is_expired() {
            let _ = fs::remove_file(&path);
            return None;
        }

        Some(entry)
    }

    /// Store response in cache
    pub fn put(
        &self,
        key: &LlmCacheKey,
        response: &str,
        gen_ms: u64,
        tokens: u32,
        ttl_secs: u64,
    ) -> std::io::Result<()> {
        self.ensure_dir()?;

        let entry = LlmCacheEntry {
            key: key.clone(),
            response: response.to_string(),
            created_at: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs(),
            ttl_secs,
            original_gen_ms: gen_ms,
            tokens,
        };

        let content = serde_json::to_string_pretty(&entry)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;

        let path = self.cache_path(key);
        crate::atomic_write(path.to_str().unwrap_or(""), &content)
    }

    /// Get cache statistics by role
    pub fn stats(&self) -> LlmCacheStats {
        let mut stats = LlmCacheStats::default();

        if let Ok(entries) = fs::read_dir(&self.cache_dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.extension().and_then(|e| e.to_str()) == Some("json") {
                    if let Ok(content) = fs::read_to_string(&path) {
                        if let Ok(cached) = serde_json::from_str::<LlmCacheEntry>(&content) {
                            if !cached.is_expired() {
                                if cached.key.role == "translator" {
                                    stats.translator_entries += 1;
                                } else if cached.key.role == "junior" {
                                    stats.junior_entries += 1;
                                }
                            }
                            if let Ok(metadata) = entry.metadata() {
                                stats.total_size_bytes += metadata.len();
                            }
                        }
                    }
                }
            }
        }

        stats
    }

    /// Clear all cached entries
    pub fn clear(&self) -> std::io::Result<usize> {
        let mut cleared = 0;

        if let Ok(entries) = fs::read_dir(&self.cache_dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.extension().and_then(|e| e.to_str()) == Some("json") {
                    let _ = fs::remove_file(&path);
                    cleared += 1;
                }
            }
        }

        Ok(cleared)
    }
}

// =============================================================================
// Cache Statistics
// =============================================================================

/// Statistics for tool cache
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct CacheStats {
    /// Total entries in cache
    pub total_entries: usize,
    /// Expired entries (pending cleanup)
    pub expired_entries: usize,
    /// Total size in bytes
    pub total_size_bytes: u64,
    /// Entries by tool name
    pub entries_by_tool: HashMap<String, usize>,
}

/// Statistics for LLM cache
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct LlmCacheStats {
    /// Translator cache entries
    pub translator_entries: usize,
    /// Junior cache entries
    pub junior_entries: usize,
    /// Total size in bytes
    pub total_size_bytes: u64,
}

// =============================================================================
// Performance Statistics
// =============================================================================

/// Latency sample for tracking
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LatencySample {
    /// Timestamp (epoch seconds)
    pub timestamp: u64,
    /// Total request latency (ms)
    pub total_ms: u64,
    /// Translator latency (ms)
    pub translator_ms: u64,
    /// Junior latency (ms)
    pub junior_ms: u64,
    /// Tool execution latency (ms)
    pub tools_ms: u64,
    /// Whether cache was used
    pub cache_hit: bool,
}

/// Performance statistics tracker
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerfStats {
    /// Schema version
    pub schema_version: u32,
    /// Date string (YYYY-MM-DD)
    pub date: String,
    /// Latency samples (rolling 24h)
    pub samples: Vec<LatencySample>,
    /// Cache hits today
    pub cache_hits: u64,
    /// Cache misses today
    pub cache_misses: u64,
    /// Tool cache entries by name with hit count
    pub tool_cache_hits: HashMap<String, u64>,
    /// Budget violations today
    pub budget_violations: Vec<BudgetViolation>,
}

impl Default for PerfStats {
    fn default() -> Self {
        let today = chrono::Local::now().format("%Y-%m-%d").to_string();
        Self {
            schema_version: 1,
            date: today,
            samples: Vec::new(),
            cache_hits: 0,
            cache_misses: 0,
            tool_cache_hits: HashMap::new(),
            budget_violations: Vec::new(),
        }
    }
}

impl PerfStats {
    /// Load stats from file, resetting if date changed
    pub fn load() -> Self {
        let today = chrono::Local::now().format("%Y-%m-%d").to_string();
        let path = Path::new(PERF_STATS_FILE);

        if path.exists() {
            if let Ok(content) = fs::read_to_string(path) {
                if let Ok(mut stats) = serde_json::from_str::<PerfStats>(&content) {
                    if stats.date == today {
                        // Prune old samples (keep last 24h)
                        let cutoff = SystemTime::now()
                            .duration_since(UNIX_EPOCH)
                            .unwrap_or_default()
                            .as_secs()
                            .saturating_sub(86400);
                        stats.samples.retain(|s| s.timestamp >= cutoff);
                        return stats;
                    }
                }
            }
        }

        // New day or no file
        Self::default()
    }

    /// Save stats to file
    pub fn save(&self) -> std::io::Result<()> {
        let parent = Path::new(PERF_STATS_FILE).parent();
        if let Some(p) = parent {
            fs::create_dir_all(p)?;
        }

        let content = serde_json::to_string_pretty(self)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;
        crate::atomic_write(PERF_STATS_FILE, &content)
    }

    /// Record a latency sample
    pub fn record_sample(&mut self, sample: LatencySample) {
        if sample.cache_hit {
            self.cache_hits += 1;
        } else {
            self.cache_misses += 1;
        }
        self.samples.push(sample);

        // Keep max 10000 samples
        if self.samples.len() > 10000 {
            self.samples.drain(0..1000);
        }
    }

    /// Record a tool cache hit
    pub fn record_tool_cache_hit(&mut self, tool_name: &str) {
        *self
            .tool_cache_hits
            .entry(tool_name.to_string())
            .or_insert(0) += 1;
        self.cache_hits += 1;
    }

    /// Record a budget violation
    pub fn record_violation(&mut self, violation: BudgetViolation) {
        self.budget_violations.push(violation);
    }

    /// Get cache hit rate (0.0 - 1.0)
    pub fn cache_hit_rate(&self) -> f64 {
        let total = self.cache_hits + self.cache_misses;
        if total == 0 {
            0.0
        } else {
            self.cache_hits as f64 / total as f64
        }
    }

    /// Get average total latency (last 24h)
    pub fn avg_total_latency_ms(&self) -> u64 {
        if self.samples.is_empty() {
            return 0;
        }
        let sum: u64 = self.samples.iter().map(|s| s.total_ms).sum();
        sum / self.samples.len() as u64
    }

    /// Get average translator latency (last 24h)
    pub fn avg_translator_latency_ms(&self) -> u64 {
        if self.samples.is_empty() {
            return 0;
        }
        let sum: u64 = self.samples.iter().map(|s| s.translator_ms).sum();
        sum / self.samples.len() as u64
    }

    /// Get average junior latency (last 24h)
    pub fn avg_junior_latency_ms(&self) -> u64 {
        if self.samples.is_empty() {
            return 0;
        }
        let sum: u64 = self.samples.iter().map(|s| s.junior_ms).sum();
        sum / self.samples.len() as u64
    }

    /// Get top 5 cached tools by hit count
    pub fn top_cached_tools(&self, n: usize) -> Vec<(String, u64)> {
        let mut sorted: Vec<_> = self
            .tool_cache_hits
            .iter()
            .map(|(k, v)| (k.clone(), *v))
            .collect();
        sorted.sort_by(|a, b| b.1.cmp(&a.1));
        sorted.into_iter().take(n).collect()
    }

    /// Get sample count
    pub fn sample_count(&self) -> usize {
        self.samples.len()
    }
}

// =============================================================================
// Snapshot Hash for Cache Invalidation
// =============================================================================

/// Generate a hash from snapshot metadata for cache invalidation
pub fn get_snapshot_hash() -> String {
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};

    // Use snapshot sequence and timestamp for invalidation
    let snapshot_path = Path::new("/var/lib/anna/internal/snapshots/status.json");

    let mut hasher = DefaultHasher::new();

    if let Ok(metadata) = fs::metadata(snapshot_path) {
        if let Ok(modified) = metadata.modified() {
            if let Ok(duration) = modified.duration_since(UNIX_EPOCH) {
                duration.as_secs().hash(&mut hasher);
            }
        }
        metadata.len().hash(&mut hasher);
    }

    format!("{:016x}", hasher.finish())
}

/// Get current policy version for cache invalidation
pub fn get_policy_version() -> String {
    let policy_dir = Path::new("/etc/anna/policy");
    if !policy_dir.exists() {
        return "default".to_string();
    }

    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};

    let mut hasher = DefaultHasher::new();

    // Hash modification times of policy files
    for name in &["capabilities.toml", "blocked.toml", "risk.toml"] {
        let path = policy_dir.join(name);
        if let Ok(metadata) = fs::metadata(&path) {
            if let Ok(modified) = metadata.modified() {
                if let Ok(duration) = modified.duration_since(UNIX_EPOCH) {
                    duration.as_secs().hash(&mut hasher);
                }
            }
        }
    }

    format!("{:016x}", hasher.finish())
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_token_budget_defaults() {
        let translator = TokenBudget::translator_default();
        assert_eq!(translator.max_tokens, 256);
        assert_eq!(translator.max_ms, 1500);

        let junior = TokenBudget::junior_default();
        assert_eq!(junior.max_tokens, 384);
        assert_eq!(junior.max_ms, 2500);
    }

    #[test]
    fn test_token_budget_exceeded() {
        let budget = TokenBudget {
            max_tokens: 256,
            max_ms: 1500,
        };

        // Not exceeded
        assert!(!budget.is_exceeded(100, 500));
        assert!(!budget.is_exceeded(256, 1500));

        // Token exceeded
        assert!(budget.is_exceeded(257, 500));

        // Time exceeded
        assert!(budget.is_exceeded(100, 1501));

        // Both exceeded
        assert!(budget.is_exceeded(300, 2000));
    }

    #[test]
    fn test_tool_cache_key_determinism() {
        let key1 = ToolCacheKey::new(
            "test_tool",
            &[
                ("b".to_string(), "2".to_string()),
                ("a".to_string(), "1".to_string()),
            ],
            "hash123",
        );

        let key2 = ToolCacheKey::new(
            "test_tool",
            &[
                ("a".to_string(), "1".to_string()),
                ("b".to_string(), "2".to_string()),
            ],
            "hash123",
        );

        // Same hash regardless of argument order
        assert_eq!(key1.to_hash(), key2.to_hash());
    }

    #[test]
    fn test_tool_cache_key_different_args() {
        let key1 = ToolCacheKey::new(
            "test_tool",
            &[("a".to_string(), "1".to_string())],
            "hash123",
        );

        let key2 = ToolCacheKey::new(
            "test_tool",
            &[("a".to_string(), "2".to_string())],
            "hash123",
        );

        // Different hashes for different args
        assert_ne!(key1.to_hash(), key2.to_hash());
    }

    #[test]
    fn test_tool_cache_key_different_snapshot() {
        let key1 = ToolCacheKey::new("test_tool", &[], "hash123");
        let key2 = ToolCacheKey::new("test_tool", &[], "hash456");

        // Different hashes for different snapshot versions
        assert_ne!(key1.to_hash(), key2.to_hash());
    }

    #[test]
    fn test_llm_cache_key_determinism() {
        let key1 = LlmCacheKey::new(
            "translator",
            "what is my cpu",
            &["e1".to_string(), "e2".to_string()],
            "pol1",
            "model1",
        );

        let key2 = LlmCacheKey::new(
            "translator",
            "what is my cpu",
            &["e2".to_string(), "e1".to_string()], // Different order
            "pol1",
            "model1",
        );

        // Same hash regardless of evidence order
        assert_eq!(key1.to_hash(), key2.to_hash());
    }

    #[test]
    fn test_llm_cache_key_policy_invalidation() {
        let key1 = LlmCacheKey::new("translator", "test request", &[], "pol_v1", "model1");

        let key2 = LlmCacheKey::new(
            "translator",
            "test request",
            &[],
            "pol_v2", // Different policy
            "model1",
        );

        // Different policy = different key
        assert_ne!(key1.to_hash(), key2.to_hash());
    }

    #[test]
    fn test_llm_cache_key_model_invalidation() {
        let key1 = LlmCacheKey::new("translator", "test request", &[], "pol1", "model_v1");

        let key2 = LlmCacheKey::new(
            "translator",
            "test request",
            &[],
            "pol1",
            "model_v2", // Different model
        );

        // Different model = different key
        assert_ne!(key1.to_hash(), key2.to_hash());
    }

    #[test]
    fn test_perf_stats_default() {
        let stats = PerfStats::default();
        assert_eq!(stats.cache_hit_rate(), 0.0);
        assert_eq!(stats.avg_total_latency_ms(), 0);
        assert_eq!(stats.sample_count(), 0);
    }

    #[test]
    fn test_perf_stats_cache_hit_rate() {
        let mut stats = PerfStats::default();
        stats.cache_hits = 7;
        stats.cache_misses = 3;
        assert!((stats.cache_hit_rate() - 0.7).abs() < 0.001);
    }

    #[test]
    fn test_budget_settings_default() {
        let settings = BudgetSettings::default();
        assert_eq!(settings.translator.max_tokens, 256);
        assert_eq!(settings.translator.max_ms, 1500);
        assert_eq!(settings.junior.max_tokens, 384);
        assert_eq!(settings.junior.max_ms, 2500);
    }

    #[test]
    fn test_tool_cache_entry_expiry() {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();

        // Not expired
        let entry1 = ToolCacheEntry {
            key: ToolCacheKey::new("test", &[], "hash"),
            result: "{}".to_string(),
            created_at: now,
            ttl_secs: 300,
            original_exec_ms: 100,
        };
        assert!(!entry1.is_expired());

        // Expired
        let entry2 = ToolCacheEntry {
            key: ToolCacheKey::new("test", &[], "hash"),
            result: "{}".to_string(),
            created_at: now - 400,
            ttl_secs: 300,
            original_exec_ms: 100,
        };
        assert!(entry2.is_expired());
    }
}
