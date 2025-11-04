//! RPC Client - Unix socket client for communicating with daemon

use anna_common::ipc::{Method, Request, Response, ResponseData};
use anyhow::{Context, Result};
use std::sync::atomic::{AtomicU64, Ordering};
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::UnixStream;

const SOCKET_PATH: &str = "/run/anna/anna.sock";

static REQUEST_ID: AtomicU64 = AtomicU64::new(1);

/// RPC Client for communicating with the daemon
pub struct RpcClient {
    reader: BufReader<tokio::net::unix::OwnedReadHalf>,
    writer: tokio::net::unix::OwnedWriteHalf,
}

impl RpcClient {
    /// Connect to the daemon
    pub async fn connect() -> Result<Self> {
        let stream = UnixStream::connect(SOCKET_PATH)
            .await
            .context("Failed to connect to daemon. Is annad running?")?;

        let (reader, writer) = stream.into_split();
        let reader = BufReader::new(reader);

        Ok(Self { reader, writer })
    }

    /// Send a request and get a response
    pub async fn call(&mut self, method: Method) -> Result<ResponseData> {
        let id = REQUEST_ID.fetch_add(1, Ordering::SeqCst);

        let request = Request { id, method };

        // Send request
        let request_json = serde_json::to_string(&request)? + "\n";
        self.writer
            .write_all(request_json.as_bytes())
            .await
            .context("Failed to send request")?;

        // Read response
        let mut line = String::new();
        self.reader
            .read_line(&mut line)
            .await
            .context("Failed to read response")?;

        let response: Response =
            serde_json::from_str(&line).context("Failed to parse response")?;

        if response.id != id {
            anyhow::bail!("Response ID mismatch");
        }

        response
            .result
            .map_err(|e| anyhow::anyhow!("RPC error: {}", e))
    }

    /// Ping the daemon (health check)
    #[allow(dead_code)]
    pub async fn ping(&mut self) -> Result<()> {
        self.call(Method::Ping).await?;
        Ok(())
    }
}
