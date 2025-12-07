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

    // v0.0.77: Meta/small-talk - bypass LLM entirely
    // "how are you", "what is your name", "are you using llm", "are you ok"
    if q == "how are you"
        || q == "how are you?"
        || q.starts_with("how are you ")
        || q.contains("what is your name")
        || q.contains("what's your name")
        || q.contains("who are you")
        || q.contains("are you ok")
        || q.contains("are you okay")
        || q.contains("are you using llm")
        || q.contains("are you an ai")
        || q.contains("are you a bot")
        || q.contains("are you human")
        || q.contains("are you real")
        || q == "hello"
        || q == "hi"
        || q == "hey"
        || q == "thanks"
        || q == "thank you"
        || q == "good morning"
        || q == "good afternoon"
        || q == "good evening"
    {
        return QueryClass::MetaSmallTalk;
    }

    // v0.0.77: Kernel version - "kernel version", "uname", "linux version"
    if q.contains("kernel version")
        || q.contains("kernel release")
        || q == "uname"
        || q == "uname -a"
        || q.contains("linux version")
        || q.contains("what kernel")
    {
        return QueryClass::KernelVersion;
    }

    // v0.0.77: Config file location - "where is vim config", "hyprland config path"
    let config_location_query = (q.contains("where is") || q.contains("where's")
        || q.contains("path to") || q.contains("location of") || q.contains("find the"))
        && q.contains("config");
    let specific_config_query = (q.contains("vim") || q.contains("nvim") || q.contains("hyprland")
        || q.contains("sway") || q.contains("alacritty") || q.contains("kitty")
        || q.contains("bash") || q.contains("zsh") || q.contains("fish"))
        && (q.contains("config") || q.contains("rc file") || q.contains("dotfile"));
    if config_location_query || specific_config_query {
        return QueryClass::ConfigFileLocation;
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

    // v0.45.4: InstalledToolCheck - "do I have nano", "is vim installed", "have I got firefox"
    // Check BEFORE ServiceStatus to avoid "is X running" collision
    // Exclude hardware queries (cpu, ram, memory, gpu, disk)
    // Generic pattern: any "do I have <word>" is a tool check
    let is_hardware_query = q.contains("cpu") || q.contains("ram") || q.contains("memory")
        || q.contains("gpu") || q.contains("disk") || q.contains("core");
    let is_tool_check_query = q.contains("do i have")
        || q.contains("do you have")
        || q.contains("have i got")
        || (q.contains("is") && q.contains("installed"))
        || (q.contains("have") && q.contains("installed"));
    if !is_hardware_query && is_tool_check_query {
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

    // v0.0.45: MemoryFree - "free ram", "available ram", "free memory", "available memory"
    // v0.0.80: B1 fix - "free memory" and "available memory" should be MemoryFree, not MemoryUsage
    // Check BEFORE MemoryUsage since it's more specific
    if (q.contains("free") && q.contains("ram"))
        || (q.contains("available") && q.contains("ram"))
        || q.contains("how much free ram")
        || q.contains("how much available ram")
        || q.contains("free memory")
        || q.contains("available memory")
        || q.contains("how much free memory")
        || q.contains("how much available memory")
    {
        return QueryClass::MemoryFree;
    }

    // Memory usage (dynamic): "memory usage", "how much memory used"
    // Check before RamInfo since these are more specific
    // v0.0.80: Removed "free memory" and "available memory" - those are MemoryFree
    if (q.contains("memory") && q.contains("usage"))
        || (q.contains("memory") && q.contains("used"))
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

    // v0.45.5: Configure editor - "enable syntax highlighting", "turn on line numbers"
    // Requires clarification about which editor before action
    if (q.contains("enable") || q.contains("turn on") || q.contains("activate") || q.contains("set up"))
        && (q.contains("syntax highlight")
            || q.contains("line number")
            || q.contains("word wrap")
            || q.contains("auto indent")
            || q.contains("tab size")
            || q.contains("color scheme")
            || q.contains("theme"))
    {
        return QueryClass::ConfigureEditor;
    }

    // Also match "how do I enable X in vim/nano/etc"
    if (q.contains("how") || q.contains("configure") || q.contains("setup"))
        && (q.contains("vim") || q.contains("nvim") || q.contains("nano") || q.contains("emacs"))
        && (q.contains("syntax") || q.contains("highlight") || q.contains("line number")
            || q.contains("color") || q.contains("theme"))
    {
        return QueryClass::ConfigureEditor;
    }

    // v0.0.99: Install package - "install htop", "install vim", "add package htop"
    // Check for install/add commands with package names
    if q.starts_with("install ")
        || q.starts_with("add ")
        || q.contains("install package")
        || q.contains("install the")
        || (q.contains("can you install") && !q.contains("installed"))
        || q.contains("please install")
        || q.contains("i need to install")
        || q.contains("how do i install")
    {
        return QueryClass::InstallPackage;
    }

    // v0.0.99: Manage service - "restart docker", "start sshd", "stop nginx"
    // Common service control verbs at the start of query
    let service_verbs = ["start ", "stop ", "restart ", "enable ", "disable ", "reload "];
    for verb in &service_verbs {
        if q.starts_with(verb) {
            return QueryClass::ManageService;
        }
    }
    // Also match "can you restart X", "please start X"
    if (q.contains("can you") || q.contains("please") || q.contains("could you"))
        && (q.contains("start ") || q.contains("stop ") || q.contains("restart ")
            || q.contains("enable ") || q.contains("disable "))
    {
        return QueryClass::ManageService;
    }

    // v0.0.101: Configure shell - "colored prompt in bash", "syntax highlighting zsh"
    let is_shell_config = (q.contains("bash") || q.contains("zsh") || q.contains("fish")
        || q.contains("bashrc") || q.contains("zshrc"))
        && (q.contains("color") || q.contains("prompt") || q.contains("syntax")
            || q.contains("highlight") || q.contains("history") || q.contains("alias")
            || q.contains("auto") && q.contains("suggest"));
    if is_shell_config {
        return QueryClass::ConfigureShell;
    }

    // v0.0.101: Configure git - "configure git aliases", "git username", "git email"
    let is_git_config = q.contains("git")
        && (q.contains("config") || q.contains("alias") || q.contains("username")
            || q.contains("user") || q.contains("email") || q.contains("editor")
            || q.contains("default branch") || q.contains("color")
            || q.contains("autocorrect") || q.contains("pull")
            || q.contains("credential") || q.contains("gpg") || q.contains("sign"));
    if is_git_config {
        return QueryClass::ConfigureGit;
    }

    // v0.0.104: SSH key management - "generate ssh key", "copy ssh key", "ssh config"
    let is_ssh = q.contains("ssh")
        && (q.contains("key") || q.contains("keygen") || q.contains("generate")
            || q.contains("create") || q.contains("copy") || q.contains("ssh-copy")
            || q.contains("config") || q.contains("agent") || q.contains("github")
            || q.contains("gitlab") || q.contains("authorized") || q.contains("passphrase"));
    if is_ssh {
        return QueryClass::SshKeyManagement;
    }

    // v0.0.111: Ticket history - "show my tickets", "recent cases", "ticket history"
    // v0.0.116: Added inbox queries
    if q.contains("ticket")
        || q.contains("case number")
        || q.contains("my cases")
        || q.contains("recent cases")
        || q.contains("past questions")
        || q.contains("previous questions")
        || q.contains("what have i asked")
        || q.contains("support history")
        || q.contains("inbox")
        || q.contains("pending queries")
        || q.contains("pending questions")
        || q.contains("queued questions")
    {
        return QueryClass::TicketHistory;
    }

    // v0.0.111: Staff roster - "who is on shift", "show IT team", "who works here"
    if q.contains("who is on shift")
        || q.contains("who's on shift")
        || q.contains("on duty")
        || q.contains("it team")
        || q.contains("it department")
        || q.contains("who works here")
        || q.contains("staff")
        || q.contains("team roster")
        || q.contains("support team")
        || (q.contains("who") && q.contains("available"))
    {
        return QueryClass::StaffRoster;
    }

    // v0.0.122: Package updates - "any updates", "check for updates"
    if q.contains("updates available")
        || q.contains("any updates")
        || q.contains("check for updates")
        || q.contains("pending updates")
        || q.contains("upgradable")
        || q.contains("need to update")
        || (q.contains("package") && q.contains("update"))
        || q.contains("checkupdates")
    {
        return QueryClass::PackageUpdates;
    }

    // v0.0.122: Swap info - "swap usage", "show swap"
    if q.contains("swap usage")
        || q.contains("swap space")
        || q.contains("show swap")
        || q.contains("how much swap")
        || q.contains("swap status")
        || q.trim() == "swap"
    {
        return QueryClass::SwapInfo;
    }

    // v0.0.122: Timezone info - "what timezone", "show locale"
    if q.contains("timezone")
        || q.contains("time zone")
        || q.contains("locale")
        || q.contains("what time is it")
        || q.contains("current time")
        || q.contains("system time")
        || q.contains("timedatectl")
    {
        return QueryClass::TimezoneInfo;
    }

    // v0.0.122: System uptime - "uptime", "how long running"
    if q.trim() == "uptime"
        || q.contains("how long")
        && (q.contains("running") || q.contains("been on") || q.contains("up"))
        || q.contains("system uptime")
        || q.contains("uptime?")
    {
        return QueryClass::SystemUptime;
    }

    // v0.0.123: Logged in users - "who is logged in", "show users"
    if q.contains("logged in")
        || q.contains("who is on")
        || q.contains("active users")
        || q.contains("current users")
        || q.contains("show users")
        || q.contains("list users")
        || (q.contains("who") && q.contains("logged"))
        || q.trim() == "who"
        || q.trim() == "w"
    {
        return QueryClass::LoggedInUsers;
    }

    // v0.0.123: Battery status - "battery", "power status"
    if q.contains("battery")
        || q.contains("power status")
        || q.contains("charge level")
        || q.contains("charging")
        || q.contains("power level")
        || (q.contains("laptop") && q.contains("power"))
    {
        return QueryClass::BatteryStatus;
    }

    // v0.0.123: System load - "load average", "system load"
    if q.contains("load average")
        || q.contains("system load")
        || q.contains("cpu load")
        || q.trim() == "load"
        || (q.contains("how busy") && q.contains("system"))
    {
        return QueryClass::SystemLoad;
    }

    // v0.0.123: Last boot - "when did system start", "last reboot"
    if q.contains("last boot")
        || q.contains("last reboot")
        || q.contains("when did") && (q.contains("boot") || q.contains("start") || q.contains("reboot"))
        || q.contains("when was") && (q.contains("boot") || q.contains("reboot"))
        || q.contains("boot time")
        || q.contains("reboot time")
    {
        return QueryClass::LastBoot;
    }

    // v0.0.124: Hostname - "hostname", "what is my hostname"
    if q.trim() == "hostname"
        || q.contains("my hostname")
        || q.contains("computer name")
        || q.contains("machine name")
        || (q.contains("what") && q.contains("hostname"))
    {
        return QueryClass::Hostname;
    }

    // v0.0.124: OS info - "what distro", "which linux"
    if q.contains("what distro")
        || q.contains("which distro")
        || q.contains("which linux")
        || q.contains("what linux")
        || q.contains("os version")
        || q.contains("what os")
        || q.contains("which os")
        || q.contains("os-release")
        || q.contains("linux version")
        || (q.contains("running") && (q.contains("distro") || q.contains("linux")))
    {
        return QueryClass::OsInfo;
    }

    // v0.0.124: Network connectivity - "am I online", "check internet"
    if q.contains("am i online")
        || q.contains("internet connection")
        || q.contains("check internet")
        || q.contains("network connectivity")
        || q.contains("connected to internet")
        || q.contains("online?")
        || q.contains("can i reach")
        || (q.contains("ping") && !q.contains("pinging"))
    {
        return QueryClass::NetworkConnectivity;
    }

    // v0.0.124: Mounted filesystems - "mounted drives", "show mounts"
    if q.contains("mounted")
        || q.contains("mount points")
        || q.contains("show mounts")
        || q.contains("list mounts")
        || q.contains("filesystems")
        || q.trim() == "mounts"
        || q.trim() == "findmnt"
    {
        return QueryClass::MountedFilesystems;
    }

    // v0.0.124: USB devices - "usb devices", "what's plugged in"
    if q.contains("usb device")
        || q.contains("usb")
        || q.contains("plugged in")
        || q.contains("connected device")
        || q.trim() == "lsusb"
        || (q.contains("what") && q.contains("plugged"))
    {
        return QueryClass::UsbDevices;
    }

    // v0.0.125: Listening ports - "open ports", "listening ports"
    if q.contains("listening port")
        || q.contains("open port")
        || q.contains("port listen")
        || q.contains("network port")
        || q.contains("what ports")
        || q.trim() == "ss"
        || q.trim() == "netstat"
        || (q.contains("port") && q.contains("open"))
    {
        return QueryClass::ListeningPorts;
    }

    // v0.0.125: Running services - "running services", "active services"
    if q.contains("running service")
        || q.contains("active service")
        || q.contains("started service")
        || q.contains("enabled service")
        || (q.contains("service") && q.contains("running"))
        || (q.contains("service") && q.contains("active"))
        || q.contains("list services")
    {
        return QueryClass::RunningServices;
    }

    // v0.0.125: Current user - "whoami", "current user"
    if q.trim() == "whoami"
        || q.trim() == "id"
        || q.contains("current user")
        || q.contains("logged in as")
        || q.contains("my user")
        || q.contains("who am i")
        || (q.contains("what") && q.contains("user") && q.contains("am"))
    {
        return QueryClass::CurrentUser;
    }

    // v0.0.125: System architecture - "architecture", "32 or 64 bit"
    if q.contains("architecture")
        || q.contains("32 bit")
        || q.contains("64 bit")
        || q.contains("x86_64")
        || q.contains("arm64")
        || q.contains("aarch64")
        || q.trim() == "arch"
        || (q.contains("what") && q.contains("arch"))
    {
        return QueryClass::SystemArchitecture;
    }

    // v0.0.125: Environment variables - "env vars", "environment"
    if q.contains("environment variable")
        || q.contains("env var")
        || q.trim() == "env"
        || q.trim() == "printenv"
        || q.contains("show env")
        || q.contains("list env")
        || (q.contains("what") && q.contains("env"))
    {
        return QueryClass::EnvironmentVars;
    }

    // v0.0.126: Process tree - "pstree", "process tree"
    if q.trim() == "pstree"
        || q.contains("process tree")
        || q.contains("process hierarchy")
        || q.contains("parent process")
        || (q.contains("show") && q.contains("process") && q.contains("tree"))
    {
        return QueryClass::ProcessTree;
    }

    // v0.0.126: DNS servers - "dns servers", "nameservers", "resolv.conf"
    if q.contains("dns server")
        || q.contains("nameserver")
        || q.contains("resolv.conf")
        || q.contains("dns config")
        || (q.contains("what") && q.contains("dns"))
        || (q.contains("which") && q.contains("dns"))
    {
        return QueryClass::DnsServers;
    }

    // v0.0.126: Default gateway - "default gateway", "gateway", "default route"
    if q.contains("default gateway")
        || q.contains("gateway ip")
        || q.contains("default route")
        || (q.contains("what") && q.contains("gateway"))
        || (q.contains("my") && q.contains("gateway"))
        || q.trim() == "gateway"
    {
        return QueryClass::DefaultGateway;
    }

    // v0.0.126: Open files - "open files", "lsof count"
    if q.contains("open file")
        || q.contains("file handle")
        || q.contains("file descriptor")
        || q.trim() == "lsof"
        || (q.contains("how many") && q.contains("file") && q.contains("open"))
    {
        return QueryClass::OpenFiles;
    }

    // v0.0.126: System locale - "locale", "language settings"
    if q.trim() == "locale"
        || q.contains("system locale")
        || q.contains("language setting")
        || q.contains("character set")
        || q.contains("encoding")
        || (q.contains("what") && q.contains("locale"))
    {
        return QueryClass::SystemLocale;
    }

    // v0.0.127: Block devices - "lsblk", "block devices", "partitions"
    if q.trim() == "lsblk"
        || q.contains("block device")
        || q.contains("partition")
        || q.contains("show disk")
        || q.contains("list disk")
        || (q.contains("disk") && q.contains("layout"))
    {
        return QueryClass::BlockDevices;
    }

    // v0.0.127: Installed kernels - "installed kernels", "available kernels"
    if q.contains("installed kernel")
        || q.contains("available kernel")
        || q.contains("linux kernel")
        || (q.contains("what") && q.contains("kernel") && q.contains("install"))
        || (q.contains("list") && q.contains("kernel"))
    {
        return QueryClass::InstalledKernels;
    }

    // v0.0.127: CPU frequency - "cpu frequency", "clock speed"
    if q.contains("cpu freq")
        || q.contains("clock speed")
        || q.contains("cpu speed")
        || q.contains("processor speed")
        || q.contains("cpu mhz")
        || q.contains("cpu ghz")
        || (q.contains("how fast") && q.contains("cpu"))
    {
        return QueryClass::CpuFrequency;
    }

    // v0.0.127: Memory slots - "memory slots", "ram slots", "dimm"
    if q.contains("memory slot")
        || q.contains("ram slot")
        || q.contains("dimm")
        || q.contains("memory stick")
        || (q.contains("how many") && q.contains("ram") && q.contains("slot"))
    {
        return QueryClass::MemorySlots;
    }

    // v0.0.127: ZFS status - "zfs status", "zpool status"
    if q.contains("zfs")
        || q.contains("zpool")
        || (q.contains("storage pool") && (q.contains("status") || q.contains("health")))
    {
        return QueryClass::ZfsStatus;
    }

    // v0.0.128: Boot loader - "bootloader", "grub", "systemd-boot"
    if q.contains("bootloader")
        || q.contains("boot loader")
        || q.contains("grub")
        || q.contains("systemd-boot")
        || q.contains("bootctl")
        || (q.contains("what") && q.contains("boot") && !q.contains("last boot"))
    {
        return QueryClass::BootLoader;
    }

    // v0.0.128: Firewall status - "firewall", "iptables", "nftables"
    if q.contains("firewall")
        || q.contains("iptables")
        || q.contains("nftables")
        || q.contains("ufw")
        || (q.contains("port") && q.contains("block"))
    {
        return QueryClass::FirewallStatus;
    }

    // v0.0.128: Systemd units - "systemd units", "list units"
    if q.contains("systemd unit")
        || q.contains("list unit")
        || q.contains("all unit")
        || q.contains("enabled unit")
        || (q.contains("show") && q.contains("unit"))
    {
        return QueryClass::SystemdUnits;
    }

    // v0.0.128: Crontabs - "crontab", "scheduled tasks", "cron jobs"
    if q.contains("crontab")
        || q.contains("cron job")
        || q.contains("scheduled task")
        || q.contains("scheduled job")
        || q.trim() == "cron"
        || (q.contains("show") && q.contains("cron"))
    {
        return QueryClass::Crontabs;
    }

    // v0.0.128: SSH connections - "ssh connections", "who is connected via ssh"
    if q.contains("ssh connection")
        || q.contains("ssh session")
        || (q.contains("who") && q.contains("ssh"))
        || (q.contains("connected") && q.contains("ssh"))
        || q.contains("remote connection")
    {
        return QueryClass::SshConnections;
    }

    // v0.0.129: Docker containers - "docker ps", "running containers"
    if q.contains("docker container")
        || q.contains("docker ps")
        || q.contains("running container")
        || (q.contains("container") && q.contains("running"))
        || (q.contains("list") && q.contains("container"))
    {
        return QueryClass::DockerContainers;
    }

    // v0.0.129: Docker images - "docker images", "list images"
    if q.contains("docker image")
        || (q.contains("list") && q.contains("image") && !q.contains("disk"))
        || (q.contains("show") && q.contains("image") && q.contains("docker"))
    {
        return QueryClass::DockerImages;
    }

    // v0.0.129: Systemd timers - "systemd timers", "scheduled timers"
    if q.contains("systemd timer")
        || q.contains("list timer")
        || q.contains("scheduled timer")
        || (q.contains("timer") && q.contains("systemd"))
    {
        return QueryClass::SystemdTimers;
    }

    // v0.0.129: Last logins - "last logins", "login history"
    if q.contains("last login")
        || q.contains("login history")
        || q.contains("recent login")
        || q.contains("who logged in")
        || q.trim() == "last"
    {
        return QueryClass::LastLogins;
    }

    // v0.0.129: Failed logins - "failed logins", "login failures"
    if q.contains("failed login")
        || q.contains("login failure")
        || q.contains("unsuccessful login")
        || q.contains("bad login")
        || q.trim() == "lastb"
    {
        return QueryClass::FailedLogins;
    }

    QueryClass::Unknown
}
