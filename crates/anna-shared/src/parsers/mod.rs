//! Probe output parsers for STRUCT-lite phase.
//!
//! This module provides deterministic parsers for common Linux command outputs.
//! All parsing uses integer arithmetic (no floats) and produces typed structs.
//!
//! # Supported Probes
//!
//! - `free -h`: Memory and swap information → `MemoryInfo`
//! - `df -h`: Disk usage per mount → `Vec<DiskUsage>`
//! - `systemctl --failed` / `systemctl is-active`: Service status → `Vec<ServiceStatus>` / `ServiceStatus`
//! - `lsblk`: Block device information → `Vec<BlockDevice>` (v0.0.22 STRUCT+)
//! - `lscpu`: CPU information → `CpuInfo` (v0.0.22 STRUCT+)
//!
//! # Design Principles
//!
//! - **No floats**: All sizes are stored as `u64` bytes.
//! - **Exact rounding**: Size parsing uses rational arithmetic with deterministic tie-breaking.
//! - **Binary prefixes**: K/M/G/T are treated as base-2 (1024), matching Linux tool defaults.
//! - **Explicit errors**: Parse failures return `ParsedProbeData::Error` with context.

pub mod atoms;
pub mod df;
pub mod free;
pub mod journalctl;
pub mod lsblk;
pub mod lscpu;
pub mod systemctl;

// Re-export main types
pub use atoms::{
    normalize_service_name, parse_display_size, parse_percent, parse_size, ParseError,
    ParseErrorReason,
};
pub use df::{find_by_mount, parse_df, resolve_mount_alias, DiskUsage};
pub use free::{parse_free, MemoryInfo};
pub use journalctl::{
    parse_journalctl_priority, parse_boot_time,
    JournalSummary, JournalTopItem, BootTimeInfo,
    FailedUnit as JournalFailedUnit, // Alias to avoid conflict with systemctl
    parse_failed_units as parse_journal_failed_units,
};
pub use lsblk::{parse_lsblk, find_root_device, total_disk_size, BlockDevice, BlockDeviceType};
pub use lscpu::{parse_lscpu, CpuInfo};
pub use systemctl::{
    parse_failed_units, parse_is_active, parse_status_verbose, ServiceState, ServiceStatus,
};

// v0.45.7: Tool/package evidence helper functions
/// Find tool existence evidence for a given tool name
pub fn find_tool_evidence<'a>(parsed: &'a [ParsedProbeData], name: &str) -> Option<&'a ToolExists> {
    parsed.iter()
        .filter_map(|p| p.as_tool())
        .find(|t| t.name.to_lowercase() == name.to_lowercase())
}

/// Find package installation evidence for a given package name
pub fn find_package_evidence<'a>(parsed: &'a [ParsedProbeData], name: &str) -> Option<&'a PackageInstalled> {
    parsed.iter()
        .filter_map(|p| p.as_package())
        .find(|p| p.name.to_lowercase() == name.to_lowercase())
}

/// Check if any tool/package evidence exists (positive or negative) for a name
pub fn has_evidence_for(parsed: &[ParsedProbeData], name: &str) -> bool {
    find_tool_evidence(parsed, name).is_some() || find_package_evidence(parsed, name).is_some()
}

/// Find audio devices evidence (v0.45.8, v0.0.56, v0.0.60 merged).
/// If multiple sources exist (lspci + pactl), merge them:
/// - Use lspci for hardware identity (PCI slot, device name)
/// - Use pactl for card names/profiles if lspci found nothing
/// Never return "No audio" if either source has devices.
/// v0.0.60: Improved merging with deduplication by (pci_slot, description).
pub fn find_audio_evidence(parsed: &[ParsedProbeData]) -> Option<AudioDevices> {
    let all_audio: Vec<&AudioDevices> = parsed.iter()
        .filter_map(|p| p.as_audio())
        .collect();

    if all_audio.is_empty() {
        return None;
    }

    // If only one source, return it
    if all_audio.len() == 1 {
        return Some(all_audio[0].clone());
    }

    // v0.0.60: Merge all sources with deduplication
    let lspci_audio = all_audio.iter().find(|a| a.source == "lspci");
    let pactl_audio = all_audio.iter().find(|a| a.source == "pactl");

    match (lspci_audio, pactl_audio) {
        (Some(lspci), Some(pactl)) => {
            // v0.0.60: Merge devices from both sources, deduplicate
            let merged = merge_audio_devices(&lspci.devices, &pactl.devices);
            if merged.is_empty() {
                // Both empty - return lspci (grounded negative evidence)
                Some(AudioDevices {
                    devices: vec![],
                    source: "lspci+pactl".to_string(),
                })
            } else {
                Some(AudioDevices {
                    devices: merged,
                    source: "lspci+pactl".to_string(),
                })
            }
        },
        (Some(lspci), None) => Some((*lspci).clone()),
        (None, Some(pactl)) => Some((*pactl).clone()),
        (None, None) => {
            // Unknown sources - return first with devices, or first
            all_audio.iter()
                .find(|a| !a.devices.is_empty())
                .or(all_audio.first())
                .map(|a| (*a).clone())
        }
    }
}

/// v0.0.60: Merge audio devices from lspci and pactl, deduplicating by description.
/// Prefers lspci devices (have PCI slot) over pactl (no PCI slot).
fn merge_audio_devices(lspci: &[AudioDevice], pactl: &[AudioDevice]) -> Vec<AudioDevice> {
    let mut merged: Vec<AudioDevice> = Vec::new();

    // Add all lspci devices first (preferred source)
    for dev in lspci {
        merged.push(dev.clone());
    }

    // Add pactl devices that aren't duplicates
    for pactl_dev in pactl {
        // Check if this pactl device is a duplicate of an lspci device
        // Compare by normalized description (case-insensitive, trim whitespace)
        let is_duplicate = merged.iter().any(|existing| {
            // Check if descriptions overlap (one contains the other)
            let existing_lower = existing.description.to_lowercase();
            let pactl_lower = pactl_dev.description.to_lowercase();
            existing_lower.contains(&pactl_lower) || pactl_lower.contains(&existing_lower)
        });

        if !is_duplicate {
            merged.push(pactl_dev.clone());
        }
    }

    merged
}

