//! Synonym expansion for query matching.

use super::contains_word;

/// Synonym pairs for query expansion.
/// Each pair: (word, synonym) - if word is in query, also check with synonym.
pub const SYNONYMS: &[(&str, &str)] = &[
    // Memory
    ("memory", "ram"),
    ("ram", "memory"),
    ("mem", "memory"),
    // Storage
    ("disk", "storage"),
    ("storage", "disk"),
    ("drive", "disk"),
    ("ssd", "disk"),
    ("hdd", "disk"),
    ("nvme", "disk"),
    ("hard drive", "disk"),
    ("filesystem", "disk"),
    // Network
    ("internet", "network"),
    ("network", "internet"),
    ("wifi", "wireless"),
    ("wireless", "wifi"),
    ("ethernet", "wired"),
    ("lan", "network"),
    ("wan", "network"),
    ("connection", "network"),
    ("connectivity", "connection"),
    // CPU
    ("processor", "cpu"),
    ("cpu", "processor"),
    ("cores", "cpu"),
    // GPU
    ("graphics", "gpu"),
    ("gpu", "graphics"),
    ("video card", "gpu"),
    ("nvidia", "gpu"),
    ("amd", "gpu"),
    ("intel", "gpu"),
    // Temperature
    ("temperature", "temp"),
    ("temp", "temperature"),
    ("hot", "temperature"),
    ("overheating", "temperature"),
    ("thermal", "temperature"),
    ("heat", "temperature"),
    // Power
    ("power", "battery"),
    ("charging", "battery"),
    ("suspend", "sleep"),
    ("sleep", "suspend"),
    ("hibernate", "suspend"),
    // Package management
    ("package", "packages"),
    ("packages", "package"),
    ("software", "package"),
    ("app", "package"),
    ("application", "package"),
    ("program", "package"),
    // Service
    ("daemon", "service"),
    ("service", "daemon"),
    ("unit", "service"),
    // User
    ("account", "user"),
    ("user", "account"),
    ("login", "user"),
    // Files
    ("folder", "directory"),
    ("directory", "folder"),
    ("file", "files"),
    ("files", "file"),
    ("path", "directory"),
    // Boot
    ("bootloader", "grub"),
    ("startup", "boot"),
    ("reboot", "boot"),
    // Audio
    ("audio", "sound"),
    ("sound", "audio"),
    ("speaker", "audio"),
    ("speakers", "audio"),
    ("volume", "audio"),
    ("headphone", "audio"),
    ("headphones", "audio"),
    ("microphone", "audio"),
    ("mic", "microphone"),
    // Display
    ("screen", "display"),
    ("display", "screen"),
    ("monitor", "display"),
    ("monitors", "display"),
    ("resolution", "display"),
    // Printing
    ("printer", "print"),
    ("print", "printer"),
    ("printing", "print"),
    ("cups", "printer"),
    // Time/Date
    ("clock", "time"),
    ("time", "clock"),
    ("date", "time"),
    ("timezone", "time"),
    // Processes
    ("process", "processes"),
    ("processes", "process"),
    ("task", "process"),
    ("tasks", "process"),
    ("running", "process"),
    ("pid", "process"),
    // Configuration
    ("config", "configuration"),
    ("configuration", "config"),
    ("settings", "config"),
    ("options", "config"),
    ("preferences", "settings"),
    // Logs
    ("log", "logs"),
    ("logs", "log"),
    ("journal", "logs"),
    ("journalctl", "logs"),
    ("dmesg", "logs"),
    // SSH
    ("ssh", "remote"),
    ("remote", "ssh"),
    // Backup
    ("backup", "backups"),
    ("backups", "backup"),
    ("restore", "backup"),
    // Security
    ("firewall", "security"),
    ("password", "security"),
    ("permissions", "security"),
    // Cron/Schedule
    ("cron", "schedule"),
    ("schedule", "cron"),
    ("scheduled", "cron"),
    ("crontab", "cron"),
    // Swap
    ("swap", "swapfile"),
    ("swapfile", "swap"),
    // Common verbs
    ("show", "list"),
    ("display", "show"),
    ("check", "show"),
    ("view", "show"),
    ("get", "show"),
    ("find", "search"),
    ("search", "find"),
    ("look for", "find"),
    ("start", "enable"),
    ("stop", "disable"),
    ("restart", "reload"),
    ("install", "add"),
    ("remove", "uninstall"),
    ("delete", "remove"),
    ("update", "upgrade"),
    ("upgrade", "update"),
    // Common adjectives
    ("current", "active"),
    ("active", "running"),
    ("failed", "error"),
    ("broken", "failed"),
    ("slow", "performance"),
    ("fast", "performance"),
    ("high", "usage"),
    ("low", "usage"),
];

/// Expand query with synonyms for better pattern matching.
pub fn expand_with_synonyms(query: &str) -> String {
    let mut expanded = query.to_string();
    for (word, synonym) in SYNONYMS {
        if contains_word(query, word) && !contains_word(query, synonym) {
            expanded.push(' ');
            expanded.push_str(synonym);
        }
    }
    expanded
}
