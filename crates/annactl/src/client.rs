//! Unix socket client for communicating with annad.
//! v0.0.73: Added get_daemon_info for version truth.
//! v0.0.312: Added execute_command for user-approved commands.

use anna_shared::progress::ProgressEvent;
use anna_shared::rpc::{
    CommandExecutionResult, DaemonInfo, RpcMethod, RpcRequest, RpcResponse, ServiceDeskResult,
};
use anna_shared::stats::GlobalStats;
use anna_shared::status::DaemonStatus;
use anna_shared::status_snapshot::StatusSnapshot;
use anna_shared::socket_path;
use anyhow::{anyhow, Result};
use std::path::Path;
use std::time::Duration;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::UnixStream;
use tokio::time::timeout;

/// Default RPC call timeout (covers translator + probes + specialist + supervisor)
const RPC_TIMEOUT_SECS: u64 = 45;

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
        let socket_file = socket_path();
        let socket_path_obj = Path::new(&socket_file);

        if !socket_path_obj.exists() {
            return Err(anyhow!(
                "Anna daemon not running.\n\
                 The socket at {} does not exist.\n\n\
                 To fix this, re-run the installer:\n\
                 curl -sSL https://raw.githubusercontent.com/jjgarcianorway/anna-assistant/main/scripts/install.sh | bash",
                socket_file
            ));
        }

        let stream = UnixStream::connect(socket_path_obj).await.map_err(|e| {
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

    /// Send an RPC request and get the response with timeout
    pub async fn call(
        &mut self,
        method: RpcMethod,
        params: Option<serde_json::Value>,
    ) -> Result<RpcResponse> {
        self.call_with_timeout(method, params, RPC_TIMEOUT_SECS)
            .await
    }

    /// Send an RPC request with custom timeout
    pub async fn call_with_timeout(
        &mut self,
        method: RpcMethod,
        params: Option<serde_json::Value>,
        timeout_secs: u64,
    ) -> Result<RpcResponse> {
        let request = RpcRequest::new(method, params);
        let request_json = serde_json::to_string(&request)?;

        // Send request with timeout
        timeout(Duration::from_secs(5), async {
            self.stream
                .write_all(format!("{}\n", request_json).as_bytes())
                .await
        })
        .await
        .map_err(|_| anyhow!("Timeout writing to daemon"))?
        .map_err(|e| anyhow!("Failed to write to daemon: {}", e))?;

        // Read response with timeout
        let (reader, _) = self.stream.split();
        let mut reader = BufReader::new(reader);
        let mut line = String::new();

        timeout(
            Duration::from_secs(timeout_secs),
            reader.read_line(&mut line),
        )
        .await
        .map_err(|_| anyhow!("Request timed out after {}s", timeout_secs))?
        .map_err(|e| anyhow!("Failed to read from daemon: {}", e))?;

        let response: RpcResponse = serde_json::from_str(&line)
            .map_err(|e| anyhow!("Invalid response from daemon: {}", e))?;
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

    /// Get comprehensive status snapshot (versions, helpers, config)
    pub async fn status_snapshot(&mut self) -> Result<StatusSnapshot> {
        let response = self.call(RpcMethod::StatusSnapshot, None).await?;

        if let Some(error) = response.error {
            return Err(anyhow!("StatusSnapshot error: {}", error.message));
        }

        let result = response
            .result
            .ok_or_else(|| anyhow!("No result in response"))?;
        let snapshot: StatusSnapshot = serde_json::from_value(result)?;
        Ok(snapshot)
    }

    /// Send a natural language request - returns full service desk result
    pub async fn request(&mut self, prompt: &str) -> Result<ServiceDeskResult> {
        let params = serde_json::json!({ "prompt": prompt });
        let response = self.call(RpcMethod::Request, Some(params)).await?;

        if let Some(error) = response.error {
            return Err(anyhow!("{}", error.message));
        }

        let result = response
            .result
            .ok_or_else(|| anyhow!("No result in response"))?;

        let service_result: ServiceDeskResult = serde_json::from_value(result)?;
        Ok(service_result)
    }

    /// Reset learned data (v0.0.144: kept for future natural language "reset anna")
    #[allow(dead_code)]
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

    /// Get progress events for current/last request (v0.0.148: used by live_request)
    pub async fn progress(&mut self) -> Result<Vec<ProgressEvent>> {
        let response = self.call(RpcMethod::Progress, None).await?;

        if let Some(error) = response.error {
            return Err(anyhow!("Progress error: {}", error.message));
        }

        let result = response
            .result
            .ok_or_else(|| anyhow!("No result in response"))?;
        let events: Vec<ProgressEvent> = serde_json::from_value(result)?;
        Ok(events)
    }

    /// Get per-team statistics (v0.0.27)
    pub async fn stats(&mut self) -> Result<GlobalStats> {
        let response = self.call(RpcMethod::Stats, None).await?;

        if let Some(error) = response.error {
            return Err(anyhow!("Stats error: {}", error.message));
        }

        let result = response
            .result
            .ok_or_else(|| anyhow!("No result in response"))?;
        let stats: GlobalStats = serde_json::from_value(result)?;
        Ok(stats)
    }

    /// v0.0.73: Get daemon version info
    pub async fn get_daemon_info(&mut self) -> Result<DaemonInfo> {
        let response = self.call(RpcMethod::GetDaemonInfo, None).await?;

        if let Some(error) = response.error {
            return Err(anyhow!("GetDaemonInfo error: {}", error.message));
        }

        let result = response
            .result
            .ok_or_else(|| anyhow!("No result in response"))?;
        let info: DaemonInfo = serde_json::from_value(result)?;
        Ok(info)
    }

    /// v0.0.275: Generate personalized greeting via LLM
    #[allow(dead_code)]
    pub async fn generate_greeting(
        &mut self,
        ctx: &anna_shared::greeting_context::GreetingContext,
    ) -> Result<anna_shared::greeting_context::GreetingResponse> {
        let params = serde_json::to_value(ctx)?;
        let response = self.call(RpcMethod::GenerateGreeting, Some(params)).await?;

        if let Some(error) = response.error {
            return Err(anyhow!("GenerateGreeting error: {}", error.message));
        }

        let result = response
            .result
            .ok_or_else(|| anyhow!("No result in response"))?;
        let greeting: anna_shared::greeting_context::GreetingResponse =
            serde_json::from_value(result)?;
        Ok(greeting)
    }

    /// v0.0.312: Execute a user-approved command via daemon (with elevated privileges)
    pub async fn execute_command(
        &mut self,
        command: &str,
        request_id: &str,
    ) -> Result<CommandExecutionResult> {
        let params = serde_json::json!({
            "command": command,
            "request_id": request_id
        });
        // Use longer timeout for commands (5 minutes)
        let response = self
            .call_with_timeout(RpcMethod::ExecuteCommand, Some(params), 300)
            .await?;

        if let Some(error) = response.error {
            return Err(anyhow!("ExecuteCommand error: {}", error.message));
        }

        let result = response
            .result
            .ok_or_else(|| anyhow!("No result in response"))?;
        let exec_result: CommandExecutionResult = serde_json::from_value(result)?;
        Ok(exec_result)
    }
}

/// Client for streaming requests with progress polling (v0.0.144: kept for future streaming UI)
#[allow(dead_code)]
pub struct StreamingClient;

#[allow(dead_code)]
impl StreamingClient {
    /// Send request with progress polling - returns (result, events)
    pub async fn request_with_progress(
        prompt: &str,
        mut on_progress: impl FnMut(&ProgressEvent),
    ) -> Result<ServiceDeskResult> {
        // Start request in background
        let prompt_owned = prompt.to_string();
        let request_handle = tokio::spawn(async move {
            let mut client = AnnadClient::connect().await?;
            client.request(&prompt_owned).await
        });

        // Poll for progress every 250ms
        let poll_interval = std::time::Duration::from_millis(250);
        let mut last_event_count = 0;

        loop {
            tokio::time::sleep(poll_interval).await;

            // Check if request completed
            if request_handle.is_finished() {
                break;
            }

            // Poll for new progress events
            if let Ok(mut poll_client) = AnnadClient::connect().await {
                if let Ok(events) = poll_client.progress().await {
                    // Report only new events
                    for event in events.iter().skip(last_event_count) {
                        on_progress(event);
                    }
                    last_event_count = events.len();
                }
            }
        }

        // Get final result
        request_handle.await?
    }
}
