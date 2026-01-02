//! Classification tests for fastpath module (v0.0.39)

use anna_shared::fastpath::{classify_fast_path, FastPathClass};

#[test]
fn test_classify_system_health_basic() {
    assert_eq!(
        classify_fast_path("how is my computer"),
        FastPathClass::SystemHealth
    );
    assert_eq!(
        classify_fast_path("any errors"),
        FastPathClass::SystemHealth
    );
    assert_eq!(
        classify_fast_path("any problems"),
        FastPathClass::SystemHealth
    );
    assert_eq!(
        classify_fast_path("any warnings"),
        FastPathClass::SystemHealth
    );
    assert_eq!(classify_fast_path("status"), FastPathClass::SystemHealth);
    assert_eq!(classify_fast_path("health"), FastPathClass::SystemHealth);
}

#[test]
fn test_classify_system_health_with_greeting() {
    // The exact test case from definition of done
    assert_eq!(
        classify_fast_path("hello anna :) how is my computer? any errors or problems so far?"),
        FastPathClass::SystemHealth
    );
}

#[test]
fn test_classify_disk_usage() {
    assert_eq!(classify_fast_path("disk usage"), FastPathClass::DiskUsage);
    assert_eq!(classify_fast_path("disk space"), FastPathClass::DiskUsage);
    assert_eq!(
        classify_fast_path("how much disk"),
        FastPathClass::DiskUsage
    );
}

#[test]
fn test_classify_memory_usage() {
    assert_eq!(
        classify_fast_path("memory usage"),
        FastPathClass::MemoryUsage
    );
    assert_eq!(
        classify_fast_path("how much memory"),
        FastPathClass::MemoryUsage
    );
    assert_eq!(classify_fast_path("ram usage"), FastPathClass::MemoryUsage);
}

#[test]
fn test_classify_failed_services() {
    assert_eq!(
        classify_fast_path("failed services"),
        FastPathClass::FailedServices
    );
    assert_eq!(
        classify_fast_path("failed units"),
        FastPathClass::FailedServices
    );
}

#[test]
fn test_classify_what_changed() {
    assert_eq!(
        classify_fast_path("what changed since last time"),
        FastPathClass::WhatChanged
    );
    assert_eq!(classify_fast_path("what's new"), FastPathClass::WhatChanged);
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
    assert_eq!(
        classify_fast_path("configure nginx"),
        FastPathClass::NotFastPath
    );
    assert_eq!(
        classify_fast_path("why is my network slow"),
        FastPathClass::NotFastPath
    );
}

#[test]
fn test_fast_path_class_display() {
    assert_eq!(FastPathClass::SystemHealth.to_string(), "system_health");
    assert_eq!(FastPathClass::DiskUsage.to_string(), "disk_usage");
    assert_eq!(FastPathClass::MemoryUsage.to_string(), "memory_usage");
    assert_eq!(FastPathClass::FailedServices.to_string(), "failed_services");
    assert_eq!(FastPathClass::WhatChanged.to_string(), "what_changed");
    assert_eq!(FastPathClass::NotFastPath.to_string(), "not_fast_path");
}
