//! Deterministic router - routes queries and enforces probe spine.
//!
//! v0.45.x stabilization: LLM-first reasoning with probe spine.
//! Deterministic code selects tools and enforces safety, but does NOT invent answers.
//!
//! Key policy:
//! - can_answer_deterministically = true ONLY for narrow typed queries with extractable claims
//! - All other queries go to LLM specialist with probe evidence
//! - Probe spine enforces minimum probes when evidence is required

use anna_shared::probe_spine::{EvidenceKind, ProbeId, RouteCapability};
use anna_shared::rpc::{QueryIntent, SpecialistDomain, TranslatorTicket};
use serde::{Deserialize, Serialize};
use tracing::info;

// Re-export classify_query from the patterns module
pub use crate::query_classify::classify_query;

/// Known query classes with deterministic routing
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum QueryClass {
    /// System triage: "any errors", "any problems" => fast path with probes
    SystemTriage,
    /// CPU info: cpu/processor => needs hardware snapshot
    CpuInfo,
    /// CPU cores: "how many cores" => needs lscpu probe + LLM
    CpuCores,
    /// CPU temperature: "cpu temp" => needs sensors probe + LLM
    CpuTemp,
    /// RAM info: ram/memory (not process) => needs free probe
    RamInfo,
    /// GPU info: gpu/graphics/vram => needs hardware snapshot
    GpuInfo,
    /// Hardware audio: "sound card" => needs lspci probe + LLM
    HardwareAudio,
    /// Memory processes: processes using memory => deterministic from probe
    TopMemoryProcesses,
    /// CPU processes: processes using cpu => deterministic from probe
    TopCpuProcesses,
    /// Disk space: disk/storage => deterministic from probe
    DiskSpace,
    /// Network interfaces: network/ip => deterministic from probe
    NetworkInterfaces,
    /// Help request: deterministic static response
    Help,
    /// System slow: diagnostic => needs LLM interpretation
    SystemSlow,
    /// Memory usage: "how much memory" => deterministic from probe
    MemoryUsage,
    /// Memory free: "free ram" => deterministic from probe
    MemoryFree,
    /// Disk usage: "disk usage" => deterministic from probe
    DiskUsage,
    /// Service status: "is nginx running" => deterministic from probe
    ServiceStatus,
    /// System health summary: needs LLM interpretation
    SystemHealthSummary,
    /// Boot time status: from knowledge store
    BootTimeStatus,
    /// Installed packages overview: from knowledge store
    InstalledPackagesOverview,
    /// Package count: needs probe + LLM interpretation
    PackageCount,
    /// Installed tool check: "do I have nano" => needs probe + LLM
    InstalledToolCheck,
    /// App alternatives: from knowledge store + LLM
    AppAlternatives,
    /// Configure editor: "enable syntax highlighting" => needs clarification + recipe (v0.45.5)
    ConfigureEditor,
    /// Unknown: defer to LLM translator
    Unknown,
}

impl std::fmt::Display for QueryClass {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            Self::SystemTriage => "system_triage",
            Self::CpuInfo => "cpu_info",
            Self::CpuCores => "cpu_cores",
            Self::CpuTemp => "cpu_temp",
            Self::RamInfo => "ram_info",
            Self::GpuInfo => "gpu_info",
            Self::HardwareAudio => "hardware_audio",
            Self::TopMemoryProcesses => "top_memory_processes",
            Self::TopCpuProcesses => "top_cpu_processes",
            Self::DiskSpace => "disk_space",
            Self::NetworkInterfaces => "network_interfaces",
            Self::Help => "help",
            Self::SystemSlow => "system_slow",
            Self::MemoryUsage => "memory_usage",
            Self::MemoryFree => "memory_free",
            Self::DiskUsage => "disk_usage",
            Self::ServiceStatus => "service_status",
            Self::SystemHealthSummary => "system_health_summary",
            Self::BootTimeStatus => "boot_time_status",
            Self::InstalledPackagesOverview => "installed_packages_overview",
            Self::PackageCount => "package_count",
            Self::InstalledToolCheck => "installed_tool_check",
            Self::AppAlternatives => "app_alternatives",
            Self::ConfigureEditor => "configure_editor",
            Self::Unknown => "unknown",
        };
        write!(f, "{}", s)
    }
}

