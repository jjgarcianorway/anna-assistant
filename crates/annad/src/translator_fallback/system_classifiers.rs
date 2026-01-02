//! System-level classification functions
//!
//! Handles health, storage, memory, CPU, process, hardware, and general system queries

use anna_shared::answer_contract::AnswerContract;
use anna_shared::rpc::{QueryIntent, SpecialistDomain, TranslatorTicket};

use super::helpers::classify_intent;

pub fn classify_health_query(q: &str, orig: &str) -> Option<TranslatorTicket> {
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

pub fn classify_storage_query(q: &str, orig: &str) -> Option<TranslatorTicket> {
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

pub fn classify_memory_query(q: &str, orig: &str) -> Option<TranslatorTicket> {
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

pub fn classify_cpu_query(q: &str, orig: &str) -> Option<TranslatorTicket> {
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

pub fn classify_process_query(q: &str, orig: &str) -> Option<TranslatorTicket> {
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

pub fn classify_hardware_query(q: &str, orig: &str) -> Option<TranslatorTicket> {
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

pub fn classify_log_query(q: &str, orig: &str) -> Option<TranslatorTicket> {
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

pub fn classify_config_query(q: &str, orig: &str) -> Option<TranslatorTicket> {
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

pub fn classify_docker_query(q: &str, orig: &str) -> Option<TranslatorTicket> {
    if q.contains("docker") || q.contains("container") || q.contains("podman") {
        return Some(TranslatorTicket {
            intent: classify_intent(q),
            domain: SpecialistDomain::System,
            entities: vec![],
            needs_probes: vec!["docker_containers".to_string(), "docker_images".to_string()],
            clarification_question: None,
            confidence: 0.9,
            answer_contract: Some(AnswerContract::from_query(orig)),
        });
    }
    None
}

pub fn classify_user_query(q: &str, orig: &str) -> Option<TranslatorTicket> {
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
