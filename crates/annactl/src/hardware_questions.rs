//! Hardware Questions - Knowledge-First Answers (6.12.1)
//!
//! Answers hardware questions directly from SystemKnowledgeBase
//! without suggesting commands to run.
//!
//! 6.12.2: Extended to include non-hardware system questions (desktop/WM)

use anna_common::system_knowledge::SystemKnowledgeBase;

/// Hardware question types we can answer
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HardwareQuestion {
    WhatComputer,
    CpuInfo,
    RamAmount,
    GpuPresence,
    GpuDetails,
    SoundCardPresence,
}

/// System/desktop question types (6.12.2)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SystemQuestion {
    WhatDesktop,
}

/// Power management question types (6.13.0)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PowerQuestion {
    TlpNotWorking,
    TlpStatus,
}

/// Question classification result
#[derive(Debug)]
pub enum QuestionKind {
    Hardware(HardwareQuestion),
    System(SystemQuestion),  // 6.12.2: Non-hardware system questions
    Power(PowerQuestion),    // 6.13.0: Power management questions
    Other,
}

/// Classify a user question
pub fn classify_question(query: &str) -> QuestionKind {
    let query_lower = query.to_lowercase();

    // What computer / machine
    if query_lower.contains("what computer")
        || query_lower.contains("which computer")
        || query_lower.contains("what machine")
        || query_lower.contains("my computer")
        || query_lower.contains("my machine")
    {
        return QuestionKind::Hardware(HardwareQuestion::WhatComputer);
    }

    // CPU info
    if (query_lower.contains("cpu") || query_lower.contains("processor"))
        && (query_lower.contains("what")
            || query_lower.contains("which")
            || query_lower.contains("model")
            || query_lower.contains("info"))
    {
        return QuestionKind::Hardware(HardwareQuestion::CpuInfo);
    }

    // RAM amount
    if (query_lower.contains("how much ram")
        || query_lower.contains("how much memory")
        || query_lower.contains("total ram")
        || query_lower.contains("total memory"))
        && !query_lower.contains("using")
        && !query_lower.contains("free")
    {
        return QuestionKind::Hardware(HardwareQuestion::RamAmount);
    }

    // GPU presence (discrete)
    if (query_lower.contains("discrete")
        || query_lower.contains("dedicated")
        || query_lower.contains("do i have a")
        || query_lower.contains("have a gpu")
        || query_lower.contains("have a graphic"))
        && (query_lower.contains("gpu")
            || query_lower.contains("graphic")
            || query_lower.contains("video card"))
    {
        return QuestionKind::Hardware(HardwareQuestion::GpuPresence);
    }

    // GPU details
    if (query_lower.contains("what gpu")
        || query_lower.contains("which gpu")
        || query_lower.contains("what graphic")
        || query_lower.contains("which graphic")
        || query_lower.contains("gpu model"))
        && !query_lower.contains("discrete")
        && !query_lower.contains("dedicated")
    {
        return QuestionKind::Hardware(HardwareQuestion::GpuDetails);
    }

    // Sound card
    if (query_lower.contains("sound card")
        || query_lower.contains("audio device")
        || query_lower.contains("sound device"))
        && (query_lower.contains("do i have")
            || query_lower.contains("have a")
            || query_lower.contains("what")
            || query_lower.contains("which"))
    {
        return QuestionKind::Hardware(HardwareQuestion::SoundCardPresence);
    }

    // 6.12.2: Desktop/WM question
    if (query_lower.contains("what desktop")
        || query_lower.contains("which desktop")
        || query_lower.contains("what wm")
        || query_lower.contains("which wm")
        || query_lower.contains("window manager")
        || query_lower.contains("desktop environment"))
        && (query_lower.contains("using")
            || query_lower.contains("am i")
            || query_lower.contains("do i")
            || query_lower.contains("running"))
    {
        return QuestionKind::System(SystemQuestion::WhatDesktop);
    }

    // 6.13.0: TLP power management questions
    if query_lower.contains("tlp") {
        // TLP not working / needs enabling
        if query_lower.contains("not working")
            || query_lower.contains("warning")
            || query_lower.contains("not applied")
            || query_lower.contains("not apply")
            || query_lower.contains("not enabled")
            || query_lower.contains("enable tlp")
            || query_lower.contains("will not apply on boot")
        {
            return QuestionKind::Power(PowerQuestion::TlpNotWorking);
        }

        // TLP status check
        if query_lower.contains("status")
            || query_lower.contains("is tlp running")
            || query_lower.contains("is tlp enabled")
        {
            return QuestionKind::Power(PowerQuestion::TlpStatus);
        }
    }

    QuestionKind::Other
}

