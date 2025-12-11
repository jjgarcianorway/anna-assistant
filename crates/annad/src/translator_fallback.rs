//! Comprehensive keyword-based translator (v0.0.402).
//!
//! This is now the PRIMARY classification path. The LLM translator is only used
//! as a fallback for truly ambiguous queries. This approach is:
//! - Fast: No LLM calls needed for common queries
//! - Reliable: Deterministic, testable behavior
//! - Accurate: Domain-specific probe selection
//!
//! v0.0.164: Extracted from translator.rs
//! v0.0.402: Massive expansion to handle 95%+ of common IT queries

use anna_shared::answer_contract::AnswerContract;
use anna_shared::rpc::{QueryIntent, SpecialistDomain, TranslatorTicket};
use tracing::info;

/// Main entry point - classify query using comprehensive keyword matching
pub fn translate_fallback(query: &str) -> TranslatorTicket {
    let q = query.to_lowercase();
    let stripped = strip_greetings(&q);

    // Try classification in order of specificity
    if let Some(ticket) = classify_health_query(&stripped, query) {
        return ticket;
    }
    if let Some(ticket) = classify_storage_query(&stripped, query) {
        return ticket;
    }
    if let Some(ticket) = classify_memory_query(&stripped, query) {
        return ticket;
    }
    if let Some(ticket) = classify_cpu_query(&stripped, query) {
        return ticket;
    }
    if let Some(ticket) = classify_process_query(&stripped, query) {
        return ticket;
    }
    if let Some(ticket) = classify_network_query(&stripped, query) {
        return ticket;
    }
    if let Some(ticket) = classify_graphics_query(&stripped, query) {
        return ticket;
    }
    if let Some(ticket) = classify_audio_query(&stripped, query) {
        return ticket;
    }
    if let Some(ticket) = classify_bluetooth_query(&stripped, query) {
        return ticket;
    }
    if let Some(ticket) = classify_boot_query(&stripped, query) {
        return ticket;
    }
    if let Some(ticket) = classify_service_query(&stripped, query) {
        return ticket;
    }
    if let Some(ticket) = classify_package_query(&stripped, query) {
        return ticket;
    }
    if let Some(ticket) = classify_hardware_query(&stripped, query) {
        return ticket;
    }
    if let Some(ticket) = classify_security_query(&stripped, query) {
        return ticket;
    }
    if let Some(ticket) = classify_log_query(&stripped, query) {
        return ticket;
    }
    if let Some(ticket) = classify_config_query(&stripped, query) {
        return ticket;
    }
    if let Some(ticket) = classify_docker_query(&stripped, query) {
        return ticket;
    }
    if let Some(ticket) = classify_user_query(&stripped, query) {
        return ticket;
    }

    // Generic system fallback
    info!("Fallback: no keyword match, using generic system domain");
    TranslatorTicket {
        intent: classify_intent(&stripped),
        domain: SpecialistDomain::System,
        entities: Vec::new(),
        needs_probes: vec!["memory_info".to_string(), "disk_usage".to_string()],
        clarification_question: None,
        confidence: 0.4,
        answer_contract: Some(AnswerContract::from_query(query)),
    }
}

// ============================================================================
// Classification functions by category
// ============================================================================

fn classify_health_query(q: &str, orig: &str) -> Option<TranslatorTicket> {
    if q.contains("how is my computer")
        || q.contains("how's my computer")
        || q.contains("how is the system")
        || q.contains("any errors")
        || q.contains("any problems")
        || q.contains("problems so far")
        || q.contains("what's wrong")
        || q.contains("is everything ok")
        || q.contains("check my system")
        || q.contains("health")
        || q.contains("status report")
        || q.contains("overview")
        || q.contains("system summary")
        || q.trim() == "status"
        || q.trim() == "report"
    {
        return Some(TranslatorTicket {
            intent: QueryIntent::Question,
            domain: SpecialistDomain::System,
            entities: vec![],
            needs_probes: vec![
                "memory_info".into(),
                "disk_usage".into(),
                "cpu_info".into(),
                "failed_services".into(),
                "load_average".into(),
            ],
            clarification_question: None,
            confidence: 0.9,
            answer_contract: Some(AnswerContract::from_query(orig)),
        });
    }
    None
}