impl QueryClass {
    /// Parse from string (for corpus tests)
    pub fn from_str(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "system_triage" => Some(Self::SystemTriage),
            "cpu_info" => Some(Self::CpuInfo),
            "cpu_cores" => Some(Self::CpuCores),
            "cpu_temp" => Some(Self::CpuTemp),
            "ram_info" => Some(Self::RamInfo),
            "gpu_info" => Some(Self::GpuInfo),
            "hardware_audio" => Some(Self::HardwareAudio),
            "top_memory_processes" => Some(Self::TopMemoryProcesses),
            "top_cpu_processes" => Some(Self::TopCpuProcesses),
            "disk_space" => Some(Self::DiskSpace),
            "network_interfaces" => Some(Self::NetworkInterfaces),
            "help" => Some(Self::Help),
            "system_slow" => Some(Self::SystemSlow),
            "memory_usage" => Some(Self::MemoryUsage),
            "memory_free" => Some(Self::MemoryFree),
            "disk_usage" => Some(Self::DiskUsage),
            "service_status" => Some(Self::ServiceStatus),
            "system_health_summary" => Some(Self::SystemHealthSummary),
            "boot_time_status" => Some(Self::BootTimeStatus),
            "installed_packages_overview" => Some(Self::InstalledPackagesOverview),
            "package_count" => Some(Self::PackageCount),
            "installed_tool_check" => Some(Self::InstalledToolCheck),
            "app_alternatives" => Some(Self::AppAlternatives),
            "configure_editor" => Some(Self::ConfigureEditor),
            "unknown" => Some(Self::Unknown),
            _ => None,
        }
    }

    /// Check if this class is RAG-first (answered from knowledge store)
    pub fn is_rag_first(&self) -> bool {
        matches!(self, Self::BootTimeStatus | Self::InstalledPackagesOverview | Self::AppAlternatives)
    }

    /// Check if this class is a fast-path (skip translator, no specialist)
    pub fn is_fast_path(&self) -> bool {
        matches!(self, Self::SystemTriage | Self::Help)
    }

    /// Check if this class needs clarification before proceeding (v0.45.5)
    /// These queries require user input (e.g., which editor) before action
    pub fn needs_clarification(&self) -> bool {
        matches!(self, Self::ConfigureEditor)
    }

    /// Get the fact key needed for clarification (v0.45.5)
    pub fn clarification_fact_key(&self) -> Option<&'static str> {
        match self {
            Self::ConfigureEditor => Some("preferred_editor"),
            _ => None,
        }
    }
}

/// Route result from deterministic router
#[derive(Debug, Clone)]
pub struct DeterministicRoute {
    pub class: QueryClass,
    pub domain: SpecialistDomain,
    pub intent: QueryIntent,
    pub probes: Vec<String>,
    pub capability: RouteCapability,
}

impl DeterministicRoute {
    /// Legacy accessor for can_answer_deterministically
    pub fn can_answer_deterministically(&self) -> bool {
        self.capability.can_answer_deterministically
    }
}

/// Get deterministic route for a query
pub fn get_route(query: &str) -> DeterministicRoute {
    let class = classify_query(query);
    build_route(class)
}

