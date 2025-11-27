//! Anna Presentation Layer v0.12.0
//!
//! ASCII-only, sysadmin-style output formatting.
//! No emojis. No Unicode box characters. Professional and compact.

use owo_colors::OwoColorize;

/// ASCII separator line (60 chars)
pub const SEPARATOR: &str = "============================================================";
/// ASCII thin separator
pub const THIN_SEPARATOR: &str = "------------------------------------------------------------";

/// Reliability score thresholds
pub const THRESHOLD_HIGH: f64 = 0.9;
pub const THRESHOLD_MEDIUM: f64 = 0.7;

/// Reliability color category
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ReliabilityColor {
    Green,  // >= 0.9
    Yellow, // 0.7 - 0.9
    Red,    // < 0.7
}

impl ReliabilityColor {
    pub fn from_score(score: f64) -> Self {
        if score >= THRESHOLD_HIGH {
            ReliabilityColor::Green
        } else if score >= THRESHOLD_MEDIUM {
            ReliabilityColor::Yellow
        } else {
            ReliabilityColor::Red
        }
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            ReliabilityColor::Green => "green",
            ReliabilityColor::Yellow => "yellow",
            ReliabilityColor::Red => "red",
        }
    }
}

/// Format reliability score with ANSI color
pub fn format_reliability_score(score: f64) -> String {
    let color = ReliabilityColor::from_score(score);
    let score_str = format!("{:.2}", score);
    let colored = match color {
        ReliabilityColor::Green => score_str.green().to_string(),
        ReliabilityColor::Yellow => score_str.yellow().to_string(),
        ReliabilityColor::Red => score_str.red().to_string(),
    };
    format!("{} ({})", colored, color.as_str())
}

/// Report verbosity level inferred from user prompt
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Verbosity {
    /// Brief, single line or very short block
    Short,
    /// Standard response
    #[default]
    Normal,
    /// Multi-section with all headings
    Detailed,
}

impl Verbosity {
    /// Infer verbosity from user prompt
    pub fn from_prompt(prompt: &str) -> Self {
        let lower = prompt.to_lowercase();

        // Short answer indicators
        if lower.contains("short answer")
            || lower.contains("brief")
            || lower.contains("quick")
            || lower.contains("one line")
            || lower.contains("tl;dr")
        {
            return Verbosity::Short;
        }

        // Detailed report indicators
        if lower.contains("detailed")
            || lower.contains("full report")
            || lower.contains("comprehensive")
            || lower.contains("in depth")
            || lower.contains("complete report")
        {
            return Verbosity::Detailed;
        }

        Verbosity::Normal
    }
}

/// Section headers for structured reports
pub mod sections {
    pub const SUMMARY: &str = "[SUMMARY]";
    pub const DETAILS: &str = "[DETAILS]";
    pub const EVIDENCE: &str = "[EVIDENCE]";
    pub const RELIABILITY: &str = "[RELIABILITY]";
    pub const NEXT_STEPS: &str = "[NEXT STEPS]";
}

/// Terminal hyperlink support
pub fn hyperlink(url: &str, text: &str) -> String {
    // OSC 8 hyperlink format: \x1b]8;;URL\x1b\\TEXT\x1b]8;;\x1b\\
    format!("\x1b]8;;{}\x1b\\{}\x1b]8;;\x1b\\", url, text)
}

/// Common documentation links
pub mod links {
    use super::hyperlink;

    pub fn arch_wiki(page: &str, display: &str) -> String {
        let url = format!("https://wiki.archlinux.org/title/{}", page);
        hyperlink(&url, display)
    }

    pub fn pacman() -> String {
        arch_wiki("Pacman", "Pacman (Arch Wiki)")
    }

    pub fn systemd() -> String {
        arch_wiki("Systemd", "systemd (Arch Wiki)")
    }

    pub fn network_manager() -> String {
        arch_wiki("NetworkManager", "NetworkManager (Arch Wiki)")
    }
}

/// Format a section header
pub fn section_header(name: &str) -> String {
    format!("{}", name.bold())
}

/// Format a bullet point
pub fn bullet(text: &str) -> String {
    format!("  * {}", text)
}

