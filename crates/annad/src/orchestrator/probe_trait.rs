//! Probe Executor Trait Abstraction v1.0.0
//!
//! See `docs/architecture.md` Section 9 for design rationale.
//!
//! This module provides a trait abstraction over probe execution to enable:
//! - Deterministic testing with fake implementations
//! - No system calls required for testing
//! - Clear interface boundaries
//!
//! ## Usage
//!
//! Production code uses `RealProbeExecutor` which runs actual shell commands.
//! Test code uses `FakeProbeExecutor` with pre-configured responses.

use anna_common::{EvidenceStatus, ProbeCatalog, ProbeEvidenceV10};
use async_trait::async_trait;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

// ============================================================================
// Probe Executor Trait
// ============================================================================

/// Trait abstraction for probe execution
///
/// This trait defines the minimal interface needed by UnifiedEngine
/// to execute probes and get evidence.
#[async_trait]
pub trait ProbeExecutor: Send + Sync {
    /// Execute a single probe and return evidence
    async fn execute_probe(&self, probe_id: &str) -> ProbeEvidenceV10;

    /// Execute multiple probes (may be in parallel or sequential)
    async fn execute_probes(&self, probe_ids: &[String]) -> Vec<ProbeEvidenceV10>;

    /// Check if a probe ID is valid in this executor's catalog
    fn is_valid(&self, probe_id: &str) -> bool;

    /// Get the list of available probe IDs
    fn available_probes(&self) -> Vec<String>;
}

// ============================================================================
// Real Probe Executor (Production)
// ============================================================================

/// Real probe executor that runs actual shell commands
pub struct RealProbeExecutor {
    catalog: ProbeCatalog,
}

impl RealProbeExecutor {
    pub fn new() -> Self {
        Self {
            catalog: ProbeCatalog::standard(),
        }
    }

    pub fn with_catalog(catalog: ProbeCatalog) -> Self {
        Self { catalog }
    }
}

impl Default for RealProbeExecutor {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl ProbeExecutor for RealProbeExecutor {
    async fn execute_probe(&self, probe_id: &str) -> ProbeEvidenceV10 {
        match super::probe_executor::execute_probe(&self.catalog, probe_id).await {
            Ok(evidence) => evidence,
            Err(e) => ProbeEvidenceV10 {
                probe_id: probe_id.to_string(),
                timestamp: chrono::Utc::now().to_rfc3339(),
                status: EvidenceStatus::Error,
                command: "".to_string(),
                raw: Some(format!("Execution error: {}", e)),
                parsed: None,
            },
        }
    }

    async fn execute_probes(&self, probe_ids: &[String]) -> Vec<ProbeEvidenceV10> {
        super::probe_executor::execute_probes(&self.catalog, probe_ids).await
    }

    fn is_valid(&self, probe_id: &str) -> bool {
        self.catalog.is_valid(probe_id)
    }

    fn available_probes(&self) -> Vec<String> {
        self.catalog
            .available_probes()
            .iter()
            .map(|p| p.probe_id.clone())
            .collect()
    }
}

// ============================================================================
// Fake Probe Executor (Testing)
// ============================================================================

/// Pre-configured probe response for testing
#[derive(Debug, Clone)]
pub struct FakeProbeResponse {
    pub probe_id: String,
    pub status: EvidenceStatus,
    pub raw: Option<String>,
    pub parsed: Option<serde_json::Value>,
}

impl FakeProbeResponse {
    /// Create a successful probe response
    pub fn ok(probe_id: &str, output: &str) -> Self {
        Self {
            probe_id: probe_id.to_string(),
            status: EvidenceStatus::Ok,
            raw: Some(output.to_string()),
            parsed: None,
        }
    }

    /// Create a successful probe response with JSON
    pub fn ok_json(probe_id: &str, output: &str, json: serde_json::Value) -> Self {
        Self {
            probe_id: probe_id.to_string(),
            status: EvidenceStatus::Ok,
            raw: Some(output.to_string()),
            parsed: Some(json),
        }
    }

    /// Create an error probe response
    pub fn error(probe_id: &str, error_msg: &str) -> Self {
        Self {
            probe_id: probe_id.to_string(),
            status: EvidenceStatus::Error,
            raw: Some(format!("Error: {}", error_msg)),
            parsed: None,
        }
    }

