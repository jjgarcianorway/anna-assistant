//! Hardware-related probe rules (GPU, audio, bluetooth, sensors).

use crate::deterministic_probes::types::ProbeRule;

pub fn hardware_rules() -> Vec<ProbeRule> {
    vec![
        // Bluetooth queries
        ProbeRule {
            intent_id: "bluetooth.status",
            keywords: &["bluetooth"],
            negative_keywords: &["install", "package"],
            probes: &["bluetooth_service", "bluetooth_devices"],
            description: "Bluetooth status",
        },
        ProbeRule {
            intent_id: "bluetooth.enabled",
            keywords: &["bluetooth", "enabled"],
            negative_keywords: &[],
            probes: &["bluetooth_service", "bluetooth_devices"],
            description: "Is bluetooth enabled",
        },
        ProbeRule {
            intent_id: "bluetooth.working",
            keywords: &["bluetooth", "working"],
            negative_keywords: &[],
            probes: &["bluetooth_service", "bluetooth_devices"],
            description: "Is bluetooth working",
        },
        // GPU/Graphics queries
        ProbeRule {
            intent_id: "gpu.info",
            keywords: &["gpu"],
            negative_keywords: &["install"],
            probes: &["gpu_info", "gpu_drivers", "glxinfo_renderer"],
            description: "GPU information",
        },
        ProbeRule {
            intent_id: "gpu.driver",
            keywords: &["graphics", "driver"],
            negative_keywords: &[],
            probes: &["gpu_drivers", "gpu_info"],
            description: "Graphics drivers",
        },
        ProbeRule {
            intent_id: "gpu.acceleration",
            keywords: &["hardware", "acceleration"],
            negative_keywords: &[],
            probes: &[
                "vaapi_status",
                "vdpau_status",
                "vulkan_status",
                "glxinfo_renderer",
            ],
            description: "Hardware acceleration",
        },
        // Audio queries
        ProbeRule {
            intent_id: "audio.status",
            keywords: &["audio"],
            negative_keywords: &["install"],
            probes: &["audio_devices", "audio_server"],
            description: "Audio status",
        },
        ProbeRule {
            intent_id: "audio.sound",
            keywords: &["sound"],
            negative_keywords: &["install"],
            probes: &["audio_devices", "audio_server"],
            description: "Sound status",
        },
        // Temperature/Sensors queries
        ProbeRule {
            intent_id: "sensors.temp",
            keywords: &["temperature"],
            negative_keywords: &[],
            probes: &["sensors_temp"],
            description: "System temperature",
        },
        ProbeRule {
            intent_id: "sensors.cpu_temp",
            keywords: &["cpu", "temp"],
            negative_keywords: &[],
            probes: &["sensors_temp"],
            description: "CPU temperature",
        },
    ]
}
