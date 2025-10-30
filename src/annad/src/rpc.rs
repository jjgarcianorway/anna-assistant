use anyhow::Result;
use serde::{Deserialize, Serialize};
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::{UnixListener, UnixStream};
use tracing::{info, error, debug};
use std::sync::Arc;

use crate::config::{self, Config, Scope};
use crate::diagnostics;
use crate::telemetry;
use crate::autonomy;
use crate::persistence;
use crate::policy;
use crate::events;
use crate::learning;
use crate::state::DaemonState;

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
    // Sprint 2: Autonomy
    AutonomyStatus,
    AutonomyRun {
        task: String,
    },
    // Sprint 2: Persistence
    StateSave {
        component: String,
        data: serde_json::Value,
    },
    StateLoad {
        component: String,
    },
    StateList,
    // Sprint 2: Auto-fix
    DoctorAutoFix,
    // Sprint 3: Policy Engine
    PolicyEvaluate {
        context: serde_json::Value,
    },
    PolicyReload,
    PolicyList,
    // Sprint 3: Events
    EventsList {
        filter: Option<String>,
        limit: Option<usize>,
    },
    EventsShow {
        event_type: Option<String>,
        severity: Option<String>,
    },
    EventsClear,
    // Sprint 3: Learning
    LearningStats {
        action: Option<String>,
    },
    LearningRecommendations,
    LearningReset,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "status", rename_all = "snake_case")]
pub enum Response {
    Success { data: serde_json::Value },
    Error { message: String },
}

pub async fn serve(listener: UnixListener, config: Config, state: Arc<DaemonState>) -> Result<()> {
    loop {
        match listener.accept().await {
            Ok((stream, _)) => {
                let config = config.clone();
                let state = state.clone();
                tokio::spawn(async move {
                    if let Err(e) = handle_connection(stream, config, state).await {
                        error!("Connection error: {}", e);
                    }
                });
            }
            Err(e) => error!("Accept error: {}", e),
        }
    }
}