/// Format a key-value pair with source
pub fn key_value(key: &str, value: &str, source: &str) -> String {
    format!(
        "  * {}: {} {}",
        key.bold(),
        value,
        format!("[source: {}]", source).dimmed()
    )
}

/// Format an evidence entry
pub fn evidence_entry(probe_id: &str, description: &str, freshness: &str) -> String {
    format!(
        "  * {} - {} ({})",
        probe_id.cyan(),
        description,
        freshness.dimmed()
    )
}

/// Build a structured report
#[derive(Debug, Clone, Default)]
pub struct ReportBuilder {
    pub title: Option<String>,
    pub summary: Vec<String>,
    pub details: Vec<String>,
    pub evidence: Vec<String>,
    pub reliability_score: f64,
    pub reliability_factors: Vec<String>,
    pub reliability_risks: Vec<String>,
    pub internal_passes: u8,
    pub threshold_reached: bool,
    pub main_limitations: Vec<String>,
    pub next_steps: Vec<String>,
    pub verbosity: Verbosity,
}

impl ReportBuilder {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_verbosity(mut self, verbosity: Verbosity) -> Self {
        self.verbosity = verbosity;
        self
    }

    pub fn title(mut self, title: &str) -> Self {
        self.title = Some(title.to_string());
        self
    }

    pub fn add_summary(mut self, item: &str) -> Self {
        self.summary.push(item.to_string());
        self
    }

    pub fn add_detail(mut self, item: &str) -> Self {
        self.details.push(item.to_string());
        self
    }

    pub fn add_evidence(mut self, probe_id: &str, desc: &str, freshness: &str) -> Self {
        self.evidence
            .push(evidence_entry(probe_id, desc, freshness));
        self
    }

    pub fn reliability(mut self, score: f64) -> Self {
        self.reliability_score = score;
        self
    }

    pub fn add_reliability_factor(mut self, factor: &str) -> Self {
        self.reliability_factors.push(factor.to_string());
        self
    }

    pub fn add_risk(mut self, risk: &str) -> Self {
        self.reliability_risks.push(risk.to_string());
        self
    }

    pub fn passes(mut self, count: u8, reached: bool) -> Self {
        self.internal_passes = count;
        self.threshold_reached = reached;
        self
    }

    pub fn add_limitation(mut self, limitation: &str) -> Self {
        self.main_limitations.push(limitation.to_string());
        self
    }

    pub fn add_next_step(mut self, step: &str) -> Self {
        self.next_steps.push(step.to_string());
        self
    }

    /// Build the final report string
    pub fn build(&self) -> String {
        match self.verbosity {
            Verbosity::Short => self.build_short(),
            Verbosity::Normal => self.build_normal(),
            Verbosity::Detailed => self.build_detailed(),
        }
    }

    fn build_short(&self) -> String {
        let mut lines = Vec::new();

        // Just summary, one line per item
        for item in &self.summary {
            lines.push(item.clone());
        }

        // Compact reliability
        lines.push(format!(
            "Reliability: {}",
            format_reliability_score(self.reliability_score)
        ));

        lines.join("\n")
    }

    fn build_normal(&self) -> String {
        let mut lines = Vec::new();

        // Title if present
        if let Some(ref title) = self.title {
            lines.push(format!("{}", title.bold()));
            lines.push(THIN_SEPARATOR.dimmed().to_string());
        }

        // Summary
        if !self.summary.is_empty() {
            for item in &self.summary {
                lines.push(bullet(item));
            }
            lines.push(String::new());
        }

        // Reliability inline
        lines.push(format!(
            "Reliability: {}",
            format_reliability_score(self.reliability_score)
        ));

        // Evidence summary
        if !self.evidence.is_empty() {
            lines.push(format!("Evidence: {} sources", self.evidence.len()));
        }

        lines.join("\n")
    }