/// Handle a hardware question using the knowledge base
///
/// Returns None if the knowledge base doesn't have the required info
pub fn handle_hardware_question(
    question: HardwareQuestion,
    kb: &SystemKnowledgeBase,
) -> Option<String> {
    match question {
        HardwareQuestion::WhatComputer => answer_what_computer(kb),
        HardwareQuestion::CpuInfo => answer_cpu_info(kb),
        HardwareQuestion::RamAmount => answer_ram_amount(kb),
        HardwareQuestion::GpuPresence => answer_gpu_presence(kb),
        HardwareQuestion::GpuDetails => answer_gpu_details(kb),
        HardwareQuestion::SoundCardPresence => answer_sound_card_presence(kb),
    }
}

/// Handle a system question using the knowledge base (6.12.2)
///
/// Returns None if the knowledge base doesn't have the required info
pub fn handle_system_question(
    question: SystemQuestion,
    kb: &SystemKnowledgeBase,
) -> Option<String> {
    match question {
        SystemQuestion::WhatDesktop => answer_what_desktop(kb),
    }
}

fn answer_what_computer(kb: &SystemKnowledgeBase) -> Option<String> {
    let hw = &kb.hardware;

    let mut answer = String::from("From my knowledge of your system:\n\n");

    let mut has_info = false;

    if let Some(ref machine) = hw.machine_model {
        answer.push_str(&format!("- Machine: {}\n", machine));
        has_info = true;
    }

    if let Some(ref cpu) = hw.cpu_model {
        let cores_info = match (hw.cpu_physical_cores, hw.cpu_logical_cores) {
            (Some(phys), Some(log)) if phys != log => {
                format!(" ({} cores / {} threads)", phys, log)
            }
            (Some(cores), _) | (_, Some(cores)) => format!(" ({} cores)", cores),
            _ => String::new(),
        };
        answer.push_str(&format!("- CPU: {}{}\n", cpu, cores_info));
        has_info = true;
    }

    if let Some(ref gpu) = hw.gpu_model {
        let gpu_type_str = hw
            .gpu_type
            .as_ref()
            .map(|t| format!(" ({})", t))
            .unwrap_or_default();
        answer.push_str(&format!("- GPU: {}{}\n", gpu, gpu_type_str));
        has_info = true;
    }

    if let Some(ram) = hw.total_ram_bytes {
        answer.push_str(&format!("- RAM: {} GiB\n", ram / (1024 * 1024 * 1024)));
        has_info = true;
    }

    if !has_info {
        return None;
    }

    answer.push_str("\nThis is all derived from my hardware knowledge base. No extra commands were needed.\n\n");
    answer.push_str(&format!(
        "Source: {}",
        wiki_url_for_hardware_topic(HardwareQuestion::WhatComputer)
    ));

    Some(answer)
}

fn answer_cpu_info(kb: &SystemKnowledgeBase) -> Option<String> {
    let hw = &kb.hardware;

    let cpu_model = hw.cpu_model.as_ref()?;

    let mut answer = format!("Your CPU is: {}\n\n", cpu_model);

    if let (Some(phys), Some(log)) = (hw.cpu_physical_cores, hw.cpu_logical_cores) {
        if phys != log {
            answer.push_str(&format!(
                "It has {} physical cores and {} logical cores (threads).\n\n",
                phys, log
            ));
        } else {
            answer.push_str(&format!("It has {} cores.\n\n", phys));
        }
    } else if let Some(cores) = hw.cpu_physical_cores.or(hw.cpu_logical_cores) {
        answer.push_str(&format!("It has {} cores.\n\n", cores));
    }

    answer.push_str(&format!(
        "Source: {}",
        wiki_url_for_hardware_topic(HardwareQuestion::CpuInfo)
    ));

    Some(answer)
}

fn answer_ram_amount(kb: &SystemKnowledgeBase) -> Option<String> {
    let ram_bytes = kb.hardware.total_ram_bytes?;
    let ram_gib = ram_bytes / (1024 * 1024 * 1024);

    let answer = format!(
        "You have {} GiB of RAM installed.\n\nSource: {}",
        ram_gib,
        wiki_url_for_hardware_topic(HardwareQuestion::RamAmount)
    );

    Some(answer)
}