/// Find audio devices evidence returning a reference (v0.45.8 legacy)
pub fn find_audio_evidence_ref(parsed: &[ParsedProbeData]) -> Option<&AudioDevices> {
    parsed.iter()
        .filter_map(|p| p.as_audio())
        .next()
}

/// Get all tool evidence from parsed probes (v0.45.8, v0.0.56 fix).
/// Returns both positive (exists=true) and negative (exists=false) evidence.
/// Caller should filter by `.exists` if only installed tools are needed.
pub fn get_installed_tools(parsed: &[ParsedProbeData]) -> Vec<&ToolExists> {
    parsed.iter()
        .filter_map(|p| p.as_tool())
        .collect()
}

/// v0.0.59: Extract installed editor names from parsed probe evidence.
/// Only returns editors that exist (exists=true) in current probe results.
/// Maps tool names to canonical editor identifiers.
/// Returns sorted, deduplicated list for stable output.
pub fn installed_editors_from_parsed(parsed: &[ParsedProbeData]) -> Vec<String> {
    // Supported editor mappings: tool_name -> canonical_name
    const EDITOR_MAP: &[(&str, &str)] = &[
        ("vim", "vim"),
        ("nvim", "nvim"),
        ("nano", "nano"),
        ("emacs", "emacs"),
        ("micro", "micro"),
        ("hx", "helix"),
        ("helix", "helix"),
        ("code", "code"),
        ("kate", "kate"),
        ("gedit", "gedit"),
    ];

    let tools = get_installed_tools(parsed);
    let mut editors: Vec<String> = tools.iter()
        .filter(|t| t.exists)
        .filter_map(|t| {
            EDITOR_MAP.iter()
                .find(|(tool, _)| *tool == t.name.as_str())
                .map(|(_, canonical)| canonical.to_string())
        })
        .collect();

    // Deduplicate (in case hx and helix both map to helix)
    editors.sort();
    editors.dedup();
    editors
}

use crate::rpc::ProbeResult;

/// Count how many probes produced valid evidence (v0.0.56).
/// A probe produces valid evidence if parse_probe_result returns is_valid_evidence().
/// This is used for reliability scoring - exit_code=1 for tool checks is valid negative evidence!
pub fn count_valid_evidence_probes(probes: &[ProbeResult]) -> usize {
    probes.iter()
        .filter(|p| parse_probe_result(p).is_valid_evidence())
        .count()
}

/// Check if a probe result produced valid evidence (v0.0.56).
/// Tool/package probes with exit_code=1 are VALID negative evidence, not failures!
pub fn is_probe_valid_evidence(probe: &ProbeResult) -> bool {
    parse_probe_result(probe).is_valid_evidence()
}
use serde::{Deserialize, Serialize};

/// Method used to check tool existence (v0.45.7)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ToolExistsMethod {
    /// `command -v <name>` (POSIX)
    CommandV,
    /// `which <name>` (less portable)
    Which,
    /// `type <name>` (bash builtin)
    Type,
}

/// Tool existence evidence (v0.45.7)
/// Note: exit code 1 is VALID NEGATIVE EVIDENCE, not an error!
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ToolExists {
    /// Name of the tool/binary
    pub name: String,
    /// Whether the tool exists (false = valid negative evidence)
    pub exists: bool,
    /// Method used to check
    pub method: ToolExistsMethod,
    /// Path if found (from stdout)
    pub path: Option<String>,
}

/// Package installation evidence (v0.45.7)
/// Note: exit code 1 is VALID NEGATIVE EVIDENCE, not an error!
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PackageInstalled {
    /// Name of the package
    pub name: String,
    /// Whether the package is installed (false = valid negative evidence)
    pub installed: bool,
    /// Version if installed
    pub version: Option<String>,
}

/// Audio device from lspci or pactl (v0.45.8)
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AudioDevice {
    /// Device description (e.g., "Intel Corporation Cannon Lake PCH cAVS")
    pub description: String,
    /// PCI slot if from lspci (e.g., "00:1f.3")
    pub pci_slot: Option<String>,
    /// Vendor name extracted from description
    pub vendor: Option<String>,
}

/// Audio devices evidence (v0.45.8)
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AudioDevices {
    /// List of detected audio devices
    pub devices: Vec<AudioDevice>,
    /// Source of the information (lspci, pactl)
    pub source: String,
}

/// Parsed probe data or error.
/// Used internally for enrichment; not serialized over the wire.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ParsedProbeData {
    /// Memory info from `free -h`
    Memory(MemoryInfo),
    /// Disk usage from `df -h`
    Disk(Vec<DiskUsage>),
    /// Service status list (from `systemctl --failed` or similar)
    Services(Vec<ServiceStatus>),
    /// Single service status (from `systemctl is-active`)
    Service(ServiceStatus),
    /// Block devices from `lsblk` (v0.0.22 STRUCT+)
    BlockDevices(Vec<BlockDevice>),
    /// CPU info from `lscpu` (v0.0.22 STRUCT+)
    Cpu(CpuInfo),
    /// Journal errors from `journalctl -p 3` (v0.0.35)
    JournalErrors(JournalSummary),
    /// Journal warnings from `journalctl -p 4` (v0.0.35)
    JournalWarnings(JournalSummary),
    /// Boot time from `systemd-analyze` (v0.0.35)
    BootTime(BootTimeInfo),
    /// Tool existence check (v0.45.7) - exit 1 = valid negative evidence
    Tool(ToolExists),
    /// Package installation check (v0.45.7) - exit 1 = valid negative evidence
    Package(PackageInstalled),
    /// Audio devices from lspci/pactl (v0.45.8)
    Audio(AudioDevices),
    /// Parse error with diagnostic context
    Error(ParseError),
    /// Probe type not supported for structured parsing
    Unsupported,
}

impl ParsedProbeData {
    /// Check if this is an error variant.
    pub fn is_error(&self) -> bool {
        matches!(self, Self::Error(_))
    }

    /// Get the error if this is an error variant.
    pub fn as_error(&self) -> Option<&ParseError> {
        match self {
            Self::Error(e) => Some(e),
            _ => None,
        }
    }

    /// Get memory info if this is the Memory variant.
    pub fn as_memory(&self) -> Option<&MemoryInfo> {
        match self {
            Self::Memory(m) => Some(m),
            _ => None,
        }
    }

