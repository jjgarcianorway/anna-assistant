//! Agent trait definition for multi-agent orchestration.

use async_trait::async_trait;
use super::types::{AgentTask, AgentResult, AgentContext, AgentCapability};
use super::memory::AgentMemory;

/// Domain specializations for agents.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum AgentDomain {
    Network,
    Desktop,
    System,
    Packages,
    Hardware,
    Audio,
    Storage,
    Security,
    General,
}

impl AgentDomain {
    pub fn from_str(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "network" => Self::Network,
            "desktop" => Self::Desktop,
            "system" => Self::System,
            "packages" => Self::Packages,
            "hardware" => Self::Hardware,
            "audio" => Self::Audio,
            "storage" => Self::Storage,
            "security" => Self::Security,
            _ => Self::General,
        }
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Network => "network",
            Self::Desktop => "desktop",
            Self::System => "system",
            Self::Packages => "packages",
            Self::Hardware => "hardware",
            Self::Audio => "audio",
            Self::Storage => "storage",
            Self::Security => "security",
            Self::General => "general",
        }
    }

    /// Keywords that indicate this domain.
    pub fn keywords(&self) -> &'static [&'static str] {
        match self {
            Self::Network => &["wifi", "network", "ethernet", "ip", "dns", "vpn", "firewall", "routing", "iptables", "nftables"],
            Self::Desktop => &["display", "monitor", "window", "hyprland", "i3", "kde", "gnome", "wayland", "x11", "compositor"],
            Self::System => &["systemd", "boot", "kernel", "service", "journal", "cron", "init", "grub"],
            Self::Packages => &["pacman", "yay", "aur", "package", "install", "update", "upgrade", "remove"],
            Self::Hardware => &["gpu", "nvidia", "amd", "driver", "firmware", "cpu", "ram", "usb", "pci", "power"],
            Self::Audio => &["audio", "sound", "pipewire", "pulseaudio", "alsa", "volume", "speaker", "microphone"],
            Self::Storage => &["disk", "partition", "mount", "fstab", "btrfs", "ext4", "nvme", "ssd", "storage"],
            Self::Security => &["ssh", "firewall", "permission", "sudo", "encryption", "gpg", "password", "key"],
            Self::General => &[],
        }
    }
}

/// Model tier for complexity-based routing.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum ModelTier {
    /// Fast model for simple queries (qwen2.5:7b)
    Fast,
    /// Balanced model for standard tasks (qwen2.5:14b)
    Standard,
    /// Deep model for complex debugging (qwen2.5:32b)
    Deep,
}

impl ModelTier {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Fast => "fast",
            Self::Standard => "standard",
            Self::Deep => "deep",
        }
    }

    pub fn from_str(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "fast" => Self::Fast,
            "deep" => Self::Deep,
            _ => Self::Standard,
        }
    }
}

/// Core trait that all agents must implement.
#[async_trait]
pub trait Agent: Send + Sync {
    /// Unique identifier for this agent.
    fn id(&self) -> &str;

    /// Human-readable name.
    fn name(&self) -> &str;

    /// Domain this agent specializes in.
    fn domain(&self) -> AgentDomain;

    /// Capabilities this agent provides.
    fn capabilities(&self) -> Vec<AgentCapability>;

    /// Preferred model tier for this agent.
    fn model_tier(&self) -> ModelTier;

    /// Score how well this agent can handle the given task (0.0-1.0).
    fn can_handle(&self, task: &AgentTask) -> f32;

    /// Execute the task and return a result.
    async fn execute(&self, task: AgentTask, ctx: &AgentContext) -> AgentResult;

    /// Get agent's memory/state (immutable).
    fn memory(&self) -> &AgentMemory;

    /// Persist learning from a completed task.
    fn learn(&mut self, task: &AgentTask, result: &AgentResult);
}
