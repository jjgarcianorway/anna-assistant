//! RPC unit tests (v0.0.220).

#[cfg(test)]
mod tests {
    use crate::rpc::{RpcMethod, RpcRequest, RpcResponse};

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
