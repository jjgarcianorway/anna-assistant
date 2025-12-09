//! Fast path query classification (v0.0.261).
//!
//! v0.0.259: Added Uptime, CpuUsage, and NetworkStatus classification.
//! v0.0.261: Added TopProcesses classification.

use super::types::FastPathClass;

/// Classify a query as fast path or not
pub fn classify_fast_path(query: &str) -> FastPathClass {
    let q = query.to_lowercase();

    // Strip common greetings
    let stripped = strip_greetings(&q);

    // SystemHealth: "how is my computer", "any errors", "status", "health"
    if stripped.contains("how is my computer")
        || stripped.contains("how's my computer")
        || stripped.contains("computer doing")
        || stripped.contains("any errors")
        || stripped.contains("any problems")
        || stripped.contains("any issues")
        || stripped.contains("any warnings")
        || stripped.contains("errors so far")
        || stripped.contains("problems so far")
        || stripped.contains("is everything ok")
        || stripped.contains("is everything okay")
        || q.contains("health")
        || q.trim() == "status"
        || q.trim() == "errors"
        || q.trim() == "warnings"
        || q.trim() == "problems"
    {
        return FastPathClass::SystemHealth;
    }

    // WhatChanged: "what changed", "changes since", "since last time"
    if stripped.contains("what changed")
        || stripped.contains("changes since")
        || stripped.contains("since last time")
        || stripped.contains("what's new")
        || stripped.contains("what's different")
    {
        return FastPathClass::WhatChanged;
    }

    // v0.0.259: Uptime: "uptime", "how long running", "when did I boot"
    // v0.0.265: Added "boot time" pattern
    if stripped.contains("uptime")
        || stripped.contains("how long has")
        || stripped.contains("how long been running")
        || stripped.contains("when did") && stripped.contains("boot")
        || stripped.contains("last boot")
        || stripped.contains("last reboot")
        || stripped.contains("time since boot")
        || stripped.contains("boot time")
        || stripped.contains("boottime")
    {
        return FastPathClass::Uptime;
    }

    // v0.0.259: CpuUsage: "cpu usage", "cpu load", "processor"
    if stripped.contains("cpu usage")
        || stripped.contains("cpu load")
        || stripped.contains("processor usage")
        || stripped.contains("processor load")
        || stripped.contains("how busy is") && stripped.contains("cpu")
        || stripped.contains("load average")
    {
        return FastPathClass::CpuUsage;
    }

    // v0.0.259: NetworkStatus: "network status", "connected", "internet"
    if stripped.contains("network status")
        || stripped.contains("am i connected")
        || stripped.contains("internet connection")
        || stripped.contains("online")
        || stripped.contains("network connection")
        || stripped.contains("wifi status")
        || stripped.contains("ethernet status")
    {
        return FastPathClass::NetworkStatus;
    }

    // v0.0.261: TopProcesses: "what's using my cpu", "top processes", "what's eating memory"
    if stripped.contains("top process")
        || stripped.contains("what's using")
        || stripped.contains("whats using")
        || stripped.contains("what is using")
        || stripped.contains("using my cpu")
        || stripped.contains("using my memory")
        || stripped.contains("using my ram")
        || stripped.contains("eating memory")
        || stripped.contains("eating cpu")
        || stripped.contains("hogging")
        || stripped.contains("consuming cpu")
        || stripped.contains("consuming memory")
        || stripped.contains("heavy process")
        || stripped.contains("resource hog")
    {
        return FastPathClass::TopProcesses;
    }

    // DiskUsage: "disk usage", "disk space", "how much disk"
    if stripped.contains("disk usage")
        || stripped.contains("disk space")
        || stripped.contains("how much disk")
        || stripped.contains("storage space")
        || stripped.contains("free space")
        || stripped.contains("space left")
    {
        return FastPathClass::DiskUsage;
    }

    // MemoryUsage: "memory usage", "how much memory", "ram usage"
    if stripped.contains("memory usage")
        || stripped.contains("how much memory")
        || stripped.contains("ram usage")
        || stripped.contains("how much ram")
        || stripped.contains("free memory")
        || stripped.contains("memory free")
    {
        return FastPathClass::MemoryUsage;
    }

    // FailedServices: "failed services", "failed units"
    if stripped.contains("failed service")
        || stripped.contains("failed unit")
        || stripped.contains("service failures")
    {
        return FastPathClass::FailedServices;
    }

    FastPathClass::NotFastPath
}

/// Strip common greetings from query for better classification
fn strip_greetings(query: &str) -> String {
    let q = query.to_lowercase();
    let patterns = [
        "hello",
        "hi ",
        "hey ",
        "good morning",
        "good afternoon",
        "good evening",
        "anna",
        ":)",
        ":(",
        ";)",
        ":d",
        ":p",
        "!",
        "?",
        "…",
        "...",
    ];
    let mut result = q;
    for p in patterns {
        result = result.replace(p, " ");
    }
    result.split_whitespace().collect::<Vec<_>>().join(" ")
}
