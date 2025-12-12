//! Command explanations database.
//!
//! Contains explanations for common Linux commands used by Anna.

use super::{FlagExplanation, LearningContext};

/// Known command explanations database
pub struct CommandExplainer;

impl CommandExplainer {
    /// Get explanation for common commands
    pub fn explain(command: &str) -> Option<LearningContext> {
        let cmd_base = command.split_whitespace().next()?;
        let mut ctx = LearningContext::new(command);

        match cmd_base {
            "df" => Self::explain_df(&mut ctx),
            "free" => Self::explain_free(&mut ctx),
            "lsblk" => Self::explain_lsblk(&mut ctx),
            "systemctl" => Self::explain_systemctl(&mut ctx, command),
            "journalctl" => Self::explain_journalctl(&mut ctx),
            "ip" => Self::explain_ip(&mut ctx),
            "lscpu" => Self::explain_lscpu(&mut ctx),
            "uname" => Self::explain_uname(&mut ctx),
            "cat" => Self::explain_cat(&mut ctx),
            "sensors" => Self::explain_sensors(&mut ctx),
            "lspci" => Self::explain_lspci(&mut ctx),
            "pacman" => Self::explain_pacman(&mut ctx),
            "ps" => Self::explain_ps(&mut ctx),
            "top" | "htop" => Self::explain_top(&mut ctx),
            "grep" => Self::explain_grep(&mut ctx),
            "find" => Self::explain_find(&mut ctx),
            "chmod" => Self::explain_chmod(&mut ctx),
            "chown" => Self::explain_chown(&mut ctx),
            "du" => Self::explain_du(&mut ctx),
            "mount" => Self::explain_mount(&mut ctx),
            "ss" | "netstat" => Self::explain_ss(&mut ctx),
            "ping" => Self::explain_ping(&mut ctx),
            "curl" | "wget" => Self::explain_curl(&mut ctx),
            "tar" => Self::explain_tar(&mut ctx),
            "git" => Self::explain_git(&mut ctx),
            "docker" => Self::explain_docker(&mut ctx),
            _ => return None,
        }

        Some(ctx)
    }

    fn explain_df(ctx: &mut LearningContext) {
        ctx.add_why("Checking disk space usage on mounted filesystems");
        ctx.add_how(
            "df (disk free) shows how much space is available on each mounted filesystem",
            vec![
                flag("-h", "Human-readable sizes (GB, MB)"),
                flag("-T", "Show filesystem type"),
            ],
        );
        ctx.add_output_meaning(
            "Use%",
            "Percentage of disk space used. Above 90% may cause issues",
        );
    }

    fn explain_free(ctx: &mut LearningContext) {
        ctx.add_why("Checking memory (RAM) usage");
        ctx.add_how(
            "free shows total, used, and available memory",
            vec![
                flag("-h", "Human-readable sizes"),
                flag("-m", "Show in megabytes"),
            ],
        );
        ctx.add_output_meaning(
            "available",
            "Memory that can be used without swapping. More important than 'free'",
        );
    }

    fn explain_lsblk(ctx: &mut LearningContext) {
        ctx.add_why("Listing block devices (disks and partitions)");
        ctx.add_how(
            "lsblk shows storage devices in a tree structure",
            vec![
                flag("-f", "Show filesystem info"),
                flag("-o", "Specify output columns"),
            ],
        );
    }

    fn explain_systemctl(ctx: &mut LearningContext, command: &str) {
        if command.contains("status") {
            ctx.add_why("Checking the status of a systemd service");
            ctx.add_output_meaning("Active: active (running)", "Service is running normally");
            ctx.add_output_meaning("Active: failed", "Service crashed or failed to start");
        } else if command.contains("list-units") {
            ctx.add_why("Listing all systemd units and their states");
        } else {
            ctx.add_why("Managing systemd services");
        }
    }

    fn explain_journalctl(ctx: &mut LearningContext) {
        ctx.add_why("Reading system logs from the journal");
        ctx.add_how(
            "journalctl queries the systemd journal for log entries",
            vec![
                flag("-u", "Filter by unit/service"),
                flag("-b", "Show logs since boot"),
                flag("-p", "Filter by priority (err, warning)"),
            ],
        );
    }

    fn explain_ip(ctx: &mut LearningContext) {
        ctx.add_why("Querying network interface information");
        ctx.add_how(
            "ip command manages network interfaces, addresses, and routing",
            vec![
                flag("addr", "Show IP addresses"),
                flag("link", "Show link-layer info"),
                flag("route", "Show routing table"),
            ],
        );
    }

    fn explain_lscpu(ctx: &mut LearningContext) {
        ctx.add_why("Getting CPU information");
        ctx.add_how(
            "lscpu displays CPU architecture information from /proc/cpuinfo",
            vec![],
        );
    }

    fn explain_uname(ctx: &mut LearningContext) {
        ctx.add_why("Getting system/kernel information");
        ctx.add_how(
            "uname prints system information",
            vec![
                flag("-r", "Kernel release version"),
                flag("-a", "All information"),
            ],
        );
    }

    fn explain_cat(ctx: &mut LearningContext) {
        ctx.add_why("Reading file contents");
        ctx.add_how("cat outputs the contents of files", vec![]);
    }

    fn explain_sensors(ctx: &mut LearningContext) {
        ctx.add_why("Reading hardware sensor data (temperature, fans, voltage)");
        ctx.add_how(
            "sensors displays readings from lm-sensors compatible chips",
            vec![],
        );
    }

    fn explain_lspci(ctx: &mut LearningContext) {
        ctx.add_why("Listing PCI devices (graphics, network, sound cards)");
        ctx.add_how(
            "lspci shows all PCI buses and devices",
            vec![
                flag("-v", "Verbose output"),
                flag("-k", "Show kernel drivers"),
            ],
        );
    }

