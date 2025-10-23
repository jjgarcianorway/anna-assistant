use anyhow::{Context, Result};
use serde::Deserialize;
use std::fs;
use std::path::Path;

const CONFIG_PATH: &str = "/etc/anna/config.toml";

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct Config {
    pub persona: PersonaConfig,
    pub advice: AdviceConfig,
    pub signals: SignalsConfig,
    pub quickscan: QuickscanConfig,
}

#[derive(Debug, Clone)]
pub struct PersonaConfig {
    pub enabled: bool,
    pub confidence_threshold: f32,
    pub min_observation_days: u32,
    pub sampler: SamplerConfig,
    pub infer: InferConfig,
    pub trigger: TriggerConfig,
}

impl Default for PersonaConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            confidence_threshold: 0.6,
            min_observation_days: 3,
            sampler: SamplerConfig::default(),
            infer: InferConfig::default(),
            trigger: TriggerConfig::default(),
        }
    }
}

#[derive(Debug, Deserialize, Default)]
struct RawConfig {
    #[serde(default)]
    persona: RawPersonaConfig,
    #[serde(default)]
    advice: RawAdviceConfig,
    #[serde(default)]
    signals: RawSignalsConfig,
    #[serde(default)]
    quickscan: RawQuickscanConfig,
}

#[derive(Debug, Deserialize, Default)]
struct RawPersonaConfig {
    #[serde(default)]
    enabled: Option<bool>,
    #[serde(default)]
    confidence_threshold: Option<f32>,
    #[serde(default)]
    min_observation_days: Option<u32>,
    #[serde(default)]
    sampler: RawSamplerConfig,
    #[serde(default)]
    infer: RawInferConfig,
    #[serde(default)]
    trigger: RawTriggerConfig,
}

