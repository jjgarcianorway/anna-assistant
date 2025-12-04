//! JSON-RPC 2.0 types for annad communication.

use serde::{Deserialize, Serialize};

/// RPC methods supported by annad
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum RpcMethod {
    Status,
    Request,
    Reset,
    Uninstall,
    Autofix,
    Probe,
}

/// JSON-RPC 2.0 request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RpcRequest {
    pub jsonrpc: String,
    pub method: RpcMethod,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub params: Option<serde_json::Value>,
    pub id: String,
}

impl RpcRequest {
    pub fn new(method: RpcMethod, params: Option<serde_json::Value>) -> Self {
        Self {
            jsonrpc: "2.0".to_string(),
            method,
            params,
            id: uuid::Uuid::new_v4().to_string(),
        }
    }
}

/// JSON-RPC 2.0 response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RpcResponse {
    pub jsonrpc: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub result: Option<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<RpcError>,
    pub id: String,
}

impl RpcResponse {
    pub fn success(id: String, result: serde_json::Value) -> Self {
        Self {
            jsonrpc: "2.0".to_string(),
            result: Some(result),
            error: None,
            id,
        }
    }

    pub fn error(id: String, code: i32, message: String) -> Self {
        Self {
            jsonrpc: "2.0".to_string(),
            result: None,
            error: Some(RpcError {
                code,
                message,
                data: None,
            }),
            id,
        }
    }
}

/// JSON-RPC 2.0 error object
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RpcError {
    pub code: i32,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<serde_json::Value>,
}

/// Parameters for the request method
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RequestParams {
    pub prompt: String,
}

/// Parameters for the probe method
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProbeParams {
    pub probe_type: ProbeType,
}

/// Types of probes that can be run
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum ProbeType {
    TopMemory,
    TopCpu,
    DiskUsage,
    NetworkInterfaces,
}

/// Runtime context injected into every LLM request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RuntimeContext {
    pub version: String,
    pub daemon_running: bool,
    pub capabilities: Capabilities,
    pub hardware: HardwareSummary,
    pub probes: std::collections::HashMap<String, String>,
}

/// Capability flags for the assistant
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Capabilities {
    pub can_read_system_info: bool,
    pub can_run_probes: bool,
    pub can_modify_files: bool,
    pub can_install_packages: bool,
}

impl Default for Capabilities {
    fn default() -> Self {
        Self {
            can_read_system_info: true,
            can_run_probes: true,
            can_modify_files: false,
            can_install_packages: false,
        }
    }
}

/// Hardware summary for context
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HardwareSummary {
    pub cpu_model: String,
    pub cpu_cores: u32,
    pub ram_gb: f64,
    pub gpu: Option<String>,
    pub gpu_vram_gb: Option<f64>,
}

/// Specialist domain for service desk routing
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SpecialistDomain {
    System,
    Network,
    Storage,
    Security,
    Packages,
}

impl std::fmt::Display for SpecialistDomain {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::System => write!(f, "system"),
            Self::Network => write!(f, "network"),
            Self::Storage => write!(f, "storage"),
            Self::Security => write!(f, "security"),
            Self::Packages => write!(f, "packages"),
        }
    }
}

/// Unified response from service desk pipeline
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceDeskResult {
    /// The LLM's answer text
    pub answer: String,
    /// Reliability score 0-100
    pub reliability_score: u8,
    /// Which specialist handled this
    pub domain: SpecialistDomain,
    /// Probes that were run
    pub probes_used: Vec<String>,
    /// Whether clarification is needed
    pub needs_clarification: bool,
    /// Question to ask if clarification needed
    pub clarification_question: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rpc_request_serialization() {
        let req = RpcRequest::new(RpcMethod::Status, None);
        let json = serde_json::to_string(&req).unwrap();
        assert!(json.contains("\"method\":\"status\""));
    }

    #[test]
    fn test_rpc_response_success() {
        let resp = RpcResponse::success("test-id".to_string(), serde_json::json!({"status": "ok"}));
        assert!(resp.result.is_some());
        assert!(resp.error.is_none());
    }

    #[test]
    fn test_rpc_response_error() {
        let resp = RpcResponse::error("test-id".to_string(), -32600, "Invalid request".to_string());
        assert!(resp.result.is_none());
        assert!(resp.error.is_some());
    }
}
