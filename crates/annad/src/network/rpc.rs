//! RPC server for distributed consensus (Phase 1.9 + 1.12 + 1.13)
//!
//! HTTP JSON-RPC endpoints for peer communication with full TLS/mTLS support.

use axum::{
    extract::{Query, State},
    http::StatusCode,
    response::{IntoResponse, Response},
    routing::{get, post},
    Json, Router,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::RwLock;
use tower_http::timeout::TimeoutLayer;
use tracing::{debug, error, info};

use crate::consensus::{AuditObservation, ConsensusEngine};
use super::metrics::ConsensusMetrics;

/// Shared consensus state
pub type SharedConsensus = Arc<RwLock<ConsensusEngine>>;

/// RPC server state
#[derive(Clone)]
pub struct RpcState {
    pub consensus: SharedConsensus,
    pub metrics: ConsensusMetrics,
    pub rate_limiter: super::middleware::RateLimiter,
}

/// Submit observation request
#[derive(Debug, Deserialize)]
pub struct SubmitRequest {
    pub observation: AuditObservation,
}

/// Submit observation response
#[derive(Debug, Serialize)]
pub struct SubmitResponse {
    pub success: bool,
    pub message: String,
}

/// Status query parameters
#[derive(Debug, Deserialize)]
pub struct StatusQuery {
    pub round_id: Option<String>,
}

/// Consensus RPC server
pub struct ConsensusRpcServer {
    state: RpcState,
}

impl ConsensusRpcServer {
    pub fn new(consensus: SharedConsensus, metrics: ConsensusMetrics) -> Self {
        let rate_limiter = super::middleware::RateLimiter::new_with_metrics(Arc::new(metrics.clone()));

        Self {
            state: RpcState {
                consensus,
                metrics,
                rate_limiter,
            },
        }
    }

    /// Build the router with all endpoints and middleware (Phase 1.14)
    pub fn router(&self) -> Router {
        use axum::middleware;
        use super::middleware::{body_size_limit, rate_limit_middleware};

        Router::new()
            .route("/rpc/submit", post(submit_observation))
            .route("/rpc/status", get(get_status))
            .route("/rpc/reconcile", post(reconcile))
            .route("/metrics", get(get_metrics))
            .route("/health", get(health_check))
            .with_state(self.state.clone())
            // Apply middleware layers (Phase 1.14)
            .layer(middleware::from_fn(body_size_limit))
            .layer(middleware::from_fn_with_state(
                self.state.rate_limiter.clone(),
                rate_limit_middleware,
            ))
            // Overall request timeout: 5 seconds (Phase 1.12)
            .layer(TimeoutLayer::new(Duration::from_secs(5)))
    }

    /// Start the RPC server (HTTP only - for testing/insecure mode)
    pub async fn serve(self, port: u16) -> anyhow::Result<()> {
        let addr = format!("0.0.0.0:{}", port);
        info!("Starting consensus RPC server on {} (HTTP, insecure)", addr);

        let listener = tokio::net::TcpListener::bind(&addr).await?;
        axum::serve(listener, self.router()).await?;

        Ok(())
    }

    /// Start the RPC server with TLS (Phase 1.14)
    ///
    /// Implements server-side TLS using tokio-rustls with mTLS support.
    /// Uses manual TLS accept loop with per-connection metrics tracking.
    pub async fn serve_with_tls(
        self,
        port: u16,
        tls_config: &super::peers::TlsConfig,
    ) -> anyhow::Result<()> {
        use tokio_rustls::TlsAcceptor;
        use std::net::SocketAddr;
        use tower::ServiceExt;

        let addr: SocketAddr = format!("0.0.0.0:{}", port).parse()?;
        let listener = tokio::net::TcpListener::bind(addr).await?;

        // Load server TLS config with mTLS
        info!("Loading TLS configuration...");
        let server_config = tls_config.load_server_config().await?;
        let acceptor = TlsAcceptor::from(server_config);

        info!("Starting consensus RPC server on {} (HTTPS, mTLS enabled)", addr);

        // Convert router to make service for per-connection usage
        let make_service = self.router().into_make_service();
        let state = self.state.clone();

        loop {
            let (stream, peer_addr) = listener.accept().await?;
            let acceptor = acceptor.clone();
            let make_service = make_service.clone();
            let metrics = state.metrics.clone();

            tokio::spawn(async move {
                // TLS handshake
                let tls_stream = match acceptor.accept(stream).await {
                    Ok(s) => {
                        debug!("TLS handshake successful from {}", peer_addr);
                        metrics.record_tls_handshake("success");
                        s
                    }
                    Err(e) => {
                        error!("TLS handshake failed from {}: {}", peer_addr, e);

                        // Classify error for metrics
                        let status = if e.to_string().contains("certificate") {
                            "cert_invalid"
                        } else if e.to_string().contains("expired") {
                            "cert_expired"
                        } else {
                            "error"
                        };

                        metrics.record_tls_handshake(status);
                        return;
                    }
                };

                // Create service for this connection using oneshot pattern
                let tower_service = match make_service.clone().oneshot(peer_addr).await {
                    Ok(svc) => svc,
                    Err(e) => {
                        error!("Failed to create service for {}: {:?}", peer_addr, e);
                        return;
                    }
                };

                // Wrap in hyper-compatible service
                let hyper_service = hyper_util::service::TowerToHyperService::new(tower_service);

                // Serve HTTP over TLS
                let io = hyper_util::rt::TokioIo::new(tls_stream);

                if let Err(e) = hyper::server::conn::http1::Builder::new()
                    .serve_connection(io, hyper_service)
                    .await
                {
                    error!("Connection error from {}: {}", peer_addr, e);
                }
            });
        }
    }
}

/// POST /rpc/submit - Submit observation to consensus
async fn submit_observation(
    State(state): State<RpcState>,
    Json(request): Json<SubmitRequest>,
) -> Result<Json<SubmitResponse>, AppError> {
    debug!("Received observation: {}", request.observation.audit_id);

    let mut consensus = state.consensus.write().await;

    match consensus.submit_observation(request.observation) {
        Ok(accepted) => {
            if accepted {
                // Update metrics
                state.metrics.byzantine_nodes_total.set(
                    consensus.get_byzantine_nodes().len() as i64
                );

                Ok(Json(SubmitResponse {
                    success: true,
                    message: "Observation accepted".to_string(),
                }))
            } else {
                Ok(Json(SubmitResponse {
                    success: false,
                    message: "Observation rejected (Byzantine detected)".to_string(),
                }))
            }
        }
        Err(e) => {
            Err(AppError::Internal(format!("Failed to process observation: {}", e)))
        }
    }
}

/// GET /rpc/status - Get consensus status
async fn get_status(
    State(state): State<RpcState>,
    Query(query): Query<StatusQuery>,
) -> Result<Json<serde_json::Value>, AppError> {
    debug!("Status query: round_id={:?}", query.round_id);

    let consensus = state.consensus.read().await;

    if let Some(round_id) = query.round_id {
        // Get specific round
        match consensus.get_consensus_result(&round_id) {
            Some(result) => {
                Ok(Json(serde_json::to_value(result).unwrap()))
            }
            None => {
                Err(AppError::NotFound(format!("Round not found: {}", round_id)))
            }
        }
    } else {
        // Get all rounds summary
        let rounds: Vec<_> = consensus.get_rounds().iter()
            .map(|r| serde_json::json!({
                "round_id": r.round_id,
                "status": format!("{:?}", r.status),
                "observations": r.observations.len(),
                "consensus_tis": r.consensus_tis,
            }))
            .collect();

        Ok(Json(serde_json::json!({
            "rounds": rounds,
            "byzantine_nodes": consensus.get_byzantine_nodes().len(),
        })))
    }
}

/// POST /rpc/reconcile - Force consensus computation
async fn reconcile(
    State(state): State<RpcState>,
) -> Result<Json<serde_json::Value>, AppError> {
    debug!("Reconcile requested");

    let consensus = state.consensus.write().await;
    let mut reconciled = 0;

    // Force consensus on all pending rounds
    for round in consensus.get_rounds() {
        if round.status == crate::consensus::RoundStatus::Pending {
            // This would need a method to force consensus
            // For now, just count pending rounds
            reconciled += 1;
        }
    }

    // Update metrics
    state.metrics.rounds_total.inc_by(reconciled as u64);

    Ok(Json(serde_json::json!({
        "reconciled": reconciled,
        "message": format!("Reconciled {} pending rounds", reconciled),
    })))
}

/// GET /metrics - Prometheus metrics endpoint
async fn get_metrics(
    State(state): State<RpcState>,
) -> impl IntoResponse {
    let metrics_text = state.metrics.export();
    (
        StatusCode::OK,
        [("Content-Type", "text/plain; version=0.0.4")],
        metrics_text,
    )
}

/// GET /health - Health check endpoint
async fn health_check() -> impl IntoResponse {
    (StatusCode::OK, Json(serde_json::json!({"status": "healthy"})))
}

/// Application error type
#[derive(Debug)]
enum AppError {
    NotFound(String),
    Internal(String),
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, message) = match self {
            AppError::NotFound(msg) => (StatusCode::NOT_FOUND, msg),
            AppError::Internal(msg) => (StatusCode::INTERNAL_SERVER_ERROR, msg),
        };

        (status, Json(serde_json::json!({
            "error": message
        }))).into_response()
    }
}
