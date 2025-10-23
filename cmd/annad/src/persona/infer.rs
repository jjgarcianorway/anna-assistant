use super::store::{create_state, Store, TriggerSnapshot};
use super::types::{Persona, PersonaSource, PersonaState};
use crate::config::Config;
use anyhow::{Context, Result};
use serde::Deserialize;
use std::collections::HashMap;
use std::fs::{self as stdfs, File};
use std::io::Read;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::Duration as StdDuration;
use time::{
    format_description::well_known::Rfc3339, Duration as TimeDuration, OffsetDateTime,
    PrimitiveDateTime, Time,
};
use tokio::time as tokio_time;
use tracing::{info, warn};

#[derive(Debug, Deserialize)]
struct RollupFile {
    #[serde(default)]
    _date: String,
    total: u64,
    by_cat: HashMap<String, u64>,
    top_execs: Vec<(String, u64)>,
    #[serde(default)]
    _generated_at: String,
}

#[derive(Default)]
pub struct WindowData {
    totals: HashMap<String, u64>,
    total_all: u64,
    top_execs: HashMap<String, u64>,
    days: u32,
}

impl WindowData {
    fn add_rollup(&mut self, rollup: RollupFile) {
        self.total_all += rollup.total;
        for (cat, count) in rollup.by_cat {
            *self.totals.entry(cat.to_lowercase()).or_insert(0) += count;
        }
        for (exe, count) in rollup.top_execs {
            *self.top_execs.entry(exe.to_lowercase()).or_insert(0) += count;
        }
        self.days += 1;
    }

    fn share(&self, cat: &str) -> f32 {
        if self.total_all == 0 {
            return 0.0;
        }
        let cat_total = self.totals.get(cat).copied().unwrap_or(0);
        (cat_total as f32) / (self.total_all as f32)
    }

    fn contains_exec(&self, name: &str) -> bool {
        self.top_execs.contains_key(name)
    }

    fn exec_count(&self, name: &str) -> u64 {
        self.top_execs.get(name).copied().unwrap_or(0)
    }
}

pub fn maybe_update_current(cfg: &Config) -> Result<Option<PersonaState>> {
    let store = Store::new()?;
    if store.read_override()?.is_some() {
        return Ok(None);
    }

    let window = load_window(cfg.persona.infer.window_days as usize)?;
    if window.days < cfg.persona.min_observation_days {
        return Ok(None);
    }

    let signals = store.read_last_trigger()?;
    let (persona, confidence, explanations) = score(&window, signals.as_ref());
    if confidence < cfg.persona.confidence_threshold {
        return Ok(None);
    }

    let mut current = store
        .read_current()?
        .unwrap_or_else(|| create_state(Persona::Unknown, 0.0, PersonaSource::Default));

    let confidence_delta = (confidence - current.confidence).abs();
    if current.persona == persona && confidence_delta < cfg.persona.infer.change_epsilon {
        return Ok(None);
    }

    current.persona = persona;
    current.confidence = confidence;
    current.source = PersonaSource::Inferred;
    current.updated = now_rfc3339();
    current.window_days = window.days;
    current.explanations = explanations;

    store.write_current(&current)?;
    let summary = current
        .explanations
        .iter()
        .take(2)
        .cloned()
        .collect::<Vec<_>>()
        .join("; ");
    info!(
        target: "annad",
        "persona updated: {} ({:.2}) — {}",
        current.persona.as_str(),
        current.confidence,
        summary
    );

    Ok(Some(current))
}

pub fn load_window(days: usize) -> Result<WindowData> {
    let mut window = WindowData::default();
    let mut rollups = gather_rollup_paths()?;
    rollups.sort();
    rollups.reverse();
    for path in rollups.into_iter().take(days) {
        let mut data = Vec::new();
        File::open(&path)
            .with_context(|| format!("open rollup {}", path.display()))?
            .read_to_end(&mut data)?;
        let rollup: RollupFile = serde_json::from_slice(&data)
            .with_context(|| format!("parse rollup {}", path.display()))?;
        window.add_rollup(rollup);
    }
    Ok(window)
}