    /// Get disk usage if this is the Disk variant.
    pub fn as_disk(&self) -> Option<&Vec<DiskUsage>> {
        match self {
            Self::Disk(d) => Some(d),
            _ => None,
        }
    }

    /// Get services if this is the Services variant.
    pub fn as_services(&self) -> Option<&Vec<ServiceStatus>> {
        match self {
            Self::Services(s) => Some(s),
            _ => None,
        }
    }

    /// Get single service if this is the Service variant.
    pub fn as_service(&self) -> Option<&ServiceStatus> {
        match self {
            Self::Service(s) => Some(s),
            _ => None,
        }
    }

    /// Get block devices if this is the BlockDevices variant.
    pub fn as_block_devices(&self) -> Option<&Vec<BlockDevice>> {
        match self {
            Self::BlockDevices(b) => Some(b),
            _ => None,
        }
    }

    /// Get CPU info if this is the Cpu variant.
    pub fn as_cpu(&self) -> Option<&CpuInfo> {
        match self {
            Self::Cpu(c) => Some(c),
            _ => None,
        }
    }

    /// Get tool existence if this is the Tool variant (v0.45.7).
    pub fn as_tool(&self) -> Option<&ToolExists> {
        match self {
            Self::Tool(t) => Some(t),
            _ => None,
        }
    }

    /// Get package installation if this is the Package variant (v0.45.7).
    pub fn as_package(&self) -> Option<&PackageInstalled> {
        match self {
            Self::Package(p) => Some(p),
            _ => None,
        }
    }

    /// Get audio devices if this is the Audio variant (v0.45.8).
    pub fn as_audio(&self) -> Option<&AudioDevices> {
        match self {
            Self::Audio(a) => Some(a),
            _ => None,
        }
    }

    /// Check if this represents valid evidence (not error/unsupported) (v0.45.7).
    pub fn is_valid_evidence(&self) -> bool {
        !matches!(self, Self::Error(_) | Self::Unsupported)
    }
}

/// Probe ID constants for matching.
pub mod probe_ids {
    pub const FREE: &str = "free -h";
    pub const DF: &str = "df -h";
    pub const SYSTEMCTL_FAILED: &str = "systemctl --failed";
    pub const TOP_MEMORY: &str = "ps aux --sort=-%mem";
    pub const TOP_CPU: &str = "ps aux --sort=-%cpu";
    pub const LSBLK: &str = "lsblk";
    pub const LSCPU: &str = "lscpu";
}

/// Parse a ProbeResult into structured data.
/// Returns `ParsedProbeData::Unsupported` for probes we don't have parsers for.
/// v0.45.7: Tool/package probes with exit code 1 are VALID NEGATIVE EVIDENCE, not errors!
/// v0.45.8: Audio probes from lspci/pactl are now parsed.
pub fn parse_probe_result(probe: &ProbeResult) -> ParsedProbeData {
    let cmd_lower = probe.command.to_lowercase();

    // v0.45.7: Handle tool existence probes - exit 1 = tool not found (valid evidence!)
    if let Some(parsed) = try_parse_tool_exists(probe, &cmd_lower) {
        return parsed;
    }

    // v0.45.7: Handle package probes - exit 1 = package not installed (valid evidence!)
    if let Some(parsed) = try_parse_package_installed(probe, &cmd_lower) {
        return parsed;
    }

    // v0.45.8: Handle audio probes - lspci audio and pactl
    if let Some(parsed) = try_parse_audio_devices(probe, &cmd_lower) {
        return parsed;
    }

    // For other probes, non-zero exit code is an error
    if probe.exit_code != 0 {
        return ParsedProbeData::Error(ParseError::new(
            &probe.command,
            ParseErrorReason::MissingSection(format!("exit code {}", probe.exit_code)),
            &probe.stderr,
        ));
    }

    parse_probe_output(&probe.command, &probe.stdout)
}

/// Try to parse a tool existence probe (v0.45.7, v0.0.57 hardening).
/// Handles `command -v`, `which`, and `type` commands.
/// Returns Some if this is a tool check probe, None otherwise.
///
/// v0.0.57: exit_code=127 ("command not found") is an ERROR, not valid evidence.
/// Only exit_code=0 (found) and exit_code=1 (not found) are valid evidence.
fn try_parse_tool_exists(probe: &ProbeResult, cmd_lower: &str) -> Option<ParsedProbeData> {
    // Pattern: "command -v <name>" or "sh -lc 'command -v <name>'"
    if cmd_lower.contains("command -v") {
        // v0.0.57: exit_code=127 means the shell itself failed - this is an error
        if probe.exit_code == 127 {
            return Some(ParsedProbeData::Error(ParseError::new(
                &probe.command,
                ParseErrorReason::MissingSection("shell error: command not found".to_string()),
                &probe.stderr,
            )));
        }

        let name = extract_tool_name_from_command_v(&probe.command);
        let exists = probe.exit_code == 0;
        let path = if exists && !probe.stdout.trim().is_empty() {
            Some(probe.stdout.trim().to_string())
        } else {
            None
        };
        return Some(ParsedProbeData::Tool(ToolExists {
            name,
            exists,
            method: ToolExistsMethod::CommandV,
            path,
        }));
    }

    // Pattern: "which <name>"
    if cmd_lower.starts_with("which ") {
        // v0.0.57: exit_code=127 means the shell itself failed - this is an error
        if probe.exit_code == 127 {
            return Some(ParsedProbeData::Error(ParseError::new(
                &probe.command,
                ParseErrorReason::MissingSection("shell error: command not found".to_string()),
                &probe.stderr,
            )));
        }

        let name = probe.command.split_whitespace().nth(1)
            .unwrap_or("unknown").to_string();
        let exists = probe.exit_code == 0;
        let path = if exists && !probe.stdout.trim().is_empty() {
            Some(probe.stdout.trim().to_string())
        } else {
            None
        };
        return Some(ParsedProbeData::Tool(ToolExists {
            name,
            exists,
            method: ToolExistsMethod::Which,
            path,
        }));
    }

    None
}