    fn build_detailed(&self) -> String {
        let mut lines = Vec::new();

        lines.push(SEPARATOR.to_string());

        // Title
        if let Some(ref title) = self.title {
            lines.push(format!("{}", title.bold()));
            lines.push(String::new());
        }

        // Summary section
        if !self.summary.is_empty() {
            lines.push(section_header(sections::SUMMARY));
            for item in &self.summary {
                lines.push(bullet(item));
            }
            lines.push(String::new());
        }

        // Details section
        if !self.details.is_empty() {
            lines.push(section_header(sections::DETAILS));
            for item in &self.details {
                lines.push(format!("  {}", item));
            }
            lines.push(String::new());
        }

        // Evidence section
        if !self.evidence.is_empty() {
            lines.push(section_header(sections::EVIDENCE));
            for item in &self.evidence {
                lines.push(item.clone());
            }
            lines.push(String::new());
        }

        // Reliability section
        lines.push(section_header(sections::RELIABILITY));
        lines.push(format!(
            "  * score: {}",
            format_reliability_score(self.reliability_score)
        ));
        lines.push(format!(
            "  * internal_passes: {}",
            self.internal_passes
        ));
        lines.push(format!(
            "  * threshold_reached: {}",
            if self.threshold_reached { "yes" } else { "no" }
        ));

        if !self.reliability_factors.is_empty() {
            lines.push("  * factors:".to_string());
            for factor in &self.reliability_factors {
                lines.push(format!("    - {}", factor));
            }
        }

        if !self.reliability_risks.is_empty() {
            lines.push("  * main_risks:".to_string());
            for risk in &self.reliability_risks {
                lines.push(format!("    - {}", risk));
            }
        } else {
            lines.push("  * main_risks: none".to_string());
        }

        if !self.main_limitations.is_empty() {
            lines.push("  * main_limitations:".to_string());
            for limitation in &self.main_limitations {
                lines.push(format!("    - {}", limitation));
            }
        }

        // Next steps section (optional)
        if !self.next_steps.is_empty() {
            lines.push(String::new());
            lines.push(section_header(sections::NEXT_STEPS));
            for step in &self.next_steps {
                lines.push(bullet(step));
            }
        }

        lines.push(String::new());
        lines.push(SEPARATOR.to_string());

        lines.join("\n")
    }
}

