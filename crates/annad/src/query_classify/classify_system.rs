//! System query classification patterns (v0.0.803).
//!
//! Uptime, load, boot, users, hostname, OS, architecture, locale, kernel, desktop.

use crate::router::QueryClass;

/// Classify system queries.
/// Returns Some if matched, None otherwise.
pub fn classify_system(q: &str) -> Option<QueryClass> {
    // v0.0.799: Boot blame - "why is my boot slow?", "slow boot", "boot analysis"
    // MUST come before BootTimeStatus to catch slow boot queries
    if (q.contains("boot") && q.contains("slow"))
        || (q.contains("slow") && q.contains("startup"))
        || (q.contains("why") && q.contains("boot"))
        || q.contains("boot blame")
        || q.contains("boot analysis")
        || (q.contains("boot") && q.contains("long"))
        || (q.contains("takes") && q.contains("boot"))
    {
        return Some(QueryClass::BootBlame);
    }

    // Boot time status
    // v0.0.803: Added "boot times" pattern for "what are my boot times?"
    if q.contains("boot time")
        || q.contains("boot times")
        || q.contains("bootup")
        || q.contains("startup time")
        || q.contains("how long to boot")
        || q.contains("how fast does it boot")
        || (q.contains("boot") && q.contains("seconds"))
    {
        return Some(QueryClass::BootTimeStatus);
    }

    // v0.0.122: System uptime
    if q.trim() == "uptime"
        || q.contains("how long")
            && (q.contains("running") || q.contains("been on") || q.contains("up"))
        || q.contains("system uptime")
        || q.contains("uptime?")
    {
        return Some(QueryClass::SystemUptime);
    }

    // v0.0.111: Staff roster - MUST come BEFORE LoggedInUsers due to "staff" overlap
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
        return Some(QueryClass::StaffRoster);
    }

    // v0.0.123: Logged in users
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
        return Some(QueryClass::LoggedInUsers);
    }

    // v0.0.123: Battery status
    if q.contains("battery")
        || q.contains("power status")
        || q.contains("charge level")
        || q.contains("charging")
        || q.contains("power level")
        || (q.contains("laptop") && q.contains("power"))
    {
        return Some(QueryClass::BatteryStatus);
    }

    // v0.0.123: System load
    if q.contains("load average")
        || q.contains("system load")
        || q.contains("cpu load")
        || q.trim() == "load"
        || (q.contains("how busy") && q.contains("system"))
    {
        return Some(QueryClass::SystemLoad);
    }

    // v0.0.123: Last boot
    if q.contains("last boot")
        || q.contains("last reboot")
        || q.contains("when did")
            && (q.contains("boot") || q.contains("start") || q.contains("reboot"))
        || q.contains("when was") && (q.contains("boot") || q.contains("reboot"))
        || q.contains("boot time")
        || q.contains("reboot time")
    {
        return Some(QueryClass::LastBoot);
    }

    // v0.0.124: Hostname
    if q.trim() == "hostname"
        || q.contains("my hostname")
        || q.contains("computer name")
        || q.contains("machine name")
        || (q.contains("what") && q.contains("hostname"))
    {
        return Some(QueryClass::Hostname);
    }

    // v0.0.124: OS info
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
        return Some(QueryClass::OsInfo);
    }

    // v0.0.125: Current user
    if q.trim() == "whoami"
        || q.trim() == "id"
        || q.contains("current user")
        || q.contains("logged in as")
        || q.contains("my user")
        || q.contains("who am i")
        || (q.contains("what") && q.contains("user") && q.contains("am"))
    {
        return Some(QueryClass::CurrentUser);
    }

    // v0.0.125: System architecture
    if q.contains("architecture")
        || q.contains("32 bit")
        || q.contains("64 bit")
        || q.contains("x86_64")
        || q.contains("arm64")
        || q.contains("aarch64")
        || q.trim() == "arch"
        || (q.contains("what") && q.contains("arch"))
    {
        return Some(QueryClass::SystemArchitecture);
    }

    // v0.0.125: Environment variables
    if q.contains("environment variable")
        || q.contains("env var")
        || q.trim() == "env"
        || q.trim() == "printenv"
        || q.contains("show env")
        || q.contains("list env")
        || (q.contains("what") && q.contains("env"))
    {
        return Some(QueryClass::EnvironmentVars);
    }

    // v0.0.126: Process tree
    if q.trim() == "pstree"
        || q.contains("process tree")
        || q.contains("process hierarchy")
        || q.contains("parent process")
        || (q.contains("show") && q.contains("process") && q.contains("tree"))
    {
        return Some(QueryClass::ProcessTree);
    }

    // v0.0.126: Open files
    if q.contains("open file")
        || q.contains("file handle")
        || q.contains("file descriptor")
        || q.trim() == "lsof"
        || (q.contains("how many") && q.contains("file") && q.contains("open"))
    {
        return Some(QueryClass::OpenFiles);
    }

    // v0.0.126: System locale
    if q.trim() == "locale"
        || q.contains("system locale")
        || q.contains("language setting")
        || q.contains("character set")
        || q.contains("encoding")
        || (q.contains("what") && q.contains("locale"))
    {
        return Some(QueryClass::SystemLocale);
    }

    // v0.0.122: Timezone info
    // v0.0.807: Added more date/time patterns
    if q.contains("timezone")
        || q.contains("time zone")
        || q.contains("locale")
        || q.contains("what time is it")
        || q.contains("what time")
        || q.contains("current time")
        || q.contains("system time")
        || q.contains("timedatectl")
        || q.contains("what date")
        || q.contains("current date")
        || q.contains("today's date")
        || q.trim() == "date"
        || q.trim() == "time"
    {
        return Some(QueryClass::TimezoneInfo);
    }

    // v0.0.130: Available shells
    if q.contains("available shell")
        || q.contains("installed shell")
        || q.contains("list shell")
        || q.contains("/etc/shells")
        || (q.contains("what") && q.contains("shell") && q.contains("available"))
    {
        return Some(QueryClass::AvailableShells);
    }

    // v0.0.309: Desktop wallpaper - MUST come before InstalledDesktops
    if q.contains("wallpaper")
        || q.contains("desktop background")
        || (q.contains("background") && q.contains("image"))
        || (q.contains("what") && q.contains("background") && !q.contains("process"))
    {
        return Some(QueryClass::DesktopWallpaper);
    }

    // v0.0.130: Installed desktops
    if q.contains("installed desktop")
        || q.contains("desktop environment")
        || q.contains("which de")
        || q.contains("what de")
        || (q.contains("gnome") && q.contains("install"))
        || (q.contains("kde") && q.contains("install"))
        || (q.contains("xfce") && q.contains("install"))
    {
        return Some(QueryClass::InstalledDesktops);
    }

    // v0.0.801: Device type - laptop vs desktop
    if (q.contains("laptop") && q.contains("desktop"))
        || q.contains("device type")
        || (q.contains("is this") && (q.contains("laptop") || q.contains("desktop")))
        || (q.contains("am i on") && (q.contains("laptop") || q.contains("desktop")))
        || q.contains("chassis")
        || (q.contains("what type") && q.contains("computer"))
        || (q.contains("what kind") && q.contains("computer"))
    {
        return Some(QueryClass::DeviceType);
    }

    // v0.0.131: Virtualization info
    if q.contains("virtualization")
        || q.contains("systemd-detect-virt")
        || (q.contains("running") && (q.contains("vm") || q.contains("container")))
        || (q.contains("inside")
            && (q.contains("vm") || q.contains("container") || q.contains("virtual")))
        || (q.contains("is this")
            && (q.contains("vm") || q.contains("virtual") || q.contains("container")))
    {
        return Some(QueryClass::VirtualizationInfo);
    }

    // v0.0.131: Coredump list
    if q.contains("coredump")
        || q.contains("core dump")
        || q.contains("crash dump")
        || q.trim() == "coredumpctl"
        || (q.contains("crash") && q.contains("list"))
    {
        return Some(QueryClass::CoredumpList);
    }

    // v0.0.133: Tmp files
    if q.contains("tmp file")
        || q.contains("temp file")
        || q.contains("/tmp")
        || q.contains("temporary file")
        || (q.contains("what") && q.contains("tmp"))
    {
        return Some(QueryClass::TmpFiles);
    }

    // v0.0.133: User groups
    if q.contains("my group")
        || q.contains("user group")
        || q.trim() == "groups"
        || (q.contains("what") && q.contains("group") && q.contains("am i"))
        || (q.contains("which") && q.contains("group"))
    {
        return Some(QueryClass::UserGroups);
    }

    // v0.0.134: Xorg log
    if q.contains("xorg")
        || q.contains("x11 error")
        || q.contains("x server")
        || q.contains("xorg.log")
        || (q.contains("display") && q.contains("error"))
    {
        return Some(QueryClass::XorgLog);
    }

    // v0.0.136: Loginctl sessions
    if q.trim() == "loginctl"
        || q.contains("loginctl session")
        || q.contains("user session")
        || q.contains("active session")
        || (q.contains("list") && q.contains("session"))
    {
        return Some(QueryClass::LoginctlSessions);
    }

    // v0.0.139: Environment variables (duplicate check)
    if q.contains("env var")
        || q.contains("environment variable")
        || q.trim() == "printenv"
        || q.trim() == "env"
        || (q.contains("show") && q.contains("environment"))
        || (q.contains("list") && q.contains("env"))
    {
        return Some(QueryClass::EnvironmentVariables);
    }

    None
}
