//! Performance and resource usage patterns

use anna_shared::rpc::{DeepUnderstanding, IntentCategory};

/// Match performance-related queries
pub fn match_patterns(q: &str) -> Option<DeepUnderstanding> {
    // Thermal/fan issues
    if let Some(u) = match_thermal(q) {
        return Some(u);
    }
    // Memory issues
    if let Some(u) = match_memory(q) {
        return Some(u);
    }
    // CPU/process issues
    if let Some(u) = match_cpu(q) {
        return Some(u);
    }
    // Service/shutdown issues
    if let Some(u) = match_services(q) {
        return Some(u);
    }
    // General slowness
    if let Some(u) = match_slowness(q) {
        return Some(u);
    }
    None
}

fn match_thermal(q: &str) -> Option<DeepUnderstanding> {
    let patterns: &[(&[&str], &str)] = &[
        (&["fan", "spin", "idle"], "fan running when idle"),
        (&["fan", "loud"], "loud fan noise"),
        (&["overheating"], "system overheating"),
        (&["thermal", "throttl"], "thermal throttling"),
        (&["cpu", "temp", "high"], "high CPU temperature"),
        (&["hot", "laptop"], "laptop overheating"),
    ];

    for (keywords, interpreted) in patterns {
        if keywords.iter().all(|kw| q.contains(kw)) {
            return Some(DeepUnderstanding {
                interpreted_as: interpreted.to_string(),
                category: IntentCategory::Troubleshoot,
                confidence: 0.9,
                topic: Some("hardware".to_string()),
                needs_confirmation: false,
                ..Default::default()
            });
        }
    }
    None
}

fn match_memory(q: &str) -> Option<DeepUnderstanding> {
    let patterns: &[(&[&str], &str)] = &[
        (&["memory", "leak"], "memory leak detection"),
        (&["ram", "usage", "high"], "high RAM usage"),
        (&["ram", "full"], "RAM full"),
        (&["using", "all", "ram"], "high RAM usage"),
        (&["firefox", "memory"], "Firefox memory usage"),
        (&["chrome", "memory"], "Chrome memory usage"),
        (&["browser", "memory"], "browser memory usage"),
        (&["oom", "killer"], "OOM killer triggered"),
        (&["out of memory"], "out of memory error"),
        (&["swap", "full"], "swap space full"),
    ];

    for (keywords, interpreted) in patterns {
        if keywords.iter().all(|kw| q.contains(kw)) {
            return Some(DeepUnderstanding {
                interpreted_as: interpreted.to_string(),
                category: IntentCategory::Troubleshoot,
                confidence: 0.9,
                topic: Some("performance".to_string()),
                needs_confirmation: false,
                ..Default::default()
            });
        }
    }
    None
}

fn match_cpu(q: &str) -> Option<DeepUnderstanding> {
    let patterns: &[(&[&str], &str)] = &[
        (&["cpu", "usage", "high"], "high CPU usage"),
        (&["cpu", "100"], "CPU at 100%"),
        (&["what", "using", "cpu"], "CPU usage query"),
        (&["process", "cpu"], "process CPU usage"),
        (&["zombie", "process"], "zombie processes"),
        (&["process", "still", "running"], "orphan process"),
    ];

    for (keywords, interpreted) in patterns {
        if keywords.iter().all(|kw| q.contains(kw)) {
            return Some(DeepUnderstanding {
                interpreted_as: interpreted.to_string(),
                category: IntentCategory::Troubleshoot,
                confidence: 0.9,
                topic: Some("performance".to_string()),
                needs_confirmation: false,
                ..Default::default()
            });
        }
    }
    None
}

fn match_services(q: &str) -> Option<DeepUnderstanding> {
    let patterns: &[(&[&str], &str, &str, IntentCategory)] = &[
        (&["failed", "service"], "failed systemd services", "services", IntentCategory::Troubleshoot),
        (&["service", "fail"], "service failure", "services", IntentCategory::Troubleshoot),
        (&["what", "using", "port"], "port usage query", "network", IntentCategory::Factual),
        (&["port", "in", "use"], "port in use query", "network", IntentCategory::Factual),
        (&["won't", "shut", "down"], "shutdown hanging", "services", IntentCategory::Troubleshoot),
        (&["shutdown", "stuck"], "shutdown stuck", "services", IntentCategory::Troubleshoot),
        (&["prevent", "shutdown"], "process preventing shutdown", "services", IntentCategory::Troubleshoot),
    ];

    for (keywords, interpreted, topic, category) in patterns {
        if keywords.iter().all(|kw| q.contains(kw)) {
            return Some(DeepUnderstanding {
                interpreted_as: interpreted.to_string(),
                category: category.clone(),
                confidence: 0.9,
                topic: Some(topic.to_string()),
                needs_confirmation: false,
                ..Default::default()
            });
        }
    }
    None
}

fn match_slowness(q: &str) -> Option<DeepUnderstanding> {
    let patterns: &[(&[&str], &str, &str)] = &[
        // Boot time
        (&["boot", "time", "slow"], "slow boot time", "boot"),
        (&["boot", "takes", "long"], "slow boot time", "boot"),
        (&["slow", "boot"], "slow boot time", "boot"),
        // System slow
        (&["system", "slow"], "system performance issue", "performance"),
        (&["computer", "slow"], "system performance issue", "performance"),
        (&["it's", "slow"], "system performance issue", "performance"),
        (&["everything", "slow"], "system performance issue", "performance"),
        // Desktop/UI slow
        (&["workspace", "stutter"], "workspace switching stutter", "display"),
        (&["compositor", "lag"], "compositor lag", "display"),
        (&["animation", "stutter"], "animation stuttering", "display"),
        // Network slow
        (&["bandwidth", "using"], "bandwidth usage query", "network"),
        (&["what", "using", "network"], "network usage query", "network"),
        (&["internet", "slow"], "slow internet connection", "network"),
        (&["download", "slow"], "slow download speed", "network"),
    ];

    for (keywords, interpreted, topic) in patterns {
        if keywords.iter().all(|kw| q.contains(kw)) {
            return Some(DeepUnderstanding {
                interpreted_as: interpreted.to_string(),
                category: IntentCategory::Troubleshoot,
                confidence: 0.9,
                topic: Some(topic.to_string()),
                needs_confirmation: false,
                ..Default::default()
            });
        }
    }
    None
}
