//! Direct answer generation from probe results (v0.0.403).
//!
//! This module bypasses the LLM specialist entirely for queries where
//! probe data directly and unambiguously answers the question.
//!
//! The key insight: if we have the right probes and they succeeded, we can
//! generate accurate answers deterministically - the LLM often fails at this.

mod hardware;
mod network;
mod service;
mod system;

use anna_shared::rpc::ProbeResult;

// Re-export all try_* functions from submodules
pub(crate) use hardware::{try_audio_answer, try_bluetooth_answer, try_cpu_answer, try_gpu_answer, try_webcam_answer};
pub(crate) use network::{try_network_answer, try_port_answer};
pub(crate) use service::try_service_answer;
pub(crate) use system::{try_disk_answer, try_memory_answer, try_swap_answer};

/// Result of direct probe answer generation
pub struct DirectAnswerResult {
    pub answer: String,
    pub confidence: u8,
}

/// Try to generate a direct answer from probe results based on query pattern.
/// Returns Some if we can confidently answer without LLM.
pub fn try_direct_answer(query: &str, probes: &[ProbeResult]) -> Option<DirectAnswerResult> {
    let q = query.to_lowercase();

    // Service status queries - most common LLM failure case
    if let Some(r) = try_service_answer(&q, probes) {
        return Some(r);
    }

    // Swap queries
    if let Some(r) = try_swap_answer(&q, probes) {
        return Some(r);
    }

    // Disk queries
    if let Some(r) = try_disk_answer(&q, probes) {
        return Some(r);
    }

    // Memory queries
    if let Some(r) = try_memory_answer(&q, probes) {
        return Some(r);
    }

    // Network/IP queries
    if let Some(r) = try_network_answer(&q, probes) {
        return Some(r);
    }

    // v0.0.792: Port/listening queries
    if let Some(r) = try_port_answer(&q, probes) {
        return Some(r);
    }

    // Bluetooth queries
    if let Some(r) = try_bluetooth_answer(&q, probes) {
        return Some(r);
    }

    // GPU queries
    if let Some(r) = try_gpu_answer(&q, probes) {
        return Some(r);
    }

    // Webcam queries
    if let Some(r) = try_webcam_answer(&q, probes) {
        return Some(r);
    }

    // CPU queries
    if let Some(r) = try_cpu_answer(&q, probes) {
        return Some(r);
    }

    // Audio queries
    if let Some(r) = try_audio_answer(&q, probes) {
        return Some(r);
    }

    None
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_probe(cmd: &str, stdout: &str) -> ProbeResult {
        ProbeResult {
            command: cmd.to_string(),
            exit_code: 0,
            stdout: stdout.to_string(),
            stderr: String::new(),
            timing_ms: 100,
        }
    }

    #[test]
    fn test_service_running() {
        let probe = make_probe(
            "systemctl status bluetooth.service",
            "● bluetooth.service - Bluetooth service\n   Loaded: loaded\n   Active: active (running)",
        );
        let result = try_direct_answer("is bluetooth running", &[probe]).unwrap();
        assert!(result.answer.contains("active and running"));
    }

    #[test]
    fn test_no_swap() {
        let probe = make_probe("cat /proc/swaps", "Filename\tType\tSize\tUsed\tPriority");
        let result = try_direct_answer("do i have swap", &[probe]).unwrap();
        assert!(result.answer.contains("No swap"));
    }

    #[test]
    fn test_disk_usage() {
        let probe = make_probe(
            "df -h",
            "Filesystem      Size  Used Avail Use% Mounted on\n/dev/sda1       100G   50G   50G  50% /",
        );
        let result = try_direct_answer("disk space", &[probe]).unwrap();
        assert!(result.answer.contains("Disk Usage"));
    }

    #[test]
    fn test_port_answer() {
        let probe = make_probe(
            "ss -tulpn",
            "Netid State  Recv-Q Send-Q Local Address:Port  Peer Address:Port Process\ntcp   LISTEN 0      128    127.0.0.1:8080  0.0.0.0:*     users:((\"node\",pid=1234,fd=3))",
        );
        let result = try_direct_answer("open ports", &[probe]).unwrap();
        assert!(result.answer.contains("Listening Ports"));
        assert!(result.answer.contains("8080"));
    }

    #[test]
    fn test_port_answer_what_ports() {
        let probe = make_probe(
            "ss -tulpn",
            "Netid State  Recv-Q Send-Q Local Address:Port  Peer Address:Port Process\ntcp   LISTEN 0      128    0.0.0.0:3000  0.0.0.0:*",
        );
        let result = try_direct_answer("what's using port 3000", &[probe]).unwrap();
        assert!(result.answer.contains("Listening Ports"));
        assert!(result.answer.contains("3000"));
    }
}