fn classify_storage_query(q: &str, orig: &str) -> Option<TranslatorTicket> {
    // Disk space queries
    if q.contains("disk")
        || q.contains("storage")
        || q.contains("space")
        || q.contains("folder")
        || q.contains("directory")
        || q.contains("mount")
        || q.contains("partition")
        || q.contains("drive")
        || q.contains("filesystem")
        || q.contains("du ")
        || q.contains("df ")
        || q.contains("lsblk")
        || q.contains("taking up space")
        || q.contains("using space")
        || q.contains("largest files")
        || q.contains("largest folder")
        || q.contains("biggest folder")
        || q.contains("what's taking")
        || q.contains("what is taking")
        || q.contains("how much space")
    {
        // Determine specific probes based on query
        let mut probes = vec!["disk_usage".to_string()];

        if q.contains("largest")
            || q.contains("biggest")
            || q.contains("taking")
            || q.contains("using space")
        {
            probes.push("largest_dirs".to_string());
            probes.push("largest_home".to_string());
        }
        if q.contains("mount") || q.contains("partition") {
            probes.push("findmnt".to_string());
            probes.push("block_devices".to_string());
        }
        if q.contains("block") || q.contains("lsblk") {
            probes.push("block_devices".to_string());
        }
        if q.contains("ssd") || q.contains("nvme") || q.contains("hdd") {
            probes.push("block_devices".to_string());
        }

        return Some(TranslatorTicket {
            intent: classify_intent(q),
            domain: SpecialistDomain::Storage,
            entities: vec![],
            needs_probes: probes,
            clarification_question: None,
            confidence: 0.85,
            answer_contract: Some(AnswerContract::from_query(orig)),
        });
    }
    None
}

fn classify_memory_query(q: &str, orig: &str) -> Option<TranslatorTicket> {
    if q.contains("memory")
        || q.contains("ram")
        || q.contains("swap")
        || q.contains("free -")
        || (q.contains("using") && q.contains("gb"))
    {
        let mut probes = vec!["memory_info".to_string()];

        if q.contains("process") || q.contains("using") || q.contains("consuming") {
            probes.push("top_memory".to_string());
        }
        if q.contains("swap") {
            probes.push("swap_files".to_string());
        }

        return Some(TranslatorTicket {
            intent: QueryIntent::Question,
            domain: SpecialistDomain::System,
            entities: vec![],
            needs_probes: probes,
            clarification_question: None,
            confidence: 0.9,
            answer_contract: Some(AnswerContract::from_query(orig)),
        });
    }
    None
}

fn classify_cpu_query(q: &str, orig: &str) -> Option<TranslatorTicket> {
    if q.contains("cpu")
        || q.contains("processor")
        || q.contains("core")
        || q.contains("load average")
        || q.contains("lscpu")
    {
        let mut probes = vec!["cpu_info".to_string()];

        if q.contains("usage") || q.contains("load") || q.contains("busy") {
            probes.push("load_average".to_string());
            probes.push("top_cpu".to_string());
        }
        if q.contains("temperature") || q.contains("temp") || q.contains("hot") {
            probes.push("sensors_temp".to_string());
        }
        if q.contains("frequency") || q.contains("speed") || q.contains("ghz") {
            probes.push("cpu_frequency".to_string());
            probes.push("cpu_governor".to_string());
        }

        return Some(TranslatorTicket {
            intent: QueryIntent::Question,
            domain: SpecialistDomain::System,
            entities: vec![],
            needs_probes: probes,
            clarification_question: None,
            confidence: 0.9,
            answer_contract: Some(AnswerContract::from_query(orig)),
        });
    }
    None
}