/// Try to parse a package installation probe (v0.45.7).
/// Handles `pacman -Q` commands.
fn try_parse_package_installed(probe: &ProbeResult, cmd_lower: &str) -> Option<ParsedProbeData> {
    // Pattern: "pacman -Q <name>" or "pacman -Q <name> 2>/dev/null"
    // Note: cmd_lower is already lowercase, so we check for lowercase -q
    if cmd_lower.contains("pacman -q") {
        let name = extract_package_name_from_pacman(&probe.command);
        let installed = probe.exit_code == 0;
        let version = if installed {
            // pacman -Q outputs: "<name> <version>"
            probe.stdout.split_whitespace().nth(1).map(|v| v.to_string())
        } else {
            None
        };
        return Some(ParsedProbeData::Package(PackageInstalled {
            name,
            installed,
            version,
        }));
    }

    None
}

/// v0.0.60, v0.0.61: Check if a command is an lspci audio probe.
/// Matches:
/// - "lspci | grep -i audio"
/// - "lspci_audio" probe ID
/// - Raw lspci output when context suggests audio
/// v0.0.61: Also check stdout for audio controller patterns
fn is_lspci_audio_command(cmd: &str) -> bool {
    let cmd_lower = cmd.to_lowercase();
    // Explicit audio grep pattern
    if cmd_lower.contains("lspci") && cmd_lower.contains("audio") {
        return true;
    }
    // Probe ID form
    if cmd_lower == "lspci_audio" {
        return true;
    }
    false
}

/// v0.0.61: Check if stdout contains lspci audio device output.
/// This catches cases where the command doesn't match but output is clearly lspci audio.
fn stdout_contains_audio_device(stdout: &str) -> bool {
    let lower = stdout.to_lowercase();
    // Check for common lspci audio device class patterns
    // Note: lspci may show "[0403]" PCI class code between name and colon
    lower.contains("audio device") ||
    lower.contains("multimedia audio controller") ||
    lower.contains("audio controller") ||
    (lower.contains("multimedia controller") && lower.contains("audio"))
}

/// Try to parse audio devices from lspci or pactl (v0.45.8, v0.0.60, v0.0.61 expanded).
/// Handles `lspci | grep -i audio` and `pactl list cards` commands.
/// v0.0.60: Improved to handle more lspci variants and grep exit codes.
/// v0.0.61: Also detect audio device output in stdout (fallback for command mismatch).
fn try_parse_audio_devices(probe: &ProbeResult, cmd_lower: &str) -> Option<ParsedProbeData> {
    // v0.0.61: First check if stdout contains audio device output
    // This catches cases where command pattern doesn't match exactly
    let has_lspci_audio_output = stdout_contains_audio_device(&probe.stdout);

    // Pattern: lspci audio probe (by command or by output content)
    if is_lspci_audio_command(&probe.command) || has_lspci_audio_output {
        // v0.0.60: Handle grep exit codes correctly
        // exit_code 0 = matches found (devices present)
        // exit_code 1 = no matches (valid empty evidence for grep)
        // exit_code 2+ = grep error

        // v0.0.61: If we detected audio output, always try to parse it
        if has_lspci_audio_output && probe.exit_code == 0 {
            let devices = parse_lspci_audio_output(&probe.stdout);
            if !devices.is_empty() {
                return Some(ParsedProbeData::Audio(AudioDevices {
                    devices,
                    source: "lspci".to_string(),
                }));
            }
        }

        if probe.exit_code == 1 && probe.stdout.trim().is_empty() {
            // grep found no matches - valid negative evidence
            return Some(ParsedProbeData::Audio(AudioDevices {
                devices: vec![],
                source: "lspci".to_string(),
            }));
        }

        if probe.exit_code != 0 && probe.exit_code != 1 {
            // Real error (exit code 2+)
            return Some(ParsedProbeData::Audio(AudioDevices {
                devices: vec![],
                source: "lspci".to_string(),
            }));
        }

        let devices = parse_lspci_audio_output(&probe.stdout);
        return Some(ParsedProbeData::Audio(AudioDevices {
            devices,
            source: "lspci".to_string(),
        }));
    }

    // Pattern: "pactl list cards"
    if cmd_lower.contains("pactl") && cmd_lower.contains("cards") {
        // pactl may return empty or error if no pulseaudio - still valid evidence
        if probe.exit_code != 0 || probe.stdout.trim().is_empty() {
            return Some(ParsedProbeData::Audio(AudioDevices {
                devices: vec![],
                source: "pactl".to_string(),
            }));
        }

        let devices = parse_pactl_cards_output(&probe.stdout);
        return Some(ParsedProbeData::Audio(AudioDevices {
            devices,
            source: "pactl".to_string(),
        }));
    }

    // v0.0.61: Also detect pactl output by content (Card # blocks)
    if probe.stdout.contains("Card #") && probe.exit_code == 0 {
        let devices = parse_pactl_cards_output(&probe.stdout);
        return Some(ParsedProbeData::Audio(AudioDevices {
            devices,
            source: "pactl".to_string(),
        }));
    }

    None
}

/// Parse lspci audio output (v0.0.55 fix: handles PCI class codes in brackets).
/// Handles multiple lspci formats:
/// - "00:1f.3 Audio device: Intel Corporation Cannon Lake PCH cAVS (rev 10)"
/// - "00:1f.3 Multimedia audio controller: Intel Corporation Cannon Lake PCH cAVS"
/// - "00:1f.3 Audio controller [0403]: Some Device" (with PCI class code)
/// - "00:1f.3 Multimedia audio controller [0403]: Intel Corporation..." (common format)
/// v0.0.55: Fixed to handle PCI class codes like [0403] between device class and colon.
fn parse_lspci_audio_output(stdout: &str) -> Vec<AudioDevice> {
    let mut devices = Vec::new();

    for line in stdout.lines() {
        let line = line.trim();
        if line.is_empty() {
            continue;
        }

        let line_lower = line.to_lowercase();

        // v0.0.55: Check for audio-related device class markers (case-insensitive)
        // Note: The colon check is relaxed because PCI class codes [XXXX] may appear before colon
        let is_audio_line = line_lower.contains("audio device")
            || line_lower.contains("multimedia audio controller")
            || line_lower.contains("audio controller")
            || (line_lower.contains("multimedia controller") && line_lower.contains("audio"));

        // v0.0.55: If the line contains "audio" and has PCI slot format, trust it
        // This catches output from `lspci | grep -i audio`
        let has_pci_slot = line.len() > 7 && line.chars().nth(2) == Some(':');
        let is_grep_match = line_lower.contains("audio") && has_pci_slot;

        if !is_audio_line && !is_grep_match {
            continue;
        }

        // Parse format: "XX:XX.X <device_class> [XXXX]: <description>"
        let pci_slot = extract_pci_slot(line);

        // v0.0.55: Extract description after device class marker (handles [XXXX] codes)
        let description = extract_lspci_description_v055(line);

        // Extract vendor from description (usually first word)
        let vendor = extract_vendor_from_description(&description);

        if !description.is_empty() {
            devices.push(AudioDevice {
                description,
                pci_slot,
                vendor,
            });
        }
    }

    devices
}