fn answer_gpu_presence(kb: &SystemKnowledgeBase) -> Option<String> {
    let hw = &kb.hardware;

    let gpu_model = hw.gpu_model.as_ref()?;
    let gpu_type = hw.gpu_type.as_ref()?;

    let answer = if gpu_type == "discrete" {
        format!(
            "Yes, you have a discrete GPU: {}\n\nA discrete GPU is a separate graphics card with its own dedicated memory. According to the Arch Wiki, discrete GPUs typically offer better performance than integrated graphics for gaming and GPU-intensive workloads.\n\nSource: {}",
            gpu_model,
            wiki_url_for_hardware_topic(HardwareQuestion::GpuPresence)
        )
    } else if gpu_type == "integrated" {
        format!(
            "You do not have a discrete GPU. Your graphics are integrated: {}\n\nIntegrated graphics are built into the CPU and share system RAM. According to the Arch Wiki, integrated GPUs are sufficient for general use and light gaming, but discrete GPUs offer better performance for demanding applications.\n\nSource: {}",
            gpu_model,
            wiki_url_for_hardware_topic(HardwareQuestion::GpuPresence)
        )
    } else {
        format!(
            "I found a GPU: {}\n\nHowever, I couldn't determine if it's integrated or discrete.\n\nSource: {}",
            gpu_model,
            wiki_url_for_hardware_topic(HardwareQuestion::GpuPresence)
        )
    };

    Some(answer)
}

fn answer_gpu_details(kb: &SystemKnowledgeBase) -> Option<String> {
    let hw = &kb.hardware;

    let gpu_model = hw.gpu_model.as_ref()?;

    let mut answer = format!("Your GPU is: {}\n\n", gpu_model);

    if let Some(ref gpu_type) = hw.gpu_type {
        answer.push_str(&format!("Type: {}\n\n", gpu_type));
    }

    answer.push_str(&format!(
        "Source: {}",
        wiki_url_for_hardware_topic(HardwareQuestion::GpuDetails)
    ));

    Some(answer)
}

fn answer_sound_card_presence(kb: &SystemKnowledgeBase) -> Option<String> {
    let devices = &kb.hardware.sound_devices;

    if devices.is_empty() {
        Some(format!(
            "I do not see any sound devices in my hardware knowledge base.\n\nIf you believe you have a sound card, it may not be detected or may require additional drivers.\n\nSource: {}",
            wiki_url_for_hardware_topic(HardwareQuestion::SoundCardPresence)
        ))
    } else {
        let mut answer = format!("Yes, I see these sound devices:\n\n");
        for device in devices {
            answer.push_str(&format!("- {}\n", device));
        }
        answer.push_str(&format!(
            "\nSource: {}",
            wiki_url_for_hardware_topic(HardwareQuestion::SoundCardPresence)
        ));
        Some(answer)
    }
}

/// 6.12.2: Answer desktop/WM question
fn answer_what_desktop(kb: &SystemKnowledgeBase) -> Option<String> {
    let wm_or_de = kb.wallpaper.wm_or_de.as_ref()?;

    let mut answer = format!("You are using: {}\n\n", wm_or_de);

    // Add wallpaper tool info if available
    if let Some(ref tool) = kb.wallpaper.wallpaper_tool {
        answer.push_str(&format!("Wallpaper tool: {}\n\n", tool));
    }

    answer.push_str("This information comes from my system knowledge base.\n\n");
    answer.push_str(&format!(
        "Source: {}",
        wiki_url_for_system_topic(SystemQuestion::WhatDesktop)
    ));

    Some(answer)
}

/// Get Arch Wiki URL for a hardware topic
pub fn wiki_url_for_hardware_topic(topic: HardwareQuestion) -> &'static str {
    match topic {
        HardwareQuestion::GpuPresence | HardwareQuestion::GpuDetails => {
            "https://wiki.archlinux.org/title/Hardware_video_acceleration"
        }
        HardwareQuestion::SoundCardPresence => {
            "https://wiki.archlinux.org/title/Advanced_Linux_Sound_Architecture"
        }
        HardwareQuestion::RamAmount | HardwareQuestion::WhatComputer | HardwareQuestion::CpuInfo => {
            "https://wiki.archlinux.org/title/System_maintenance#Inspect_hardware"
        }
    }
}