async fn handle_connection(stream: UnixStream, mut config: Config, state: Arc<DaemonState>) -> Result<()> {
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

        let response = match handle_request(request, &mut config, &state).await {
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

async fn handle_request(request: Request, config: &mut Config, state: &Arc<DaemonState>) -> Result<Response> {
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

        Request::AutonomyStatus => {
            let status = autonomy::get_status(config);
            Ok(Response::Success {
                data: serde_json::to_value(status)?,
            })
        }

        Request::AutonomyRun { task } => {
            // Parse task name
            let task_obj = match task.as_str() {
                "doctor" => autonomy::Task::Doctor,
                "telemetry_cleanup" => autonomy::Task::TelemetryCleanup,
                "config_sync" => autonomy::Task::ConfigSync,
                _ => anyhow::bail!("Unknown task: {}", task),
            };

            let result = autonomy::run_task(task_obj, config).await?;
            Ok(Response::Success {
                data: serde_json::to_value(result)?,
            })
        }

        Request::StateSave { component, data } => {
            persistence::save_state(&component, data)?;
            Ok(Response::Success {
                data: serde_json::json!({
                    "component": component,
                    "saved": true,
                }),
            })
        }

        Request::StateLoad { component } => {
            let state = persistence::load_state(&component)?;
            if let Some(state) = state {
                Ok(Response::Success {
                    data: serde_json::to_value(state)?,
                })
            } else {
                Ok(Response::Success {
                    data: serde_json::json!({
                        "component": component,
                        "found": false,
                    }),
                })
            }
        }

        Request::StateList => {
            let components = persistence::list_states()?;
            Ok(Response::Success {
                data: serde_json::to_value(components)?,
            })
        }

        Request::DoctorAutoFix => {
            let results = diagnostics::run_autofix().await;
            Ok(Response::Success {
                data: serde_json::to_value(results)?,
            })
        }

        // Sprint 3: Policy Engine handlers
        Request::PolicyEvaluate { context } => {
            // Build policy context from provided JSON
            let mut policy_context = policy::PolicyContext::new();

            if let Some(obj) = context.as_object() {
                for (key, value) in obj {
                    match value {
                        serde_json::Value::Number(n) => {
                            if let Some(f) = n.as_f64() {
                                policy_context.set_metric(key, f);
                            }
                        }
                        serde_json::Value::Bool(b) => {
                            policy_context.set_flag(key, *b);
                        }
                        serde_json::Value::String(s) => {
                            policy_context.set_string(key, s.clone());
                        }
                        _ => {}
                    }
                }
            }

            let result = state.policy_engine.evaluate(&policy_context)?;

            Ok(Response::Success {
                data: serde_json::json!({
                    "matched": result.matched,
                    "actions": result.actions,
                    "rule_count": state.policy_engine.rule_count(),
                }),
            })
        }

        Request::PolicyReload => {
            let count = state.reload_policies()?;
            Ok(Response::Success {
                data: serde_json::json!({
                    "loaded": count,
                }),
            })
        }

        Request::PolicyList => {
            let rules = state.policy_engine.list_rules();
            let rules_json: Vec<serde_json::Value> = rules.iter().map(|r| {
                serde_json::json!({
                    "condition": r.condition,
                    "action": format!("{:?}", r.action),
                    "enabled": r.enabled,
                })
            }).collect();

            Ok(Response::Success {
                data: serde_json::json!({
                    "rules": rules_json,
                    "total": rules.len(),
                }),
            })
        }

        // Sprint 3: Events handlers
        Request::EventsList { filter: _, limit } => {
            let limit = limit.unwrap_or(50);
            let events = state.event_dispatcher.get_recent_events(limit);

            let events_json: Vec<serde_json::Value> = events.iter().map(|e| {
                serde_json::json!({
                    "id": e.id,
                    "timestamp": e.timestamp,
                    "event_type": format!("{:?}", e.event_type),
                    "severity": format!("{:?}", e.severity),
                    "source": e.source,
                    "message": e.message,
                    "metadata": e.metadata,
                })
            }).collect();

            Ok(Response::Success {
                data: serde_json::json!({
                    "events": events_json,
                    "total": state.event_dispatcher.event_count(),
                    "showing": events.len(),
                }),
            })
        }

        Request::EventsShow { event_type, severity } => {
            // Get events based on filters
            let events = if let Some(ref severity_str) = severity {
                use crate::events::EventSeverity;
                let min_severity = match severity_str.to_lowercase().as_str() {
                    "info" => EventSeverity::Info,
                    "warning" => EventSeverity::Warning,
                    "error" => EventSeverity::Error,
                    "critical" => EventSeverity::Critical,
                    _ => EventSeverity::Info,
                };
                state.event_dispatcher.get_events_by_severity(min_severity)
            } else {
                state.event_dispatcher.get_recent_events(50)
            };

            let events_json: Vec<serde_json::Value> = events.iter().map(|e| {
                serde_json::json!({
                    "id": e.id,
                    "timestamp": e.timestamp,
                    "event_type": format!("{:?}", e.event_type),
                    "severity": format!("{:?}", e.severity),
                    "source": e.source,
                    "message": e.message,
                    "metadata": e.metadata,
                })
            }).collect();

            Ok(Response::Success {
                data: serde_json::json!({
                    "events": events_json,
                    "filter": {
                        "event_type": event_type,
                        "severity": severity,
                    },
                }),
            })
        }

        Request::EventsClear => {
            let count = state.event_dispatcher.event_count();
            state.event_dispatcher.clear_history();
            Ok(Response::Success {
                data: serde_json::json!({
                    "cleared": count,
                }),
            })
        }

        // Sprint 3: Learning handlers
        Request::LearningStats { action } => {
            let cache = state.learning_cache.lock().unwrap();

            let stats = if let Some(action_name) = action {
                if let Some(stats) = cache.get_stats(&action_name) {
                    serde_json::to_value(vec![stats])?
                } else {
                    serde_json::json!([])
                }
            } else {
                serde_json::to_value(cache.get_all_stats())?
            };

            Ok(Response::Success {
                data: serde_json::json!({
                    "stats": stats,
                    "global": {
                        "total_actions": cache.action_count(),
                        "total_outcomes": cache.total_outcomes(),
                        "success_rate": cache.global_success_rate(),
                    },
                }),
            })
        }

        Request::LearningRecommendations => {
            let cache = state.learning_cache.lock().unwrap();
            let recommendations = cache.get_recommended_actions();
            Ok(Response::Success {
                data: serde_json::json!({
                    "recommendations": recommendations,
                }),
            })
        }

        Request::LearningReset => {
            let mut cache = state.learning_cache.lock().unwrap();
            cache.clear()?;
            telemetry::log_event(telemetry::Event::RpcCall {
                name: "learning_reset".to_string(),
                status: "cleared".to_string(),
            })?;

            Ok(Response::Success {
                data: serde_json::json!({
                    "reset": true,
                }),
            })
        }
    }
}
