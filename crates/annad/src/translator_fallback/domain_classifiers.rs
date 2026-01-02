//! Domain-specific classification functions
//!
//! Handles network, graphics, audio, bluetooth, boot, services, packages, and security queries

use anna_shared::answer_contract::AnswerContract;
use anna_shared::rpc::{QueryIntent, SpecialistDomain, TranslatorTicket};

use super::helpers::{classify_intent, extract_tool_name_from_query};

pub fn classify_network_query(q: &str, orig: &str) -> Option<TranslatorTicket> {
    if q.contains("network")
        || q.contains("ip ")
        || q.contains("ip address")
        || q.contains("interface")
        || q.contains("dns")
        || q.contains("port")
        || q.contains("route")
        || q.contains("wifi")
        || q.contains("wireless")
        || q.contains("ethernet")
        || q.contains("internet")
        || q.contains("connected")
        || q.contains("connection")
        || q.contains("ping")
        || q.contains("latency")
        || q.contains("bandwidth")
    {
        let mut probes = vec!["network_addrs".to_string()];

        if q.contains("route") || q.contains("gateway") {
            probes.push("network_routes".to_string());
            probes.push("default_gateway".to_string());
        }
        if q.contains("port") || q.contains("listen") || q.contains("service") {
            probes.push("listening_ports".to_string());
        }
        if q.contains("dns") || q.contains("resolve") {
            probes.push("dns_servers".to_string());
        }
        if q.contains("wifi") || q.contains("wireless") {
            probes.push("wireless_networks".to_string());
        }
        if q.contains("ping") || q.contains("connection") || q.contains("internet") {
            probes.push("ping_check".to_string());
        }

        return Some(TranslatorTicket {
            intent: classify_intent(q),
            domain: SpecialistDomain::Network,
            entities: vec![],
            needs_probes: probes,
            clarification_question: None,
            confidence: 0.85,
            answer_contract: Some(AnswerContract::from_query(orig)),
        });
    }
    None
}

pub fn classify_graphics_query(q: &str, orig: &str) -> Option<TranslatorTicket> {
    if q.contains("gpu")
        || q.contains("graphics")
        || q.contains("video")
        || q.contains("screen")
        || q.contains("display")
        || q.contains("monitor")
        || q.contains("resolution")
        || q.contains("tearing")
        || q.contains("nvidia")
        || q.contains("amd")
        || q.contains("radeon")
        || q.contains("intel")
        || q.contains("wayland")
        || q.contains("xorg")
        || q.contains("x11")
        || q.contains("vulkan")
        || q.contains("opengl")
        || q.contains("glx")
        || q.contains("vaapi")
        || q.contains("vdpau")
        || q.contains("cuda")
        || q.contains("driver")
        || q.contains("acceleration")
    {
        let mut probes = vec!["gpu_info".to_string(), "display_server".to_string()];

        if q.contains("driver") || q.contains("nvidia") || q.contains("amd") {
            probes.push("gpu_drivers".to_string());
            probes.push("kernel_modules".to_string());
        }
        if q.contains("vulkan") {
            probes.push("vulkan_status".to_string());
        }
        if q.contains("opengl") || q.contains("glx") {
            probes.push("glxinfo_renderer".to_string());
        }
        if q.contains("vaapi") || q.contains("hardware") || q.contains("acceleration") {
            probes.push("vaapi_status".to_string());
            probes.push("vdpau_status".to_string());
        }
        if q.contains("monitor") || q.contains("resolution") {
            probes.push("display_info".to_string());
        }

        return Some(TranslatorTicket {
            intent: classify_intent(q),
            domain: SpecialistDomain::Display, // v0.0.405: Use Display domain
            entities: vec![],
            needs_probes: probes,
            clarification_question: None,
            confidence: 0.85,
            answer_contract: Some(AnswerContract::from_query(orig)),
        });
    }
    None
}