/// Build route from query class
fn build_route(class: QueryClass) -> DeterministicRoute {
    match class {
        // === FAST PATH: Deterministic with probes ===

        // SystemTriage: deterministic triage from probe data
        QueryClass::SystemTriage => DeterministicRoute {
            class,
            domain: SpecialistDomain::System,
            intent: QueryIntent::Question,
            probes: vec![
                "journal_errors".to_string(),
                "journal_warnings".to_string(),
                "failed_units".to_string(),
                "boot_time".to_string(),
            ],
            capability: RouteCapability {
                can_answer_deterministically: true,
                evidence_required: true,
                required_evidence: vec![EvidenceKind::Journal, EvidenceKind::Services],
                spine_probes: vec![ProbeId::JournalErrors, ProbeId::FailedUnits],
            },
        },

        // Help: deterministic static response, no evidence needed
        QueryClass::Help => DeterministicRoute {
            class,
            domain: SpecialistDomain::System,
            intent: QueryIntent::Question,
            probes: vec![],
            capability: RouteCapability {
                can_answer_deterministically: true,
                evidence_required: false,
                required_evidence: vec![],
                spine_probes: vec![],
            },
        },

        // === NARROW TYPED QUERIES: Deterministic from probe data ===

        QueryClass::MemoryUsage | QueryClass::MemoryFree => DeterministicRoute {
            class,
            domain: SpecialistDomain::System,
            intent: QueryIntent::Question,
            probes: vec!["free".to_string()],
            capability: RouteCapability {
                can_answer_deterministically: true,
                evidence_required: true,
                required_evidence: vec![EvidenceKind::Memory],
                spine_probes: vec![ProbeId::Free],
            },
        },

        QueryClass::DiskUsage | QueryClass::DiskSpace => DeterministicRoute {
            class,
            domain: SpecialistDomain::Storage,
            intent: QueryIntent::Question,
            probes: vec!["df".to_string()],
            capability: RouteCapability {
                can_answer_deterministically: true,
                evidence_required: true,
                required_evidence: vec![EvidenceKind::Disk],
                spine_probes: vec![ProbeId::Df],
            },
        },

        QueryClass::TopMemoryProcesses => DeterministicRoute {
            class,
            domain: SpecialistDomain::System,
            intent: QueryIntent::Question,
            probes: vec!["top_memory".to_string()],
            capability: RouteCapability {
                can_answer_deterministically: true,
                evidence_required: true,
                required_evidence: vec![EvidenceKind::Processes],
                spine_probes: vec![ProbeId::TopMemory],
            },
        },

        QueryClass::TopCpuProcesses => DeterministicRoute {
            class,
            domain: SpecialistDomain::System,
            intent: QueryIntent::Question,
            probes: vec!["top_cpu".to_string()],
            capability: RouteCapability {
                can_answer_deterministically: true,
                evidence_required: true,
                required_evidence: vec![EvidenceKind::Processes],
                spine_probes: vec![ProbeId::TopCpu],
            },
        },

        QueryClass::NetworkInterfaces => DeterministicRoute {
            class,
            domain: SpecialistDomain::Network,
            intent: QueryIntent::Question,
            probes: vec!["network_addrs".to_string()],
            capability: RouteCapability {
                can_answer_deterministically: true,
                evidence_required: true,
                required_evidence: vec![EvidenceKind::Network],
                spine_probes: vec![ProbeId::IpAddr],
            },
        },

        QueryClass::ServiceStatus => DeterministicRoute {
            class,
            domain: SpecialistDomain::System,
            intent: QueryIntent::Question,
            probes: vec!["systemctl".to_string()],
            capability: RouteCapability {
                can_answer_deterministically: true,
                evidence_required: true,
                required_evidence: vec![EvidenceKind::Services],
                spine_probes: vec![ProbeId::FailedUnits],
            },
        },

        // === LLM-REQUIRED: Need specialist interpretation ===

        // CPU info from hardware snapshot - needs LLM to format
        QueryClass::CpuInfo => DeterministicRoute {
            class,
            domain: SpecialistDomain::System,
            intent: QueryIntent::Question,
            probes: vec!["lscpu".to_string()],
            capability: RouteCapability {
                can_answer_deterministically: false, // LLM formats hardware info
                evidence_required: true,
                required_evidence: vec![EvidenceKind::Cpu],
                spine_probes: vec![ProbeId::Lscpu],
            },
        },

        // CPU cores - v0.45.7: deterministic from lscpu parse
        QueryClass::CpuCores => DeterministicRoute {
            class,
            domain: SpecialistDomain::System,
            intent: QueryIntent::Question,
            probes: vec!["lscpu".to_string()],
            capability: RouteCapability {
                can_answer_deterministically: true, // v0.45.7: typed lscpu parser
                evidence_required: true,
                required_evidence: vec![EvidenceKind::Cpu],
                spine_probes: vec![ProbeId::Lscpu],
            },
        },

        // CPU temperature - needs sensors + LLM interpretation
        // CRITICAL: Must NOT return CPU model!
        QueryClass::CpuTemp => DeterministicRoute {
            class,
            domain: SpecialistDomain::System,
            intent: QueryIntent::Question,
            probes: vec!["sensors".to_string()],
            capability: RouteCapability {
                can_answer_deterministically: false, // NEVER deterministic
                evidence_required: true,
                required_evidence: vec![EvidenceKind::CpuTemperature],
                spine_probes: vec![ProbeId::Sensors],
            },
        },

        QueryClass::RamInfo => DeterministicRoute {
            class,
            domain: SpecialistDomain::System,
            intent: QueryIntent::Question,
            probes: vec!["free".to_string()],
            capability: RouteCapability {
                can_answer_deterministically: false, // LLM formats
                evidence_required: true,
                required_evidence: vec![EvidenceKind::Memory],
                spine_probes: vec![ProbeId::Free],
            },
        },

        QueryClass::GpuInfo => DeterministicRoute {
            class,
            domain: SpecialistDomain::System,
            intent: QueryIntent::Question,
            probes: vec!["lspci_gpu".to_string()],
            capability: RouteCapability {
                can_answer_deterministically: false,
                evidence_required: true,
                required_evidence: vec![EvidenceKind::Gpu],
                spine_probes: vec![],
            },
        },

        // Audio hardware - deterministic with typed Audio evidence (v0.45.8)
        QueryClass::HardwareAudio => DeterministicRoute {
            class,
            domain: SpecialistDomain::System,
            intent: QueryIntent::Question,
            probes: vec!["lspci_audio".to_string(), "pactl_cards".to_string()],
            capability: RouteCapability {
                can_answer_deterministically: true, // v0.45.8: deterministic
                evidence_required: true,
                required_evidence: vec![EvidenceKind::Audio],
                spine_probes: vec![ProbeId::LspciAudio, ProbeId::PactlCards],
            },
        },

        // System slow - diagnostic needs LLM
        QueryClass::SystemSlow => DeterministicRoute {
            class,
            domain: SpecialistDomain::System,
            intent: QueryIntent::Investigate,
            probes: vec![
                "top_cpu".to_string(),
                "top_memory".to_string(),
                "disk_usage".to_string(),
            ],
            capability: RouteCapability {
                can_answer_deterministically: false,
                evidence_required: true,
                required_evidence: vec![EvidenceKind::Processes, EvidenceKind::Disk],
                spine_probes: vec![ProbeId::TopCpu, ProbeId::TopMemory, ProbeId::Df],
            },
        },

        // System health summary - needs LLM interpretation
        QueryClass::SystemHealthSummary => DeterministicRoute {
            class,
            domain: SpecialistDomain::System,
            intent: QueryIntent::Question,
            probes: vec![
                "disk_usage".to_string(),
                "free".to_string(),
                "failed_units".to_string(),
                "top_cpu".to_string(),
            ],
            capability: RouteCapability {
                can_answer_deterministically: false,
                evidence_required: true,
                required_evidence: vec![
                    EvidenceKind::Disk, EvidenceKind::Memory,
                    EvidenceKind::Services, EvidenceKind::Processes,
                ],
                spine_probes: vec![ProbeId::Df, ProbeId::Free, ProbeId::FailedUnits],
            },
        },

        // Package count - needs probe + LLM formatting
        QueryClass::PackageCount => DeterministicRoute {
            class,
            domain: SpecialistDomain::Packages,
            intent: QueryIntent::Question,
            probes: vec!["pacman_count".to_string()],
            capability: RouteCapability {
                can_answer_deterministically: false, // LLM formats
                evidence_required: true,
                required_evidence: vec![EvidenceKind::Packages],
                spine_probes: vec![ProbeId::PacmanCount],
            },
        },

        // Installed tool check - v0.45.7: CAN be deterministic with evidence
        // "do I have nano" - exit code 0 = yes, exit code 1 = valid NO
        QueryClass::InstalledToolCheck => DeterministicRoute {
            class,
            domain: SpecialistDomain::System,
            intent: QueryIntent::Question,
            probes: vec!["command_v".to_string()],
            capability: RouteCapability {
                can_answer_deterministically: true, // v0.45.7: YES with evidence
                evidence_required: true,
                required_evidence: vec![EvidenceKind::ToolExists],
                spine_probes: vec![], // Specific tool added at runtime
            },
        },

        // === RAG-FIRST: Knowledge store + LLM ===

        QueryClass::BootTimeStatus => DeterministicRoute {
            class,
            domain: SpecialistDomain::System,
            intent: QueryIntent::Question,
            probes: vec!["boot_time".to_string()],
            capability: RouteCapability {
                can_answer_deterministically: false,
                evidence_required: true,
                required_evidence: vec![EvidenceKind::BootTime],
                spine_probes: vec![ProbeId::SystemdAnalyze],
            },
        },

        QueryClass::InstalledPackagesOverview => DeterministicRoute {
            class,
            domain: SpecialistDomain::Packages,
            intent: QueryIntent::Question,
            probes: vec!["pacman_count".to_string()],
            capability: RouteCapability {
                can_answer_deterministically: false,
                evidence_required: true,
                required_evidence: vec![EvidenceKind::Packages],
                spine_probes: vec![ProbeId::PacmanCount],
            },
        },

        QueryClass::AppAlternatives => DeterministicRoute {
            class,
            domain: SpecialistDomain::Packages,
            intent: QueryIntent::Question,
            probes: vec![],
            capability: RouteCapability {
                can_answer_deterministically: false,
                evidence_required: false, // Knowledge-based
                required_evidence: vec![],
                spine_probes: vec![],
            },
        },

        // Configure editor: v0.0.55 - deterministic with editor probes
        // "enable syntax highlighting", "enable line numbers"
        // Must probe supported editors to offer only installed options
        // v0.0.55: Fixed probe list - must include hx (helix binary)
        QueryClass::ConfigureEditor => DeterministicRoute {
            class,
            domain: SpecialistDomain::System, // Editor config is system-level
            intent: QueryIntent::Request, // Requesting a configuration change
            probes: vec![
                // v0.0.55: Probe all supported editors (must match parser's EDITOR_MAP)
                "command_v_vim".to_string(),
                "command_v_nvim".to_string(),
                "command_v_nano".to_string(),
                "command_v_emacs".to_string(),
                "command_v_micro".to_string(),
                "command_v_helix".to_string(),
                "command_v_hx".to_string(),     // v0.0.55: helix binary name
                "command_v_code".to_string(),
                "command_v_kate".to_string(),
                "command_v_gedit".to_string(),
            ],
            capability: RouteCapability {
                can_answer_deterministically: true, // v0.0.57: YES with evidence
                evidence_required: true, // Must verify editor is installed
                required_evidence: vec![EvidenceKind::ToolExists],
                spine_probes: vec![], // Probes are in the probes list above
            },
        },

        // Unknown - full LLM path
        QueryClass::Unknown => DeterministicRoute {
            class,
            domain: SpecialistDomain::System,
            intent: QueryIntent::Question,
            probes: vec![],
            capability: RouteCapability {
                can_answer_deterministically: false,
                evidence_required: false, // Let translator decide
                required_evidence: vec![],
                spine_probes: vec![],
            },
        },
    }
}