fn gather_rollup_paths() -> Result<Vec<PathBuf>> {
    let mut paths = Vec::new();
    let dir = PathBuf::from(super::fs::ROLLUPS_DIR);
    if dir.exists() {
        for entry in stdfs::read_dir(&dir)? {
            let entry = entry?;
            let path = entry.path();
            if path.extension().and_then(|s| s.to_str()) == Some("json") {
                paths.push(path);
            }
        }
    }
    Ok(paths)
}

pub fn score(
    window: &WindowData,
    trigger: Option<&TriggerSnapshot>,
) -> (Persona, f32, Vec<String>) {
    let shares = |cat: &str| window.share(cat);
    let s_editor = shares("editor");
    let s_terminal = shares("terminal");
    let s_browser = shares("browser");
    let s_build = shares("build");
    let s_shell = shares("shell");
    let s_other = shares("other");

    let mut dev = (0.45 * s_editor + 0.35 * s_terminal + 0.20 * s_build).min(1.0);
    let admin_browser_bonus = (s_browser * 0.3).min(0.1);
    let mut admin = (0.5 * (s_shell + s_terminal) + 0.3 * admin_browser_bonus).min(1.0);
    let tiling_tool = detect_power_tool(window);
    let mut power = (0.4 * s_terminal + 0.25 * s_editor + 0.2 * s_shell + 0.15 * s_other).min(1.0);
    if tiling_tool.is_some() {
        power = (power + 0.1).min(1.0);
    }
    let mut creator = (0.5 * s_browser + 0.3 * s_editor).min(1.0);
    if s_build > 0.15 {
        creator = (creator - 0.2).max(0.0);
    }
    let combined_dev = s_editor + s_terminal;
    if combined_dev >= 0.6 {
        dev = (dev + 0.3).min(1.0);
    }
    if s_shell + s_terminal >= 0.5 {
        admin = (admin + 0.25).min(1.0);
    }
    if s_browser >= 0.4 {
        creator = (creator + 0.25).min(1.0);
    }
    const DEV_TOOL_EXECUTABLES: &[&str] = &[
        "cargo", "rustup", "pip", "pip3", "poetry", "npm", "pnpm", "yarn", "go", "deno",
    ];
    const POWER_DESKTOP_EXECUTABLES: &[&str] = &[
        "hyprland",
        "sway",
        "river",
        "qtile",
        "waybar",
        "dunst",
        "alacritty",
        "kitty",
        "wezterm",
    ];
    const EDITOR_PLUGINS: &[&str] = &["nvim", "vim", "neovide", "astronvim"];
    const ADMIN_TOGGLES: &[&str] = &["powerprofilesctl", "tlp", "tuned-adm"];

    let mut bonus_notes = Vec::new();

    if let Some(snapshot) = trigger {
        if snapshot.pkg_churn >= 30 {
            dev = (dev + 0.15).min(1.0);
            power = (power + 0.15).min(1.0);
            bonus_notes.push(format!(
                "Package churn (~{} operations) suggests active tweaking.",
                snapshot.pkg_churn
            ));
        } else if snapshot.pkg_churn >= 10 {
            dev = (dev + 0.10).min(1.0);
            power = (power + 0.10).min(1.0);
            bonus_notes.push(format!(
                "Recent package activity (~{} ops) nudged us toward dev/power personas.",
                snapshot.pkg_churn
            ));
        }
    }

    if let Some(tool) = DEV_TOOL_EXECUTABLES
        .iter()
        .find(|name| window.exec_count(name) >= 3)
    {
        dev = (dev + 0.12).min(1.0);
        power = (power + 0.08).min(1.0);
        bonus_notes.push(format!(
            "Commands like `{}` showed up repeatedly, signalling developer workflows.",
            tool
        ));
    }

    if let Some(tool) = POWER_DESKTOP_EXECUTABLES
        .iter()
        .find(|name| window.exec_count(name) >= 1)
    {
        power = (power + 0.1).min(1.0);
        bonus_notes.push(format!(
            "Desktop tooling such as `{}` points toward a power-user setup.",
            tool
        ));
    }

    if let Some(tool) = EDITOR_PLUGINS
        .iter()
        .find(|name| window.exec_count(name) >= 5)
    {
        dev = (dev + 0.08).min(1.0);
        bonus_notes.push(format!(
            "Editor-focused processes like `{}` dominated samples, leaning dev-heavy.",
            tool
        ));
    }

    if let Some(tool) = ADMIN_TOGGLES
        .iter()
        .find(|name| window.exec_count(name) >= 2)
    {
        admin = (admin + 0.1).min(1.0);
        bonus_notes.push(format!(
            "Power management commands such as `{}` suggest hands-on system tuning.",
            tool
        ));
    }

    let max_other = dev.max(admin).max(power).max(creator);
    let casual = (1.0 - max_other).max(0.0);

    let candidates = [
        (Persona::DevEnthusiast, dev),
        (Persona::AdminPragmatic, admin),
        (Persona::PowerNerd, power),
        (Persona::CreatorWriter, creator),
        (Persona::CasualMinimal, casual),
    ];

    let mut best = (Persona::Unknown, 0.0);
    for (persona, score) in candidates {
        if score >= best.1 {
            best = (persona, score);
        }
    }

    let mut explanations = bonus_notes;
    let days = window.days.max(1);
    match best.0 {
        Persona::DevEnthusiast => {
            explanations.push(format!(
                "Editors and terminals made up ~{}% of your active time over {} day(s).",
                pct(s_editor + s_terminal),
                days
            ));
            if s_build > 0.05 {
                explanations.push(format!(
                    "Build tools appeared in roughly {}% of samples, supporting developer focus.",
                    pct(s_build)
                ));
            }
            if explanations.len() < 2 && s_browser > 0.05 {
                explanations.push(
                    "Frequent dev browsing paired with coding tools reinforced this pick."
                        .to_string(),
                );
            }
        }
        Persona::AdminPragmatic => {
            explanations.push(format!(
                "Shell and terminal commands were about {}% of activity across {} day(s).",
                pct(s_shell + s_terminal),
                days
            ));
            if admin_browser_bonus > 0.0 {
                explanations.push(
                    "Admin-flavoured browsing and tooling appeared alongside those sessions."
                        .to_string(),
                );
            }
            if explanations.len() < 2 && s_build > 0.05 {
                explanations.push(
                    "System maintenance commands outweighed build or editor activity.".to_string(),
                );
            }
        }
        Persona::PowerNerd => {
            explanations.push(format!(
                "Terminal usage (~{}%) plus steady editor time (~{}%) signalled a power-user workflow.",
                pct(s_terminal),
                pct(s_editor)
            ));
            if let Some(tool) = tiling_tool {
                explanations.push(format!(
                    "Advanced tooling like {} was spotted, nudging this toward power-nerd.",
                    tool
                ));
            } else {
                explanations.push("Mixed usage of scripting/shell tools kept things firmly in the tweak-friendly lane.".to_string());
            }
        }
        Persona::CreatorWriter => {
            explanations.push(format!(
                "Writing/browsing tools covered roughly {}% of activity over {} day(s).",
                pct(s_browser + s_editor),
                days
            ));
            if s_build <= 0.05 {
                explanations.push(
                    "Very little build automation showed up, keeping focus on content work."
                        .to_string(),
                );
            }
            if explanations.len() < 2 && s_terminal < 0.2 {
                explanations
                    .push("Terminal usage stayed light compared to writing tools.".to_string());
            }
        }
        Persona::CasualMinimal => {
            explanations.push(format!(
                "No single category dominated in the last {} day(s); treating this as light/casual usage.",
                days
            ));
            let top_cat = window
                .totals
                .iter()
                .max_by_key(|(_cat, count)| *count)
                .map(|(cat, count)| (cat.as_str(), *count));
            if let Some((cat, count)) = top_cat {
                if window.total_all > 0 {
                    let share = count as f32 / window.total_all as f32;
                    explanations.push(format!(
                        "{} topped the chart at ~{}% but never dominated enough to switch personas.",
                        cat,
                        pct(share)
                    ));
                }
            }
        }
        Persona::Unknown => {
            explanations.push("Not enough recent activity to learn a persona yet.".to_string());
            explanations.push("Keep using the system normally and we’ll check again.".to_string());
        }
    }

    if explanations.len() > 4 {
        explanations.truncate(4);
    }

    if explanations.len() < 2 {
        explanations
            .push("Signals were sparse but consistent with the current assignment.".to_string());
    }

    (best.0, best.1.min(1.0), explanations)
}

