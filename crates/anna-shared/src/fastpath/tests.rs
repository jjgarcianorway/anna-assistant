//! Tests for fastpath module (v0.0.259).
//!
//! v0.0.259: Added tests for Uptime, CpuUsage, and NetworkStatus.

#[cfg(test)]
mod tests {
    use crate::fastpath::{
        classify_fast_path, try_fast_path, FastPathClass, FastPathInput, FastPathPolicy,
    };

    #[test]
    fn test_classify_system_health() {
        assert_eq!(
            classify_fast_path("how is my computer"),
            FastPathClass::SystemHealth
        );
        assert_eq!(
            classify_fast_path("any errors"),
            FastPathClass::SystemHealth
        );
        assert_eq!(
            classify_fast_path("hello anna :) how is my computer? any errors or problems so far?"),
            FastPathClass::SystemHealth
        );
    }

    #[test]
    fn test_classify_what_changed() {
        assert_eq!(
            classify_fast_path("what changed since last time"),
            FastPathClass::WhatChanged
        );
    }

    #[test]
    fn test_classify_disk_usage() {
        assert_eq!(classify_fast_path("disk usage"), FastPathClass::DiskUsage);
        assert_eq!(classify_fast_path("how much free space"), FastPathClass::DiskUsage);
        assert_eq!(classify_fast_path("space left on disk"), FastPathClass::DiskUsage);
    }

    #[test]
    fn test_classify_uptime() {
        assert_eq!(classify_fast_path("uptime"), FastPathClass::Uptime);
        assert_eq!(classify_fast_path("how long has my computer been running"), FastPathClass::Uptime);
        assert_eq!(classify_fast_path("when was last boot"), FastPathClass::Uptime);
    }

    #[test]
    fn test_classify_cpu_usage() {
        assert_eq!(classify_fast_path("cpu usage"), FastPathClass::CpuUsage);
        assert_eq!(classify_fast_path("cpu load"), FastPathClass::CpuUsage);
        assert_eq!(classify_fast_path("load average"), FastPathClass::CpuUsage);
    }

    #[test]
    fn test_classify_network_status() {
        assert_eq!(classify_fast_path("network status"), FastPathClass::NetworkStatus);
        assert_eq!(classify_fast_path("am i connected to internet"), FastPathClass::NetworkStatus);
        assert_eq!(classify_fast_path("wifi status"), FastPathClass::NetworkStatus);
    }

    #[test]
    fn test_classify_not_fast_path() {
        assert_eq!(
            classify_fast_path("install vim"),
            FastPathClass::NotFastPath
        );
        assert_eq!(
            classify_fast_path("edit my vimrc"),
            FastPathClass::NotFastPath
        );
    }

    #[test]
    fn test_fast_path_disabled() {
        let policy = FastPathPolicy {
            enabled: false,
            ..Default::default()
        };
        let input = FastPathInput {
            request: "how is my computer",
            snapshot: None,
            facts: None,
            policy: &policy,
        };
        let result = try_fast_path(&input);
        assert!(!result.handled);
        assert!(result.trace_note.contains("disabled"));
    }
}
