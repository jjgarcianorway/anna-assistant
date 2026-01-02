//! Safe Change Engine handlers for planning, applying, and rolling back configuration changes.
//! v0.0.95: Introduced safe change engine

use super::types::*;
use anna_shared::change::{apply_change, plan_ensure_line, rollback};
use anna_shared::rpc::{ChangeParams, PlanChangeParams};
use std::path::Path;

/// Handle PlanChange request - creates a change plan for user confirmation
pub async fn handle_plan_change(id: String, params: Option<serde_json::Value>) -> RpcResponse {
    let params: PlanChangeParams = match params {
        Some(p) => match serde_json::from_value(p) {
            Ok(p) => p,
            Err(e) => {
                return RpcResponse::error(id, -32602, format!("Invalid params: {}", e));
            }
        },
        None => {
            return RpcResponse::error(id, -32602, "Missing params".to_string());
        }
    };

    let config_path = Path::new(&params.config_path);
    match plan_ensure_line(config_path, &params.line) {
        Ok(plan) => {
            info!("Change planned: {}", plan.summary());
            RpcResponse::success(id, serde_json::to_value(&plan).unwrap())
        }
        Err(e) => {
            error!("Failed to plan change: {}", e);
            RpcResponse::error(id, -32006, format!("Failed to plan change: {}", e))
        }
    }
}

/// Handle ApplyChange request - applies a confirmed change plan
pub async fn handle_apply_change(id: String, params: Option<serde_json::Value>) -> RpcResponse {
    let params: ChangeParams = match params {
        Some(p) => match serde_json::from_value(p) {
            Ok(p) => p,
            Err(e) => {
                return RpcResponse::error(id, -32602, format!("Invalid params: {}", e));
            }
        },
        None => {
            return RpcResponse::error(id, -32602, "Missing params".to_string());
        }
    };

    let result = apply_change(&params.plan);
    if result.applied {
        info!("Change applied successfully");
    } else if result.was_noop {
        info!("Change was no-op (already in desired state)");
    } else if let Some(ref err) = result.error {
        error!("Change failed: {}", err);
    }
    RpcResponse::success(id, serde_json::to_value(&result).unwrap())
}

/// Handle RollbackChange request - rolls back a change using backup
pub async fn handle_rollback_change(id: String, params: Option<serde_json::Value>) -> RpcResponse {
    let params: ChangeParams = match params {
        Some(p) => match serde_json::from_value(p) {
            Ok(p) => p,
            Err(e) => {
                return RpcResponse::error(id, -32602, format!("Invalid params: {}", e));
            }
        },
        None => {
            return RpcResponse::error(id, -32602, "Missing params".to_string());
        }
    };

    let result = rollback(&params.plan);
    if result.applied {
        info!("Change rolled back successfully");
    } else if let Some(ref err) = result.error {
        error!("Rollback failed: {}", err);
    }
    RpcResponse::success(id, serde_json::to_value(&result).unwrap())
}
