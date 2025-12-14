// v0.0.630: Settings Connector (Phase 206)
// Connector for external settings providers

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::unified_settings::SettingsCategory;

/// Connector protocol
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub enum ConnectorProtocol {
    /// HTTP protocol
    #[default]
    Http,
    /// HTTPS protocol
    Https,
    /// Unix socket
    UnixSocket,
    /// TCP socket
    Tcp,
    /// Local file
    File,
}

impl std::fmt::Display for ConnectorProtocol {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Http => write!(f, "http"),
            Self::Https => write!(f, "https"),
            Self::UnixSocket => write!(f, "unix_socket"),
            Self::Tcp => write!(f, "tcp"),
            Self::File => write!(f, "file"),
        }
    }
}

/// Connection state
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum ConnectionState {
    /// Disconnected
    #[default]
    Disconnected,
    /// Connecting
    Connecting,
    /// Connected
    Connected,
    /// Reconnecting
    Reconnecting,
    /// Failed
    Failed,
}

impl std::fmt::Display for ConnectionState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Disconnected => write!(f, "disconnected"),
            Self::Connecting => write!(f, "connecting"),
            Self::Connected => write!(f, "connected"),
            Self::Reconnecting => write!(f, "reconnecting"),
            Self::Failed => write!(f, "failed"),
        }
    }
}

/// Connector config
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConnectorConfig {
    /// Name
    pub name: String,
    /// Protocol
    pub protocol: ConnectorProtocol,
    /// Endpoint
    pub endpoint: String,
    /// Timeout ms
    pub timeout_ms: u64,
    /// Retry count
    pub retry_count: u32,
}

impl ConnectorConfig {
    /// Create new config
    pub fn new(name: impl Into<String>, protocol: ConnectorProtocol, endpoint: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            protocol,
            endpoint: endpoint.into(),
            timeout_ms: 5000,
            retry_count: 3,
        }
    }

    /// Set timeout
    pub fn timeout(mut self, ms: u64) -> Self {
        self.timeout_ms = ms;
        self
    }

    /// Set retry count
    pub fn retries(mut self, count: u32) -> Self {
        self.retry_count = count;
        self
    }
}

/// Connection info
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConnectionInfo {
    /// State
    pub state: ConnectionState,
    /// Connected at
    pub connected_at: u64,
    /// Last activity
    pub last_activity: u64,
    /// Error message
    pub error: Option<String>,
}

impl ConnectionInfo {
    /// Create new info
    pub fn new() -> Self {
        Self {
            state: ConnectionState::Disconnected,
            connected_at: 0,
            last_activity: 0,
            error: None,
        }
    }

    /// Mark connected
    pub fn connect(&mut self, timestamp: u64) {
        self.state = ConnectionState::Connected;
        self.connected_at = timestamp;
        self.last_activity = timestamp;
        self.error = None;
    }

    /// Mark disconnected
    pub fn disconnect(&mut self) {
        self.state = ConnectionState::Disconnected;
    }

    /// Mark failed
    pub fn fail(&mut self, error: impl Into<String>) {
        self.state = ConnectionState::Failed;
        self.error = Some(error.into());
    }

    /// Update activity
    pub fn touch(&mut self, timestamp: u64) {
        self.last_activity = timestamp;
    }
}

impl Default for ConnectionInfo {
    fn default() -> Self {
        Self::new()
    }
}

/// Connector statistics
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ConnectorStats {
    /// Request count
    pub request_count: usize,
    /// Success count
    pub success_count: usize,
    /// Failure count
    pub failure_count: usize,
    /// Reconnect count
    pub reconnect_count: usize,
}

impl ConnectorStats {
    /// Record request
    pub fn record_request(&mut self, success: bool) {
        self.request_count += 1;
        if success {
            self.success_count += 1;
        } else {
            self.failure_count += 1;
        }
    }

    /// Record reconnect
    pub fn record_reconnect(&mut self) {
        self.reconnect_count += 1;
    }

    /// Success rate
    pub fn success_rate(&self) -> f64 {
        if self.request_count == 0 {
            1.0
        } else {
            self.success_count as f64 / self.request_count as f64
        }
    }
}

/// Settings connector
#[derive(Debug, Clone, Default)]
pub struct SettingsConnector {
    /// Connectors by name
    connectors: HashMap<String, ConnectorConfig>,
    /// Connection info by name
    connections: HashMap<String, ConnectionInfo>,
    /// Statistics by name
    stats: HashMap<String, ConnectorStats>,
}

impl SettingsConnector {
    /// Create new connector manager
    pub fn new() -> Self {
        Self::default()
    }

