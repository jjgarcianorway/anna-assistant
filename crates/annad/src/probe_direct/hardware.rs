//! Hardware queries (bluetooth, GPU, webcam, CPU, audio)

use anna_shared::parsers::{parse_probe_result, ParsedProbeData};
use anna_shared::rpc::ProbeResult;
use tracing::info;

use super::DirectAnswerResult;

/// Bluetooth answer
pub(crate) fn try_bluetooth_answer(query: &str, probes: &[ProbeResult]) -> Option<DirectAnswerResult> {
    if !query.contains("bluetooth") {
        return None;
    }

    for probe in probes {
        let cmd = probe.command.to_lowercase();
        if cmd.contains("systemctl") && cmd.contains("bluetooth") {
            let output = &probe.stdout;

            if output.contains("Active: active (running)") {
                info!("v0.0.403: Direct bluetooth answer - running");
                return Some(DirectAnswerResult {
                    answer: "**Bluetooth** service is **active and running**.".to_string(),
                    confidence: 95,
                });
            }

            if output.contains("Active: inactive") {
                return Some(DirectAnswerResult {
                    answer: "**Bluetooth** service is **inactive**.\n\nTo start: `sudo systemctl start bluetooth`".to_string(),
                    confidence: 95,
                });
            }

            if output.contains("could not be found") {
                return Some(DirectAnswerResult {
                    answer: "**Bluetooth** service is not installed.\n\nInstall with your package manager (e.g., `bluez`).".to_string(),
                    confidence: 95,
                });
            }
        }

        if cmd.contains("bluetoothctl") {
            if probe.stdout.contains("Controller") {
                return Some(DirectAnswerResult {
                    answer: format!(
                        "**Bluetooth** hardware detected:\n{}",
                        probe.stdout.lines().take(5).collect::<Vec<_>>().join("\n")
                    ),
                    confidence: 85,
                });
            }
        }
    }

    None
}

/// GPU answer
pub(crate) fn try_gpu_answer(query: &str, probes: &[ProbeResult]) -> Option<DirectAnswerResult> {
    if !query.contains("gpu") && !query.contains("graphic") && !query.contains("video") {
        return None;
    }

    for probe in probes {
        let cmd = probe.command.to_lowercase();
        if cmd.contains("lspci") && (cmd.contains("vga") || cmd.contains("3d")) {
            let gpus: Vec<&str> = probe.stdout.lines().filter(|l| !l.is_empty()).collect();

            if gpus.is_empty() {
                return Some(DirectAnswerResult {
                    answer: "No GPU detected via lspci.".to_string(),
                    confidence: 80,
                });
            }

            let mut answer = "**Graphics Hardware:**\n".to_string();
            for gpu in &gpus {
                answer.push_str(&format!("- {}\n", gpu.trim()));
            }
            info!("v0.0.403: Direct GPU answer");
            return Some(DirectAnswerResult {
                answer,
                confidence: 90,
            });
        }
    }

    None
}

/// Webcam answer
pub(crate) fn try_webcam_answer(query: &str, probes: &[ProbeResult]) -> Option<DirectAnswerResult> {
    if !query.contains("webcam") && !query.contains("camera") {
        return None;
    }

    for probe in probes {
        let cmd = probe.command.to_lowercase();
        if cmd.contains("v4l2") || cmd.contains("/dev/video") {
            if probe.stdout.trim().is_empty() || probe.exit_code != 0 {
                return Some(DirectAnswerResult {
                    answer: "**No webcam detected**. Check if camera is connected or drivers are loaded.".to_string(),
                    confidence: 85,
                });
            }

            let devices: Vec<&str> = probe.stdout.lines().filter(|l| !l.is_empty()).collect();
            if !devices.is_empty() {
                let mut answer = "**Webcam Devices:**\n".to_string();
                for dev in &devices {
                    answer.push_str(&format!("- {}\n", dev.trim()));
                }
                info!("v0.0.403: Direct webcam answer");
                return Some(DirectAnswerResult {
                    answer,
                    confidence: 90,
                });
            }
        }
    }

    None
}

/// CPU answer
pub(crate) fn try_cpu_answer(query: &str, probes: &[ProbeResult]) -> Option<DirectAnswerResult> {
    if !query.contains("cpu") && !query.contains("processor") {
        return None;
    }

    for probe in probes {
        let cmd = probe.command.to_lowercase();
        if cmd.contains("lscpu") {
            let parsed = parse_probe_result(probe);
            if let ParsedProbeData::Cpu(cpu) = parsed {
                let answer = format!(
                    "**CPU:** {}\n- Cores: {}\n- Threads: {}",
                    cpu.model_name,
                    cpu.physical_cores().unwrap_or(cpu.cpu_count),
                    cpu.cpu_count
                );
                info!("v0.0.403: Direct CPU answer");
                return Some(DirectAnswerResult {
                    answer,
                    confidence: 95,
                });
            }
        }
    }

    None
}

/// Audio answer
pub(crate) fn try_audio_answer(query: &str, probes: &[ProbeResult]) -> Option<DirectAnswerResult> {
    if !query.contains("audio") && !query.contains("sound") && !query.contains("speaker") {
        return None;
    }

    for probe in probes {
        let cmd = probe.command.to_lowercase();
        if cmd.contains("pactl")
            || cmd.contains("aplay")
            || (cmd.contains("lspci") && cmd.contains("audio"))
        {
            if !probe.stdout.trim().is_empty() {
                let mut answer = "**Audio Devices:**\n".to_string();
                for line in probe.stdout.lines().take(10) {
                    if !line.trim().is_empty() {
                        answer.push_str(&format!("- {}\n", line.trim()));
                    }
                }
                info!("v0.0.403: Direct audio answer");
                return Some(DirectAnswerResult {
                    answer,
                    confidence: 85,
                });
            }
        }
    }

    None
}