fn now_rfc3339() -> String {
    OffsetDateTime::now_utc()
        .format(&Rfc3339)
        .unwrap_or_else(|_| "1970-01-01T00:00:00Z".into())
}

fn pct(value: f32) -> i32 {
    (value * 100.0).round() as i32
}

fn detect_power_tool(window: &WindowData) -> Option<&'static str> {
    const TOOLS: &[&str] = &[
        "hyprland", "bspwm", "sway", "btrfs", "zfs", "nix", "paru", "yay",
    ];
    TOOLS
        .iter()
        .copied()
        .find(|tool| window.contains_exec(tool))
}

fn parse_daily_at(infer_cfg: &crate::config::InferConfig) -> (u8, u8) {
    let parts: Vec<&str> = infer_cfg.daily_at.split(':').collect();
    if parts.len() != 2 {
        return (3, 15);
    }
    let hour = parts[0].parse::<u8>().ok();
    let minute = parts[1].parse::<u8>().ok();
    match (hour, minute) {
        (Some(h), Some(m)) if h < 24 && m < 60 => (h, m),
        _ => (3, 15),
    }
}

fn next_delay(infer_cfg: &crate::config::InferConfig) -> StdDuration {
    let (hour, minute) = parse_daily_at(infer_cfg);
    let now_local = OffsetDateTime::now_local().unwrap_or_else(|_| OffsetDateTime::now_utc());
    let time =
        Time::from_hms(hour, minute, 0).unwrap_or_else(|_| Time::from_hms(3, 15, 0).unwrap());
    let mut target =
        PrimitiveDateTime::new(now_local.date(), time).assume_offset(now_local.offset());
    if target <= now_local {
        target += TimeDuration::days(1);
    }
    let duration = target - OffsetDateTime::now_utc();
    if duration.is_negative() {
        StdDuration::from_secs(0)
    } else {
        let secs = duration.whole_seconds() as u64;
        let nanos = duration.subsec_nanoseconds() as u32;
        StdDuration::from_secs(secs) + StdDuration::from_nanos(nanos as u64)
    }
}

