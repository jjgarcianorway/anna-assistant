//! Centralized category definitions for Anna Assistant
//!
//! This module provides the canonical list of categories and their display properties.
//! All UI code should reference this module to ensure consistency.

/// Category display information with emoji and ordering
pub struct CategoryDisplay {
    pub emoji: &'static str,
    pub display_order: usize,
}

/// Get all categories in display order
pub fn get_category_order() -> Vec<&'static str> {
    vec![
        "Security & Privacy",
        "Hardware Support",
        "System Maintenance",
        "Performance & Optimization",
        "Power Management",
        "Package Management",
        "Development Tools",
        "Desktop Environment",
        "Gaming & Entertainment",
        "Multimedia & Graphics",
        "Network Configuration",
        "Utilities",
        "System Utilities",
        "Desktop Utilities",
        "System Configuration",
        "Productivity",
        "Terminal & CLI Tools",
        "Shell & Terminal",
        "Communication",
        "Engineering & CAD",
        "Desktop Customization",
    ]
}

/// Get emoji for a category
pub fn get_category_emoji(category: &str) -> &'static str {
    match category {
        "Security & Privacy" => "🔒",
        "Hardware Support" => "🔌",
        "System Maintenance" => "🔧",
        "Performance & Optimization" => "⚡",
        "Power Management" => "🔋",
        "Package Management" => "📦",
        "Development Tools" => "💻",
        "Desktop Environment" => "🖥️",
        "Gaming & Entertainment" => "🎮",
        "Multimedia & Graphics" => "🎬",
        "Network Configuration" => "📡",
        "Utilities" | "System Utilities" | "Desktop Utilities" => "🛠️",
        "System Configuration" => "⚙️",
        "Productivity" => "📊",
        "Terminal & CLI Tools" | "Shell & Terminal" => "🐚",
        "Communication" => "💬",
        "Engineering & CAD" => "📐",
        "Desktop Customization" => "🎨",
        _ => "💡",
    }
}
