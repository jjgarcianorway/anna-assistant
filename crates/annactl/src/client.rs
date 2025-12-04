//! Unix socket client for communicating with annad.

use anna_shared::rpc::{RpcMethod, RpcRequest, RpcResponse};
use anna_shared::status::DaemonStatus;
use anna_shared::SOCKET_PATH;
use anyhow::{anyhow, Result};
use std::path::Path;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::UnixStream;

/// Uninstall information returned by daemon
pub struct UninstallInfo {
    pub commands: Vec<String>,
    pub ollama_installed: bool,
    pub models: Vec<String>,
}

/// Client for communicating with annad
pub struct AnnadClient {
    stream: UnixStream,
}

impl AnnadClient {
    /// Connect to annad
    pub async fn connect() -> Result<Self> {
        let socket_path = Path::new(SOCKET_PATH);

        if !socket_path.exists() {
            return Err(anyhow!(
                "Anna daemon not running.\n\
                 The socket at {} does not exist.\n\n\
                 To fix this, re-run the installer:\n\
                 curl -sSL https://raw.githubusercontent.com/jjgarcianorway/anna-assistant/main/scripts/install.sh | bash",
                SOCKET_PATH
            ));
        }

        let stream = UnixStream::connect(socket_path).await.map_err(|e| {
            anyhow!(
                "Cannot connect to Anna daemon: {}\n\n\
                 The daemon may have crashed. To fix this:\n\
                 sudo systemctl restart annad\n\n\
                 If that doesn't work, re-run the installer.",
                e
            )
        })?;

        Ok(Self { stream })
    }

    /// Send an RPC request and get the response
    pub async fn call(
        &mut self,
        method: RpcMethod,
        params: Option<serde_json::Value>,
    ) -> Result<RpcResponse> {
        let request = RpcRequest::new(method, params);
        let request_json = serde_json::to_string(&request)?;

        // Send request
        self.stream
            .write_all(format!("{}\n", request_json).as_bytes())
            .await?;

        // Read response
        let (reader, _) = self.stream.split();
        let mut reader = BufReader::new(reader);
        let mut line = String::new();
        reader.read_line(&mut line).await?;

        let response: RpcResponse = serde_json::from_str(&line)?;
        Ok(response)
    }

    /// Get daemon status
    pub async fn status(&mut self) -> Result<DaemonStatus> {
        let response = self.call(RpcMethod::Status, None).await?;

        if let Some(error) = response.error {
            return Err(anyhow!("Status error: {}", error.message));
        }

        let result = response
            .result
            .ok_or_else(|| anyhow!("No result in response"))?;
        let status: DaemonStatus = serde_json::from_value(result)?;
        Ok(status)
    }

    /// Send a natural language request
    pub async fn request(&mut self, prompt: &str) -> Result<String> {
        let params = serde_json::json!({ "prompt": prompt });
        let response = self.call(RpcMethod::Request, Some(params)).await?;

        if let Some(error) = response.error {
            return Err(anyhow!("{}", error.message));
        }

        let result = response
            .result
            .ok_or_else(|| anyhow!("No result in response"))?;
        let response_text = result
            .get("response")
            .and_then(|r| r.as_str())
            .unwrap_or("")
            .to_string();

        Ok(response_text)
    }

    /// Reset learned data
    pub async fn reset(&mut self) -> Result<()> {
        let response = self.call(RpcMethod::Reset, None).await?;

        if let Some(error) = response.error {
            return Err(anyhow!("Reset error: {}", error.message));
        }

        Ok(())
    }

    /// Get uninstall information
    pub async fn uninstall_info(&mut self) -> Result<UninstallInfo> {
        let response = self.call(RpcMethod::Uninstall, None).await?;

        if let Some(error) = response.error {
            return Err(anyhow!("Uninstall error: {}", error.message));
        }

        let result = response.result.ok_or_else(|| anyhow!("No result"))?;

        let commands: Vec<String> = result
            .get("commands")
            .and_then(|c| c.as_array())
            .map(|arr| {
                arr.iter()
                    .filter_map(|v| v.as_str().map(String::from))
                    .collect()
            })
            .unwrap_or_default();

        let helpers = result.get("helpers");
        let ollama_installed = helpers
            .and_then(|h| h.get("ollama"))
            .and_then(|o| o.as_bool())
            .unwrap_or(false);

        let models: Vec<String> = helpers
            .and_then(|h| h.get("models"))
            .and_then(|m| m.as_array())
            .map(|arr| {
                arr.iter()
                    .filter_map(|v| v.as_str().map(String::from))
                    .collect()
            })
            .unwrap_or_default();

        Ok(UninstallInfo {
            commands,
            ollama_installed,
            models,
        })
    }

    /// Trigger autofix
    #[allow(dead_code)]
    pub async fn autofix(&mut self) -> Result<Vec<String>> {
        let response = self.call(RpcMethod::Autofix, None).await?;

        if let Some(error) = response.error {
            return Err(anyhow!("Autofix error: {}", error.message));
        }

        let result = response.result.ok_or_else(|| anyhow!("No result"))?;
        let fixes: Vec<String> = result
            .get("fixes_applied")
            .and_then(|f| f.as_array())
            .map(|arr| {
                arr.iter()
                    .filter_map(|v| v.as_str().map(String::from))
                    .collect()
            })
            .unwrap_or_default();

        Ok(fixes)
    }
}