fn classify_process_query(q: &str, orig: &str) -> Option<TranslatorTicket> {
    // v0.0.403: Be more specific - "running" alone should not trigger process query
    // as it could be "is X running" for any service
    let is_process_query = q.contains("process")
        || q.contains("processes")
        || q.contains("ps ")
        || q.contains("top ")
        || q.contains("htop")
        || q.contains("using cpu")
        || q.contains("using memory")
        || q.contains("what's eating")
        || q.contains("resource hog")
        || q.contains("hogging")
        // "running" only counts if it's in a process-related context
        || (q.contains("running") && (q.contains("what") || q.contains("which") || q.contains("show") || q.contains("list")));

    if is_process_query {
        let mut probes = vec![];

        if q.contains("cpu") || q.contains("eating") {
            probes.push("top_cpu".to_string());
        }
        if q.contains("memory") || q.contains("ram") || q.contains("eating") {
            probes.push("top_memory".to_string());
        }
        if probes.is_empty() {
            probes.push("top_cpu".to_string());
            probes.push("top_memory".to_string());
        }

        return Some(TranslatorTicket {
            intent: QueryIntent::Question,
            domain: SpecialistDomain::System,
            entities: vec![],
            needs_probes: probes,
            clarification_question: None,
            confidence: 0.9,
            answer_contract: Some(AnswerContract::from_query(orig)),
        });
    }
    None
}

fn classify_network_query(q: &str, orig: &str) -> Option<TranslatorTicket> {
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

fn classify_graphics_query(q: &str, orig: &str) -> Option<TranslatorTicket> {
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
        let mut probes = vec!["gpu_drivers".to_string(), "display_server".to_string()];

        if q.contains("driver") || q.contains("nvidia") || q.contains("amd") {
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
        if q.contains("cuda") {
            probes.push("cuda_installed".to_string());
        }
        if q.contains("xorg") || q.contains("tearing") || q.contains("screen") {
            probes.push("xorg_log".to_string());
        }
        if q.contains("temperature") || q.contains("temp") {
            probes.push("sensors_temp".to_string());
            probes.push("gpu_memory".to_string());
        }

        return Some(TranslatorTicket {
            intent: classify_intent(q),
            domain: SpecialistDomain::System, // Graphics is a system concern
            entities: vec![],
            needs_probes: probes,
            clarification_question: None,
            confidence: 0.85,
            answer_contract: Some(AnswerContract::from_query(orig)),
        });
    }
    None
}

fn classify_audio_query(q: &str, orig: &str) -> Option<TranslatorTicket> {
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
            domain: SpecialistDomain::System,
            entities: vec![],
            needs_probes: vec![
                "audio_devices".to_string(),
                "pactl_cards".to_string(),
                "lspci_audio".to_string(),
            ],
            clarification_question: None,
            confidence: 0.85,
            answer_contract: Some(AnswerContract::from_query(orig)),
        });
    }
    None
}