impl From<RawPersonaConfig> for PersonaConfig {
    fn from(raw: RawPersonaConfig) -> Self {
        let defaults = PersonaConfig::default();
        Self {
            enabled: raw.enabled.unwrap_or(defaults.enabled),
            confidence_threshold: raw
                .confidence_threshold
                .unwrap_or(defaults.confidence_threshold),
            min_observation_days: raw
                .min_observation_days
                .unwrap_or(defaults.min_observation_days),
            sampler: raw.sampler.into(),
            infer: raw.infer.into(),
            trigger: raw.trigger.into(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct AdviceConfig {
    pub enabled: bool,
    pub disk_free_threshold: f32,
    pub check_interval_minutes: u64,
    pub cooldown_hours: u64,
}

impl Default for AdviceConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            disk_free_threshold: 0.15,
            check_interval_minutes: 60,
            cooldown_hours: 12,
        }
    }
}

#[derive(Debug, Clone)]
pub struct TriggerConfig {
    pub enable: bool,
    pub debounce_secs: u64,
    pub pkg_churn_threshold: u32,
    pub shell_hist_threshold: u32,
    pub browser_nav_threshold: u32,
}

impl Default for TriggerConfig {
    fn default() -> Self {
        Self {
            enable: true,
            debounce_secs: 300,
            pkg_churn_threshold: 20,
            shell_hist_threshold: 200,
            browser_nav_threshold: 100,
        }
    }
}

#[derive(Debug, Deserialize, Default)]
struct RawTriggerConfig {
    #[serde(default)]
    enable: Option<bool>,
    #[serde(default)]
    debounce_secs: Option<u64>,
    #[serde(default)]
    pkg_churn_threshold: Option<u32>,
    #[serde(default)]
    shell_hist_threshold: Option<u32>,
    #[serde(default)]
    browser_nav_threshold: Option<u32>,
}

impl From<RawTriggerConfig> for TriggerConfig {
    fn from(raw: RawTriggerConfig) -> Self {
        let defaults = TriggerConfig::default();
        let debounce = raw.debounce_secs.unwrap_or(defaults.debounce_secs).max(60);
        let pkg_threshold = raw
            .pkg_churn_threshold
            .unwrap_or(defaults.pkg_churn_threshold)
            .max(1);
        let shell_threshold = raw
            .shell_hist_threshold
            .unwrap_or(defaults.shell_hist_threshold)
            .max(1);
        let browser_threshold = raw
            .browser_nav_threshold
            .unwrap_or(defaults.browser_nav_threshold)
            .max(1);
        Self {
            enable: raw.enable.unwrap_or(defaults.enable),
            debounce_secs: debounce,
            pkg_churn_threshold: pkg_threshold,
            shell_hist_threshold: shell_threshold,
            browser_nav_threshold: browser_threshold,
        }
    }
}

#[derive(Debug, Deserialize, Default)]
struct RawAdviceConfig {
    #[serde(default)]
    enabled: Option<bool>,
    #[serde(default)]
    disk_free_threshold: Option<f32>,
    #[serde(default)]
    check_interval_minutes: Option<u64>,
    #[serde(default)]
    cooldown_hours: Option<u64>,
}

impl From<RawAdviceConfig> for AdviceConfig {
    fn from(raw: RawAdviceConfig) -> Self {
        let defaults = AdviceConfig::default();
        let threshold = raw
            .disk_free_threshold
            .unwrap_or(defaults.disk_free_threshold)
            .clamp(0.05, 0.5);
        let interval = raw
            .check_interval_minutes
            .unwrap_or(defaults.check_interval_minutes)
            .max(5);
        let cooldown = raw.cooldown_hours.unwrap_or(defaults.cooldown_hours).max(1);
        Self {
            enabled: raw.enabled.unwrap_or(defaults.enabled),
            disk_free_threshold: threshold,
            check_interval_minutes: interval,
            cooldown_hours: cooldown,
        }
    }
}

#[derive(Debug, Clone, Default)]
pub struct SignalsConfig {
    pub allow_shell_history: bool,
    pub allow_browser_history: bool,
}

#[derive(Debug, Deserialize, Default)]
struct RawSignalsConfig {
    #[serde(default)]
    allow_shell_history: Option<bool>,
    #[serde(default)]
    allow_browser_history: Option<bool>,
}

impl From<RawSignalsConfig> for SignalsConfig {
    fn from(raw: RawSignalsConfig) -> Self {
        let defaults = SignalsConfig::default();
        Self {
            allow_shell_history: raw
                .allow_shell_history
                .unwrap_or(defaults.allow_shell_history),
            allow_browser_history: raw
                .allow_browser_history
                .unwrap_or(defaults.allow_browser_history),
        }
    }
}

#[derive(Debug, Clone)]
pub struct QuickscanConfig {
    pub enable: bool,
    pub timeout_secs: u64,
    pub check_network: bool,
    pub check_cpu_power: bool,
    pub check_memory_swap: bool,
    pub check_storage: bool,
    pub check_fs_trim: bool,
    pub check_ntp: bool,
    pub check_pkg_cache: bool,
    pub check_orphans: bool,
}

impl Default for QuickscanConfig {
    fn default() -> Self {
        Self {
            enable: true,
            timeout_secs: 12,
            check_network: true,
            check_cpu_power: true,
            check_memory_swap: true,
            check_storage: true,
            check_fs_trim: true,
            check_ntp: true,
            check_pkg_cache: true,
            check_orphans: true,
        }
    }
}

#[derive(Debug, Deserialize, Default)]
struct RawQuickscanConfig {
    #[serde(default)]
    enable: Option<bool>,
    #[serde(default)]
    timeout_secs: Option<u64>,
    #[serde(default)]
    check_network: Option<bool>,
    #[serde(default)]
    check_cpu_power: Option<bool>,
    #[serde(default)]
    check_memory_swap: Option<bool>,
    #[serde(default)]
    check_storage: Option<bool>,
    #[serde(default)]
    check_fs_trim: Option<bool>,
    #[serde(default)]
    check_ntp: Option<bool>,
    #[serde(default)]
    check_pkg_cache: Option<bool>,
    #[serde(default)]
    check_orphans: Option<bool>,
}

impl From<RawQuickscanConfig> for QuickscanConfig {
    fn from(raw: RawQuickscanConfig) -> Self {
        let defaults = QuickscanConfig::default();
        Self {
            enable: raw.enable.unwrap_or(defaults.enable),
            timeout_secs: raw.timeout_secs.unwrap_or(defaults.timeout_secs).max(5),
            check_network: raw.check_network.unwrap_or(defaults.check_network),
            check_cpu_power: raw.check_cpu_power.unwrap_or(defaults.check_cpu_power),
            check_memory_swap: raw.check_memory_swap.unwrap_or(defaults.check_memory_swap),
            check_storage: raw.check_storage.unwrap_or(defaults.check_storage),
            check_fs_trim: raw.check_fs_trim.unwrap_or(defaults.check_fs_trim),
            check_ntp: raw.check_ntp.unwrap_or(defaults.check_ntp),
            check_pkg_cache: raw.check_pkg_cache.unwrap_or(defaults.check_pkg_cache),
            check_orphans: raw.check_orphans.unwrap_or(defaults.check_orphans),
        }
    }
}

#[derive(Debug, Clone)]
pub struct SamplerConfig {
    pub enable: bool,
    pub interval_secs: u64,
    pub max_procs: usize,
    pub loadavg_cap: f64,
}

impl Default for SamplerConfig {
    fn default() -> Self {
        Self {
            enable: true,
            interval_secs: 60,
            max_procs: 20000,
            loadavg_cap: 8.0,
        }
    }
}

#[derive(Debug, Deserialize, Default)]
struct RawSamplerConfig {
    #[serde(default)]
    enable: Option<bool>,
    #[serde(default)]
    interval_secs: Option<u64>,
    #[serde(default)]
    max_procs: Option<usize>,
    #[serde(default)]
    loadavg_cap: Option<f64>,
}

impl From<RawSamplerConfig> for SamplerConfig {
    fn from(raw: RawSamplerConfig) -> Self {
        let defaults = SamplerConfig::default();
        let interval = raw.interval_secs.unwrap_or(defaults.interval_secs).max(15);
        let max_procs = raw.max_procs.unwrap_or(defaults.max_procs).max(1000);
        let loadavg = raw.loadavg_cap.unwrap_or(defaults.loadavg_cap).max(0.0);
        Self {
            enable: raw.enable.unwrap_or(defaults.enable),
            interval_secs: interval,
            max_procs,
            loadavg_cap: loadavg,
        }
    }
}

#[derive(Debug, Clone)]
pub struct InferConfig {
    pub daily_at: String,
    pub window_days: u32,
    pub change_epsilon: f32,
}

impl Default for InferConfig {
    fn default() -> Self {
        Self {
            daily_at: "03:15".into(),
            window_days: 14,
            change_epsilon: 0.05,
        }
    }
}

#[derive(Debug, Deserialize, Default)]
struct RawInferConfig {
    #[serde(default)]
    daily_at: Option<String>,
    #[serde(default)]
    window_days: Option<u32>,
    #[serde(default)]
    change_epsilon: Option<f32>,
}

impl From<RawInferConfig> for InferConfig {
    fn from(raw: RawInferConfig) -> Self {
        let defaults = InferConfig::default();
        let window = raw.window_days.unwrap_or(defaults.window_days).clamp(7, 30);
        let daily = raw.daily_at.unwrap_or_else(|| defaults.daily_at.clone());
        let epsilon = raw
            .change_epsilon
            .unwrap_or(defaults.change_epsilon)
            .max(0.0);
        Self {
            daily_at: daily,
            window_days: window,
            change_epsilon: epsilon,
        }
    }
}

pub fn load() -> Result<Config> {
    let path = Path::new(CONFIG_PATH);
    if !path.exists() {
        return Ok(Config {
            persona: PersonaConfig::default(),
            advice: AdviceConfig::default(),
            signals: SignalsConfig::default(),
            quickscan: QuickscanConfig::default(),
        });
    }
    let raw = fs::read_to_string(path).with_context(|| format!("read {}", path.display()))?;
    let file_cfg: RawConfig =
        toml::from_str(&raw).with_context(|| format!("parse {}", path.display()))?;
    Ok(Config {
        persona: file_cfg.persona.into(),
        advice: file_cfg.advice.into(),
        signals: file_cfg.signals.into(),
        quickscan: file_cfg.quickscan.into(),
    })
}