    fn explain_pacman(ctx: &mut LearningContext) {
        ctx.add_why("Arch Linux package management");
        ctx.add_how(
            "pacman is the Arch Linux package manager",
            vec![
                flag("-S", "Sync/install packages"),
                flag("-Q", "Query local database"),
                flag("-R", "Remove packages"),
            ],
        );
    }

    fn explain_ps(ctx: &mut LearningContext) {
        ctx.add_why("Listing running processes");
        ctx.add_how(
            "ps shows snapshot of current processes",
            vec![
                flag("aux", "All users, detailed format"),
                flag("-ef", "Full-format listing"),
            ],
        );
    }

    fn explain_top(ctx: &mut LearningContext) {
        ctx.add_why("Interactive process monitoring");
        ctx.add_how(
            "Shows real-time process activity, CPU, and memory usage",
            vec![],
        );
        ctx.add_output_meaning("%CPU", "CPU usage percentage. High values indicate busy process");
        ctx.add_output_meaning("%MEM", "Memory usage percentage of total RAM");
    }

    fn explain_grep(ctx: &mut LearningContext) {
        ctx.add_why("Searching for patterns in text");
        ctx.add_how(
            "grep searches for patterns using regular expressions",
            vec![
                flag("-i", "Case-insensitive search"),
                flag("-r", "Recursive search in directories"),
                flag("-n", "Show line numbers"),
            ],
        );
    }

    fn explain_find(ctx: &mut LearningContext) {
        ctx.add_why("Searching for files in directory hierarchy");
        ctx.add_how(
            "find walks directory trees looking for files matching criteria",
            vec![
                flag("-name", "Match by filename pattern"),
                flag("-type", "Filter by type (f=file, d=directory)"),
                flag("-mtime", "Filter by modification time"),
            ],
        );
    }

    fn explain_chmod(ctx: &mut LearningContext) {
        ctx.add_why("Changing file permissions");
        ctx.add_how(
            "chmod modifies read/write/execute permissions for files",
            vec![
                flag("+x", "Add execute permission"),
                flag("-R", "Recursive (apply to all files)"),
            ],
        );
    }

    fn explain_chown(ctx: &mut LearningContext) {
        ctx.add_why("Changing file ownership");
        ctx.add_how(
            "chown sets the user and group owner of files",
            vec![flag("-R", "Recursive (apply to all files)")],
        );
    }

    fn explain_du(ctx: &mut LearningContext) {
        ctx.add_why("Checking disk usage of files/directories");
        ctx.add_how(
            "du estimates file space usage",
            vec![
                flag("-h", "Human-readable sizes"),
                flag("-s", "Summary (total only)"),
                flag("--max-depth", "Limit directory depth"),
            ],
        );
    }

    fn explain_mount(ctx: &mut LearningContext) {
        ctx.add_why("Listing or mounting filesystems");
        ctx.add_how(
            "mount attaches filesystems to the directory tree",
            vec![
                flag("-t", "Specify filesystem type"),
                flag("-o", "Mount options (ro, rw, etc.)"),
            ],
        );
    }

    fn explain_ss(ctx: &mut LearningContext) {
        ctx.add_why("Showing network connections and listening ports");
        ctx.add_how(
            "ss (socket statistics) displays network socket information",
            vec![
                flag("-t", "TCP connections"),
                flag("-l", "Listening sockets only"),
                flag("-p", "Show process using socket"),
                flag("-n", "Numeric (don't resolve names)"),
            ],
        );
    }

    fn explain_ping(ctx: &mut LearningContext) {
        ctx.add_why("Testing network connectivity to a host");
        ctx.add_how(
            "ping sends ICMP echo requests to test if host is reachable",
            vec![flag("-c", "Count (number of pings to send)")],
        );
        ctx.add_output_meaning(
            "time=",
            "Round-trip time in ms. Lower is better, >100ms may indicate issues",
        );
    }

    fn explain_curl(ctx: &mut LearningContext) {
        ctx.add_why("Downloading files or making HTTP requests");
        ctx.add_how(
            "curl transfers data from or to a server",
            vec![
                flag("-O", "Save with remote filename"),
                flag("-L", "Follow redirects"),
                flag("-s", "Silent mode"),
            ],
        );
    }

    fn explain_tar(ctx: &mut LearningContext) {
        ctx.add_why("Creating or extracting archive files");
        ctx.add_how(
            "tar archives multiple files into a single file",
            vec![
                flag("-c", "Create archive"),
                flag("-x", "Extract archive"),
                flag("-z", "Compress with gzip"),
                flag("-v", "Verbose (show files)"),
                flag("-f", "Specify archive filename"),
            ],
        );
    }

    fn explain_git(ctx: &mut LearningContext) {
        ctx.add_why("Version control operations");
        ctx.add_how(
            "git is a distributed version control system",
            vec![
                flag("status", "Show working tree status"),
                flag("pull", "Fetch and merge remote changes"),
                flag("push", "Upload local commits to remote"),
            ],
        );
    }

    fn explain_docker(ctx: &mut LearningContext) {
        ctx.add_why("Container management operations");
        ctx.add_how(
            "docker manages application containers",
            vec![
                flag("ps", "List running containers"),
                flag("images", "List images"),
                flag("run", "Create and start container"),
            ],
        );
    }
}

/// Helper to create flag explanation
fn flag(name: &str, meaning: &str) -> FlagExplanation {
    FlagExplanation {
        flag: name.to_string(),
        meaning: meaning.to_string(),
    }
}
