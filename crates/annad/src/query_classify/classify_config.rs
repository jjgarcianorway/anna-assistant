//! Configuration query classification patterns (v0.0.174).
//!
//! Editor, shell, git, packages, kernel modules.

use crate::router::QueryClass;

/// Classify configuration queries.
/// Returns Some if matched, None otherwise.
pub fn classify_config(q: &str) -> Option<QueryClass> {
    // v0.0.45: PackageCount
    if (q.contains("how many") && q.contains("package"))
        || q.contains("package count")
        || q.contains("count packages")
    {
        return Some(QueryClass::PackageCount);
    }

    // Installed packages overview
    // v0.0.390: Added (list && packages) to match "list installed packages"
    if q.contains("how many packages")
        || q.contains("packages installed")
        || q.contains("what's installed")
        || q.contains("what is installed")
        || (q.contains("list") && q.contains("packages"))
        || q.contains("installed software")
        || (q.contains("packages") && q.contains("count"))
        || q.contains("installed packages")
    {
        return Some(QueryClass::InstalledPackagesOverview);
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
        return Some(QueryClass::AppAlternatives);
    }

    // v0.45.5: Configure editor
    if (q.contains("enable")
        || q.contains("turn on")
        || q.contains("activate")
        || q.contains("set up"))
        && (q.contains("syntax highlight")
            || q.contains("line number")
            || q.contains("word wrap")
            || q.contains("auto indent")
            || q.contains("tab size")
            || q.contains("color scheme")
            || q.contains("theme"))
    {
        return Some(QueryClass::ConfigureEditor);
    }
    if (q.contains("how") || q.contains("configure") || q.contains("setup"))
        && (q.contains("vim") || q.contains("nvim") || q.contains("nano") || q.contains("emacs"))
        && (q.contains("syntax")
            || q.contains("highlight")
            || q.contains("line number")
            || q.contains("color")
            || q.contains("theme"))
    {
        return Some(QueryClass::ConfigureEditor);
    }

    // v0.0.99: Install package
    if q.starts_with("install ")
        || q.starts_with("add ")
        || q.contains("install package")
        || q.contains("install the")
        || (q.contains("can you install") && !q.contains("installed"))
        || q.contains("please install")
        || q.contains("i need to install")
        || q.contains("how do i install")
    {
        return Some(QueryClass::InstallPackage);
    }

    // v0.0.101: Configure shell
    let is_shell_config = (q.contains("bash")
        || q.contains("zsh")
        || q.contains("fish")
        || q.contains("bashrc")
        || q.contains("zshrc"))
        && (q.contains("color")
            || q.contains("prompt")
            || q.contains("syntax")
            || q.contains("highlight")
            || q.contains("history")
            || q.contains("alias")
            || q.contains("auto") && q.contains("suggest"));
    if is_shell_config {
        return Some(QueryClass::ConfigureShell);
    }

    // v0.0.101: Configure git
    let is_git_config = q.contains("git")
        && (q.contains("config")
            || q.contains("alias")
            || q.contains("username")
            || q.contains("user")
            || q.contains("email")
            || q.contains("editor")
            || q.contains("default branch")
            || q.contains("color")
            || q.contains("autocorrect")
            || q.contains("pull")
            || q.contains("credential")
            || q.contains("gpg")
            || q.contains("sign"));
    if is_git_config {
        return Some(QueryClass::ConfigureGit);
    }

    // v0.0.111: Ticket history
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
        return Some(QueryClass::TicketHistory);
    }

    // v0.0.311: System update request (ACTION to update, not just checking)
    // Must come BEFORE PackageUpdates to catch "update my system" vs "are there updates"
    let update_action_verbs = q.contains("please update")
        || q.contains("update my")
        || q.contains("run update")
        || q.contains("do update")
        || q.contains("perform update")
        || q.contains("apply update")
        || q.contains("upgrade my")
        || q.contains("upgrade system")
        || q.contains("update the system")
        || q.contains("update system")
        || (q.contains("can you") && q.contains("update"));

    if update_action_verbs {
        return Some(QueryClass::SystemUpdate);
    }

    // v0.0.122: Package updates (just checking what's available)
    if q.contains("updates available")
        || q.contains("any updates")
        || q.contains("check for updates")
        || q.contains("pending updates")
        || q.contains("upgradable")
        || q.contains("need to update")
        || (q.contains("package") && q.contains("update"))
        || q.contains("checkupdates")
    {
        return Some(QueryClass::PackageUpdates);
    }

    // v0.0.127: Installed kernels
    if q.contains("installed kernel")
        || q.contains("available kernel")
        || q.contains("linux kernel")
        || (q.contains("what") && q.contains("kernel") && q.contains("install"))
        || (q.contains("list") && q.contains("kernel"))
    {
        return Some(QueryClass::InstalledKernels);
    }

    // v0.0.132: Kernel modules
    if q.trim() == "lsmod"
        || q.contains("kernel module")
        || q.contains("loaded module")
        || (q.contains("what") && q.contains("module") && q.contains("load"))
        || (q.contains("list") && q.contains("module"))
    {
        return Some(QueryClass::KernelModules);
    }

    // v0.0.133: Dmesg errors
    if q.contains("dmesg error")
        || q.contains("kernel error")
        || q.contains("dmesg warn")
        || q.trim() == "dmesg"
        || (q.contains("kernel") && q.contains("log"))
    {
        return Some(QueryClass::DmesgErrors);
    }

    // v0.0.134: NTP status
    if q.contains("ntp")
        || q.contains("time sync")
        || q.contains("chrony")
        || (q.contains("clock") && q.contains("sync"))
        || (q.contains("time") && q.contains("server"))
    {
        return Some(QueryClass::NtpStatus);
    }

    // v0.0.136: Sysctl settings
    if q.contains("sysctl")
        || q.contains("kernel parameter")
        || q.contains("kernel setting")
        || (q.contains("show") && q.contains("kernel") && q.contains("param"))
        || q.contains("/proc/sys")
    {
        return Some(QueryClass::SysctlSettings);
    }

    // v0.0.139: Kernel command line
    if q.contains("kernel cmdline")
        || q.contains("boot param")
        || q.contains("/proc/cmdline")
        || q.contains("kernel command line")
        || (q.contains("boot") && q.contains("option"))
    {
        return Some(QueryClass::KernelCmdline);
    }

    // v0.0.139: Module parameters
    if q.contains("module param")
        || q.trim() == "modinfo"
        || q.contains("module option")
        || (q.contains("kernel") && q.contains("module") && q.contains("param"))
        || (q.contains("driver") && q.contains("param"))
    {
        return Some(QueryClass::ModuleParams);
    }

    None
}
