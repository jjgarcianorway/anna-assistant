//! Factual query patterns - simple questions with known commands
//!
//! These are common "what is X" questions that can be answered immediately
//! with pre-cached commands, bypassing the LLM command selection pipeline.
//! v0.0.937: Added thermal, process, audio, and logs patterns
//! v0.0.945: Added time/date, environment, and shell patterns
//! v0.1.0: Use word boundary matching to prevent "update" matching "date"

mod environment;
mod network;
mod services;
mod system;

use anna_shared::rpc::DeepUnderstanding;

/// Pattern with keywords, description, topic, and pre-cached commands
pub type FactualPattern = (&'static [&'static str], &'static str, &'static str, &'static [&'static str]);

/// Match factual queries that have simple, direct answers
pub fn match_patterns(q: &str) -> Option<DeepUnderstanding> {
    // System info queries
    if let Some(u) = system::match_system_info(q) {
        return Some(u);
    }
    // Storage queries
    if let Some(u) = system::match_storage(q) {
        return Some(u);
    }
    // Network queries
    if let Some(u) = network::match_network(q) {
        return Some(u);
    }
    // v0.1.0: Process/load queries checked BEFORE hardware
    // so "cpu usage" matches before "what cpu" (which uses lscpu)
    if let Some(u) = services::match_processes(q) {
        return Some(u);
    }
    // v0.0.937: Thermal/temperature queries
    if let Some(u) = services::match_thermal(q) {
        return Some(u);
    }
    // Hardware queries (after process queries to avoid "what cpu" matching usage questions)
    if let Some(u) = system::match_hardware(q) {
        return Some(u);
    }
    // Package queries
    if let Some(u) = services::match_packages(q) {
        return Some(u);
    }
    // Service queries
    if let Some(u) = services::match_services(q) {
        return Some(u);
    }
    // v0.0.937: Audio/sound queries
    if let Some(u) = environment::match_audio(q) {
        return Some(u);
    }
    // v0.0.937: Boot/log queries
    if let Some(u) = environment::match_logs(q) {
        return Some(u);
    }
    // v0.0.945: Time/date queries
    if let Some(u) = environment::match_time(q) {
        return Some(u);
    }
    // v0.0.945: Environment/shell queries
    if let Some(u) = environment::match_environment(q) {
        return Some(u);
    }
    // v0.0.945: User/group queries
    if let Some(u) = environment::match_users(q) {
        return Some(u);
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_disk_usage() {
        let result = match_patterns("what is my disk usage");
        assert!(result.is_some());
        let u = result.unwrap();
        assert!(!u.suggested_commands.is_empty());
        assert!(u.suggested_commands.iter().any(|c| c.contains("df")));
    }

    #[test]
    fn test_ram() {
        let result = match_patterns("how much ram do I have");
        assert!(result.is_some());
        let u = result.unwrap();
        assert!(u.suggested_commands.iter().any(|c| c.contains("free")));
    }

    #[test]
    fn test_gpu() {
        let result = match_patterns("what gpu do I have");
        assert!(result.is_some());
        let u = result.unwrap();
        assert!(u.suggested_commands.iter().any(|c| c.contains("lspci")));
    }

    #[test]
    fn test_ip() {
        let result = match_patterns("what is my ip address");
        assert!(result.is_some());
        let u = result.unwrap();
        assert!(u.suggested_commands.iter().any(|c| c.contains("ip")));
    }

    #[test]
    fn test_kernel() {
        let result = match_patterns("what kernel am I running");
        assert!(result.is_some());
        let u = result.unwrap();
        assert!(u.suggested_commands.iter().any(|c| c.contains("uname")));
    }

    #[test]
    fn test_failed_services() {
        let result = match_patterns("list failed services");
        assert!(result.is_some());
        let u = result.unwrap();
        assert!(u.suggested_commands.iter().any(|c| c.contains("systemctl")));
    }
}
