//! Helper functions for streaming responses.

use anna_shared::rpc::{AskResult, DialogueStep, StepType, StreamingResponse};
use anyhow::Result;

/// Send a step over the streaming connection.
pub async fn send_step<W: tokio::io::AsyncWriteExt + Unpin>(
    writer: &mut W,
    step: DialogueStep,
) -> Result<()> {
    let resp = StreamingResponse::Step { step };
    let json = serde_json::to_string(&resp)?;
    writer.write_all(format!("{}\n", json).as_bytes()).await?;
    writer.flush().await?;
    Ok(())
}

/// Create a step, push it to dialogue, and send it.
pub async fn push_and_send<W: tokio::io::AsyncWriteExt + Unpin>(
    writer: &mut W,
    dialogue: &mut Vec<DialogueStep>,
    step_type: StepType,
    content: String,
) -> Result<()> {
    let step = DialogueStep { step_type, content };
    dialogue.push(step.clone());
    send_step(writer, step).await
}

/// Send the final Done response.
pub async fn send_done<W: tokio::io::AsyncWriteExt + Unpin>(
    writer: &mut W,
    result: &AskResult,
) -> Result<()> {
    let resp = StreamingResponse::Done {
        result: result.clone(),
    };
    let json = serde_json::to_string(&resp)?;
    writer.write_all(format!("{}\n", json).as_bytes()).await?;
    writer.flush().await?;
    Ok(())
}

/// Send all dialogue steps from a result.
pub async fn send_dialogue_steps<W: tokio::io::AsyncWriteExt + Unpin>(
    writer: &mut W,
    dialogue: &[DialogueStep],
) -> Result<()> {
    for step in dialogue {
        send_step(writer, step.clone()).await?;
    }
    Ok(())
}

/// Build the final answer with evidence and teaching block.
pub fn build_final_answer(
    answer: &str,
    evidence_line: &str,
    teaching_block: Option<String>,
) -> String {
    let mut final_answer = answer.to_string();
    if !evidence_line.is_empty() {
        final_answer = format!("{}\n\n{}", final_answer, evidence_line);
    }
    if let Some(teaching) = teaching_block {
        final_answer = format!("{}{}", final_answer, teaching);
    }
    final_answer
}