/// Check if a string contains any emoji characters
pub fn contains_emoji(s: &str) -> bool {
    s.chars().any(|c| {
        let code = c as u32;
        // Common emoji ranges
        (0x1F600..=0x1F64F).contains(&code)  // Emoticons
            || (0x1F300..=0x1F5FF).contains(&code)  // Misc Symbols and Pictographs
            || (0x1F680..=0x1F6FF).contains(&code)  // Transport and Map
            || (0x1F1E0..=0x1F1FF).contains(&code)  // Flags
            || (0x2600..=0x26FF).contains(&code)    // Misc symbols
            || (0x2700..=0x27BF).contains(&code)    // Dingbats
            || (0xFE00..=0xFE0F).contains(&code)    // Variation Selectors
            || (0x1F900..=0x1F9FF).contains(&code)  // Supplemental Symbols
            || (0x1FA00..=0x1FA6F).contains(&code)  // Chess Symbols
            || (0x1FA70..=0x1FAFF).contains(&code)  // Symbols and Pictographs Extended-A
            || (0x231A..=0x231B).contains(&code)    // Watch, Hourglass
            || (0x23E9..=0x23F3).contains(&code)    // Various symbols
            || (0x23F8..=0x23FA).contains(&code)    // Various symbols
            || (0x25AA..=0x25AB).contains(&code)    // Squares
            || (0x25B6..=0x25C0).contains(&code)    // Triangles
            || (0x25FB..=0x25FE).contains(&code)    // Squares
            || (0x2614..=0x2615).contains(&code)    // Umbrella, Hot Beverage
            || (0x2648..=0x2653).contains(&code)    // Zodiac
            || (0x267F..=0x267F).contains(&code)    // Wheelchair
            || (0x2693..=0x2693).contains(&code)    // Anchor
            || (0x26A1..=0x26A1).contains(&code)    // High Voltage
            || (0x26AA..=0x26AB).contains(&code)    // Circles
            || (0x26BD..=0x26BE).contains(&code)    // Soccer, Baseball
            || (0x26C4..=0x26C5).contains(&code)    // Snowman, Sun
            || (0x26CE..=0x26CE).contains(&code)    // Ophiuchus
            || (0x26D4..=0x26D4).contains(&code)    // No Entry
            || (0x26EA..=0x26EA).contains(&code)    // Church
            || (0x26F2..=0x26F3).contains(&code)    // Fountain, Golf
            || (0x26F5..=0x26F5).contains(&code)    // Sailboat
            || (0x26FA..=0x26FA).contains(&code)    // Tent
            || (0x26FD..=0x26FD).contains(&code)    // Fuel Pump
            || (0x2702..=0x2702).contains(&code)    // Scissors
            || (0x2705..=0x2705).contains(&code)    // Check Mark
            || (0x2708..=0x270D).contains(&code)    // Airplane to Writing Hand
            || (0x270F..=0x270F).contains(&code)    // Pencil
            || (0x2712..=0x2712).contains(&code)    // Black Nib
            || (0x2714..=0x2714).contains(&code)    // Check Mark
            || (0x2716..=0x2716).contains(&code)    // X Mark
            || (0x271D..=0x271D).contains(&code)    // Latin Cross
            || (0x2721..=0x2721).contains(&code)    // Star of David
            || (0x2728..=0x2728).contains(&code)    // Sparkles
            || (0x2733..=0x2734).contains(&code)    // Eight Spoked Asterisk
            || (0x2744..=0x2744).contains(&code)    // Snowflake
            || (0x2747..=0x2747).contains(&code)    // Sparkle
            || (0x274C..=0x274C).contains(&code)    // Cross Mark
            || (0x274E..=0x274E).contains(&code)    // Cross Mark
            || (0x2753..=0x2755).contains(&code)    // Question Marks
            || (0x2757..=0x2757).contains(&code)    // Exclamation Mark
            || (0x2763..=0x2764).contains(&code)    // Heart Exclamation, Heart
            || (0x2795..=0x2797).contains(&code)    // Plus, Minus, Division
            || (0x27A1..=0x27A1).contains(&code)    // Right Arrow
            || (0x27B0..=0x27B0).contains(&code)    // Curly Loop
            || (0x27BF..=0x27BF).contains(&code)    // Double Curly Loop
            || (0x2934..=0x2935).contains(&code)    // Arrows
            || (0x2B05..=0x2B07).contains(&code)    // Arrows
            || (0x2B1B..=0x2B1C).contains(&code)    // Squares
            || (0x2B50..=0x2B50).contains(&code)    // Star
            || (0x2B55..=0x2B55).contains(&code)    // Circle
            || (0x3030..=0x3030).contains(&code)    // Wavy Dash
            || (0x303D..=0x303D).contains(&code)    // Part Alternation Mark
            || (0x3297..=0x3297).contains(&code)    // Circled Ideograph Congratulation
            || (0x3299..=0x3299).contains(&code)    // Circled Ideograph Secret
    })
}

