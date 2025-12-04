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
        let resp = RpcResponse::success(
            "test-id".to_string(),
            serde_json::json!({"status": "ok"}),
        );
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