/// v0.0.60: Extract PCI slot from lspci line.
/// Expects format like "00:1f.3" at the beginning of the line.
fn extract_pci_slot(line: &str) -> Option<String> {
    let first_token = line.split_whitespace().next()?;
    // PCI slot format: XX:XX.X (e.g., 00:1f.3)
    if first_token.contains(':') && first_token.contains('.') {
        Some(first_token.to_string())
    } else {
        None
    }
}

/// v0.0.55: Extract description from lspci line, handling PCI class codes [XXXX].
/// Handles formats like:
/// - "00:1f.3 Audio device: Intel..."
/// - "00:1f.3 Multimedia audio controller [0403]: Intel..."
fn extract_lspci_description_v055(line: &str) -> String {
    // v0.0.55: Find the LAST colon before the description
    // This handles PCI class codes like [0403] that appear before the colon
    // Format: "XX:XX.X Device Type [XXXX]: Description"

    // Skip the PCI slot colon (first colon at position ~2)
    let after_slot = if line.len() > 8 && line.chars().nth(2) == Some(':') {
        &line[7..] // Skip "XX:XX.X"
    } else {
        line
    };

    // Find the colon that separates device class from description
    if let Some(colon_pos) = after_slot.find(':') {
        let description = after_slot[colon_pos + 1..].trim();
        if !description.is_empty() {
            return description.to_string();
        }
    }

    // Fallback: try to extract after common device class patterns
    let patterns = [
        "audio device",
        "multimedia audio controller",
        "audio controller",
    ];

    let line_lower = line.to_lowercase();
    for pattern in patterns {
        if let Some(pos) = line_lower.find(pattern) {
            let after_pattern = &line[pos + pattern.len()..];
            // Skip any [XXXX] class code and colon
            if let Some(colon_pos) = after_pattern.find(':') {
                let desc = after_pattern[colon_pos + 1..].trim();
                if !desc.is_empty() {
                    return desc.to_string();
                }
            }
        }
    }

    // Last resort: return the whole line (minus PCI slot)
    if line.len() > 8 {
        line[8..].trim().to_string()
    } else {
        line.to_string()
    }
}

/// v0.0.58, v0.0.60: Extract description from lspci line after the device class.
/// Deprecated: Use extract_lspci_description_v055 instead.
#[allow(dead_code)]
fn extract_lspci_description(line: &str) -> String {
    extract_lspci_description_v055(line)
}

/// Parse pactl list cards output (v0.45.8, v0.0.60 expanded).
/// v0.0.60: Also looks for driver, card.name, and other properties.
fn parse_pactl_cards_output(stdout: &str) -> Vec<AudioDevice> {
    let mut devices = Vec::new();
    let mut current_card_name: Option<String> = None;
    let mut current_card_description: Option<String> = None;
    let mut in_card_block = false;

    for line in stdout.lines() {
        let line = line.trim();

        // Detect card block start
        if line.starts_with("Card #") {
            // Save previous card if any
            if in_card_block {
                if let Some(desc) = current_card_description.take().or(current_card_name.take()) {
                    let vendor = extract_vendor_from_description(&desc);
                    devices.push(AudioDevice {
                        description: desc,
                        pci_slot: None,
                        vendor,
                    });
                }
            }
            in_card_block = true;
            current_card_name = None;
            current_card_description = None;
        }

        // Look for "Name:" lines
        if line.starts_with("Name:") {
            current_card_name = Some(line.trim_start_matches("Name:").trim().to_string());
        }
        // Look for card description properties
        else if line.contains("alsa.card_name") || line.contains("device.description")
            || line.contains("card.name") || line.contains("device.product.name")
        {
            if let Some(pos) = line.find('=') {
                let value = line[pos + 1..].trim().trim_matches('"').to_string();
                if !value.is_empty() && current_card_description.is_none() {
                    current_card_description = Some(value);
                }
            }
        }
    }

    // Save last card
    if in_card_block {
        if let Some(desc) = current_card_description.take().or(current_card_name.take()) {
            let vendor = extract_vendor_from_description(&desc);
            devices.push(AudioDevice {
                description: desc,
                pci_slot: None,
                vendor,
            });
        }
    }

    // Fallback: if we found a name but no description anywhere
    if devices.is_empty() {
        if let Some(name) = current_card_name {
            devices.push(AudioDevice {
                description: name,
                pci_slot: None,
                vendor: None,
            });
        }
    }

    devices
}

/// Extract vendor name from audio device description.
fn extract_vendor_from_description(description: &str) -> Option<String> {
    let known_vendors = [
        "Intel", "NVIDIA", "AMD", "Realtek", "Creative", "C-Media",
        "VIA", "SoundBlaster", "Logitech", "Corsair", "HyperX",
    ];

    for vendor in known_vendors {
        if description.to_lowercase().contains(&vendor.to_lowercase()) {
            return Some(vendor.to_string());
        }
    }

    // Try to extract first word if it looks like a vendor (capitalized)
    let first_word = description.split_whitespace().next()?;
    if first_word.chars().next()?.is_uppercase() && first_word.len() > 2 {
        return Some(first_word.to_string());
    }

    None
}

/// Extract tool name from "command -v <name>" or "sh -lc 'command -v <name>'"
fn extract_tool_name_from_command_v(cmd: &str) -> String {
    // Handle: sh -lc 'command -v nano'
    if let Some(pos) = cmd.find("command -v") {
        let rest = &cmd[pos + "command -v".len()..];
        let trimmed = rest.trim();
        // Extract the tool name (first alphanumeric word, stop at quotes/pipes)
        let name: String = trimmed.chars()
            .take_while(|c| c.is_alphanumeric() || *c == '-' || *c == '_')
            .collect();
        if !name.is_empty() {
            return name;
        }
    }
    "unknown".to_string()
}

