use anyhow::Result;
use serde::{Deserialize, Serialize};
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::{UnixListener, UnixStream};
use tracing::{info, error, debug};

use crate::config::{self, Config, Scope};
use crate::diagnostics;
use crate::telemetry;

#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum Request {
    Ping,
    Doctor,
    Status,
    GetConfig,
    ConfigGet {
        key: String,
    },
    ConfigSet {
        scope: Scope,
        key: String,
        value: String,
    },
    ConfigList,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "status", rename_all = "snake_case")]
pub enum Response {
    Success { data: serde_json::Value },
    Error { message: String },
}

pub async fn serve(listener: UnixListener, mut config: Config) -> Result<()> {
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

async fn handle_connection(stream: UnixStream, mut config: Config) -> Result<()> {
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

        // Log RPC call
        let rpc_name = format!("{:?}", request).split_whitespace().next().unwrap_or("unknown").to_string();

        let response = match handle_request(request, &mut config).await {
            Ok(resp) => {
                let _ = telemetry::log_event(telemetry::Event::RpcCall {
                    name: rpc_name,
                    status: "success".to_string(),
                });
                resp
            }
            Err(e) => {
                let _ = telemetry::log_event(telemetry::Event::RpcCall {
                    name: rpc_name,
                    status: "error".to_string(),
                });
                Response::Error {
                    message: e.to_string(),
                }
            }
        };

        let json = serde_json::to_string(&response)?;
        writer.write_all(json.as_bytes()).await?;
        writer.write_all(b"\n").await?;

        line.clear();
    }

    Ok(())
}

async fn handle_request(request: Request, config: &mut Config) -> Result<Response> {
    match request {
        Request::Ping => Ok(Response::Success {
            data: serde_json::json!({ "message": "pong" }),
        }),

        Request::Doctor => {
            let results = diagnostics::run_diagnostics().await;
            Ok(Response::Success {
                data: serde_json::to_value(results)?,
            })
        }

        Request::Status => {
            Ok(Response::Success {
                data: serde_json::json!({
                    "version": env!("CARGO_PKG_VERSION"),
                    "uptime": "running",
                    "autonomy_level": config.autonomy.level,
                }),
            })
        }

        Request::GetConfig => Ok(Response::Success {
            data: serde_json::to_value(&config)?,
        }),

        Request::ConfigGet { key } => {
            // Reload config to get latest values
            *config = config::load_config()?;

            if let Some(value) = config::get_value(config, &key) {
                Ok(Response::Success {
                    data: serde_json::json!({ "key": key, "value": value }),
                })
            } else {
                anyhow::bail!("Unknown configuration key: {}", key);
            }
        }

        Request::ConfigSet { scope, key, value } => {
            // Reload config first
            *config = config::load_config()?;

            // Set the value
            config::set_value(config, &key, &value)?;

            // Save to the appropriate scope
            config::save_config(config, scope)?;

            // Log the change
            telemetry::log_event(telemetry::Event::ConfigChanged {
                scope: format!("{:?}", scope).to_lowercase(),
                key: key.clone(),
            })?;

            Ok(Response::Success {
                data: serde_json::json!({
                    "key": key,
                    "value": value,
                    "scope": format!("{:?}", scope).to_lowercase(),
                }),
            })
        }

        Request::ConfigList => {
            // Reload config to get latest values
            *config = config::load_config()?;

            let values = config::list_values(config);
            Ok(Response::Success {
                data: serde_json::to_value(values)?,
            })
        }
    }
}
