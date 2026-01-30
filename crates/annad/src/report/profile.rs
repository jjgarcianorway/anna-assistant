//! User profile aggregation for personalized reports.
//!
//! Aggregates data from sessions, learning, and patterns to build
//! a profile of the user's interests and usage patterns.

use std::collections::HashMap;

/// Aggregated user profile for report personalization.
#[derive(Debug, Default)]
pub struct UserProfile {
    pub name: String,
    pub top_topics: Vec<(String, u32)>,
    pub frequent_packages: Vec<String>,
    pub watched_services: Vec<String>,
    pub recurring_issues: Vec<String>,
    pub questions_this_week: u32,
    pub favorite_commands: Vec<String>,
}

impl UserProfile {
    /// Load and aggregate user profile from various sources.
    pub fn load() -> Self {
        let name = super::user::detect_logged_in_user().unwrap_or_default();

        let mut profile = Self {
            name,
            ..Default::default()
        };

        // Load topics from session patterns
        profile.load_topics_from_sessions();

        // Load package history
        profile.load_package_history();

        // Load service interactions
        profile.load_service_interactions();

        // Load recurring issues
        profile.load_recurring_issues();

        profile
    }

    fn load_topics_from_sessions(&mut self) {
        // Try to read sessions.json and extract topics
        let path = std::path::Path::new("/var/lib/anna/sessions.json");
        if let Ok(content) = std::fs::read_to_string(path) {
            if let Ok(sessions) = serde_json::from_str::<serde_json::Value>(&content) {
                let mut topic_counts: HashMap<String, u32> = HashMap::new();
                let mut question_count = 0u32;

                // Count topics from session contexts
                if let Some(arr) = sessions.as_array() {
                    for session in arr {
                        // Count questions
                        if let Some(history) = session.get("history").and_then(|h| h.as_array()) {
                            question_count += history.len() as u32;

                            // Extract keywords from questions
                            for turn in history {
                                if let Some(q) = turn.get("question").and_then(|q| q.as_str()) {
                                    for topic in extract_topics(q) {
                                        *topic_counts.entry(topic).or_insert(0) += 1;
                                    }
                                }
                            }
                        }

                        // Check context topics
                        if let Some(ctx) = session.get("context") {
                            if let Some(topic) = ctx.get("current_topic").and_then(|t| t.as_str()) {
                                *topic_counts.entry(topic.to_string()).or_insert(0) += 1;
                            }
                        }
                    }
                }

                self.questions_this_week = question_count;

                // Sort topics by frequency
                let mut topics: Vec<_> = topic_counts.into_iter().collect();
                topics.sort_by(|a, b| b.1.cmp(&a.1));
                self.top_topics = topics.into_iter().take(5).collect();
            }
        }
    }

    fn load_package_history(&mut self) {
        let path = std::path::Path::new("/var/lib/anna/learning.json");
        if let Ok(content) = std::fs::read_to_string(path) {
            if let Ok(learning) = serde_json::from_str::<serde_json::Value>(&content) {
                let mut pkg_counts: HashMap<String, u32> = HashMap::new();

                // Count package interactions
                if let Some(history) = learning.get("package_history").and_then(|h| h.as_array()) {
                    for entry in history {
                        if let Some(packages) = entry.get("packages").and_then(|p| p.as_array()) {
                            for pkg in packages {
                                if let Some(name) = pkg.as_str() {
                                    *pkg_counts.entry(name.to_string()).or_insert(0) += 1;
                                }
                            }
                        }
                    }
                }

                // Sort by frequency
                let mut packages: Vec<_> = pkg_counts.into_iter().collect();
                packages.sort_by(|a, b| b.1.cmp(&a.1));
                self.frequent_packages = packages.into_iter().take(5).map(|(p, _)| p).collect();
            }
        }
    }