/// Extract package name from "pacman -Q <name>" command
fn extract_package_name_from_pacman(cmd: &str) -> String {
    // Find -Q or -Qi and take the next word
    let cmd_lower = cmd.to_lowercase();
    for pattern in ["-q ", "-qi "] {
        if let Some(pos) = cmd_lower.find(pattern) {
            let rest = &cmd[pos + pattern.len()..];
            let trimmed = rest.trim();
            // Extract package name (stop at whitespace or redirection)
            let name: String = trimmed.chars()
                .take_while(|c| c.is_alphanumeric() || *c == '-' || *c == '_' || *c == '.')
                .collect();
            if !name.is_empty() {
                return name;
            }
        }
    }
    "unknown".to_string()
}

/// Parse probe output based on the command.
/// Returns `ParsedProbeData::Unsupported` for probes we don't have parsers for.
pub fn parse_probe_output(command: &str, stdout: &str) -> ParsedProbeData {
    // Normalize command for matching
    let cmd_lower = command.to_lowercase();

    if cmd_lower.starts_with("free") {
        match parse_free(command, stdout) {
            Ok(info) => ParsedProbeData::Memory(info),
            Err(e) => ParsedProbeData::Error(e),
        }
    } else if cmd_lower.starts_with("df") {
        match parse_df(command, stdout) {
            Ok(entries) => ParsedProbeData::Disk(entries),
            Err(e) => ParsedProbeData::Error(e),
        }
    } else if cmd_lower.starts_with("lsblk") {
        match parse_lsblk(command, stdout) {
            Ok(devices) => ParsedProbeData::BlockDevices(devices),
            Err(e) => ParsedProbeData::Error(e),
        }
    } else if cmd_lower.starts_with("lscpu") {
        match parse_lscpu(command, stdout) {
            Ok(info) => ParsedProbeData::Cpu(info),
            Err(e) => ParsedProbeData::Error(e),
        }
    } else if cmd_lower.starts_with("journalctl -p 3") {
        // v0.0.35: Journal errors (priority 3 = err)
        ParsedProbeData::JournalErrors(parse_journalctl_priority(stdout))
    } else if cmd_lower.starts_with("journalctl -p 4") {
        // v0.0.35: Journal warnings (priority 4 = warning)
        ParsedProbeData::JournalWarnings(parse_journalctl_priority(stdout))
    } else if cmd_lower.starts_with("systemd-analyze") {
        // v0.0.35: Boot time
        ParsedProbeData::BootTime(parse_boot_time(stdout))
    } else if cmd_lower.contains("systemctl") && cmd_lower.contains("--failed") {
        match parse_failed_units(command, stdout) {
            Ok(units) => ParsedProbeData::Services(units),
            Err(e) => ParsedProbeData::Error(e),
        }
    } else if cmd_lower.contains("systemctl") && cmd_lower.contains("is-active") {
        // Extract service name from command
        let service_name = extract_service_from_is_active(&cmd_lower);
        match parse_is_active(command, &service_name, stdout) {
            Ok(status) => ParsedProbeData::Service(status),
            Err(e) => ParsedProbeData::Error(e),
        }
    } else {
        ParsedProbeData::Unsupported
    }
}