/// Check if a string contains Unicode box-drawing characters
pub fn contains_unicode_box_chars(s: &str) -> bool {
    s.chars().any(|c| {
        let code = c as u32;
        // Box Drawing block: U+2500 to U+257F
        // Block Elements: U+2580 to U+259F
        (0x2500..=0x257F).contains(&code) || (0x2580..=0x259F).contains(&code)
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_reliability_color_from_score() {
        assert_eq!(ReliabilityColor::from_score(0.95), ReliabilityColor::Green);
        assert_eq!(ReliabilityColor::from_score(0.90), ReliabilityColor::Green);
        assert_eq!(ReliabilityColor::from_score(0.85), ReliabilityColor::Yellow);
        assert_eq!(ReliabilityColor::from_score(0.70), ReliabilityColor::Yellow);
        assert_eq!(ReliabilityColor::from_score(0.65), ReliabilityColor::Red);
        assert_eq!(ReliabilityColor::from_score(0.0), ReliabilityColor::Red);
    }

    #[test]
    fn test_verbosity_from_prompt() {
        assert_eq!(
            Verbosity::from_prompt("short answer only: how much RAM?"),
            Verbosity::Short
        );
        assert_eq!(
            Verbosity::from_prompt("give me a brief summary"),
            Verbosity::Short
        );
        assert_eq!(
            Verbosity::from_prompt("detailed report about storage"),
            Verbosity::Detailed
        );
        assert_eq!(
            Verbosity::from_prompt("full report on system health"),
            Verbosity::Detailed
        );
        assert_eq!(
            Verbosity::from_prompt("how much disk space do I have?"),
            Verbosity::Normal
        );
    }

    #[test]
    fn test_contains_emoji() {
        assert!(contains_emoji("Hello 👋"));
        assert!(contains_emoji("🚀 Launch"));
        assert!(contains_emoji("Check ✓"));
        assert!(!contains_emoji("Hello World"));
        assert!(!contains_emoji("ASCII only: *+-="));
        assert!(!contains_emoji("[SUMMARY]"));
    }

    #[test]
    fn test_contains_unicode_box_chars() {
        assert!(contains_unicode_box_chars("┌──────┐"));
        assert!(contains_unicode_box_chars("│ box │"));
        assert!(contains_unicode_box_chars("└──────┘"));
        assert!(contains_unicode_box_chars("▀▄█"));
        assert!(!contains_unicode_box_chars("+------+"));
        assert!(!contains_unicode_box_chars("|  box |"));
        assert!(!contains_unicode_box_chars("============"));
    }

    #[test]
    fn test_hyperlink_format() {
        let link = hyperlink("https://example.com", "Example");
        assert!(link.contains("https://example.com"));
        assert!(link.contains("Example"));
        assert!(link.starts_with("\x1b]8;;"));
    }

    #[test]
    fn test_report_builder_short() {
        let report = ReportBuilder::new()
            .with_verbosity(Verbosity::Short)
            .add_summary("RAM: 16 GB total, 8 GB used")
            .reliability(0.95)
            .build();

        assert!(report.contains("RAM: 16 GB"));
        assert!(report.contains("Reliability:"));
        assert!(!report.contains("[SUMMARY]"));
    }

    #[test]
    fn test_report_builder_detailed() {
        let report = ReportBuilder::new()
            .with_verbosity(Verbosity::Detailed)
            .title("Storage Report")
            .add_summary("3 disks detected")
            .add_detail("nvme0n1: 500GB SSD")
            .add_evidence("disk.lsblk", "Block device info", "fresh")
            .reliability(0.92)
            .passes(2, true)
            .build();

        assert!(report.contains("[SUMMARY]"));
        assert!(report.contains("[DETAILS]"));
        assert!(report.contains("[EVIDENCE]"));
        assert!(report.contains("[RELIABILITY]"));
        assert!(report.contains("internal_passes: 2"));
        assert!(report.contains("threshold_reached: yes"));
        assert!(report.contains(SEPARATOR));
    }

    #[test]
    fn test_report_no_emoji() {
        let report = ReportBuilder::new()
            .with_verbosity(Verbosity::Detailed)
            .title("Test Report")
            .add_summary("Test item")
            .reliability(0.85)
            .passes(1, false)
            .build();

        // Strip ANSI codes for emoji check
        let stripped = strip_ansi_codes(&report);
        assert!(!contains_emoji(&stripped));
    }

    #[test]
    fn test_report_no_unicode_box() {
        let report = ReportBuilder::new()
            .with_verbosity(Verbosity::Detailed)
            .title("Test Report")
            .add_summary("Test item")
            .reliability(0.85)
            .passes(1, false)
            .build();

        // Strip ANSI codes for box char check
        let stripped = strip_ansi_codes(&report);
        assert!(!contains_unicode_box_chars(&stripped));
    }

    /// Helper to strip ANSI escape codes
    fn strip_ansi_codes(s: &str) -> String {
        let re = regex::Regex::new(r"\x1b\[[0-9;]*m|\x1b\]8;;[^\x1b]*\x1b\\[^\x1b]*\x1b\]8;;\x1b\\")
            .unwrap();
        re.replace_all(s, "").to_string()
    }
}
