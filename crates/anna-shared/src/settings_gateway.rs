// v0.0.625: Settings Gateway (Phase 201)
// Unified gateway for all settings access and modifications

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::unified_settings::SettingsCategory;

/// Gateway mode
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum GatewayMode {
    /// Open - all requests allowed
    #[default]
    Open,
    /// Restricted - only read requests
    Restricted,
    /// Locked - no requests allowed
    Locked,
    /// Maintenance - only admin requests
    Maintenance,
}

impl std::fmt::Display for GatewayMode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Open => write!(f, "open"),
            Self::Restricted => write!(f, "restricted"),
            Self::Locked => write!(f, "locked"),
            Self::Maintenance => write!(f, "maintenance"),
        }
    }
}

/// Request type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub enum RequestType {
    /// Read request
    #[default]
    Read,
    /// Write request
    Write,
    /// Delete request
    Delete,
    /// List request
    List,
    /// Admin request
    Admin,
}

impl std::fmt::Display for RequestType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Read => write!(f, "read"),
            Self::Write => write!(f, "write"),
            Self::Delete => write!(f, "delete"),
            Self::List => write!(f, "list"),
            Self::Admin => write!(f, "admin"),
        }
    }
}

/// Gateway request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GatewayRequest {
    /// Request ID
    pub id: String,
    /// Request type
    pub request_type: RequestType,
    /// Key
    pub key: Option<String>,
    /// Category filter
    pub category: Option<SettingsCategory>,
    /// Timestamp
    pub timestamp: u64,
}

impl GatewayRequest {
    /// Create new request
    pub fn new(id: impl Into<String>, request_type: RequestType) -> Self {
        Self {
            id: id.into(),
            request_type,
            key: None,
            category: None,
            timestamp: 0,
        }
    }

    /// Set key
    pub fn key(mut self, key: impl Into<String>) -> Self {
        self.key = Some(key.into());
        self
    }

    /// Set category
    pub fn category(mut self, category: SettingsCategory) -> Self {
        self.category = Some(category);
        self
    }

    /// Set timestamp
    pub fn timestamp(mut self, ts: u64) -> Self {
        self.timestamp = ts;
        self
    }

    /// Is read-only
    pub fn is_read_only(&self) -> bool {
        matches!(self.request_type, RequestType::Read | RequestType::List)
    }
}

/// Gateway response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GatewayResponse {
    /// Request ID
    pub request_id: String,
    /// Allowed
    pub allowed: bool,
    /// Reason
    pub reason: Option<String>,
    /// Processing time ms
    pub processing_time_ms: u64,
}

impl GatewayResponse {
    /// Create allowed response
    pub fn allowed(request_id: impl Into<String>) -> Self {
        Self {
            request_id: request_id.into(),
            allowed: true,
            reason: None,
            processing_time_ms: 0,
        }
    }

    /// Create denied response
    pub fn denied(request_id: impl Into<String>, reason: impl Into<String>) -> Self {
        Self {
            request_id: request_id.into(),
            allowed: false,
            reason: Some(reason.into()),
            processing_time_ms: 0,
        }
    }

    /// Set processing time
    pub fn with_time(mut self, ms: u64) -> Self {
        self.processing_time_ms = ms;
        self
    }
}

/// Gateway statistics
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct GatewayStats {
    /// Total requests
    pub total_requests: usize,
    /// Allowed requests
    pub allowed_requests: usize,
    /// Denied requests
    pub denied_requests: usize,
    /// Requests by type
    pub by_type: HashMap<RequestType, usize>,
}

impl GatewayStats {
    /// Record request
    pub fn record(&mut self, request_type: RequestType, allowed: bool) {
        self.total_requests += 1;
        if allowed {
            self.allowed_requests += 1;
        } else {
            self.denied_requests += 1;
        }
        *self.by_type.entry(request_type).or_default() += 1;
    }

    /// Allow rate
    pub fn allow_rate(&self) -> f64 {
        if self.total_requests == 0 {
            1.0
        } else {
            self.allowed_requests as f64 / self.total_requests as f64
        }
    }
}

/// Settings gateway
#[derive(Debug, Clone, Default)]
pub struct SettingsGateway {
    /// Mode
    mode: GatewayMode,
    /// Statistics
    stats: GatewayStats,
}