    fn load_service_interactions(&mut self) {
        let path = std::path::Path::new("/var/lib/anna/sessions.json");
        if let Ok(content) = std::fs::read_to_string(path) {
            if let Ok(sessions) = serde_json::from_str::<serde_json::Value>(&content) {
                let mut service_counts: HashMap<String, u32> = HashMap::new();

                if let Some(arr) = sessions.as_array() {
                    for session in arr {
                        // Check entities for services
                        if let Some(entities) = session.get("entities") {
                            if let Some(services) = entities.get("services").and_then(|s| s.as_array()) {
                                for svc in services {
                                    if let Some(name) = svc.as_str() {
                                        *service_counts.entry(name.to_string()).or_insert(0) += 1;
                                    }
                                }
                            }
                        }
                    }
                }

                let mut services: Vec<_> = service_counts.into_iter().collect();
                services.sort_by(|a, b| b.1.cmp(&a.1));
                self.watched_services = services.into_iter().take(5).map(|(s, _)| s).collect();
            }
        }
    }

    fn load_recurring_issues(&mut self) {
        let path = std::path::Path::new("/var/lib/anna/issues.json");
        if let Ok(content) = std::fs::read_to_string(path) {
            if let Ok(issues) = serde_json::from_str::<serde_json::Value>(&content) {
                if let Some(arr) = issues.as_array() {
                    for issue in arr.iter().take(3) {
                        if let Some(desc) = issue.get("description").and_then(|d| d.as_str()) {
                            self.recurring_issues.push(desc.to_string());
                        }
                    }
                }
            }
        }
    }

    /// Generate a personalized section based on the profile.
    pub fn generate_interests_section(&self) -> String {
        let mut lines = Vec::new();

        if !self.top_topics.is_empty() {
            let topics: Vec<_> = self.top_topics.iter()
                .map(|(t, c)| format!("{} ({}x)", t, c))
                .collect();
            lines.push(format!("Your focus areas: {}", topics.join(", ")));
        }

        if !self.watched_services.is_empty() {
            lines.push(format!("Services you monitor: {}", self.watched_services.join(", ")));
        }

        if !self.frequent_packages.is_empty() {
            lines.push(format!("Frequently used packages: {}", self.frequent_packages.join(", ")));
        }

        if self.questions_this_week > 0 {
            lines.push(format!("You've asked {} questions recently.", self.questions_this_week));
        }

        if lines.is_empty() {
            "Your usage profile is still being learned.".to_string()
        } else {
            lines.join("\n")
        }
    }
}

/// Extract topics from a question string.
fn extract_topics(question: &str) -> Vec<String> {
    let q = question.to_lowercase();
    let mut topics = Vec::new();

    let topic_keywords = [
        ("docker", "Docker"),
        ("container", "Containers"),
        ("network", "Networking"),
        ("wifi", "WiFi"),
        ("bluetooth", "Bluetooth"),
        ("audio", "Audio"),
        ("pipewire", "Audio"),
        ("pulseaudio", "Audio"),
        ("nvidia", "NVIDIA GPU"),
        ("gpu", "GPU"),
        ("disk", "Storage"),
        ("storage", "Storage"),
        ("memory", "Memory"),
        ("ram", "Memory"),
        ("cpu", "CPU"),
        ("performance", "Performance"),
        ("systemd", "Services"),
        ("service", "Services"),
        ("package", "Packages"),
        ("pacman", "Packages"),
        ("update", "Updates"),
        ("kernel", "Kernel"),
        ("boot", "Boot"),
        ("display", "Display"),
        ("monitor", "Display"),
        ("kde", "KDE"),
        ("gnome", "GNOME"),
        ("ssh", "SSH"),
        ("git", "Git"),
        ("python", "Python"),
        ("rust", "Rust"),
    ];

    for (keyword, topic) in topic_keywords {
        if q.contains(keyword) && !topics.contains(&topic.to_string()) {
            topics.push(topic.to_string());
        }
    }

    topics
}