/// Get Arch Wiki URL for a system topic (6.12.2)
pub fn wiki_url_for_system_topic(topic: SystemQuestion) -> &'static str {
    match topic {
        SystemQuestion::WhatDesktop => {
            "https://wiki.archlinux.org/title/Desktop_environment"
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use anna_common::system_knowledge::SystemKnowledgeBase;

    #[test]
    fn test_classify_what_computer() {
        match classify_question("what computer do I have?") {
            QuestionKind::Hardware(HardwareQuestion::WhatComputer) => {}
            _ => panic!("Expected WhatComputer"),
        }
    }

    #[test]
    fn test_classify_discrete_gpu() {
        match classify_question("do I have a discrete graphic card?") {
            QuestionKind::Hardware(HardwareQuestion::GpuPresence) => {}
            _ => panic!("Expected GpuPresence"),
        }
    }

    #[test]
    fn test_classify_ram() {
        match classify_question("how much ram do I have?") {
            QuestionKind::Hardware(HardwareQuestion::RamAmount) => {}
            _ => panic!("Expected RamAmount"),
        }
    }

    #[test]
    fn test_classify_sound_card() {
        match classify_question("do I have a sound card?") {
            QuestionKind::Hardware(HardwareQuestion::SoundCardPresence) => {}
            _ => panic!("Expected SoundCardPresence"),
        }
    }

    #[test]
    fn test_answer_discrete_gpu_yes() {
        let mut kb = SystemKnowledgeBase::default();
        kb.hardware.gpu_model = Some("NVIDIA GeForce RTX 4060".to_string());
        kb.hardware.gpu_type = Some("discrete".to_string());

        let answer = handle_hardware_question(HardwareQuestion::GpuPresence, &kb);
        assert!(answer.is_some());
        let text = answer.unwrap();
        assert!(text.contains("Yes"));
        assert!(text.contains("discrete"));
        assert!(text.contains("RTX 4060"));
        assert!(!text.contains("run lspci"));
        assert!(!text.contains("command"));
    }

    #[test]
    fn test_answer_integrated_gpu() {
        let mut kb = SystemKnowledgeBase::default();
        kb.hardware.gpu_model = Some("Intel UHD Graphics 630".to_string());
        kb.hardware.gpu_type = Some("integrated".to_string());

        let answer = handle_hardware_question(HardwareQuestion::GpuPresence, &kb);
        assert!(answer.is_some());
        let text = answer.unwrap();
        assert!(text.contains("do not have a discrete GPU"));
        assert!(text.contains("integrated"));
        assert!(text.contains("UHD"));
        assert!(!text.contains("run lspci"));
    }

    #[test]
    fn test_answer_ram_amount() {
        let mut kb = SystemKnowledgeBase::default();
        kb.hardware.total_ram_bytes = Some(32 * 1024 * 1024 * 1024);

        let answer = handle_hardware_question(HardwareQuestion::RamAmount, &kb);
        assert!(answer.is_some());
        let text = answer.unwrap();
        assert!(text.contains("32 GiB"));
        assert!(!text.contains("free -h"));
    }

    #[test]
    fn test_answer_sound_card_present() {
        let mut kb = SystemKnowledgeBase::default();
        kb.hardware.sound_devices = vec!["Intel HDA".to_string(), "NVIDIA HDMI".to_string()];

        let answer = handle_hardware_question(HardwareQuestion::SoundCardPresence, &kb);
        assert!(answer.is_some());
        let text = answer.unwrap();
        assert!(text.contains("Yes"));
        assert!(text.contains("Intel HDA"));
        assert!(text.contains("NVIDIA HDMI"));
        assert!(!text.contains("lspci"));
    }

    #[test]
    fn test_wiki_urls() {
        let url = wiki_url_for_hardware_topic(HardwareQuestion::GpuPresence);
        assert!(url.contains("archlinux.org"));
        assert!(url.contains("Hardware_video_acceleration"));

        let url2 = wiki_url_for_hardware_topic(HardwareQuestion::SoundCardPresence);
        assert!(url2.contains("archlinux.org"));
        assert!(url2.contains("Advanced_Linux_Sound_Architecture"));
    }

    // 6.13.0: Power question classifier tests
    #[test]
    fn test_classify_tlp_not_working() {
        match classify_question("tlp not working") {
            QuestionKind::Power(PowerQuestion::TlpNotWorking) => {}
            _ => panic!("Expected TlpNotWorking"),
        }
    }

    #[test]
    fn test_classify_tlp_enable() {
        match classify_question("how do I enable tlp") {
            QuestionKind::Power(PowerQuestion::TlpNotWorking) => {}
            _ => panic!("Expected TlpNotWorking"),
        }
    }

    #[test]
    fn test_classify_tlp_status() {
        match classify_question("is tlp running") {
            QuestionKind::Power(PowerQuestion::TlpStatus) => {}
            _ => panic!("Expected TlpStatus"),
        }
    }

    #[test]
    fn test_classify_non_tlp() {
        match classify_question("how do I install docker") {
            QuestionKind::Other => {}
            _ => panic!("Expected Other for non-TLP question"),
        }
    }
}
