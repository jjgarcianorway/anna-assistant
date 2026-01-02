//! Utility functions for helper tracking

use super::types::HelperPurpose;

/// Check if query is about helpers
pub fn is_helper_query(query: &str) -> bool {
    let q = query.to_lowercase();
    let keywords = [
        "helper",
        "helpers",
        "tools installed",
        "what tools",
        "installed tools",
        "available tools",
        "anna install",
        "which packages",
        "did anna install",
    ];
    keywords.iter().any(|k| q.contains(k))
}

/// Detect helper purpose from name
pub fn detect_purpose(name: &str) -> HelperPurpose {
    let name_lower = name.to_lowercase();

    if name_lower.contains("net") || name_lower.contains("ping") || name_lower.contains("ip") {
        HelperPurpose::NetworkDiag
    } else if name_lower.contains("disk") || name_lower.contains("df") || name_lower.contains("du") {
        HelperPurpose::DiskUtil
    } else if name_lower.contains("top") || name_lower.contains("ps") || name_lower.contains("proc") {
        HelperPurpose::ProcessMon
    } else if name_lower.contains("log") || name_lower.contains("journal") {
        HelperPurpose::LogAnalysis
    } else if name_lower.contains("sec") || name_lower.contains("crypt") || name_lower.contains("ssh") {
        HelperPurpose::Security
    } else if name_lower.contains("perf") || name_lower.contains("bench") {
        HelperPurpose::Performance
    } else if name_lower.contains("git") || name_lower.contains("make") || name_lower.contains("gcc") {
        HelperPurpose::Development
    } else if name_lower.contains("audio") || name_lower.contains("video") || name_lower.contains("ffmpeg") {
        HelperPurpose::Multimedia
    } else if name_lower.contains("sys") || name_lower.contains("info") || name_lower.contains("stat") {
        HelperPurpose::SystemInfo
    } else {
        HelperPurpose::General
    }
}