pub fn classify_audio_query(q: &str, orig: &str) -> Option<TranslatorTicket> {
    if q.contains("audio")
        || q.contains("sound")
        || q.contains("speaker")
        || q.contains("headphone")
        || q.contains("microphone")
        || q.contains("mic")
        || q.contains("volume")
        || q.contains("pulseaudio")
        || q.contains("pipewire")
        || q.contains("alsa")
        || q.contains("no sound")
    {
        return Some(TranslatorTicket {
            intent: classify_intent(q),
            domain: SpecialistDomain::Audio, // v0.0.405: Use Audio domain
            entities: vec![],
            needs_probes: vec![
                "audio_devices".to_string(),
                "audio_server".to_string(),
                "pactl_cards".to_string(),
            ],
            clarification_question: None,
            confidence: 0.85,
            answer_contract: Some(AnswerContract::from_query(orig)),
        });
    }
    None
}

pub fn classify_bluetooth_query(q: &str, orig: &str) -> Option<TranslatorTicket> {
    if q.contains("bluetooth") || q.contains("bt ") || q.contains("pair") {
        // v0.0.403: Distinguish between service status vs device queries
        let probes = if q.contains("running")
            || q.contains("active")
            || q.contains("status")
            || q.contains("service")
            || q.contains("start")
            || q.contains("stop")
        {
            // Service status query - use systemctl probe
            vec!["bluetooth_service".to_string()]
        } else {
            // General bluetooth query - check both service and devices
            vec![
                "bluetooth_service".to_string(),
                "bluetooth_devices".to_string(),
            ]
        };

        return Some(TranslatorTicket {
            intent: classify_intent(q),
            domain: SpecialistDomain::System,
            entities: vec![],
            needs_probes: probes,
            clarification_question: None,
            confidence: 0.85,
            answer_contract: Some(AnswerContract::from_query(orig)),
        });
    }
    None
}

pub fn classify_boot_query(q: &str, orig: &str) -> Option<TranslatorTicket> {
    if q.contains("boot")
        || q.contains("startup")
        || q.contains("uptime")
        || q.contains("reboot")
        || q.contains("grub")
        || q.contains("systemd-analyze")
        || q.contains("slow boot")
        || q.contains("takes long")
    {
        let mut probes = vec!["boot_time".to_string(), "boot_blame".to_string()];

        if q.contains("slow") || q.contains("long") || q.contains("analyze") {
            probes.push("failed_services".to_string());
        }
        if q.contains("grub") || q.contains("loader") {
            probes.push("boot_loader".to_string());
        }
        if q.contains("kernel") {
            probes.push("installed_kernels".to_string());
            probes.push("kernel_cmdline".to_string());
        }
        if q.contains("error") {
            probes.push("journal_errors".to_string());
        }

        return Some(TranslatorTicket {
            intent: classify_intent(q),
            domain: SpecialistDomain::Boot, // v0.0.405: Use Boot domain
            entities: vec![],
            needs_probes: probes,
            clarification_question: None,
            confidence: 0.85,
            answer_contract: Some(AnswerContract::from_query(orig)),
        });
    }
    None
}

pub fn classify_service_query(q: &str, orig: &str) -> Option<TranslatorTicket> {
    if q.contains("service")
        || q.contains("systemd")
        || q.contains("systemctl")
        || q.contains("unit")
        || q.contains("daemon")
        || q.contains("failed")
    {
        let mut probes = vec![];

        if q.contains("failed") || q.contains("error") || q.contains("broken") {
            probes.push("failed_services".to_string());
        }
        if q.contains("running") || q.contains("active") {
            probes.push("running_services".to_string());
        }
        if q.contains("timer") {
            probes.push("systemd_timers".to_string());
        }
        if probes.is_empty() {
            probes.push("running_services".to_string());
            probes.push("failed_services".to_string());
        }

        return Some(TranslatorTicket {
            intent: classify_intent(q),
            domain: SpecialistDomain::Services, // v0.0.405: Use Services domain
            entities: vec![],
            needs_probes: probes,
            clarification_question: None,
            confidence: 0.85,
            answer_contract: Some(AnswerContract::from_query(orig)),
        });
    }
    None
}

