//! Prometheus metrics for consensus (Phase 1.9 + 1.10 + 1.11)

use prometheus::{
    register_counter_vec_with_registry, register_gauge_with_registry,
    register_histogram_vec_with_registry, register_int_counter_with_registry,
    register_int_gauge_with_registry, CounterVec, Gauge, HistogramVec,
    IntCounter, IntGauge, Registry, TextEncoder, Encoder,
};
use std::sync::Arc;

/// Consensus metrics for Prometheus
#[derive(Clone)]
pub struct ConsensusMetrics {
    // Phase 1.9 metrics
    pub rounds_total: IntCounter,
    pub byzantine_nodes_total: IntGauge,
    pub quorum_size: IntGauge,

    // Phase 1.10 metrics
    pub average_tis: Gauge,
    pub peer_request_total: CounterVec,
    pub peer_reload_total: CounterVec,
    pub migration_events_total: CounterVec,

    // Phase 1.11 metrics
    pub peer_backoff_seconds: HistogramVec,

    // Phase 1.13 metrics
    pub tls_handshakes_total: CounterVec,

    // Phase 1.15 metrics
    pub rate_limit_violations_total: CounterVec,

    // Phase 2 metrics
    pub pinning_violations_total: CounterVec,

    registry: Arc<Registry>,
}

impl ConsensusMetrics {
    pub fn new() -> Self {
        let registry = Registry::new();

        // Phase 1.9 metrics
        let rounds_total = register_int_counter_with_registry!(
            "anna_consensus_rounds_total",
            "Total number of consensus rounds completed",
            registry
        ).unwrap();

        let byzantine_nodes_total = register_int_gauge_with_registry!(
            "anna_byzantine_nodes_total",
            "Current number of detected Byzantine nodes",
            registry
        ).unwrap();

        let quorum_size = register_int_gauge_with_registry!(
            "anna_quorum_size",
            "Required quorum size for consensus",
            registry
        ).unwrap();

        // Phase 1.10 metrics
        let average_tis = register_gauge_with_registry!(
            "anna_average_tis",
            "Average temporal integrity score across recent rounds",
            registry
        ).unwrap();

        let peer_request_total = register_counter_vec_with_registry!(
            "anna_peer_request_total",
            "Total number of peer requests by peer and status",
            &["peer", "status"],
            registry
        ).unwrap();

        let peer_reload_total = register_counter_vec_with_registry!(
            "anna_peer_reload_total",
            "Total number of peer configuration reloads by result",
            &["result"],
            registry
        ).unwrap();

        let migration_events_total = register_counter_vec_with_registry!(
            "anna_migration_events_total",
            "Total number of state migration events by result",
            &["result"],
            registry
        ).unwrap();

        // Phase 1.11 metrics
        let peer_backoff_seconds = register_histogram_vec_with_registry!(
            "anna_peer_backoff_seconds",
            "Peer request backoff duration in seconds",
            &["peer"],
            vec![0.1, 0.2, 0.5, 1.0, 2.0, 5.0],
            registry
        ).unwrap();

        // Phase 1.13 metrics
        let tls_handshakes_total = register_counter_vec_with_registry!(
            "anna_tls_handshakes_total",
            "Total number of TLS handshakes by status",
            &["status"],
            registry
        ).unwrap();

        // Phase 1.15 metrics
        let rate_limit_violations_total = register_counter_vec_with_registry!(
            "anna_rate_limit_violations_total",
            "Total number of rate limit violations by scope (peer or token)",
            &["scope"],
            registry
        ).unwrap();

        // Phase 2 metrics
        let pinning_violations_total = register_counter_vec_with_registry!(
            "anna_pinning_violations_total",
            "Total number of certificate pinning violations by peer",
            &["peer"],
            registry
        ).unwrap();

        Self {
            rounds_total,
            byzantine_nodes_total,
            quorum_size,
            average_tis,
            peer_request_total,
            peer_reload_total,
            migration_events_total,
            peer_backoff_seconds,
            tls_handshakes_total,
            rate_limit_violations_total,
            pinning_violations_total,
            registry: Arc::new(registry),
        }
    }

    /// Record peer request
    pub fn record_peer_request(&self, peer: &str, status: &str) {
        self.peer_request_total
            .with_label_values(&[peer, status])
            .inc();
    }

    /// Record peer reload
    pub fn record_peer_reload(&self, result: &str) {
        self.peer_reload_total
            .with_label_values(&[result])
            .inc();
    }

    /// Record migration event
    pub fn record_migration(&self, result: &str) {
        self.migration_events_total
            .with_label_values(&[result])
            .inc();
    }

    /// Update average TIS
    pub fn update_average_tis(&self, tis: f64) {
        self.average_tis.set(tis);
    }

    /// Record backoff duration
    pub fn record_backoff_duration(&self, duration_secs: f64) {
        self.peer_backoff_seconds
            .with_label_values(&["all"])
            .observe(duration_secs);
    }

    /// Record TLS handshake (Phase 1.13)
    pub fn record_tls_handshake(&self, status: &str) {
        self.tls_handshakes_total
            .with_label_values(&[status])
            .inc();
    }

    /// Record rate limit violation (Phase 1.15)
    pub fn record_rate_limit_violation(&self, scope: &str) {
        self.rate_limit_violations_total
            .with_label_values(&[scope])
            .inc();
    }

    /// Record certificate pinning violation (Phase 2)
    pub fn record_pinning_violation(&self, peer: &str) {
        self.pinning_violations_total
            .with_label_values(&[peer])
            .inc();
    }

    /// Export metrics in Prometheus text format
    pub fn export(&self) -> String {
        let encoder = TextEncoder::new();
        let metric_families = self.registry.gather();
        let mut buffer = Vec::new();
        encoder.encode(&metric_families, &mut buffer).unwrap();
        String::from_utf8(buffer).unwrap()
    }
}

impl Default for ConsensusMetrics {
    fn default() -> Self {
        Self::new()
    }
}
