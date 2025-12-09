//! Answer source detection (v0.0.179).

use anna_shared::rpc::ServiceDeskResult;
use anna_shared::transcript::{Actor, TranscriptEventKind};

/// Source of the final answer for display
pub enum AnswerSource<'a> {
    /// Answer is in the transcript (FinalAnswer event)
    Transcript,
    Clarification(&'a str),
    Answer(&'a str),
    Empty,
}

/// INVARIANT: Single source of truth for the final answer.
pub fn get_final_answer(result: &ServiceDeskResult) -> AnswerSource<'_> {
    for event in &result.transcript.events {
        if let TranscriptEventKind::FinalAnswer { .. } = &event.kind {
            debug_assert!(event.from == Actor::Anna, "FinalAnswer should be from Anna");
            debug_assert!(event.to == Some(Actor::You), "FinalAnswer should be to You");
            return AnswerSource::Transcript;
        }
    }
    if result.needs_clarification {
        return AnswerSource::Clarification(
            result
                .clarification_question
                .as_deref()
                .unwrap_or("I need more information to answer your question."),
        );
    }
    if !result.answer.is_empty() {
        return AnswerSource::Answer(&result.answer);
    }
    AnswerSource::Empty
}
