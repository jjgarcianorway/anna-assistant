//! Daemon client - communicates with annad
//!
//! v0.10.0: Added answer() method for evidence-based answers

use anna_common::{
    FinalAnswer, HealthResponse, ListProbesResponse, ProbeResult, RunProbeRequest,
    UpdateStateResponse,
};
use anyhow::{Context, Result};
use serde::Serialize;

const DAEMON_URL: &str = "http://127.0.0.1:7865";

/// Client for communicating with annad
pub struct DaemonClient {
    client: reqwest::Client,
    base_url: String,
}

impl DaemonClient {
    pub fn new() -> Self {
        Self {
            client: reqwest::Client::new(),
            base_url: DAEMON_URL.to_string(),
        }
    }

    /// Check if daemon is healthy
    pub async fn is_healthy(&self) -> bool {
        self.health().await.is_ok()
    }

    /// Get health status
    pub async fn health(&self) -> Result<HealthResponse> {
        let url = format!("{}/v1/health", self.base_url);
        let resp = self
            .client
            .get(&url)
            .send()
            .await
            .context("Failed to connect to daemon")?;

        resp.json().await.context("Failed to parse health response")
    }

    /// List available probes
    pub async fn list_probes(&self) -> Result<ListProbesResponse> {
        let url = format!("{}/v1/probe/list", self.base_url);
        let resp = self
            .client
            .get(&url)
            .send()
            .await
            .context("Failed to connect to daemon")?;

        resp.json().await.context("Failed to parse probes response")
    }

    /// Run a specific probe
    pub async fn run_probe(&self, probe_id: &str, force_refresh: bool) -> Result<ProbeResult> {
        let url = format!("{}/v1/probe/run", self.base_url);
        let req = RunProbeRequest {
            probe_id: probe_id.to_string(),
            force_refresh,
        };

        let resp = self
            .client
            .post(&url)
            .json(&req)
            .send()
            .await
            .context("Failed to connect to daemon")?;

        if !resp.status().is_success() {
            let status = resp.status();
            let text = resp.text().await.unwrap_or_default();
            anyhow::bail!("Probe failed ({}): {}", status, text);
        }

        resp.json().await.context("Failed to parse probe response")
    }

    /// Run multiple probes
    pub async fn run_probes(&self, probe_ids: &[&str]) -> Result<Vec<ProbeResult>> {
        let mut results = Vec::new();
        for id in probe_ids {
            match self.run_probe(id, false).await {
                Ok(result) => results.push(result),
                Err(e) => {
                    tracing::warn!("Probe {} failed: {}", id, e);
                }
            }
        }
        Ok(results)
    }

    /// v0.9.0: Get update state from daemon
    pub async fn update_state(&self) -> Result<UpdateStateResponse> {
        let url = format!("{}/v1/update/state", self.base_url);
        let resp = self
            .client
            .get(&url)
            .timeout(std::time::Duration::from_secs(5))
            .send()
            .await
            .context("Failed to connect to daemon")?;

        if resp.status().is_success() {
            resp.json()
                .await
                .context("Failed to parse update state response")
        } else {
            // Return default state if endpoint not available
            Ok(UpdateStateResponse::default())
        }
    }

    /// v0.10.0: Get evidence-based answer through LLM-A/LLM-B loop
    pub async fn answer(&self, question: &str) -> Result<FinalAnswer> {
        let url = format!("{}/v1/answer", self.base_url);

        #[derive(Serialize)]
        struct AnswerRequest {
            question: String,
        }

        let req = AnswerRequest {
            question: question.to_string(),
        };

        let resp = self
            .client
            .post(&url)
            .json(&req)
            .timeout(std::time::Duration::from_secs(120)) // LLM calls can take time
            .send()
            .await
            .context("Failed to connect to daemon")?;

        if !resp.status().is_success() {
            let status = resp.status();
            let text = resp.text().await.unwrap_or_default();
            anyhow::bail!("Answer request failed ({}): {}", status, text);
        }

        resp.json().await.context("Failed to parse answer response")
    }
}

impl Default for DaemonClient {
    fn default() -> Self {
        Self::new()
    }
}
