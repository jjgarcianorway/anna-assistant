use anyhow::Result;
use serde::{Deserialize, Serialize};
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::{UnixListener, UnixStream};
use tracing::{info, error, debug};

use crate::config::Config;
use crate::diagnostics;

#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum Request {
    Ping,
    Doctor,
    Status,
    GetConfig,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "status", rename_all = "snake_case")]
pub enum Response {
    Success { data: serde_json::Value },
    Error { message: String },
}

pub async fn serve(listener: UnixListener, config: Config) -> Result<()> {
    loop {
        match listener.accept().await {
            Ok((stream, _)) => {
                let config = config.clone();
                tokio::spawn(async move {
                    if let Err(e) = handle_connection(stream, config).await {
                        error!("Connection error: {}", e);
                    }
                });
            }
            Err(e) => error!("Accept error: {}", e),
        }
    }
}

async fn handle_connection(stream: UnixStream, config: Config) -> Result<()> {
    let (reader, mut writer) = stream.into_split();
    let mut reader = BufReader::new(reader);
    let mut line = String::new();

    while reader.read_line(&mut line).await? > 0 {
        let request: Request = match serde_json::from_str(&line) {
            Ok(req) => req,
            Err(e) => {
                let response = Response::Error {
                    message: format!("Invalid request: {}", e),
                };
                let json = serde_json::to_string(&response)?;
                writer.write_all(json.as_bytes()).await?;
                writer.write_all(b"\n").await?;
                line.clear();
                continue;
            }
        };

        debug!("Received request: {:?}", request);

        let response = match request {
            Request::Ping => Response::Success {
                data: serde_json::json!({ "message": "pong" }),
            },
            Request::Doctor => {
                let results = diagnostics::run_diagnostics().await;
                Response::Success {
                    data: serde_json::to_value(results)?,
                }
            }
            Request::Status => {
                let status = serde_json::json!({
                    "version": env!("CARGO_PKG_VERSION"),
                    "uptime": "running",
                    "autonomy_tier": config.autonomy.tier,
                });
                Response::Success { data: status }
            }
            Request::GetConfig => Response::Success {
                data: serde_json::to_value(&config)?,
            },
        };

        let json = serde_json::to_string(&response)?;
        writer.write_all(json.as_bytes()).await?;
        writer.write_all(b"\n").await?;

        line.clear();
    }

    Ok(())
}
