//! Micro-Thread Formatting for Humanizer v0.0.71
//!
//! Presents complex cases as short conversational threads:
//! - Main thread: Service desk acknowledges, concludes
//! - Side thread: Department gathers evidence (indented)
//! - Return to main: Conclusion with reliability
//!
//! This is purely formatting derived from event ordering.

use super::roles::DepartmentTag;
use super::transform::HumanizedMessage;

/// Thread type for formatting
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ThreadType {
    /// Main conversation thread
    Main,
    /// Side thread for evidence gathering (indented)
    Side,
}

/// A formatted thread segment
#[derive(Debug, Clone)]
pub struct ThreadSegment {
    pub thread_type: ThreadType,
    pub messages: Vec<HumanizedMessage>,
}

impl ThreadSegment {
    pub fn main() -> Self {
        Self {
            thread_type: ThreadType::Main,
            messages: Vec::new(),
        }
    }

    pub fn side() -> Self {
        Self {
            thread_type: ThreadType::Side,
            messages: Vec::new(),
        }
    }

    pub fn push(&mut self, msg: HumanizedMessage) {
        self.messages.push(msg);
    }

    pub fn is_empty(&self) -> bool {
        self.messages.is_empty()
    }
}

/// Transcript with micro-thread structure
#[derive(Debug, Clone, Default)]
pub struct ThreadedTranscript {
    pub segments: Vec<ThreadSegment>,
}

impl ThreadedTranscript {
    pub fn new() -> Self {
        Self::default()
    }

    /// Add to main thread
    pub fn main_message(&mut self, msg: HumanizedMessage) {
        if let Some(seg) = self.segments.last_mut() {
            if seg.thread_type == ThreadType::Main {
                seg.push(msg);
                return;
            }
        }
        let mut seg = ThreadSegment::main();
        seg.push(msg);
        self.segments.push(seg);
    }

    /// Start a side thread
    pub fn start_side_thread(&mut self, dept: DepartmentTag) {
        if self.segments.last().map(|s| s.thread_type) != Some(ThreadType::Side) {
            self.segments.push(ThreadSegment::side());
        }
    }

    /// Add to current side thread
    pub fn side_message(&mut self, msg: HumanizedMessage) {
        if let Some(seg) = self.segments.last_mut() {
            if seg.thread_type == ThreadType::Side {
                seg.push(msg);
                return;
            }
        }
        let mut seg = ThreadSegment::side();
        seg.push(msg);
        self.segments.push(seg);
    }

    /// Return to main thread
    pub fn return_to_main(&mut self) {
        // Just ensure next message goes to main
        if self.segments.last().map(|s| s.thread_type) == Some(ThreadType::Side) {
            self.segments.push(ThreadSegment::main());
        }
    }

    /// Render to lines
    pub fn render(&self) -> Vec<String> {
        let mut lines = Vec::new();

        for segment in &self.segments {
            for msg in &segment.messages {
                let line = match segment.thread_type {
                    ThreadType::Main => format!("[{}] {}", msg.tag, msg.text),
                    ThreadType::Side => format!("  [{}] {}", msg.tag, msg.text),
                };
                if !line.trim().is_empty() && !msg.tag.is_empty() {
                    lines.push(line);
                } else if msg.tag.is_empty() {
                    // Reliability line
                    lines.push(msg.text.clone());
                }
            }
        }

        lines
    }
}

/// Build a threaded transcript from events
pub struct ThreadBuilder {
    transcript: ThreadedTranscript,
    in_side_thread: bool,
}

impl ThreadBuilder {
    pub fn new() -> Self {
        Self {
            transcript: ThreadedTranscript::new(),
            in_side_thread: false,
        }
    }

    /// Add case open message
    pub fn case_open(&mut self, msg: HumanizedMessage) {
        self.transcript.main_message(msg);
    }

    /// Add triage message
    pub fn triage(&mut self, msg: HumanizedMessage) {
        self.transcript.main_message(msg);
    }

    /// Start evidence gathering for a department
    pub fn start_evidence(&mut self, dept: DepartmentTag) {
        self.transcript.start_side_thread(dept);
        self.in_side_thread = true;
    }

    /// Add evidence message
    pub fn evidence(&mut self, msg: HumanizedMessage) {
        if self.in_side_thread {
            self.transcript.side_message(msg);
        } else {
            self.transcript.main_message(msg);
        }
    }

    /// End evidence gathering, return to main thread
    pub fn end_evidence(&mut self) {
        self.transcript.return_to_main();
        self.in_side_thread = false;
    }

    /// Add department finding to main thread
    pub fn finding(&mut self, msg: HumanizedMessage) {
        if self.in_side_thread {
            self.end_evidence();
        }
        self.transcript.main_message(msg);
    }

    /// Add final answer
    pub fn final_answer(&mut self, messages: Vec<HumanizedMessage>) {
        if self.in_side_thread {
            self.end_evidence();
        }
        for msg in messages {
            self.transcript.main_message(msg);
        }
    }

    /// Build the final transcript
    pub fn build(self) -> ThreadedTranscript {
        self.transcript
    }
}

impl Default for ThreadBuilder {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::super::transform::HumanizedMessage;
    use super::*;

    #[test]
    fn test_simple_transcript() {
        let mut builder = ThreadBuilder::new();
        builder.case_open(HumanizedMessage::new("service desk", "Opening case."));
        builder.triage(HumanizedMessage::new(
            "service desk",
            "I'll have network look into this.",
        ));
        builder.finding(HumanizedMessage::new("network", "Network is up."));
        builder.final_answer(vec![
            HumanizedMessage::new("service desk", "Your network is working."),
            HumanizedMessage::new("", "Reliability: 85% (good evidence coverage)"),
        ]);

        let transcript = builder.build();
        let lines = transcript.render();

        assert!(lines.iter().any(|l| l.contains("service desk")));
        assert!(lines.iter().any(|l| l.contains("network")));
        assert!(lines.iter().any(|l| l.contains("Reliability")));
    }

    #[test]
    fn test_threaded_transcript() {
        let mut builder = ThreadBuilder::new();
        builder.case_open(HumanizedMessage::new("service desk", "Opening case."));
        builder.triage(HumanizedMessage::new(
            "service desk",
            "I'll have storage look into this.",
        ));

        // Side thread
        builder.start_evidence(DepartmentTag::Storage);
        builder.evidence(HumanizedMessage::new("storage", "Checking disk status..."));
        builder.evidence(HumanizedMessage::new("storage", "Found 150 GiB free."));
        builder.end_evidence();

        builder.finding(HumanizedMessage::new(
            "storage",
            "Disk has plenty of space.",
        ));
        builder.final_answer(vec![HumanizedMessage::new(
            "service desk",
            "You have 150 GiB free.",
        )]);

        let transcript = builder.build();
        let lines = transcript.render();

        // Side thread messages should be indented
        let indented = lines.iter().filter(|l| l.starts_with("  [")).count();
        assert!(indented >= 2, "Side thread messages should be indented");

        // Main thread messages should not be indented
        let main = lines
            .iter()
            .filter(|l| l.starts_with("[service desk]"))
            .count();
        assert!(main >= 2, "Main thread messages should not be indented");
    }
}