pub fn schedule_daily(cfg: Config) -> Result<()> {
    let cfg = Arc::new(cfg);
    let initial_delay = next_delay(&cfg.persona.infer);
    let cfg_clone = Arc::clone(&cfg);
    tokio::spawn(async move {
        tokio_time::sleep(initial_delay).await;
        loop {
            if let Err(err) = maybe_update_current(&cfg_clone) {
                warn!(target: "annad", "persona inference error: {err:?}");
            }
            let delay = next_delay(&cfg_clone.persona.infer);
            tokio_time::sleep(delay).await;
        }
    });
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_window(totals: &[(&str, u64)], total_all: u64) -> WindowData {
        let mut wd = WindowData::default();
        wd.total_all = total_all;
        wd.days = 7;
        for (cat, count) in totals {
            wd.totals.insert((*cat).into(), *count);
        }
        wd
    }

    #[test]
    fn dev_persona_scoring() {
        let window = make_window(&[("editor", 300), ("terminal", 200), ("build", 100)], 700);
        let (persona, score, explanations) = score(&window, None);
        assert_eq!(persona, Persona::DevEnthusiast);
        assert!(score > 0.6);
        assert!(explanations.len() >= 2);
        assert!(explanations
            .iter()
            .any(|e| e.to_lowercase().contains("editor")));
    }

    #[test]
    fn casual_persona_when_even() {
        let window = make_window(&[("editor", 100), ("browser", 120), ("terminal", 110)], 400);
        let (persona, _score, explanations) = score(&window, None);
        assert_eq!(persona, Persona::CasualMinimal);
        assert!(explanations.len() >= 2);
        assert!(explanations
            .iter()
            .any(|e| e.to_lowercase().contains("casual")));
    }
}