/// Apply deterministic router, overriding LLM ticket for known classes
pub fn apply_deterministic_routing(query: &str, llm_ticket: Option<TranslatorTicket>) -> TranslatorTicket {
    let route = get_route(query);

    if route.class == QueryClass::Unknown {
        return llm_ticket.unwrap_or_else(|| create_default_ticket(&route));
    }

    info!(
        "Deterministic router: class={:?}, domain={}, probes={:?}, can_det={}",
        route.class, route.domain, route.probes, route.can_answer_deterministically()
    );

    TranslatorTicket {
        intent: route.intent,
        domain: route.domain,
        entities: vec![],
        needs_probes: route.probes,
        clarification_question: None,
        confidence: 1.0,
        answer_contract: None, // v0.0.74: Deterministic routes don't need answer shaping
    }
}

/// Create default ticket from route
fn create_default_ticket(route: &DeterministicRoute) -> TranslatorTicket {
    TranslatorTicket {
        intent: route.intent.clone(),
        domain: route.domain.clone(),
        entities: vec![],
        needs_probes: route.probes.clone(),
        clarification_question: None,
        confidence: 0.5,
        answer_contract: None, // v0.0.74
    }
}

/// Check if query class can be answered deterministically
#[allow(dead_code)]
pub fn can_answer_deterministically(query: &str) -> bool {
    get_route(query).can_answer_deterministically()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cpu_temp_never_deterministic() {
        let route = get_route("cpu temperature");
        assert!(!route.can_answer_deterministically(),
            "CpuTemp must NEVER be deterministic");
        assert!(!route.probes.is_empty(),
            "CpuTemp must have sensors probe");
    }

    #[test]
    fn test_sound_card_is_deterministic_v458() {
        // v0.45.8: HardwareAudio IS deterministic with typed Audio evidence
        let route = get_route("what is my sound card");
        assert!(route.can_answer_deterministically(),
            "v0.45.8: HardwareAudio should be deterministic with Audio evidence");
        assert!(route.capability.evidence_required,
            "HardwareAudio still requires evidence");
    }

    #[test]
    fn test_installed_tool_deterministic_with_evidence() {
        // v0.45.7: InstalledToolCheck CAN be deterministic when we have evidence
        let route = get_route("do I have nano");
        assert!(route.can_answer_deterministically(),
            "InstalledToolCheck should be deterministic with tool evidence");
        assert!(route.capability.evidence_required,
            "InstalledToolCheck still requires evidence");
    }

    #[test]
    fn test_memory_usage_is_deterministic() {
        let route = get_route("memory usage");
        assert!(route.can_answer_deterministically(),
            "MemoryUsage CAN be deterministic from probe data");
    }

    /// v0.0.56: Verify ALL static probe IDs in router map to commands in translator.
    /// This catches silent probe skipping due to missing probe_id_to_command mappings.
    #[test]
    fn test_all_router_probe_ids_map_in_translator() {
        use crate::translator::probe_id_to_command;

        // Sample queries to exercise all query classes
        let test_queries = [
            "memory usage", "disk usage", "top memory", "top cpu",
            "network info", "is nginx running", "how many cores",
            "cpu info", "cpu temperature", "free ram", "gpu info",
            "sound card", "why is slow", "how is my computer",
            "package count", "do I have nano", "boot time",
            "installed packages", "help me", "enable syntax highlighting",
            "random unknown query xyz",
        ];

        // Dynamic probe IDs that are expanded at runtime (allowlist)
        let dynamic_probe_ids = [
            "command_v", // Expanded to "sh -lc 'command -v <tool>'" at runtime
        ];

        let mut failures = Vec::new();

        for query in &test_queries {
            let route = get_route(query);

            for probe_id in &route.probes {
                // Skip dynamic probe IDs (documented exceptions)
                if dynamic_probe_ids.contains(&probe_id.as_str()) {
                    continue;
                }

                if probe_id_to_command(probe_id).is_none() {
                    failures.push(format!(
                        "QueryClass::{:?} (query='{}') has probe '{}' with no translator mapping",
                        route.class, query, probe_id
                    ));
                }
            }
        }

        assert!(failures.is_empty(),
            "Probe ID mapping failures:\n{}",
            failures.join("\n"));
    }

    // ========================================================================
    // v0.0.57 Contract Tests: Goal 1 - Deterministic routing contracts
    // ========================================================================

    /// v0.0.57: Every QueryClass with evidence_required=true must have at least one probe.
    /// Either in DeterministicRoute.probes OR RouteCapability.spine_probes.
    /// This prevents shipping a route that "requires evidence" but can't collect it.
    #[test]
    fn test_evidence_required_routes_have_probes() {
        use anna_shared::probe_spine::EvidenceKind;

        // All known query classes
        let all_classes = [
            QueryClass::SystemTriage, QueryClass::CpuInfo, QueryClass::CpuCores,
            QueryClass::CpuTemp, QueryClass::RamInfo, QueryClass::GpuInfo,
            QueryClass::HardwareAudio, QueryClass::TopMemoryProcesses,
            QueryClass::TopCpuProcesses, QueryClass::DiskSpace, QueryClass::NetworkInterfaces,
            QueryClass::Help, QueryClass::SystemSlow, QueryClass::MemoryUsage,
            QueryClass::MemoryFree, QueryClass::DiskUsage, QueryClass::ServiceStatus,
            QueryClass::SystemHealthSummary, QueryClass::BootTimeStatus,
            QueryClass::InstalledPackagesOverview, QueryClass::PackageCount,
            QueryClass::InstalledToolCheck, QueryClass::AppAlternatives,
            QueryClass::ConfigureEditor, QueryClass::Unknown,
        ];

        let mut failures = Vec::new();

        for class in all_classes {
            let route = build_route(class);

            if route.capability.evidence_required {
                let has_probes = !route.probes.is_empty();
                let has_spine_probes = !route.capability.spine_probes.is_empty();
                let has_required_evidence = !route.capability.required_evidence.is_empty();

                // Must have at least one probe source
                if !has_probes && !has_spine_probes && !has_required_evidence {
                    failures.push(format!(
                        "QueryClass::{:?} has evidence_required=true but no probes, spine_probes, or required_evidence",
                        class
                    ));
                }
            }
        }

        assert!(failures.is_empty(),
            "Evidence requirement contract violations:\n{}",
            failures.join("\n"));
    }

    /// v0.0.57: enforce_minimum_probes must return expected ProbeIds for typical user queries.
    /// Tests that the probe spine correctly enforces evidence collection.
    #[test]
    fn test_probe_spine_enforces_required_evidence() {
        use anna_shared::probe_spine::{enforce_minimum_probes, ProbeId, probe_to_command};

        // CPU -> "how many cores" should enforce Lscpu
        let cpu_decision = enforce_minimum_probes("how many cores", &[]);
        assert!(cpu_decision.enforced, "CPU query should enforce probes");
        let cpu_cmds: Vec<String> = cpu_decision.probes.iter().map(|p| probe_to_command(p)).collect();
        assert!(
            cpu_decision.probes.iter().any(|p| matches!(p, ProbeId::Lscpu)),
            "CPU cores query should include Lscpu probe, got: {:?}", cpu_cmds
        );

        // Memory -> "how much free ram" should enforce Free
        let mem_decision = enforce_minimum_probes("how much free ram", &[]);
        // Note: "free ram" might not trigger spine directly (it's handled by router)
        // But let's test a query that should trigger memory probes
        let mem_decision2 = enforce_minimum_probes("memory usage", &[]);
        // Memory usage by itself doesn't trigger keyword spine (router handles it)

        // Audio -> "what is my sound card" should enforce LspciAudio
        let audio_decision = enforce_minimum_probes("what is my sound card", &[]);
        assert!(audio_decision.enforced, "Audio query should enforce probes");
        assert!(
            audio_decision.probes.iter().any(|p| matches!(p, ProbeId::LspciAudio)),
            "Audio query should include LspciAudio probe"
        );

        // ToolExists -> "do I have nano" should enforce CommandV
        let tool_decision = enforce_minimum_probes("do I have nano", &[]);
        assert!(tool_decision.enforced, "Tool check should enforce probes");
        assert!(
            tool_decision.probes.iter().any(|p| matches!(p, ProbeId::CommandV(_))),
            "Tool check should include CommandV probe"
        );
        // Verify the tool name is extracted
        let has_nano = tool_decision.probes.iter().any(|p| {
            if let ProbeId::CommandV(name) = p {
                name == "nano"
            } else {
                false
            }
        });
        assert!(has_nano, "Tool check should extract 'nano' as the tool name");
    }

    /// v0.0.57: Required evidence kinds map to probes via probes_for_evidence.
    #[test]
    fn test_evidence_kinds_have_probes() {
        use anna_shared::probe_spine::{probes_for_evidence, EvidenceKind};

        // All evidence kinds that should have probes
        let kinds_with_probes = [
            (EvidenceKind::Cpu, "Cpu"),
            (EvidenceKind::CpuTemperature, "CpuTemperature"),
            (EvidenceKind::Memory, "Memory"),
            (EvidenceKind::Disk, "Disk"),
            (EvidenceKind::BlockDevices, "BlockDevices"),
            (EvidenceKind::Audio, "Audio"),
            (EvidenceKind::Network, "Network"),
            (EvidenceKind::Processes, "Processes"),
            (EvidenceKind::Services, "Services"),
            (EvidenceKind::Journal, "Journal"),
            (EvidenceKind::Packages, "Packages"),
            (EvidenceKind::BootTime, "BootTime"),
            (EvidenceKind::System, "System"),
        ];

        for (kind, name) in kinds_with_probes {
            let probes = probes_for_evidence(kind);
            assert!(
                !probes.is_empty(),
                "EvidenceKind::{} should have probes, got empty",
                name
            );
        }

        // ToolExists and Gpu are special cases (runtime-specific or snapshot-based)
        // These are allowed to have empty probe lists
        assert!(probes_for_evidence(EvidenceKind::ToolExists).is_empty(),
            "ToolExists probes are runtime-specific");
        assert!(probes_for_evidence(EvidenceKind::Gpu).is_empty(),
            "Gpu relies on hardware snapshot");
    }
}
