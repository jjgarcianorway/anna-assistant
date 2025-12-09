//! Severity determination functions for health brief (v0.0.207).

use super::types::BriefSeverity;

/// Determine disk severity based on usage percent
pub fn disk_severity(use_percent: u8) -> BriefSeverity {
    if use_percent >= 95 {
        BriefSeverity::Error
    } else if use_percent >= 85 {
        BriefSeverity::Warning
    } else {
        BriefSeverity::Ok
    }
}

/// Determine memory severity based on usage percent
pub fn memory_severity(used_percent: u8) -> BriefSeverity {
    if used_percent >= 95 {
        BriefSeverity::Error
    } else if used_percent >= 90 {
        BriefSeverity::Warning
    } else {
        BriefSeverity::Ok
    }
}
