//! Streaming - Hooks for incremental dialogue output.
//!
//! Provides hooks for emitting dialogue incrementally during ticket processing.
//! No UI in this module - just the plumbing for streaming support.

use super::narrator::DialogueLine;
use std::sync::Arc;
use tokio::sync::mpsc;

/// A streaming dialogue emitter.
pub struct DialogueStream {
    /// Channel sender for dialogue lines.
    pub(crate) sender: mpsc::Sender<StreamEvent>,
}

/// Events that can be streamed.
#[derive(Debug, Clone)]
pub enum StreamEvent {
    /// A dialogue line.
    Line(DialogueLine),
    /// Spinner started.
    SpinnerStart(String),
    /// Spinner stopped.
    SpinnerStop,
    /// Partial update (for real-time feedback).
    Partial { key: String, value: String },
    /// Stream complete.
    Complete,
}

impl DialogueStream {
    /// Create a new streaming pair (stream, receiver).
    pub fn new(buffer_size: usize) -> (Self, mpsc::Receiver<StreamEvent>) {
        let (sender, receiver) = mpsc::channel(buffer_size);
        (Self { sender }, receiver)
    }

    /// Emit a dialogue line.
    pub async fn emit_line(&self, line: DialogueLine) {
        let _ = self.sender.send(StreamEvent::Line(line)).await;
    }

    /// Start a spinner with message.
    pub async fn start_spinner(&self, message: &str) {
        let _ = self.sender.send(StreamEvent::SpinnerStart(message.to_string())).await;
    }

    /// Stop the spinner.
    pub async fn stop_spinner(&self) {
        let _ = self.sender.send(StreamEvent::SpinnerStop).await;
    }

    /// Send a partial update.
    pub async fn partial(&self, key: &str, value: &str) {
        let _ = self.sender.send(StreamEvent::Partial {
            key: key.to_string(),
            value: value.to_string(),
        }).await;
    }

    /// Signal stream complete.
    pub async fn complete(&self) {
        let _ = self.sender.send(StreamEvent::Complete).await;
    }

    /// Check if the receiver is still listening.
    pub fn is_closed(&self) -> bool {
        self.sender.is_closed()
    }
}

/// Streaming hook trait for dialogue consumers.
pub trait DialogueConsumer: Send + Sync {
    /// Called when a dialogue line is ready.
    fn on_line(&mut self, line: &DialogueLine);
    /// Called when spinner should start.
    fn on_spinner_start(&mut self, message: &str);
    /// Called when spinner should stop.
    fn on_spinner_stop(&mut self);
    /// Called on partial update.
    fn on_partial(&mut self, key: &str, value: &str);
    /// Called when stream is complete.
    fn on_complete(&mut self);
}

/// A collecting consumer that stores all lines.
#[derive(Debug, Default)]
pub struct CollectingConsumer {
    pub lines: Vec<DialogueLine>,
    pub partials: Vec<(String, String)>,
    pub completed: bool,
}

impl DialogueConsumer for CollectingConsumer {
    fn on_line(&mut self, line: &DialogueLine) {
        self.lines.push(line.clone());
    }
    fn on_spinner_start(&mut self, _message: &str) {}
    fn on_spinner_stop(&mut self) {}
    fn on_partial(&mut self, key: &str, value: &str) {
        self.partials.push((key.to_string(), value.to_string()));
    }
    fn on_complete(&mut self) {
        self.completed = true;
    }
}

/// Process a stream with a consumer.
pub async fn process_stream(
    mut receiver: mpsc::Receiver<StreamEvent>,
    consumer: &mut dyn DialogueConsumer,
) {
    while let Some(event) = receiver.recv().await {
        match event {
            StreamEvent::Line(line) => consumer.on_line(&line),
            StreamEvent::SpinnerStart(msg) => consumer.on_spinner_start(&msg),
            StreamEvent::SpinnerStop => consumer.on_spinner_stop(),
            StreamEvent::Partial { key, value } => consumer.on_partial(&key, &value),
            StreamEvent::Complete => {
                consumer.on_complete();
                break;
            }
        }
    }
}

/// Shared dialogue stream type.
pub type SharedDialogueStream = Arc<DialogueStream>;

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_stream_emit_line() {
        let (stream, mut receiver) = DialogueStream::new(16);

        let line = DialogueLine::narration("test message", 0);
        stream.emit_line(line.clone()).await;

        let event = receiver.recv().await.unwrap();
        match event {
            StreamEvent::Line(l) => assert_eq!(l.message, "test message"),
            _ => panic!("Expected Line event"),
        }
    }

    #[tokio::test]
    async fn test_stream_complete() {
        let (stream, mut receiver) = DialogueStream::new(16);

        stream.complete().await;

        let event = receiver.recv().await.unwrap();
        assert!(matches!(event, StreamEvent::Complete));
    }

    #[tokio::test]
    async fn test_collecting_consumer() {
        let (stream, receiver) = DialogueStream::new(16);
        let mut consumer = CollectingConsumer::default();

        // Emit events in a separate task
        let sender = stream.sender.clone();
        tokio::spawn(async move {
            let _ = sender.send(StreamEvent::Line(DialogueLine::narration("line 1", 0))).await;
            let _ = sender.send(StreamEvent::Line(DialogueLine::narration("line 2", 100))).await;
            let _ = sender.send(StreamEvent::Partial { key: "status".into(), value: "working".into() }).await;
            let _ = sender.send(StreamEvent::Complete).await;
        });

        process_stream(receiver, &mut consumer).await;

        assert_eq!(consumer.lines.len(), 2);
        assert_eq!(consumer.partials.len(), 1);
        assert!(consumer.completed);
    }

    #[tokio::test]
    async fn test_spinner_events() {
        let (stream, mut receiver) = DialogueStream::new(16);

        stream.start_spinner("Loading...").await;
        stream.stop_spinner().await;

        let start = receiver.recv().await.unwrap();
        assert!(matches!(start, StreamEvent::SpinnerStart(_)));

        let stop = receiver.recv().await.unwrap();
        assert!(matches!(stop, StreamEvent::SpinnerStop));
    }
}
