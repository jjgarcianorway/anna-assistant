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

    /// Call a method with streaming response support
    /// Creates a dedicated connection for streaming to avoid blocking the main client
    /// Returns a receiver that yields ResponseData chunks until StreamEnd
    pub async fn call_streaming(
        &mut self,
        method: Method,
    ) -> Result<tokio::sync::mpsc::Receiver<ResponseData>> {
        // Create a dedicated connection for this streaming call
        let stream = UnixStream::connect(SOCKET_PATH)
            .await
            .context("Failed to connect for streaming")?;

        let (mut reader, mut writer) = stream.into_split();
        let mut reader = BufReader::new(reader);

        let id = REQUEST_ID.fetch_add(1, Ordering::SeqCst);
        let request = Request { id, method };

        // Send request
        let request_json = serde_json::to_string(&request)? + "\n";
        writer
            .write_all(request_json.as_bytes())
            .await
            .context("Failed to send streaming request")?;

        // Create channel for responses
        let (tx, rx) = tokio::sync::mpsc::channel(100);

        // Spawn task to read responses
        tokio::spawn(async move {
            loop {
                let mut line = String::new();
                match reader.read_line(&mut line).await {
                    Ok(0) | Ok(_) if line.is_empty() => break, // Connection closed
                    Ok(_) => {
                        let response: Response = match serde_json::from_str(&line) {
                            Ok(r) => r,
                            Err(e) => {
                                eprintln!("Failed to parse streaming response: {}", e);
                                break;
                            }
                        };

                        if response.id != id {
                            eprintln!("Response ID mismatch in streaming");
                            break;
                        }

                        match response.result {
                            Ok(data) => {
                                let is_end = matches!(data, ResponseData::StreamEnd { .. });

                                if tx.send(data).await.is_err() {
                                    break; // Receiver dropped
                                }

                                if is_end {
                                    break; // Streaming complete
                                }
                            }
                            Err(e) => {
                                eprintln!("RPC error in streaming: {}", e);
                                break;
                            }
                        }
                    }
                    Err(e) => {
                        eprintln!("Failed to read streaming response: {}", e);
                        break;
                    }
                }
            }
        });

        Ok(rx)
    }
}