fn classify_bluetooth_query(q: &str, orig: &str) -> Option<TranslatorTicket> {
    if q.contains("bluetooth") || q.contains("bt ") || q.contains("pair") {
        // v0.0.403: Distinguish between service status vs device queries
        let probes = if q.contains("running") || q.contains("active") || q.contains("status")
            || q.contains("service") || q.contains("start") || q.contains("stop")
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

fn classify_boot_query(q: &str, orig: &str) -> Option<TranslatorTicket> {
    if q.contains("boot")
        || q.contains("startup")
        || q.contains("uptime")
        || q.contains("reboot")
        || q.contains("grub")
        || q.contains("systemd-analyze")
        || q.contains("slow boot")
        || q.contains("takes long")
    {
        let mut probes = vec!["uptime".to_string(), "boot_time".to_string()];

        if q.contains("slow") || q.contains("long") || q.contains("analyze") {
            // Boot time analysis probes would go here
            probes.push("running_services".to_string());
        }
        if q.contains("grub") || q.contains("loader") {
            probes.push("boot_loader".to_string());
        }
        if q.contains("kernel") {
            probes.push("installed_kernels".to_string());
            probes.push("kernel_cmdline".to_string());
        }
        if q.contains("last") {
            probes.push("last_boot".to_string());
        }

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

fn classify_service_query(q: &str, orig: &str) -> Option<TranslatorTicket> {
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

fn classify_package_query(q: &str, orig: &str) -> Option<TranslatorTicket> {
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

fn classify_hardware_query(q: &str, orig: &str) -> Option<TranslatorTicket> {
    if q.contains("hardware")
        || q.contains("lspci")
        || q.contains("lsusb")
        || q.contains("usb")
        || q.contains("pci")
        || q.contains("device")
        || q.contains("webcam")
        || q.contains("camera")
        || q.contains("printer")
        || q.contains("sensor")
        || q.contains("temperature")
        || q.contains("battery")
    {
        let mut probes = vec![];

        if q.contains("usb") || q.contains("webcam") || q.contains("camera") {
            probes.push("lsusb".to_string());
        }
        if q.contains("pci") || q.contains("device") {
            probes.push("pci_devices".to_string());
        }
        if q.contains("temperature") || q.contains("sensor") || q.contains("temp") {
            probes.push("sensors_temp".to_string());
        }
        if q.contains("battery") || q.contains("power") || q.contains("charge") {
            probes.push("battery".to_string());
        }
        if q.contains("printer") {
            probes.push("printer_status".to_string());
        }
        if probes.is_empty() {
            probes.push("pci_devices".to_string());
            probes.push("lsusb".to_string());
        }

        return Some(TranslatorTicket {
            intent: QueryIntent::Question,
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

fn classify_security_query(q: &str, orig: &str) -> Option<TranslatorTicket> {
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

fn classify_log_query(q: &str, orig: &str) -> Option<TranslatorTicket> {
    if q.contains("log")
        || q.contains("journal")
        || q.contains("journalctl")
        || q.contains("dmesg")
        || q.contains("syslog")
        || q.contains("error log")
    {
        let mut probes = vec![];

        if q.contains("error") {
            probes.push("journal_errors".to_string());
        }
        if q.contains("warning") {
            probes.push("journal_warnings".to_string());
        }
        if q.contains("dmesg") || q.contains("kernel") {
            probes.push("dmesg_errors".to_string());
        }
        if probes.is_empty() {
            probes.push("journal_errors".to_string());
            probes.push("journal_warnings".to_string());
        }

        return Some(TranslatorTicket {
            intent: QueryIntent::Investigate,
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

fn classify_config_query(q: &str, orig: &str) -> Option<TranslatorTicket> {
    if q.contains("config")
        || q.contains("configuration")
        || q.contains(".conf")
        || q.contains("vimrc")
        || q.contains("bashrc")
        || q.contains("zshrc")
        || q.contains("nvim")
        || q.contains("hypr")
        || q.contains("setting")
    {
        let mut probes = vec![];

        if q.contains("vim") || q.contains("vimrc") {
            probes.push("vimrc_content".to_string());
        }
        if q.contains("nvim") || q.contains("neovim") {
            probes.push("nvim_config".to_string());
        }
        if q.contains("bash") || q.contains("bashrc") {
            probes.push("bashrc_content".to_string());
        }
        if q.contains("zsh") || q.contains("zshrc") {
            probes.push("zshrc_content".to_string());
        }
        if q.contains("hypr") {
            probes.push("desktop_wallpaper".to_string()); // Uses hyprpaper.conf
        }
        if probes.is_empty() {
            probes.push("os_release".to_string());
            probes.push("environment_variables".to_string());
        }

        return Some(TranslatorTicket {
            intent: QueryIntent::Question,
            domain: SpecialistDomain::System,
            entities: vec![],
            needs_probes: probes,
            clarification_question: None,
            confidence: 0.8,
            answer_contract: Some(AnswerContract::from_query(orig)),
        });
    }
    None
}

fn classify_docker_query(q: &str, orig: &str) -> Option<TranslatorTicket> {
    if q.contains("docker") || q.contains("container") || q.contains("podman") {
        return Some(TranslatorTicket {
            intent: classify_intent(q),
            domain: SpecialistDomain::System,
            entities: vec![],
            needs_probes: vec![
                "docker_containers".to_string(),
                "docker_images".to_string(),
            ],
            clarification_question: None,
            confidence: 0.9,
            answer_contract: Some(AnswerContract::from_query(orig)),
        });
    }
    None
}

fn classify_user_query(q: &str, orig: &str) -> Option<TranslatorTicket> {
    if q.contains("who is logged")
        || q.contains("who's logged")
        || q.contains("logged in")
        || q.contains("current user")
        || q.contains("my username")
        || q.contains("whoami")
    {
        return Some(TranslatorTicket {
            intent: QueryIntent::Question,
            domain: SpecialistDomain::System,
            entities: vec![],
            needs_probes: vec![
                "current_user".to_string(),
                "who".to_string(),
                "loginctl_sessions".to_string(),
            ],
            clarification_question: None,
            confidence: 0.9,
            answer_contract: Some(AnswerContract::from_query(orig)),
        });
    }
    None
}

// ============================================================================
// Helper functions
// ============================================================================

fn classify_intent(q: &str) -> QueryIntent {
    if q.contains("install")
        || q.contains("start")
        || q.contains("stop")
        || q.contains("restart")
        || q.contains("configure")
        || q.contains("enable")
        || q.contains("disable")
        || q.contains("update")
    {
        QueryIntent::Request
    } else if q.contains("why")
        || q.contains("debug")
        || q.contains("fix")
        || q.contains("error")
        || q.contains("problem")
        || q.contains("issue")
        || q.contains("not working")
        || q.contains("broken")
    {
        QueryIntent::Investigate
    } else {
        QueryIntent::Question
    }
}

fn strip_greetings(q: &str) -> String {
    let patterns = [
        "hello", "hi ", "hey ", "good morning", "good afternoon", "good evening",
        "anna", ":)", ":(", ";)", ":d", ":p", "!", "?", "…", "...", "please",
        "can you", "could you", "would you", "tell me", "show me",
    ];
    let mut result = q.to_string();
    for p in patterns {
        result = result.replace(p, " ");
    }
    result.split_whitespace().collect::<Vec<_>>().join(" ")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_graphics_classification() {
        let ticket = translate_fallback("screen tearing issues");
        assert_eq!(ticket.domain, SpecialistDomain::System);
        assert!(ticket.needs_probes.contains(&"gpu_drivers".to_string()));
    }

    #[test]
    fn test_webcam_classification() {
        let ticket = translate_fallback("is my webcam working?");
        assert!(ticket.needs_probes.contains(&"lsusb".to_string()));
    }

    #[test]
    fn test_bluetooth_classification() {
        // General bluetooth query - should include both service and devices
        let ticket = translate_fallback("bluetooth not working");
        assert!(ticket.needs_probes.contains(&"bluetooth_service".to_string()));
        assert!(ticket.needs_probes.contains(&"bluetooth_devices".to_string()));
    }

    #[test]
    fn test_bluetooth_service_status() {
        // Service status query - should only include service probe
        let ticket = translate_fallback("is bluetooth running");
        assert!(ticket.needs_probes.contains(&"bluetooth_service".to_string()));
        assert!(!ticket.needs_probes.contains(&"bluetooth_devices".to_string()));
    }

    #[test]
    fn test_audio_classification() {
        let ticket = translate_fallback("no sound from speakers");
        assert!(ticket.needs_probes.contains(&"audio_devices".to_string()));
    }

    #[test]
    fn test_disk_space_classification() {
        let ticket = translate_fallback("what is taking up disk space?");
        assert_eq!(ticket.domain, SpecialistDomain::Storage);
        assert!(ticket.needs_probes.contains(&"largest_dirs".to_string()));
    }

    #[test]
    fn test_boot_classification() {
        let ticket = translate_fallback("my system boots slowly");
        assert!(ticket.needs_probes.contains(&"boot_time".to_string()));
    }
}
