//! Instant answer dispatch - pattern-matches questions and returns direct answers.
//! Bypasses LLM entirely. Returns true if answered, false to fall through to LLM.

mod system;
mod ops;

pub use system::try_system_answer;
pub use ops::try_ops_answer;

use anyhow::Result;
use tokio::io::AsyncWriteExt;

use super::helpers::send_filtered_final_answer;

pub async fn try_instant_answer(
    question: &str,
    writer: &mut tokio::net::unix::OwnedWriteHalf,
) -> Result<bool> {
    if try_system_answer(question, writer).await? {
        return Ok(true);
    }
    if try_ops_answer(question, writer).await? {
        return Ok(true);
    }
    Ok(false)
}

// --- Shared helpers used by sub-modules ---

pub(super) fn run_cmd(bin: &str, args: &[&str]) -> Result<String> {
    let out = std::process::Command::new(bin).args(args).output()?;
    Ok(String::from_utf8_lossy(&out.stdout).to_string())
}

pub(super) fn run_shell(shell_cmd: &str) -> Result<String> {
    let out = std::process::Command::new("/bin/sh")
        .args(&["-c", shell_cmd])
        .output()?;
    Ok(String::from_utf8_lossy(&out.stdout).to_string())
}

pub(super) async fn send_answer(
    writer: &mut tokio::net::unix::OwnedWriteHalf,
    answer: String,
) -> Result<()> {
    send_filtered_final_answer(writer, &answer).await?;

    let result = anna_shared::rpc::AskResult {
        answer,
        success: true,
        iterations: 0,
        commands_executed: vec![],
        dialogue: vec![],
        needs_clarification: false,
        clarification_question: None,
        cached: false,
        citations: vec![],
        abstained: false,
        final_confidence: None,
    };

    let done = anna_shared::rpc::StreamingResponse::Done { result };
    let json = serde_json::to_string(&done)?;
    writer.write_all(format!("{}\n", json).as_bytes()).await?;
    Ok(())
}
