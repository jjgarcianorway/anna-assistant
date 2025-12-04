//! Camera and Audio Discovery v7.25.0
//!
//! Discovers cameras from /dev/video* and v4l2-ctl.
//! Discovers audio cards from /proc/asound/cards.

use super::types::{AudioCard, AudioSummary, CameraDevice, CameraSummary};
use std::collections::HashSet;
use std::process::Command;

/// Get camera summary
pub fn get_camera_summary() -> CameraSummary {
    let mut summary = CameraSummary {
        camera_count: 0,
        cameras: Vec::new(),
        source: "/dev/video*, v4l2-ctl".to_string(),
    };

    let dev_path = std::path::Path::new("/dev");
    if let Ok(entries) = std::fs::read_dir(dev_path) {
        let mut seen_cameras = HashSet::new();

        for entry in entries.flatten() {
            let name = entry.file_name().to_string_lossy().to_string();
            if !name.starts_with("video") {
                continue;
            }

            let device_path = format!("/dev/{}", name);

            if let Some(camera) = get_camera_info(&device_path) {
                if seen_cameras.insert(camera.name.clone()) {
                    summary.cameras.push(camera);
                }
            }
        }
    }

    summary.camera_count = summary.cameras.len() as u32;
    summary
}

fn get_camera_info(device_path: &str) -> Option<CameraDevice> {
    // Try v4l2-ctl first
    let output = Command::new("v4l2-ctl")
        .args(["--device", device_path, "--info"])
        .output();

    if let Ok(out) = output {
        if out.status.success() {
            let stdout = String::from_utf8_lossy(&out.stdout);
            let mut name = String::new();
            let mut driver = String::new();
            let mut bus = String::new();
            let mut capabilities = Vec::new();

            for line in stdout.lines() {
                if line.contains("Card type") {
                    if let Some(val) = line.split(':').nth(1) {
                        name = val.trim().to_string();
                    }
                } else if line.contains("Driver name") {
                    if let Some(val) = line.split(':').nth(1) {
                        driver = val.trim().to_string();
                    }
                } else if line.contains("Bus info") {
                    if let Some(val) = line.split(':').nth(1) {
                        bus = val.trim().to_string();
                    }
                } else if line.contains("Capabilities") {
                    if let Some(caps) = line.split(':').nth(1) {
                        capabilities = caps.split_whitespace().map(|s| s.to_string()).collect();
                    }
                }
            }

            if !name.is_empty() {
                return Some(CameraDevice {
                    name,
                    device_path: device_path.to_string(),
                    driver,
                    capabilities,
                    bus,
                });
            }
        }
    }

    // Fallback: try to get info from /sys
    let device_num = device_path.trim_start_matches("/dev/video");
    let sys_path = format!("/sys/class/video4linux/video{}/name", device_num);
    if let Ok(name) = std::fs::read_to_string(&sys_path) {
        let name = name.trim().to_string();
        if !name.is_empty() {
            return Some(CameraDevice {
                name,
                device_path: device_path.to_string(),
                driver: String::new(),
                capabilities: Vec::new(),
                bus: String::new(),
            });
        }
    }

    None
}

/// Get audio summary
pub fn get_audio_summary() -> AudioSummary {
    let mut summary = AudioSummary {
        card_count: 0,
        cards: Vec::new(),
        source: "/proc/asound/cards, aplay -l".to_string(),
    };

    let cards_path = "/proc/asound/cards";
    if let Ok(content) = std::fs::read_to_string(cards_path) {
        for line in content.lines() {
            let line = line.trim();
            if line.is_empty()
                || !line
                    .chars()
                    .next()
                    .map(|c| c.is_ascii_digit())
                    .unwrap_or(false)
            {
                continue;
            }

            let parts: Vec<&str> = line.splitn(2, '[').collect();
            if parts.len() < 2 {
                continue;
            }

            let card_num: u32 = parts[0].trim().parse().unwrap_or(0);

            let rest = parts[1];
            let parts2: Vec<&str> = rest.splitn(2, "]: ").collect();
            if parts2.len() < 2 {
                continue;
            }

            let _card_id = parts2[0].trim_end_matches(']').trim();
            let name_parts: Vec<&str> = parts2[1].splitn(2, " - ").collect();

            let driver = name_parts.get(0).unwrap_or(&"").to_string();
            let name = name_parts
                .get(1)
                .map(|s| s.to_string())
                .unwrap_or_else(|| driver.clone());

            let card_type = if name.to_lowercase().contains("hdmi") {
                "HDMI".to_string()
            } else if name.to_lowercase().contains("displayport") {
                "DisplayPort".to_string()
            } else if name.to_lowercase().contains("usb") {
                "USB".to_string()
            } else if name.to_lowercase().contains("bluetooth") {
                "Bluetooth".to_string()
            } else {
                "Internal".to_string()
            };

            let (has_playback, has_capture) = check_audio_capabilities(card_num);

            summary.cards.push(AudioCard {
                card_num,
                name,
                driver,
                card_type,
                has_playback,
                has_capture,
            });
        }
    }

    summary.card_count = summary.cards.len() as u32;
    summary
}

fn check_audio_capabilities(card_num: u32) -> (bool, bool) {
    let pcm_path = format!("/proc/asound/card{}/pcm0p", card_num);
    let has_playback = std::path::Path::new(&pcm_path).exists();

    let capture_path = format!("/proc/asound/card{}/pcm0c", card_num);
    let has_capture = std::path::Path::new(&capture_path).exists();

    (has_playback, has_capture)
}
