//! Streaming Debug Engine v0.43.0
//!
//! Real-time streaming of debug events during question processing.
//! Events are emitted as newline-delimited JSON (NDJSON) for live display.

use anna_common::{
    DebugEvent, DebugEventData, DebugEventEmitter, DebugEventType, DebugStreamConfig,
    ProbeResultSnippet,
};
use std::sync::{Arc, Mutex};
use tokio::sync::mpsc;

/// Channel-based debug event emitter for streaming
pub struct ChannelEmitter {
    /// Sender channel for debug events
    tx: mpsc::UnboundedSender<DebugEvent>,
    /// Whether debug mode is active
    active: bool,
    /// Configuration
    config: DebugStreamConfig,
}

impl ChannelEmitter {
    /// Create a new channel emitter
    pub fn new(tx: mpsc::UnboundedSender<DebugEvent>, active: bool) -> Self {
        Self {
            tx,
            active,
            config: DebugStreamConfig::default(),
        }
    }

    /// Create with custom config
    pub fn with_config(
        tx: mpsc::UnboundedSender<DebugEvent>,
        active: bool,
        config: DebugStreamConfig,
    ) -> Self {
        Self { tx, active, config }
    }

    /// Send stream started event
    pub fn stream_started(&self, question: &str, junior_model: &str, senior_model: &str) {
        if self.active {
            self.emit(
                DebugEvent::new(
                    DebugEventType::StreamStarted,
                    0,
                    "Debug stream started",
                )
                .with_data(DebugEventData::StreamMeta {
                    question: truncate_string(question, 200),
                    junior_model: junior_model.to_string(),
                    senior_model: senior_model.to_string(),
                }),
            );
        }
    }

    /// Send stream ended event
    pub fn stream_ended(&self, duration_secs: f64) {
        if self.active {
            self.emit(
                DebugEvent::new(DebugEventType::StreamEnded, 0, "Debug stream ended").with_data(
                    DebugEventData::KeyValue {
                        pairs: vec![("duration_secs".to_string(), format!("{:.2}", duration_secs))],
                    },
                ),
            );
        }
    }
}

impl DebugEventEmitter for ChannelEmitter {
    fn emit(&self, event: DebugEvent) {
        // Ignore send errors (receiver might have dropped)
        let _ = self.tx.send(event);
    }

    fn is_active(&self) -> bool {
        self.active
    }
}

/// No-op emitter for non-debug mode
pub struct NoopEmitter;

impl DebugEventEmitter for NoopEmitter {
    fn emit(&self, _event: DebugEvent) {
        // No-op
    }

    fn is_active(&self) -> bool {
        false
    }
}

/// Shared emitter that can be passed to multiple components
pub type SharedEmitter = Arc<dyn DebugEventEmitter + Send + Sync>;

/// Create a shared channel emitter
pub fn create_channel_emitter(
    tx: mpsc::UnboundedSender<DebugEvent>,
    active: bool,
) -> SharedEmitter {
    Arc::new(ChannelEmitter::new(tx, active))
}

/// Create a shared noop emitter
pub fn create_noop_emitter() -> SharedEmitter {
    Arc::new(NoopEmitter)
}

/// Helper: truncate string for display
fn truncate_string(s: &str, max_len: usize) -> String {
    if s.len() <= max_len {
        s.to_string()
    } else {
        let truncated: String = s.chars().take(max_len.saturating_sub(3)).collect();
        format!("{}...", truncated)
    }
}

/// Streaming response wrapper for axum
pub mod response {
    use super::*;
    use axum::body::Body;
    use axum::http::header;
    use axum::response::{IntoResponse, Response};
    use futures_util::stream::StreamExt;
    use tokio_stream::wrappers::UnboundedReceiverStream;

    /// Create a streaming response from a debug event receiver
    pub fn debug_stream_response(rx: mpsc::UnboundedReceiver<DebugEvent>) -> Response {
        let stream = UnboundedReceiverStream::new(rx).map(|event| {
            // Format as NDJSON (newline-delimited JSON)
            let json = event.to_ndjson();
            Ok::<_, std::io::Error>(format!("{}\n", json))
        });

        let body = Body::from_stream(stream);

        Response::builder()
            .status(200)
            .header(header::CONTENT_TYPE, "application/x-ndjson")
            .header(header::CACHE_CONTROL, "no-cache")
            .header(header::CONNECTION, "keep-alive")
            .header("X-Content-Type-Options", "nosniff")
            .body(body)
            .unwrap()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_channel_emitter() {
        let (tx, mut rx) = mpsc::unbounded_channel();
        let emitter = ChannelEmitter::new(tx, true);

        emitter.iteration_started(1);

        let event = rx.try_recv().unwrap();
        assert_eq!(event.event_type, DebugEventType::IterationStarted);
        assert_eq!(event.iteration, 1);
    }

    #[test]
    fn test_noop_emitter() {
        let emitter = NoopEmitter;
        assert!(!emitter.is_active());

        // Should not panic
        emitter.iteration_started(1);
    }

    #[test]
    fn test_truncate_string() {
        assert_eq!(truncate_string("hello", 10), "hello");
        assert_eq!(truncate_string("hello world test", 10), "hello w...");
    }
}