/// Extract service name from "systemctl is-active <service>" command.
fn extract_service_from_is_active(cmd: &str) -> String {
    // Find "is-active" and take the next word
    if let Some(pos) = cmd.find("is-active") {
        let rest = &cmd[pos + "is-active".len()..];
        let trimmed = rest.trim();
        if let Some(name) = trimmed.split_whitespace().next() {
            return name.to_string();
        }
    }
    "unknown".to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_probe_output_free() {
        let output = r#"              total        used        free      shared  buff/cache   available
Mem:           15Gi       8.2Gi       1.5Gi       512Mi       5.8Gi       6.5Gi
Swap:         4.0Gi       256Mi       3.8Gi
"#;
        let result = parse_probe_output("free -h", output);
        assert!(matches!(result, ParsedProbeData::Memory(_)));
    }

    #[test]
    fn test_parse_probe_output_df() {
        let output = r#"Filesystem      Size  Used Avail Use% Mounted on
/dev/sda1        50G   35G   12G  75% /
"#;
        let result = parse_probe_output("df -h", output);
        assert!(matches!(result, ParsedProbeData::Disk(_)));
    }

    #[test]
    fn test_parse_probe_output_systemctl_failed() {
        let output = r#"  UNIT LOAD ACTIVE SUB DESCRIPTION
0 loaded units listed.
"#;
        let result = parse_probe_output("systemctl --failed", output);
        assert!(matches!(result, ParsedProbeData::Services(_)));
    }

    #[test]
    fn test_parse_probe_output_systemctl_is_active() {
        let result = parse_probe_output("systemctl is-active nginx", "active\n");
        assert!(matches!(result, ParsedProbeData::Service(_)));
        if let ParsedProbeData::Service(s) = result {
            assert_eq!(s.name, "nginx.service");
            assert_eq!(s.state, ServiceState::Active);
        }
    }

    #[test]
    fn test_parse_probe_output_unsupported() {
        let result = parse_probe_output("ps aux --sort=-%mem", "some output");
        assert!(matches!(result, ParsedProbeData::Unsupported));
    }

    #[test]
    fn test_parse_probe_result() {
        let probe = ProbeResult {
            command: "free -h".to_string(),
            exit_code: 0,
            stdout: r#"              total        used        free      shared  buff/cache   available
Mem:           15Gi       8.2Gi       1.5Gi       512Mi       5.8Gi       6.5Gi
Swap:         4.0Gi       256Mi       3.8Gi
"#
            .to_string(),
            stderr: String::new(),
            timing_ms: 10,
        };
        let result = parse_probe_result(&probe);
        assert!(matches!(result, ParsedProbeData::Memory(_)));
    }

    #[test]
    fn test_parse_probe_result_failed() {
        let probe = ProbeResult {
            command: "free -h".to_string(),
            exit_code: 1,
            stdout: String::new(),
            stderr: "command not found".to_string(),
            timing_ms: 10,
        };
        let result = parse_probe_result(&probe);
        assert!(result.is_error());
    }

    #[test]
    fn test_parsed_probe_data_accessors() {
        let mem = ParsedProbeData::Memory(MemoryInfo {
            total_bytes: 1024,
            used_bytes: 512,
            free_bytes: 256,
            shared_bytes: 0,
            buff_cache_bytes: 256,
            available_bytes: 512,
            swap_total_bytes: None,
            swap_used_bytes: None,
            swap_free_bytes: None,
        });
        assert!(mem.as_memory().is_some());
        assert!(mem.as_disk().is_none());
        assert!(!mem.is_error());

        let err = ParsedProbeData::Error(ParseError::new(
            "test",
            ParseErrorReason::MalformedRow,
            "bad",
        ));
        assert!(err.is_error());
        assert!(err.as_error().is_some());
    }

    #[test]
    fn test_parse_probe_output_lsblk() {
        let output = r#"NAME        MAJ:MIN RM   SIZE RO TYPE MOUNTPOINTS
nvme0n1     259:0    0 953.9G  0 disk
├─nvme0n1p1 259:1    0   100M  0 part
└─nvme0n1p6 259:6    0 802.1G  0 part /
"#;
        let result = parse_probe_output("lsblk", output);
        assert!(matches!(result, ParsedProbeData::BlockDevices(_)));
        if let ParsedProbeData::BlockDevices(devices) = result {
            assert!(!devices.is_empty());
            assert_eq!(devices[0].name, "nvme0n1");
        }
    }

    #[test]
    fn test_parse_probe_output_lscpu() {
        let output = r#"Architecture: x86_64
CPU(s): 8
Model name: Intel Core i7
"#;
        let result = parse_probe_output("lscpu", output);
        assert!(matches!(result, ParsedProbeData::Cpu(_)));
        if let ParsedProbeData::Cpu(info) = result {
            assert_eq!(info.architecture, "x86_64");
            assert_eq!(info.cpu_count, 8);
        }
    }

    #[test]
    fn test_parsed_probe_data_accessors_block_devices() {
        let devices = ParsedProbeData::BlockDevices(vec![BlockDevice {
            name: "sda".to_string(),
            size_bytes: 1024,
            device_type: BlockDeviceType::Disk,
            mountpoints: vec![],
            parent: None,
            read_only: false,
        }]);
        assert!(devices.as_block_devices().is_some());
        assert!(devices.as_cpu().is_none());
    }

    #[test]
    fn test_parsed_probe_data_accessors_cpu() {
        let cpu = ParsedProbeData::Cpu(CpuInfo {
            architecture: "x86_64".to_string(),
            model_name: "Test".to_string(),
            cpu_count: 4,
            ..Default::default()
        });
        assert!(cpu.as_cpu().is_some());
        assert!(cpu.as_block_devices().is_none());
    }

    // v0.45.7: Tool existence evidence tests
    #[test]
    fn test_tool_exists_positive_evidence() {
        let probe = ProbeResult {
            command: "sh -lc 'command -v nano'".to_string(),
            exit_code: 0,
            stdout: "/usr/bin/nano\n".to_string(),
            stderr: String::new(),
            timing_ms: 5,
        };
        let result = parse_probe_result(&probe);
        assert!(matches!(result, ParsedProbeData::Tool(_)));
        if let ParsedProbeData::Tool(ref t) = result {
            assert_eq!(t.name, "nano");
            assert!(t.exists);
            assert_eq!(t.method, ToolExistsMethod::CommandV);
            assert_eq!(t.path, Some("/usr/bin/nano".to_string()));
        }
    }

    #[test]
    fn test_tool_exists_negative_evidence() {
        // v0.45.7: exit code 1 is VALID NEGATIVE EVIDENCE, not an error!
        let probe = ProbeResult {
            command: "sh -lc 'command -v nano'".to_string(),
            exit_code: 1,
            stdout: String::new(),
            stderr: String::new(),
            timing_ms: 5,
        };
        let result = parse_probe_result(&probe);
        assert!(matches!(result, ParsedProbeData::Tool(_)));
        if let ParsedProbeData::Tool(ref t) = result {
            assert_eq!(t.name, "nano");
            assert!(!t.exists); // Negative evidence!
            assert!(t.path.is_none());
        }
        // Must NOT be an error
        assert!(!result.is_error());
        assert!(result.is_valid_evidence());
    }

    #[test]
    fn test_package_installed_positive_evidence() {
        let probe = ProbeResult {
            command: "pacman -Q nano 2>/dev/null".to_string(),
            exit_code: 0,
            stdout: "nano 7.2-1\n".to_string(),
            stderr: String::new(),
            timing_ms: 10,
        };
        let result = parse_probe_result(&probe);
        assert!(matches!(result, ParsedProbeData::Package(_)));
        if let ParsedProbeData::Package(ref p) = result {
            assert_eq!(p.name, "nano");
            assert!(p.installed);
            assert_eq!(p.version, Some("7.2-1".to_string()));
        }
    }

    #[test]
    fn test_package_installed_negative_evidence() {
        // v0.45.7: exit code 1 is VALID NEGATIVE EVIDENCE!
        let probe = ProbeResult {
            command: "pacman -Q nano 2>/dev/null".to_string(),
            exit_code: 1,
            stdout: String::new(),
            stderr: "error: package 'nano' was not found".to_string(),
            timing_ms: 10,
        };
        let result = parse_probe_result(&probe);
        assert!(matches!(result, ParsedProbeData::Package(_)));
        if let ParsedProbeData::Package(ref p) = result {
            assert_eq!(p.name, "nano");
            assert!(!p.installed); // Negative evidence!
            assert!(p.version.is_none());
        }
        // Must NOT be an error
        assert!(!result.is_error());
        assert!(result.is_valid_evidence());
    }

    #[test]
    fn test_find_tool_evidence() {
        let parsed = vec![
            ParsedProbeData::Tool(ToolExists {
                name: "nano".to_string(),
                exists: true,
                method: ToolExistsMethod::CommandV,
                path: Some("/usr/bin/nano".to_string()),
            }),
            ParsedProbeData::Tool(ToolExists {
                name: "vim".to_string(),
                exists: false,
                method: ToolExistsMethod::CommandV,
                path: None,
            }),
        ];

        let nano = find_tool_evidence(&parsed, "nano");
        assert!(nano.is_some());
        assert!(nano.unwrap().exists);

        let vim = find_tool_evidence(&parsed, "vim");
        assert!(vim.is_some());
        assert!(!vim.unwrap().exists);

        let emacs = find_tool_evidence(&parsed, "emacs");
        assert!(emacs.is_none());
    }

    #[test]
    fn test_find_package_evidence() {
        let parsed = vec![
            ParsedProbeData::Package(PackageInstalled {
                name: "nano".to_string(),
                installed: true,
                version: Some("7.2-1".to_string()),
            }),
            ParsedProbeData::Package(PackageInstalled {
                name: "vim".to_string(),
                installed: false,
                version: None,
            }),
        ];

        let nano = find_package_evidence(&parsed, "nano");
        assert!(nano.is_some());
        assert!(nano.unwrap().installed);

        let vim = find_package_evidence(&parsed, "vim");
        assert!(vim.is_some());
        assert!(!vim.unwrap().installed);
    }

    #[test]
    fn test_has_evidence_for() {
        let parsed = vec![
            ParsedProbeData::Tool(ToolExists {
                name: "nano".to_string(),
                exists: false,
                method: ToolExistsMethod::CommandV,
                path: None,
            }),
        ];

        assert!(has_evidence_for(&parsed, "nano"));
        assert!(!has_evidence_for(&parsed, "vim"));
    }

    // v0.0.55: Audio parsing tests
    #[test]
    fn test_v055_lspci_audio_with_class_code() {
        // Real-world lspci output with PCI class code [XXXX]
        let output = "00:1f.3 Multimedia audio controller [0403]: Intel Corporation Cannon Lake PCH cAVS (rev 10)";
        let devices = parse_lspci_audio_output(output);
        assert_eq!(devices.len(), 1, "Should find one audio device");
        assert!(devices[0].description.contains("Intel"), "Description should contain Intel");
        assert!(devices[0].description.contains("Cannon Lake"), "Description should contain Cannon Lake");
        assert_eq!(devices[0].pci_slot, Some("00:1f.3".to_string()));
    }

    #[test]
    fn test_v055_lspci_audio_without_class_code() {
        // lspci output without PCI class code
        let output = "00:1f.3 Audio device: Intel Corporation Sunrise Point-LP HD Audio";
        let devices = parse_lspci_audio_output(output);
        assert_eq!(devices.len(), 1);
        assert!(devices[0].description.contains("Intel"));
    }

    #[test]
    fn test_v055_lspci_audio_empty_grep() {
        // Empty output from grep (no audio devices)
        let output = "";
        let devices = parse_lspci_audio_output(output);
        assert!(devices.is_empty(), "Empty output should produce empty list");
    }

    #[test]
    fn test_v055_lspci_audio_multiple_devices() {
        let output = "00:1f.3 Multimedia audio controller [0403]: Intel Corporation Cannon Lake PCH cAVS\n\
                      01:00.1 Audio device: NVIDIA Corporation TU104 HD Audio Controller";
        let devices = parse_lspci_audio_output(output);
        assert_eq!(devices.len(), 2, "Should find two audio devices");
        assert!(devices[0].description.contains("Intel"));
        assert!(devices[1].description.contains("NVIDIA"));
    }

    #[test]
    fn test_v055_extract_description_with_class_code() {
        let line = "00:1f.3 Multimedia audio controller [0403]: Intel Corporation Cannon Lake PCH cAVS (rev 10)";
        let desc = extract_lspci_description_v055(line);
        assert!(desc.contains("Intel"), "Description '{}' should contain Intel", desc);
        assert!(!desc.contains("[0403]"), "Description should not contain class code");
    }

    // v0.0.66: Full audio evidence flow tests
    #[test]
    fn test_v066_audio_evidence_from_lspci_probe() {
        use crate::rpc::ProbeResult;

        // Simulate lspci | grep -i audio probe with exit_code=0
        let probe = ProbeResult {
            command: "lspci | grep -i audio".to_string(),
            exit_code: 0,
            stdout: "00:1f.3 Multimedia audio controller [0403]: Intel Corporation Cannon Lake PCH cAVS (rev 10)".to_string(),
            stderr: String::new(),
            timing_ms: 10,
        };

        let parsed = parse_probe_result(&probe);
        assert!(parsed.as_audio().is_some(), "Should parse as Audio evidence");

        let audio = parsed.as_audio().unwrap();
        assert_eq!(audio.devices.len(), 1, "Should have one device");
        assert!(audio.devices[0].description.contains("Intel"), "Description should contain Intel");
        assert_eq!(audio.devices[0].pci_slot, Some("00:1f.3".to_string()));
    }

    #[test]
    fn test_v066_audio_negative_evidence_from_empty_grep() {
        use crate::rpc::ProbeResult;

        // Simulate lspci | grep -i audio with no matches (exit_code=1, empty stdout)
        let probe = ProbeResult {
            command: "lspci | grep -i audio".to_string(),
            exit_code: 1,
            stdout: String::new(),
            stderr: String::new(),
            timing_ms: 5,
        };

        let parsed = parse_probe_result(&probe);
        assert!(parsed.as_audio().is_some(), "Should parse as Audio evidence (negative)");

        let audio = parsed.as_audio().unwrap();
        assert!(audio.devices.is_empty(), "Should have zero devices (valid negative evidence)");
    }

    #[test]
    fn test_v066_find_audio_evidence_prefers_lspci() {
        // When both lspci and pactl have devices, lspci should be preferred
        let lspci = ParsedProbeData::Audio(AudioDevices {
            devices: vec![AudioDevice {
                description: "Intel Cannon Lake".to_string(),
                pci_slot: Some("00:1f.3".to_string()),
                vendor: Some("Intel".to_string()),
            }],
            source: "lspci".to_string(),
        });

        let pactl = ParsedProbeData::Audio(AudioDevices {
            devices: vec![AudioDevice {
                description: "alsa_card.pci-0000_00_1f.3".to_string(),
                pci_slot: None,
                vendor: None,
            }],
            source: "pactl".to_string(),
        });

        let parsed = vec![lspci, pactl];
        let merged = find_audio_evidence(&parsed);
        assert!(merged.is_some());

        let audio = merged.unwrap();
        assert!(!audio.devices.is_empty(), "Should have devices");
        // lspci device should be present (has PCI slot)
        assert!(audio.devices.iter().any(|d| d.pci_slot.is_some()));
    }
}