    /// Register connector
    pub fn register(&mut self, config: ConnectorConfig) {
        let name = config.name.clone();
        self.connectors.insert(name.clone(), config);
        self.connections.insert(name.clone(), ConnectionInfo::new());
        self.stats.insert(name, ConnectorStats::default());
    }

    /// Unregister connector
    pub fn unregister(&mut self, name: &str) -> bool {
        self.stats.remove(name);
        self.connections.remove(name);
        self.connectors.remove(name).is_some()
    }

    /// Connect
    pub fn connect(&mut self, name: &str, timestamp: u64) -> bool {
        if let Some(info) = self.connections.get_mut(name) {
            info.connect(timestamp);
            true
        } else {
            false
        }
    }

    /// Disconnect
    pub fn disconnect(&mut self, name: &str) -> bool {
        if let Some(info) = self.connections.get_mut(name) {
            info.disconnect();
            true
        } else {
            false
        }
    }

    /// Get connection info
    pub fn get_connection(&self, name: &str) -> Option<&ConnectionInfo> {
        self.connections.get(name)
    }

    /// Get stats
    pub fn get_stats(&self, name: &str) -> Option<&ConnectorStats> {
        self.stats.get(name)
    }

    /// Record request
    pub fn record_request(&mut self, name: &str, success: bool) {
        if let Some(stats) = self.stats.get_mut(name) {
            stats.record_request(success);
        }
    }

    /// Connector count
    pub fn count(&self) -> usize {
        self.connectors.len()
    }

    /// Connected count
    pub fn connected_count(&self) -> usize {
        self.connections.values().filter(|c| c.state == ConnectionState::Connected).count()
    }
}

/// Format connector
pub fn format_connector(connector: &SettingsConnector) -> String {
    let mut output = String::new();
    output.push_str("Settings Connector:\n");
    output.push_str(&format!("  Connectors: {}\n", connector.count()));
    output.push_str(&format!("  Connected: {}\n", connector.connected_count()));
    output
}

/// Check if query is about connector
pub fn is_connector_query(query: &str) -> bool {
    let lower = query.to_lowercase();
    lower.contains("connector")
        || lower.contains("settings connector")
        || lower.contains("external settings")
}

/// Fun fact about connector
pub fn connector_fun_fact() -> &'static str {
    "Anna's settings connectors enable integration with external settings providers!"
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_protocol_display() {
        assert_eq!(format!("{}", ConnectorProtocol::Http), "http");
        assert_eq!(format!("{}", ConnectorProtocol::Https), "https");
    }

    #[test]
    fn test_state_display() {
        assert_eq!(format!("{}", ConnectionState::Connected), "connected");
        assert_eq!(format!("{}", ConnectionState::Failed), "failed");
    }

    #[test]
    fn test_config_new() {
        let c = ConnectorConfig::new("c1", ConnectorProtocol::Http, "http://localhost");
        assert_eq!(c.timeout_ms, 5000);
    }

    #[test]
    fn test_config_builder() {
        let c = ConnectorConfig::new("c1", ConnectorProtocol::Http, "http://localhost")
            .timeout(10000)
            .retries(5);
        assert_eq!(c.retry_count, 5);
    }

    #[test]
    fn test_info_new() {
        let i = ConnectionInfo::new();
        assert_eq!(i.state, ConnectionState::Disconnected);
    }

    #[test]
    fn test_info_connect() {
        let mut i = ConnectionInfo::new();
        i.connect(100);
        assert_eq!(i.state, ConnectionState::Connected);
    }

    #[test]
    fn test_stats_record() {
        let mut s = ConnectorStats::default();
        s.record_request(true);
        s.record_request(false);
        assert_eq!(s.request_count, 2);
    }

    #[test]
    fn test_connector_new() {
        let c = SettingsConnector::new();
        assert_eq!(c.count(), 0);
    }

    #[test]
    fn test_connector_register() {
        let mut c = SettingsConnector::new();
        c.register(ConnectorConfig::new("c1", ConnectorProtocol::Http, "http://localhost"));
        assert_eq!(c.count(), 1);
    }

    #[test]
    fn test_connector_connect() {
        let mut c = SettingsConnector::new();
        c.register(ConnectorConfig::new("c1", ConnectorProtocol::Http, "http://localhost"));
        assert!(c.connect("c1", 100));
    }

    #[test]
    fn test_is_connector_query() {
        assert!(is_connector_query("settings connector"));
        assert!(!is_connector_query("hello world"));
    }

    #[test]
    fn test_fun_fact() {
        let fact = connector_fun_fact();
        assert!(fact.contains("connector"));
    }
}
