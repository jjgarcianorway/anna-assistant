//! Wiki topic classification and metadata
//!
//! Pattern-based classifier that maps user questions to WikiTopic + WikiIntent.
//! Uses keyword and regex patterns per topic, not per specific question.

use crate::wiki_reasoner::{WikiIntent, WikiTopic};

/// Classification result with confidence
#[derive(Debug, Clone)]
pub struct TopicMatch {
    pub topic: WikiTopic,
    pub intent: WikiIntent,
    pub confidence: f32,
}

/// Classify a user question into topic and intent
pub fn classify_wiki_topic(question: &str) -> Option<TopicMatch> {
    let q_lower = question.to_lowercase();

    // Try each topic classifier in order of specificity
    if let Some(m) = classify_power_management(&q_lower) {
        return Some(m);
    }
    if let Some(m) = classify_disk_space(&q_lower) {
        return Some(m);
    }
    if let Some(m) = classify_boot_performance(&q_lower) {
        return Some(m);
    }
    if let Some(m) = classify_networking(&q_lower) {
        return Some(m);
    }
    if let Some(m) = classify_gpu_stack(&q_lower) {
        return Some(m);
    }

    None
}

/// Power management patterns
fn classify_power_management(q: &str) -> Option<TopicMatch> {
    let troubleshoot_patterns = [
        "battery",
        "power saving",
        "laptop gets hot",
        "laptop hot",
        "power consumption",
        "battery drain",
        "suspend",
        "hibernate",
    ];

    let configure_patterns = ["tlp", "powertop", "power management"];

    if troubleshoot_patterns.iter().any(|p| q.contains(p)) {
        return Some(TopicMatch {
            topic: WikiTopic::PowerManagement,
            intent: WikiIntent::Troubleshoot,
            confidence: 0.8,
        });
    }

    if configure_patterns.iter().any(|p| q.contains(p)) {
        let intent = if q.contains("configure") || q.contains("setup") {
            WikiIntent::Configure
        } else {
            WikiIntent::Troubleshoot
        };
        return Some(TopicMatch {
            topic: WikiTopic::PowerManagement,
            intent,
            confidence: 0.85,
        });
    }

    None
}

/// Disk space patterns
fn classify_disk_space(q: &str) -> Option<TopicMatch> {
    let patterns = [
        "disk full",
        "no space left",
        "out of space",
        "disk space",
        "cleanup",
        "clean up disk",
        "which dirs",
        "heavy director",
        "large files",
    ];

    if patterns.iter().any(|p| q.contains(p)) {
        return Some(TopicMatch {
            topic: WikiTopic::DiskSpace,
            intent: WikiIntent::Troubleshoot,
            confidence: 0.9,
        });
    }

    None
}

/// Boot performance patterns
fn classify_boot_performance(q: &str) -> Option<TopicMatch> {
    let patterns = [
        "boot slow",
        "startup slow",
        "long boot time",
        "boot time",
        "slow startup",
        "boot performance",
    ];

    if patterns.iter().any(|p| q.contains(p)) {
        return Some(TopicMatch {
            topic: WikiTopic::BootPerformance,
            intent: WikiIntent::Troubleshoot,
            confidence: 0.85,
        });
    }

    None
}

/// Networking patterns
fn classify_networking(q: &str) -> Option<TopicMatch> {
    let troubleshoot_patterns = [
        "wifi drops",
        "wifi unstable",
        "no internet",
        "dns",
        "network unstable",
        "connection drops",
        "can't connect",
        "network problem",
    ];

    let configure_patterns = ["networkmanager", "configure network", "setup wifi"];

    if troubleshoot_patterns.iter().any(|p| q.contains(p)) {
        return Some(TopicMatch {
            topic: WikiTopic::Networking,
            intent: WikiIntent::Troubleshoot,
            confidence: 0.85,
        });
    }

    if configure_patterns.iter().any(|p| q.contains(p)) {
        return Some(TopicMatch {
            topic: WikiTopic::Networking,
            intent: WikiIntent::Configure,
            confidence: 0.8,
        });
    }

    None
}

/// GPU stack patterns
fn classify_gpu_stack(q: &str) -> Option<TopicMatch> {
    let troubleshoot_patterns = [
        "nvidia",
        "drivers",
        "screen tearing",
        "wayland problem",
        "xorg problem",
        "gpu",
        "graphics",
        "amdgpu",
        "intel graphics",
    ];

    if troubleshoot_patterns.iter().any(|p| q.contains(p)) {
        let intent = if q.contains("install") || q.contains("setup") {
            WikiIntent::Install
        } else {
            WikiIntent::Troubleshoot
        };
        return Some(TopicMatch {
            topic: WikiTopic::GpuStack,
            intent,
            confidence: 0.75,
        });
    }

    None
}

/// Metadata for each wiki topic
pub struct WikiTopicMetadata {
    pub page_url: &'static str,
    pub key_sections: &'static [&'static str],
}

