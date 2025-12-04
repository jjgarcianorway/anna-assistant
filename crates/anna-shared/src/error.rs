//! Error types for Anna.

use thiserror::Error;

#[derive(Error, Debug)]
pub enum AnnaError {
    #[error("Daemon not running. Re-run the installer to fix.")]
    DaemonNotRunning,

    #[error("Socket error: {0}")]
    Socket(String),

    #[error("RPC error: {0}")]
    Rpc(String),

    #[error("Ollama error: {0}")]
    Ollama(String),

    #[error("Model error: {0}")]
    Model(String),

    #[error("Ledger error: {0}")]
    Ledger(String),

    #[error("Hardware probe error: {0}")]
    HardwareProbe(String),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),

    #[error("Internal error: {0}")]
    Internal(String),
}

impl AnnaError {
    pub fn code(&self) -> i32 {
        match self {
            AnnaError::DaemonNotRunning => -32000,
            AnnaError::Socket(_) => -32001,
            AnnaError::Rpc(_) => -32600,
            AnnaError::Ollama(_) => -32002,
            AnnaError::Model(_) => -32003,
            AnnaError::Ledger(_) => -32004,
            AnnaError::HardwareProbe(_) => -32005,
            AnnaError::Io(_) => -32006,
            AnnaError::Json(_) => -32700,
            AnnaError::Internal(_) => -32603,
        }
    }
}
