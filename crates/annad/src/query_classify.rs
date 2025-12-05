//! Query classification patterns for deterministic routing.
//!
//! This module contains the pattern matching logic that classifies user queries
//! into known QueryClass categories for deterministic probe selection.

// Import QueryClass from router module
use crate::router::QueryClass;

/// Strip common greetings from query for better classification
fn strip_greetings(query: &str) -> String {
    let q = query.to_lowercase();
    // Remove common greetings and emoticons
    let patterns = [
        "hello", "hi ", "hey ", "good morning", "good afternoon", "good evening",
        "anna", ":)", ":(", ";)", ":d", ":p", "!", "?", "…", "...",
    ];
    let mut result = q;
    for p in patterns {
        result = result.replace(p, " ");
    }
    // Collapse multiple spaces
    result.split_whitespace().collect::<Vec<_>>().join(" ")
}

/// Classify query to a known class
pub fn classify_query(query: &str) -> QueryClass {
    let q = query.to_lowercase();
    let stripped = strip_greetings(query);

    // Help request (check first as it's specific)
    if q.trim() == "help" || q.contains("what can you do") || q.contains("how do i use") {
        return QueryClass::Help;
    }

    // SystemTriage (FAST PATH): error/warning focused queries - check BEFORE SystemHealthSummary
    // v0.0.35: Must match: "how is my computer", "any errors", "any problems", "is everything ok",
    //          "warnings", "errors", "health", "status"
    // No translator needed. probes=[journal_errors, journal_warnings, failed_units, boot_time]
    if stripped.contains("any errors")
        || stripped.contains("any problems")
        || stripped.contains("any issues")
        || stripped.contains("any warnings")
        || stripped.contains("errors so far")
        || stripped.contains("problems so far")
        || stripped.contains("what's wrong")
        || stripped.contains("whats wrong")
        || stripped.contains("is everything ok")
        || stripped.contains("is everything okay")
        || stripped.contains("how is my computer")
        || stripped.contains("how's my computer")
        || stripped.contains("computer doing")
        || q.contains("health")     // v0.0.35: "health" -> triage, not full report
        || q.trim() == "errors"
        || q.trim() == "warnings"
        || q.trim() == "problems"
        || q.trim() == "status"     // v0.0.35: bare "status" -> triage
        || q.trim() == "health"
    {
        return QueryClass::SystemTriage;
    }

    // System health summary: FULL system overview (explicit "summary", "report", "overview")
    // v0.0.35: Narrowed - only triggers on explicit full-report keywords
    if q.contains("summary")
        || q.contains("status report")
        || q.contains("overview")
        || q.contains("full report")
        || q.contains("system status")
        || stripped.contains("how is the system")
        || stripped.contains("how's the system")
        || stripped.contains("check my system")
        || stripped.contains("check the system")
        || stripped.contains("system check")
        || q.trim() == "report"
    {
        return QueryClass::SystemHealthSummary;
    }

    // System slow (multi-probe diagnostic)
    if q.contains("slow") || q.contains("sluggish") || q.contains("laggy") {
        return QueryClass::SystemSlow;
    }

    // === ROUTE Phase: Typed query classes (check specific patterns first) ===

    // v0.0.45: InstalledToolCheck - "do I have nano", "is vim installed"
    // Check BEFORE ServiceStatus to avoid "is X running" collision
    // Exclude hardware queries (cpu, ram, memory, gpu, disk)
    let is_hardware_query = q.contains("cpu") || q.contains("ram") || q.contains("memory")
        || q.contains("gpu") || q.contains("disk") || q.contains("core");
    if !is_hardware_query && (
        (q.contains("do i have") && (q.contains("nano") || q.contains("vim") || q.contains("git") || q.contains("emacs")))
        || (q.contains("is") && q.contains("installed"))
        || (q.contains("have") && q.contains("installed"))
    ) {
        return QueryClass::InstalledToolCheck;
    }

    // v0.0.45: HardwareAudio - "sound card", "audio device"
    if q.contains("sound card")
        || q.contains("audio device")
        || q.contains("audio card")
        || q.contains("sound device")
        || (q.contains("audio") && q.contains("hardware"))
    {
        return QueryClass::HardwareAudio;
    }

    // v0.0.45: CpuTemp - "cpu temperature", "how hot is my cpu"
    // Check BEFORE CpuInfo since it's more specific
    if q.contains("temperature")
        || q.contains("temp")
        || q.contains("how hot")
        || q.contains("thermal")
        || q.contains("sensors")
    {
        return QueryClass::CpuTemp;
    }

    // v0.0.45: CpuCores - "how many cores", "threads"
    // Check BEFORE CpuInfo since it's more specific
    if (q.contains("how many") && (q.contains("core") || q.contains("thread")))
        || q.contains("core count")
        || q.contains("thread count")
        || q.contains("number of cores")
        || q.contains("number of threads")
    {
        return QueryClass::CpuCores;
    }

    // v0.0.45: PackageCount - "how many packages"
    // Check BEFORE InstalledPackagesOverview since it's more specific
    if (q.contains("how many") && q.contains("package"))
        || q.contains("package count")
        || q.contains("count packages")
    {
        return QueryClass::PackageCount;
    }

    // v0.0.45: MemoryFree - "free ram", "available ram"
    // Check BEFORE MemoryUsage since it's more specific
    if (q.contains("free") && q.contains("ram"))
        || (q.contains("available") && q.contains("ram"))
        || q.contains("how much free ram")
        || q.contains("how much available ram")
    {
        return QueryClass::MemoryFree;
    }

    // Memory usage (dynamic): "memory usage", "how much memory used"
    // Check before RamInfo since these are more specific
    if (q.contains("memory") && q.contains("usage"))
        || (q.contains("memory") && q.contains("used"))
        || q.contains("free memory")
        || q.contains("available memory")
    {
        return QueryClass::MemoryUsage;
    }

    // Disk usage (dynamic): specific mount or usage patterns
    // Check before DiskSpace since "disk usage" is more specific
    if q.contains("disk usage") || q.contains("filesystem usage") {
        return QueryClass::DiskUsage;
    }

    // Service status: "is X running", "status of X"
    if q.contains("running")
        || q.contains("service status")
        || q.contains("systemd")
        || (q.contains("status") && q.contains("service"))
        || (q.contains("is") && (q.contains("active") || q.contains("enabled")))
    {
        return QueryClass::ServiceStatus;
    }

    // === Legacy query classes ===

    // Top memory processes (before RAM check)
    if (q.contains("process") && (q.contains("memory") || q.contains("ram")))
        || q.contains("memory hog")
        || q.contains("top memory")
        || q.contains("most memory")
        || q.contains("what's using memory")
        || q.contains("what is using memory")
    {
        return QueryClass::TopMemoryProcesses;
    }

    // Top CPU processes
    if (q.contains("process") && q.contains("cpu"))
        || q.contains("cpu hog")
        || q.contains("top cpu")
        || q.contains("most cpu")
        || q.contains("what's using cpu")
        || q.contains("what is using cpu")
    {
        return QueryClass::TopCpuProcesses;
    }

    // Hardware snapshot queries
    if q.contains("cpu") || q.contains("processor") || q.contains("core") {
        return QueryClass::CpuInfo;
    }

    if q.contains("ram") || (q.contains("memory") && !q.contains("process")) {
        return QueryClass::RamInfo;
    }

    if q.contains("gpu") || q.contains("graphics") || q.contains("vram") {
        return QueryClass::GpuInfo;
    }

    // Disk space
    if q.contains("disk")
        || q.contains("space")
        || q.contains("storage")
        || q.contains("filesystem")
        || q.contains("mount")
        || q.contains("full")
    {
        return QueryClass::DiskSpace;
    }

    // Network interfaces
    if q.contains("network")
        || q.contains("interface")
        || q.contains("ip ")
        || q.contains("ip?")
        || q.contains("ips")
        || q.contains("wifi")
        || q.contains("ethernet")
        || q.contains("wlan")
    {
        return QueryClass::NetworkInterfaces;
    }

    // === RAG-first classes (v0.0.32R): answered from knowledge store ===

    // Boot time status
    if q.contains("boot time")
        || q.contains("bootup")
        || q.contains("startup time")
        || q.contains("how long to boot")
        || q.contains("how fast does it boot")
        || (q.contains("boot") && q.contains("seconds"))
    {
        return QueryClass::BootTimeStatus;
    }

    // Installed packages overview
    if q.contains("how many packages")
        || q.contains("packages installed")
        || q.contains("what's installed")
        || q.contains("what is installed")
        || q.contains("list packages")
        || q.contains("installed software")
        || (q.contains("packages") && q.contains("count"))
    {
        return QueryClass::InstalledPackagesOverview;
    }

    // App alternatives
    if q.contains("alternative to")
        || q.contains("alternatives to")
        || q.contains("instead of")
        || q.contains("replacement for")
        || q.contains("similar to")
        || q.contains("like")
        || (q.contains("what") && q.contains("use") && q.contains("instead"))
    {
        return QueryClass::AppAlternatives;
    }

    QueryClass::Unknown
}