/// Get metadata for a wiki topic
pub fn get_topic_metadata(topic: WikiTopic) -> WikiTopicMetadata {
    match topic {
        WikiTopic::PowerManagement => WikiTopicMetadata {
            page_url: "https://wiki.archlinux.org/title/Power_management",
            key_sections: &["Troubleshooting", "Power management tools", "TLP"],
        },
        WikiTopic::DiskSpace => WikiTopicMetadata {
            page_url: "https://wiki.archlinux.org/title/Improving_performance",
            key_sections: &["File system maintenance", "Disk space"],
        },
        WikiTopic::BootPerformance => WikiTopicMetadata {
            page_url: "https://wiki.archlinux.org/title/Improving_performance",
            key_sections: &["Boot process", "systemd"],
        },
        WikiTopic::Networking => WikiTopicMetadata {
            page_url: "https://wiki.archlinux.org/title/Network_configuration",
            key_sections: &["Troubleshooting", "NetworkManager", "DNS"],
        },
        WikiTopic::GpuStack => WikiTopicMetadata {
            page_url: "https://wiki.archlinux.org/title/NVIDIA",
            key_sections: &["Installation", "Troubleshooting", "Wayland"],
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_classify_disk_space_question() {
        let result = classify_wiki_topic("my disk full, what should I do?");
        assert!(result.is_some());
        let m = result.unwrap();
        assert_eq!(m.topic, WikiTopic::DiskSpace);
        assert_eq!(m.intent, WikiIntent::Troubleshoot);
        assert!(m.confidence >= 0.8);
    }

    #[test]
    fn test_classify_power_management_question() {
        let result = classify_wiki_topic("my laptop gets very hot, help me configure power saving");
        assert!(result.is_some());
        let m = result.unwrap();
        assert_eq!(m.topic, WikiTopic::PowerManagement);
        assert!(m.confidence >= 0.7);
    }

    #[test]
    fn test_classify_network_wifi_question() {
        let result = classify_wiki_topic("my wifi drops sometimes");
        assert!(result.is_some());
        let m = result.unwrap();
        assert_eq!(m.topic, WikiTopic::Networking);
        assert_eq!(m.intent, WikiIntent::Troubleshoot);
        assert!(m.confidence >= 0.8);
    }

    #[test]
    fn test_classify_boot_performance_question() {
        let result = classify_wiki_topic("my boot slow, how do I fix it?");
        assert!(result.is_some());
        let m = result.unwrap();
        assert_eq!(m.topic, WikiTopic::BootPerformance);
        assert_eq!(m.intent, WikiIntent::Troubleshoot);
        assert!(m.confidence >= 0.8);
    }

    #[test]
    fn test_no_match() {
        let result = classify_wiki_topic("hello how are you");
        assert!(result.is_none());
    }

    #[test]
    fn test_get_power_management_metadata() {
        let meta = get_topic_metadata(WikiTopic::PowerManagement);
        assert!(meta.page_url.contains("Power_management"));
        assert!(!meta.key_sections.is_empty());
    }

    #[test]
    fn test_classify_gpu_stack_question() {
        let result = classify_wiki_topic("my nvidia drivers are broken");
        assert!(result.is_some());
        let m = result.unwrap();
        assert_eq!(m.topic, WikiTopic::GpuStack);
        assert_eq!(m.intent, WikiIntent::Troubleshoot);
        assert!(m.confidence >= 0.7);
    }

    #[test]
    fn test_classify_gpu_install_question() {
        let result = classify_wiki_topic("how do I install NVIDIA drivers?");
        assert!(result.is_some());
        let m = result.unwrap();
        assert_eq!(m.topic, WikiTopic::GpuStack);
        assert_eq!(m.intent, WikiIntent::Install);
    }

    #[test]
    fn test_all_topics_have_metadata() {
        // Ensure all 5 topics have valid metadata
        let topics = vec![
            WikiTopic::PowerManagement,
            WikiTopic::DiskSpace,
            WikiTopic::BootPerformance,
            WikiTopic::Networking,
            WikiTopic::GpuStack,
        ];

        for topic in topics {
            let meta = get_topic_metadata(topic);
            assert!(!meta.page_url.is_empty(), "Topic {:?} missing URL", topic);
            assert!(!meta.key_sections.is_empty(), "Topic {:?} missing sections", topic);
        }
    }

    #[test]
    fn test_confidence_scoring() {
        // High confidence - exact keyword match
        let high = classify_wiki_topic("wifi drops frequently");
        assert!(high.is_some());
        assert!(high.unwrap().confidence >= 0.8);

        // Lower confidence - "graphics" is less specific
        let lower = classify_wiki_topic("graphics problem");
        assert!(lower.is_some());
        assert!(lower.unwrap().confidence >= 0.7);
    }

    #[test]
    fn test_intent_classification() {
        // Troubleshoot intent
        let troubleshoot = classify_wiki_topic("my battery drains too fast");
        assert_eq!(troubleshoot.unwrap().intent, WikiIntent::Troubleshoot);

        // Install intent (GPU stack supports install detection)
        let install = classify_wiki_topic("install nvidia drivers");
        assert_eq!(install.unwrap().intent, WikiIntent::Install);

        // Configure intent
        let configure = classify_wiki_topic("configure power management");
        assert_eq!(configure.unwrap().intent, WikiIntent::Configure);

        // Setup intent (similar to configure)
        let setup = classify_wiki_topic("setup tlp");
        assert_eq!(setup.unwrap().intent, WikiIntent::Configure);
    }
}