impl SettingsGateway {
    /// Create new gateway
    pub fn new() -> Self {
        Self::default()
    }

    /// Get mode
    pub fn mode(&self) -> GatewayMode {
        self.mode
    }

    /// Set mode
    pub fn set_mode(&mut self, mode: GatewayMode) {
        self.mode = mode;
    }

    /// Process request
    pub fn process(&mut self, request: &GatewayRequest) -> GatewayResponse {
        let allowed = match self.mode {
            GatewayMode::Open => true,
            GatewayMode::Restricted => request.is_read_only(),
            GatewayMode::Locked => false,
            GatewayMode::Maintenance => request.request_type == RequestType::Admin,
        };

        self.stats.record(request.request_type, allowed);

        if allowed {
            GatewayResponse::allowed(&request.id)
        } else {
            GatewayResponse::denied(&request.id, format!("Gateway is in {} mode", self.mode))
        }
    }

    /// Get stats
    pub fn stats(&self) -> &GatewayStats {
        &self.stats
    }

    /// Is open
    pub fn is_open(&self) -> bool {
        self.mode == GatewayMode::Open
    }

    /// Is locked
    pub fn is_locked(&self) -> bool {
        self.mode == GatewayMode::Locked
    }
}

/// Format gateway
pub fn format_gateway(gateway: &SettingsGateway) -> String {
    let mut output = String::new();
    output.push_str("Settings Gateway:\n");
    output.push_str(&format!("  Mode: {}\n", gateway.mode()));
    output.push_str(&format!("  Requests: {}\n", gateway.stats().total_requests));
    output.push_str(&format!("  Allow Rate: {:.1}%\n", gateway.stats().allow_rate() * 100.0));
    output
}

/// Check if query is about gateway
pub fn is_gateway_query(query: &str) -> bool {
    let lower = query.to_lowercase();
    lower.contains("gateway")
        || lower.contains("settings gateway")
        || lower.contains("access gateway")
}

/// Fun fact about gateway
pub fn gateway_fun_fact() -> &'static str {
    "Anna's settings gateway is the unified entry point for all settings operations!"
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_gateway_mode_display() {
        assert_eq!(format!("{}", GatewayMode::Open), "open");
        assert_eq!(format!("{}", GatewayMode::Locked), "locked");
    }

    #[test]
    fn test_request_type_display() {
        assert_eq!(format!("{}", RequestType::Read), "read");
        assert_eq!(format!("{}", RequestType::Write), "write");
    }

    #[test]
    fn test_request_new() {
        let r = GatewayRequest::new("r1", RequestType::Read);
        assert!(r.is_read_only());
    }

    #[test]
    fn test_request_write_not_readonly() {
        let r = GatewayRequest::new("r1", RequestType::Write);
        assert!(!r.is_read_only());
    }

    #[test]
    fn test_response_allowed() {
        let r = GatewayResponse::allowed("r1");
        assert!(r.allowed);
    }

    #[test]
    fn test_response_denied() {
        let r = GatewayResponse::denied("r1", "locked");
        assert!(!r.allowed);
    }

    #[test]
    fn test_stats_record() {
        let mut s = GatewayStats::default();
        s.record(RequestType::Read, true);
        assert_eq!(s.total_requests, 1);
    }

    #[test]
    fn test_gateway_new() {
        let g = SettingsGateway::new();
        assert!(g.is_open());
    }

    #[test]
    fn test_gateway_process_open() {
        let mut g = SettingsGateway::new();
        let r = GatewayRequest::new("r1", RequestType::Write);
        let resp = g.process(&r);
        assert!(resp.allowed);
    }

    #[test]
    fn test_gateway_process_locked() {
        let mut g = SettingsGateway::new();
        g.set_mode(GatewayMode::Locked);
        let r = GatewayRequest::new("r1", RequestType::Read);
        let resp = g.process(&r);
        assert!(!resp.allowed);
    }

    #[test]
    fn test_is_gateway_query() {
        assert!(is_gateway_query("settings gateway"));
        assert!(!is_gateway_query("hello world"));
    }

    #[test]
    fn test_fun_fact() {
        let fact = gateway_fun_fact();
        assert!(fact.contains("gateway"));
    }
}