/// v0.0.797: Classify "is X installed" / "do I have X" tool check queries
/// Extracts the tool name and generates the correct command_v_<tool> probe
pub fn classify_tool_check_query(q: &str, orig: &str) -> Option<TranslatorTicket> {
    // Match patterns like "is nano installed", "do I have vim", "is docker running"
    // Exclude hardware queries (cpu, ram, memory, gpu, disk, storage, space)
    let hardware_keywords = [
        "cpu", "ram", "memory", "gpu", "disk", "storage", "space", "core", "drive",
    ];
    let is_hardware = hardware_keywords.iter().any(|k| q.contains(k));
    if is_hardware {
        return None;
    }

    // Extract tool name from various patterns
    let tool_name = extract_tool_name_from_query(q)?;

    // Generate the command_v_<tool> probe
    let probe_id = format!("command_v_{}", tool_name);

    // Check if this is a known tool with a registered probe, or generate dynamic command
    let probes = vec![probe_id];

    Some(TranslatorTicket {
        intent: anna_shared::rpc::QueryIntent::Question,
        domain: SpecialistDomain::System,
        entities: vec![tool_name.to_string()],
        needs_probes: probes,
        clarification_question: None,
        confidence: 0.9,
        answer_contract: Some(AnswerContract::from_query(orig)),
    })
}

pub fn classify_package_query(q: &str, orig: &str) -> Option<TranslatorTicket> {
    // v0.0.797: Skip if this looks like a specific tool check (handled by classify_tool_check_query)
    if q.contains("is ") && q.contains(" installed") && !q.contains("package") {
        return None;
    }

    if q.contains("package")
        || q.contains("install")
        || q.contains("pacman")
        || q.contains("apt")
        || q.contains("yay")
        || q.contains("paru")
        || q.contains("update")
        || q.contains("upgrade")
        || q.contains("is installed")
        || q.contains("are installed")
    {
        let mut probes = vec![];

        if q.contains("update") || q.contains("upgrade") || q.contains("available") {
            probes.push("package_updates".to_string());
        }
        if q.contains("installed") || q.contains("list") || q.contains("how many") {
            probes.push("installed_packages".to_string());
            probes.push("package_count".to_string());
        }
        if probes.is_empty() {
            probes.push("package_count".to_string());
            probes.push("package_updates".to_string());
        }

        return Some(TranslatorTicket {
            intent: classify_intent(q),
            domain: SpecialistDomain::Packages,
            entities: vec![],
            needs_probes: probes,
            clarification_question: None,
            confidence: 0.85,
            answer_contract: Some(AnswerContract::from_query(orig)),
        });
    }
    None
}

pub fn classify_security_query(q: &str, orig: &str) -> Option<TranslatorTicket> {
    if q.contains("security")
        || q.contains("firewall")
        || q.contains("iptables")
        || q.contains("nftables")
        || q.contains("permission")
        || q.contains("ssh")
        || q.contains("login")
        || q.contains("user")
        || q.contains("password")
        || q.contains("selinux")
        || q.contains("apparmor")
    {
        let mut probes = vec![];

        if q.contains("firewall") || q.contains("iptables") || q.contains("nftables") {
            probes.push("firewall_status".to_string());
            probes.push("iptables_rules".to_string());
        }
        if q.contains("ssh") || q.contains("login") {
            probes.push("ssh_connections".to_string());
            probes.push("last_logins".to_string());
            probes.push("failed_logins".to_string());
        }
        if q.contains("selinux") {
            probes.push("selinux_status".to_string());
        }
        if q.contains("apparmor") {
            probes.push("apparmor_status".to_string());
        }
        if probes.is_empty() {
            probes.push("firewall_status".to_string());
            probes.push("last_logins".to_string());
        }

        return Some(TranslatorTicket {
            intent: classify_intent(q),
            domain: SpecialistDomain::Security,
            entities: vec![],
            needs_probes: probes,
            clarification_question: None,
            confidence: 0.85,
            answer_contract: Some(AnswerContract::from_query(orig)),
        });
    }
    None
}