    /// Create a not-found probe response
    pub fn not_found(probe_id: &str) -> Self {
        Self {
            probe_id: probe_id.to_string(),
            status: EvidenceStatus::NotFound,
            raw: Some(format!("Probe '{}' not in catalog", probe_id)),
            parsed: None,
        }
    }
}

/// Fake probe executor for deterministic testing
///
/// Provides pre-configured responses without running shell commands.
///
/// ## Example
///
/// ```rust,ignore
/// let fake = FakeProbeExecutorBuilder::new()
///     .probe_response(FakeProbeResponse::ok("cpu.info", "8 cores"))
///     .probe_response(FakeProbeResponse::ok("mem.info", "16 GB"))
///     .build();
///
/// let evidence = fake.execute_probe("cpu.info").await;
/// assert_eq!(evidence.status, EvidenceStatus::Ok);
/// ```
pub struct FakeProbeExecutor {
    /// Map of probe_id -> response
    responses: HashMap<String, FakeProbeResponse>,
    /// Default response for unknown probes
    default_response: FakeProbeResponse,
    /// Valid probe IDs
    valid_probes: Vec<String>,
    /// Track call counts for assertions
    call_counts: Arc<Mutex<HashMap<String, usize>>>,
}

impl FakeProbeExecutor {
    /// Create a new fake executor with common system probes pre-configured
    pub fn new() -> Self {
        let mut responses = HashMap::new();
        let valid_probes = vec![
            "cpu.info".to_string(),
            "mem.info".to_string(),
            "disk.df".to_string(),
            "net.interfaces".to_string(),
        ];

        // Pre-configure common probes
        responses.insert(
            "cpu.info".to_string(),
            FakeProbeResponse::ok("cpu.info", "CPU(s): 8\nModel name: Test CPU\nThread(s) per core: 2"),
        );
        responses.insert(
            "mem.info".to_string(),
            FakeProbeResponse::ok("mem.info", "MemTotal: 16777216 kB\nMemFree: 8388608 kB"),
        );
        responses.insert(
            "disk.df".to_string(),
            FakeProbeResponse::ok("disk.df", "/dev/sda1  100G  50G  50G  50% /"),
        );
        responses.insert(
            "net.interfaces".to_string(),
            FakeProbeResponse::ok("net.interfaces", "1: lo: <LOOPBACK>\n2: eth0: <BROADCAST>"),
        );

        Self {
            responses,
            default_response: FakeProbeResponse::not_found("unknown"),
            valid_probes,
            call_counts: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    /// Get the number of calls to a specific probe
    pub fn call_count(&self, probe_id: &str) -> usize {
        self.call_counts
            .lock()
            .unwrap()
            .get(probe_id)
            .copied()
            .unwrap_or(0)
    }

    /// Get total call count across all probes
    pub fn total_calls(&self) -> usize {
        self.call_counts.lock().unwrap().values().sum()
    }

    /// Reset all call counts
    pub fn reset_counts(&self) {
        self.call_counts.lock().unwrap().clear();
    }
}

impl Default for FakeProbeExecutor {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl ProbeExecutor for FakeProbeExecutor {
    async fn execute_probe(&self, probe_id: &str) -> ProbeEvidenceV10 {
        // Increment call count
        {
            let mut counts = self.call_counts.lock().unwrap();
            *counts.entry(probe_id.to_string()).or_insert(0) += 1;
        }

        let response = self
            .responses
            .get(probe_id)
            .cloned()
            .unwrap_or_else(|| FakeProbeResponse::not_found(probe_id));

        ProbeEvidenceV10 {
            probe_id: response.probe_id,
            timestamp: chrono::Utc::now().to_rfc3339(),
            status: response.status,
            command: format!("fake:{}", probe_id),
            raw: response.raw,
            parsed: response.parsed,
        }
    }

    async fn execute_probes(&self, probe_ids: &[String]) -> Vec<ProbeEvidenceV10> {
        let mut results = Vec::new();
        for probe_id in probe_ids {
            results.push(self.execute_probe(probe_id).await);
        }
        results
    }

    fn is_valid(&self, probe_id: &str) -> bool {
        self.valid_probes.contains(&probe_id.to_string())
    }

    fn available_probes(&self) -> Vec<String> {
        self.valid_probes.clone()
    }
}

// ============================================================================
// Builder for FakeProbeExecutor
// ============================================================================

/// Builder for FakeProbeExecutor with convenient test setup
pub struct FakeProbeExecutorBuilder {
    responses: HashMap<String, FakeProbeResponse>,
    valid_probes: Vec<String>,
    default_response: FakeProbeResponse,
}

impl FakeProbeExecutorBuilder {
    /// Create a new builder with no pre-configured probes
    pub fn new() -> Self {
        Self {
            responses: HashMap::new(),
            valid_probes: vec![],
            default_response: FakeProbeResponse::not_found("unknown"),
        }
    }

    /// Start with the default common probes
    pub fn with_defaults() -> Self {
        let fake = FakeProbeExecutor::new();
        Self {
            responses: fake.responses,
            valid_probes: fake.valid_probes,
            default_response: fake.default_response,
        }
    }

    /// Add a probe response
    pub fn probe_response(mut self, response: FakeProbeResponse) -> Self {
        let probe_id = response.probe_id.clone();
        if !self.valid_probes.contains(&probe_id) {
            self.valid_probes.push(probe_id.clone());
        }
        self.responses.insert(probe_id, response);
        self
    }

    /// Add multiple probe responses
    pub fn probe_responses(mut self, responses: Vec<FakeProbeResponse>) -> Self {
        for response in responses {
            self = self.probe_response(response);
        }
        self
    }

    /// Set the default response for unknown probes
    pub fn default_response(mut self, response: FakeProbeResponse) -> Self {
        self.default_response = response;
        self
    }

    /// Build the FakeProbeExecutor
    pub fn build(self) -> FakeProbeExecutor {
        FakeProbeExecutor {
            responses: self.responses,
            default_response: self.default_response,
            valid_probes: self.valid_probes,
            call_counts: Arc::new(Mutex::new(HashMap::new())),
        }
    }
}

impl Default for FakeProbeExecutorBuilder {
    fn default() -> Self {
        Self::new()
    }
}

// ============================================================================
// Helper constructors for common test scenarios
// ============================================================================

impl FakeProbeExecutor {
    /// Create a fake that returns a specific CPU info
    pub fn with_cpu_info(cores: u32, model: &str) -> Self {
        FakeProbeExecutorBuilder::with_defaults()
            .probe_response(FakeProbeResponse::ok(
                "cpu.info",
                &format!("CPU(s): {}\nModel name: {}\nThread(s) per core: 2", cores, model),
            ))
            .build()
    }

    /// Create a fake that returns specific memory info
    pub fn with_memory_info(total_gb: u32, free_gb: u32) -> Self {
        let total_kb = total_gb * 1024 * 1024;
        let free_kb = free_gb * 1024 * 1024;
        FakeProbeExecutorBuilder::with_defaults()
            .probe_response(FakeProbeResponse::ok(
                "mem.info",
                &format!("MemTotal: {} kB\nMemFree: {} kB", total_kb, free_kb),
            ))
            .build()
    }

    /// Create a fake where all probes fail with an error (not not-found)
    pub fn all_failing(error_msg: &str) -> Self {
        // Start with defaults so probes are "valid" but then override to return errors
        let mut builder = FakeProbeExecutorBuilder::with_defaults();
        // Override all the default responses with errors
        for probe_id in ["cpu.info", "mem.info", "disk.df", "net.interfaces"] {
            builder = builder.probe_response(FakeProbeResponse::error(probe_id, error_msg));
        }
        builder.default_response(FakeProbeResponse::error("unknown", error_msg))
              .build()
    }

    /// Create a fake for a specific question scenario
    pub fn for_cpu_question() -> Self {
        FakeProbeExecutorBuilder::with_defaults()
            .probe_response(FakeProbeResponse::ok_json(
                "cpu.info",
                "Architecture: x86_64\nCPU(s): 16\nModel name: AMD Ryzen 7 5800X\nThread(s) per core: 2\nCore(s) per socket: 8",
                serde_json::json!({
                    "architecture": "x86_64",
                    "cpus": 16,
                    "model": "AMD Ryzen 7 5800X",
                    "threads_per_core": 2,
                    "cores_per_socket": 8
                }),
            ))
            .build()
    }

    /// Create a fake for a memory question scenario
    pub fn for_memory_question() -> Self {
        FakeProbeExecutorBuilder::with_defaults()
            .probe_response(FakeProbeResponse::ok_json(
                "mem.info",
                "MemTotal: 33554432 kB\nMemFree: 16777216 kB\nMemAvailable: 25165824 kB",
                serde_json::json!({
                    "total_kb": 33554432,
                    "free_kb": 16777216,
                    "available_kb": 25165824
                }),
            ))
            .build()
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_fake_executor_default() {
        let fake = FakeProbeExecutor::new();

        assert!(fake.is_valid("cpu.info"));
        assert!(fake.is_valid("mem.info"));
        assert!(!fake.is_valid("nonexistent.probe"));
    }

    #[tokio::test]
    async fn test_fake_executor_execute() {
        let fake = FakeProbeExecutor::new();

        let evidence = fake.execute_probe("cpu.info").await;
        assert_eq!(evidence.status, EvidenceStatus::Ok);
        assert!(evidence.raw.unwrap().contains("CPU"));
    }

    #[tokio::test]
    async fn test_fake_executor_unknown_probe() {
        let fake = FakeProbeExecutor::new();

        let evidence = fake.execute_probe("nonexistent.probe").await;
        assert_eq!(evidence.status, EvidenceStatus::NotFound);
    }

    #[tokio::test]
    async fn test_fake_executor_call_counts() {
        let fake = FakeProbeExecutor::new();

        assert_eq!(fake.call_count("cpu.info"), 0);

        fake.execute_probe("cpu.info").await;
        assert_eq!(fake.call_count("cpu.info"), 1);

        fake.execute_probe("cpu.info").await;
        assert_eq!(fake.call_count("cpu.info"), 2);

        assert_eq!(fake.total_calls(), 2);
    }

    #[tokio::test]
    async fn test_fake_executor_multiple_probes() {
        let fake = FakeProbeExecutor::new();

        let probe_ids = vec!["cpu.info".to_string(), "mem.info".to_string()];
        let results = fake.execute_probes(&probe_ids).await;

        assert_eq!(results.len(), 2);
        assert_eq!(results[0].probe_id, "cpu.info");
        assert_eq!(results[1].probe_id, "mem.info");
    }

    #[tokio::test]
    async fn test_fake_executor_builder() {
        let fake = FakeProbeExecutorBuilder::new()
            .probe_response(FakeProbeResponse::ok("custom.probe", "custom output"))
            .build();

        assert!(fake.is_valid("custom.probe"));

        let evidence = fake.execute_probe("custom.probe").await;
        assert_eq!(evidence.status, EvidenceStatus::Ok);
        assert_eq!(evidence.raw.unwrap(), "custom output");
    }

    #[tokio::test]
    async fn test_fake_executor_for_cpu() {
        let fake = FakeProbeExecutor::for_cpu_question();

        let evidence = fake.execute_probe("cpu.info").await;
        assert_eq!(evidence.status, EvidenceStatus::Ok);
        assert!(evidence.raw.unwrap().contains("Ryzen"));
        assert!(evidence.parsed.is_some());
    }

    #[tokio::test]
    async fn test_fake_executor_all_failing() {
        let fake = FakeProbeExecutor::all_failing("Connection refused");

        // Use a known probe ID that should return Error (not NotFound)
        let evidence = fake.execute_probe("cpu.info").await;
        assert_eq!(evidence.status, EvidenceStatus::Error);
        assert!(evidence.raw.unwrap().contains("Connection refused"));
    }

    #[test]
    fn test_fake_probe_response_constructors() {
        let ok = FakeProbeResponse::ok("test", "output");
        assert_eq!(ok.status, EvidenceStatus::Ok);

        let err = FakeProbeResponse::error("test", "failed");
        assert_eq!(err.status, EvidenceStatus::Error);

        let not_found = FakeProbeResponse::not_found("test");
        assert_eq!(not_found.status, EvidenceStatus::NotFound);
    }
}
