//! Common Linux patterns that should get instant answers without clarification.
//!
//! v0.0.909: Added to reduce over-clarification (80% rate in testing).
//! v0.0.910: Added factual patterns for instant answers to common info queries.
//! v0.0.916: Added development patterns for git, docker, build tools.
//! v0.0.917: Added security patterns for firewall, permissions, users, SSH.
//! v0.0.918: Added desktop patterns for GNOME, KDE, Wayland, X11.
//! These are well-known issues with standard solutions.

mod pacman;
mod errors;
mod recovery;
mod performance;
mod factual;
mod development;
mod security;
mod desktop;

use anna_shared::rpc::DeepUnderstanding;

/// Check if a question matches a common pattern that has a known solution.
/// Returns Some(DeepUnderstanding) with high confidence if matched.
pub fn match_common_pattern(question: &str) -> Option<DeepUnderstanding> {
    let q = question.to_lowercase();

    // Check each pattern category (order matters - more specific first)
    // Factual queries first (fastest path for common info questions)
    factual::match_patterns(&q)
        .or_else(|| development::match_patterns(&q))
        .or_else(|| security::match_patterns(&q))
        .or_else(|| desktop::match_patterns(&q))
        .or_else(|| pacman::match_patterns(&q))
        .or_else(|| recovery::match_patterns(&q))
        .or_else(|| errors::match_patterns(&q))
        .or_else(|| performance::match_patterns(&q))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pacman_database_locked() {
        let result = match_common_pattern("pacman says database is locked");
        assert!(result.is_some());
        let u = result.unwrap();
        assert_eq!(u.confidence, 0.95);
        assert!(!u.needs_confirmation);
    }

    #[test]
    fn test_deleted_usr_bin() {
        let result = match_common_pattern("I accidentally deleted /usr/bin");
        assert!(result.is_some());
        assert!(!result.unwrap().needs_confirmation);
    }

    #[test]
    fn test_fan_idle() {
        let result = match_common_pattern("why does my fan spin up when the system is idle");
        assert!(result.is_some());
    }

    #[test]
    fn test_no_match() {
        let result = match_common_pattern("what is the meaning of life");
        assert!(result.is_none());
    }

    // Factual pattern tests
    #[test]
    fn test_factual_disk_usage() {
        assert!(match_common_pattern("what is my disk usage").is_some());
        assert!(match_common_pattern("show disk space").is_some());
    }

    #[test]
    fn test_factual_ram() {
        assert!(match_common_pattern("how much ram do I have").is_some());
        assert!(match_common_pattern("total memory").is_some());
    }

    #[test]
    fn test_factual_gpu() {
        assert!(match_common_pattern("what gpu do I have").is_some());
        assert!(match_common_pattern("which graphics card").is_some());
    }

    #[test]
    fn test_factual_ip() {
        assert!(match_common_pattern("what is my ip address").is_some());
        assert!(match_common_pattern("show my ip").is_some());
    }

    #[test]
    fn test_factual_kernel() {
        assert!(match_common_pattern("what kernel am I running").is_some());
        assert!(match_common_pattern("kernel version").is_some());
    }

    #[test]
    fn test_factual_services() {
        assert!(match_common_pattern("list failed services").is_some());
        assert!(match_common_pattern("show running services").is_some());
    }

    // Development pattern tests
    #[test]
    fn test_dev_git() {
        assert!(match_common_pattern("git status").is_some());
        assert!(match_common_pattern("show git log").is_some());
    }

    #[test]
    fn test_dev_docker() {
        assert!(match_common_pattern("list docker containers").is_some());
        assert!(match_common_pattern("docker images").is_some());
    }

    #[test]
    fn test_dev_build_tools() {
        assert!(match_common_pattern("cargo version").is_some());
        assert!(match_common_pattern("node version").is_some());
    }

    // Security pattern tests
    #[test]
    fn test_sec_firewall() {
        assert!(match_common_pattern("firewall status").is_some());
        assert!(match_common_pattern("ufw status").is_some());
    }

    #[test]
    fn test_sec_users() {
        assert!(match_common_pattern("list all users").is_some());
        assert!(match_common_pattern("who has sudo access").is_some());
    }

    #[test]
    fn test_sec_ssh() {
        assert!(match_common_pattern("ssh key").is_some());
        assert!(match_common_pattern("ssh status").is_some());
    }

    // Desktop pattern tests
    #[test]
    fn test_desktop_display_server() {
        assert!(match_common_pattern("wayland or x11").is_some());
        assert!(match_common_pattern("which desktop am I running").is_some());
    }

    #[test]
    fn test_desktop_gnome() {
        assert!(match_common_pattern("gnome version").is_some());
        assert!(match_common_pattern("gnome extensions").is_some());
    }

    #[test]
    fn test_desktop_kde() {
        assert!(match_common_pattern("plasma version").is_some());
        assert!(match_common_pattern("kde settings").is_some());
    }

    #[test]
    fn test_desktop_monitors() {
        assert!(match_common_pattern("list connected monitors").is_some());
        assert!(match_common_pattern("screen resolution").is_some());
    }
}
