//! Type definitions for canonical intents and topics.

use serde::{Deserialize, Serialize};

/// Canonical intent - what the user wants to accomplish
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CanonicalIntent {
    // Memory intents
    CheckFreeRam,
    CheckSwapPresence,
    CheckSwapUsage,
    ListTopMemoryProcesses,

    // Storage intents
    CheckDiskUsage,
    CheckDiskHealth,
    CheckTrimService,
    FindLargestFiles,

    // Services intents
    CheckFailedServices,
    CheckServiceStatus,
    ListRunningServices,
    CheckTimers,

    // Boot intents
    CheckBootTime,
    DiagnoseSlowBoot,
    CheckBootErrors,

    // Network intents
    CheckNetworkConnectivity,
    CheckDnsHealth,
    CheckListeningPorts,
    CheckFirewallStatus,

    // Packages intents
    CheckPackageInstalled,
    ListInstalledPackages,
    CheckUpdates,

    // Process intents
    ListTopCpuProcesses,
    CheckUptime,
    CheckLoadAverage,

    // Desktop intents
    CheckDesktopEnvironment,
    FindConfigFile,

    // Audio intents
    CheckAudioDevices,
    CheckAudioServer,

    // GPU/Display intents
    CheckGpuDrivers,
    CheckDisplayInfo,

    // Generic
    ExplainConcept,
    GeneralQuery,
}

impl std::fmt::Display for CanonicalIntent {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

/// Topic - knowledge domain for documentation search
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Topic {
    pub id: String,
    pub name: String,
    pub commands: Vec<String>,
    pub keywords: Vec<String>,
}
