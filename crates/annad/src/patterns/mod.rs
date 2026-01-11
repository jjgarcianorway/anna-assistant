//! Common Linux patterns that should get instant answers without clarification.
//!
//! v0.0.909: Added to reduce over-clarification (80% rate in testing).
//! v0.0.910: Added factual patterns for instant answers to common info queries.
//! v0.0.916: Added development patterns for git, docker, build tools.
//! v0.0.917: Added security patterns for firewall, permissions, users, SSH.
//! v0.0.918: Added desktop patterns for GNOME, KDE, Wayland, X11.
//! v0.0.926: Added pattern pre-execution for instant grounded answers.
//! v0.0.947: Added howto patterns for common task instructions.
//! v0.0.948: Added network patterns for connectivity and configuration.
//! v0.0.949: Added hardware patterns for sensors, battery, CPU.
//! v0.0.950: Added gaming patterns for Steam, Wine, Proton, controllers.
//! v0.0.951: Added boot patterns for GRUB, EFI, kernel, initramfs.
//! v0.0.956: Added fuzzy matching for typo tolerance.
//! v0.0.957: Added container patterns for Docker, Podman, VMs.
//! v0.0.958: Added logs patterns for journalctl, dmesg, log analysis.
//! v0.0.959: Added audio patterns for PipeWire, PulseAudio, ALSA.
//! v0.0.960: Added power patterns for battery, suspend, hibernate.
//! v0.0.961: Added systemd patterns for services, units, timers, targets.
//! v0.0.962: Added filesystem patterns for mounts, LVM, RAID, btrfs.
//! v0.0.963: Added process patterns for ps, top, kill, zombies.
//! v0.0.964: Added cron patterns for crontab, at, anacron.
//! v0.0.965: Added users patterns for user/group management.
//! v0.0.966: Added time patterns for datetime, timezone, NTP.
//! v0.0.967: Added printing patterns for CUPS, printers, print jobs.
//! v0.0.968: Added backup patterns for rsync, borg, restic, tar.
//! v0.0.969: Added locale patterns for keyboard, language, fonts.
//! v0.0.970: Added SSH patterns for connections, config, troubleshooting.
//! v0.0.971: Added memory patterns for RAM, swap, cache, OOM.
//! v0.0.972: Added Bluetooth patterns for devices, audio, troubleshooting.
//! v0.0.974: Added virtualization patterns for KVM, QEMU, libvirt.
//! v0.0.975: Added display patterns for monitors, resolution, scaling.
//! v0.0.976: Added encryption patterns for LUKS, GPG, disk encryption.
//! v0.0.977: Added NVIDIA patterns for nvidia-smi, drivers, Optimus.
//! v0.0.978: Added AUR patterns for yay, paru, makepkg.
//! v0.0.979: Added Flatpak, Snap, AppImage patterns.
//! v0.0.980: Added system info patterns for neofetch, inxi, dmidecode.
//! v0.0.981: Fixed critical substring matching bugs (id, at) with word boundaries.
//! v0.0.982: Added bandwidth/traffic monitoring patterns.
//! v0.0.983: Added window manager patterns for Hyprland, Sway, i3.
//! v0.0.984: Added kernel/module patterns for lsmod, modprobe, sysctl.
//! v0.0.985: Added ZFS patterns for zpool, zfs, snapshots.
//! v0.0.986: Added SMART disk health patterns for smartctl, nvme.
//! These are well-known issues with standard solutions.

mod pacman;
mod errors;
mod recovery;
mod performance;
mod factual;
mod development;
mod security;
mod desktop;
mod howto;
mod network;
mod hardware;
mod gaming;
mod boot;
mod container;
mod logs;
mod audio;
mod power;
mod systemd;
mod filesystem;
mod process;
mod cron;
mod users;
mod time;
mod printing;
mod backup;
mod locale;
mod ssh;
mod memory;
mod bluetooth;
mod virtualization;
mod display;
mod encryption;
mod nvidia;
mod aur;
mod appimage;
mod sysinfo;
mod wm;
mod kernel;
mod zfs;
mod smart;

use anna_shared::rpc::DeepUnderstanding;
use std::collections::HashMap;
use std::sync::RwLock;
use tracing::debug;

/// v0.0.981: Check if query contains keyword as a whole word (not substring)
/// Prevents "bandwidth" matching "id" or "what" matching "at"
pub fn contains_word(query: &str, word: &str) -> bool {
    // For very short words (1-2 chars), require word boundaries
    if word.len() <= 2 {
        // Use regex-like word boundary check
        for (i, _) in query.match_indices(word) {
            let before_ok = i == 0 || !query.as_bytes()[i - 1].is_ascii_alphanumeric();
            let after_ok = i + word.len() >= query.len()
                || !query.as_bytes()[i + word.len()].is_ascii_alphanumeric();
            if before_ok && after_ok {
                return true;
            }
        }
        false
    } else {
        // For longer words, simple contains is fine
        query.contains(word)
    }
}

/// v0.0.954: Pattern usage statistics
static PATTERN_STATS: RwLock<Option<HashMap<String, PatternStat>>> = RwLock::new(None);

/// v0.0.954: Statistics for a pattern category
#[derive(Clone, Debug, Default)]
pub struct PatternStat {
    pub hit_count: u64,
    pub last_hit: Option<std::time::Instant>,
}

/// v0.0.954: Record a pattern hit for statistics
fn record_pattern_hit(category: &str) {
    if let Ok(mut guard) = PATTERN_STATS.write() {
        let stats = guard.get_or_insert_with(HashMap::new);
        let entry = stats.entry(category.to_string()).or_default();
        entry.hit_count += 1;
        entry.last_hit = Some(std::time::Instant::now());
    }
}

/// v0.0.954: Get pattern usage statistics
pub fn get_pattern_stats() -> Vec<(String, u64)> {
    if let Ok(guard) = PATTERN_STATS.read() {
        if let Some(ref stats) = *guard {
            let mut result: Vec<_> = stats.iter()
                .map(|(k, v)| (k.clone(), v.hit_count))
                .collect();
            result.sort_by(|a, b| b.1.cmp(&a.1)); // Sort by hit count descending
            return result;
        }
    }
    Vec::new()
}

/// v0.0.954: Get total pattern hits
pub fn get_total_pattern_hits() -> u64 {
    if let Ok(guard) = PATTERN_STATS.read() {
        if let Some(ref stats) = *guard {
            return stats.values().map(|s| s.hit_count).sum();
        }
    }
    0
}

/// v0.0.952: Synonym pairs for query expansion
/// Each pair: (word, synonym) - if word is in query, also check with synonym
/// v0.0.973: Expanded synonym mappings for better query coverage
const SYNONYMS: &[(&str, &str)] = &[
    // Memory
    ("memory", "ram"),
    ("ram", "memory"),
    ("mem", "memory"),
    // Storage
    ("disk", "storage"),
    ("storage", "disk"),
    ("drive", "disk"),
    ("ssd", "disk"),
    ("hdd", "disk"),
    ("nvme", "disk"),
    ("hard drive", "disk"),
    ("filesystem", "disk"),
    // Network
    ("internet", "network"),
    ("network", "internet"),
    ("wifi", "wireless"),
    ("wireless", "wifi"),
    ("ethernet", "wired"),
    ("lan", "network"),
    ("wan", "network"),
    ("connection", "network"),
    ("connectivity", "connection"),
    // CPU
    ("processor", "cpu"),
    ("cpu", "processor"),
    ("cores", "cpu"),
    // GPU
    ("graphics", "gpu"),
    ("gpu", "graphics"),
    ("video card", "gpu"),
    ("nvidia", "gpu"),
    ("amd", "gpu"),
    ("intel", "gpu"),
    // Temperature
    ("temperature", "temp"),
    ("temp", "temperature"),
    ("hot", "temperature"),
    ("overheating", "temperature"),
    ("thermal", "temperature"),
    ("heat", "temperature"),
    // Power
    ("power", "battery"),
    ("charging", "battery"),
    ("suspend", "sleep"),
    ("sleep", "suspend"),
    ("hibernate", "suspend"),
    // Package management
    ("package", "packages"),
    ("packages", "package"),
    ("software", "package"),
    ("app", "package"),
    ("application", "package"),
    ("program", "package"),
    // Service
    ("daemon", "service"),
    ("service", "daemon"),
    ("unit", "service"),
    // User
    ("account", "user"),
    ("user", "account"),
    ("login", "user"),
    // Files
    ("folder", "directory"),
    ("directory", "folder"),
    ("file", "files"),
    ("files", "file"),
    ("path", "directory"),
    // Boot
    ("bootloader", "grub"),
    ("startup", "boot"),
    ("reboot", "boot"),
    // Audio
    ("audio", "sound"),
    ("sound", "audio"),
    ("speaker", "audio"),
    ("speakers", "audio"),
    ("volume", "audio"),
    ("headphone", "audio"),
    ("headphones", "audio"),
    ("microphone", "audio"),
    ("mic", "microphone"),
    // Display
    ("screen", "display"),
    ("display", "screen"),
    ("monitor", "display"),
    ("monitors", "display"),
    ("resolution", "display"),
    // Printing
    ("printer", "print"),
    ("print", "printer"),
    ("printing", "print"),
    ("cups", "printer"),
    // Time/Date
    ("clock", "time"),
    ("time", "clock"),
    ("date", "time"),
    ("timezone", "time"),
    // Processes
    ("process", "processes"),
    ("processes", "process"),
    ("task", "process"),
    ("tasks", "process"),
    ("running", "process"),
    ("pid", "process"),
    // Configuration
    ("config", "configuration"),
    ("configuration", "config"),
    ("settings", "config"),
    ("options", "config"),
    ("preferences", "settings"),
    // Logs
    ("log", "logs"),
    ("logs", "log"),
    ("journal", "logs"),
    ("journalctl", "logs"),
    ("dmesg", "logs"),
    // SSH
    ("ssh", "remote"),
    ("remote", "ssh"),
    // Backup
    ("backup", "backups"),
    ("backups", "backup"),
    ("restore", "backup"),
    // Security
    ("firewall", "security"),
    ("password", "security"),
    ("permissions", "security"),
    // Cron/Schedule
    ("cron", "schedule"),
    ("schedule", "cron"),
    ("scheduled", "cron"),
    ("crontab", "cron"),
    // Swap
    ("swap", "swapfile"),
    ("swapfile", "swap"),
    // Common verbs
    ("show", "list"),
    ("display", "show"),
    ("check", "show"),
    ("view", "show"),
    ("get", "show"),
    ("find", "search"),
    ("search", "find"),
    ("look for", "find"),
    ("start", "enable"),
    ("stop", "disable"),
    ("restart", "reload"),
    ("install", "add"),
    ("remove", "uninstall"),
    ("delete", "remove"),
    ("update", "upgrade"),
    ("upgrade", "update"),
    // Common adjectives
    ("current", "active"),
    ("active", "running"),
    ("failed", "error"),
    ("broken", "failed"),
    ("slow", "performance"),
    ("fast", "performance"),
    ("high", "usage"),
    ("low", "usage"),
];

/// v0.0.952: Expand query with synonyms for better pattern matching
fn expand_with_synonyms(query: &str) -> String {
    let mut expanded = query.to_string();
    for (word, synonym) in SYNONYMS {
        if query.contains(word) && !query.contains(synonym) {
            // Add synonym to query for pattern matching
            expanded.push(' ');
            expanded.push_str(synonym);
        }
    }
    expanded
}

/// v0.0.955: Normalize query for better pattern matching
/// Removes extra whitespace, punctuation, and common filler words
fn normalize_query(q: &str) -> String {
    let mut result = q.to_lowercase();

    // Remove common punctuation
    result = result.replace(['?', '!', '.', ',', ':', ';', '"', '\''], " ");

    // Remove filler words that don't add meaning
    let fillers = ["please", "can you", "could you", "would you", "i want to",
                   "i need to", "help me", "tell me", "show me how to",
                   "how do i", "how can i", "what's the", "what is the"];
    for filler in fillers {
        result = result.replace(filler, " ");
    }

    // Collapse multiple spaces into one
    let mut prev_space = false;
    result = result.chars().filter(|c| {
        if c.is_whitespace() {
            if prev_space { return false; }
            prev_space = true;
        } else {
            prev_space = false;
        }
        true
    }).collect();

    result.trim().to_string()
}

/// v0.0.956: Common misspellings of Linux/tech terms
/// Format: (misspelling, correct_spelling)
/// v0.0.973: Expanded typo corrections
const TYPO_CORRECTIONS: &[(&str, &str)] = &[
    // Package managers
    ("pacaman", "pacman"), ("pacmn", "pacman"), ("packman", "pacman"),
    ("pamcan", "pacman"), ("pacmam", "pacman"),
    ("systemclt", "systemctl"), ("sytemctl", "systemctl"), ("systemcl", "systemctl"),
    ("systmctl", "systemctl"),
    ("journalclt", "journalctl"), ("journctl", "journalctl"), ("jounalctl", "journalctl"),
    // Common terms
    ("kernal", "kernel"), ("kerne", "kernel"), ("kernle", "kernel"),
    ("wif", "wifi"), ("wfii", "wifi"), ("wiif", "wifi"),
    ("bluetoth", "bluetooth"), ("bluethooth", "bluetooth"), ("blutooth", "bluetooth"),
    ("bluetooh", "bluetooth"), ("bluettoth", "bluetooth"),
    ("netwrok", "network"), ("newtork", "network"), ("netowrk", "network"),
    ("memroy", "memory"), ("memeory", "memory"), ("memor", "memory"),
    ("stoarge", "storage"), ("stroage", "storage"), ("sotrage", "storage"),
    ("direcotry", "directory"), ("dirctory", "directory"), ("directroy", "directory"),
    ("permisions", "permissions"), ("permsisions", "permissions"), ("permssions", "permissions"),
    ("temperture", "temperature"), ("temprature", "temperature"), ("tempurature", "temperature"),
    // Commands
    ("grb", "grub"), ("grbu", "grub"),
    ("dokcer", "docker"), ("docekr", "docker"), ("dcoker", "docker"),
    ("firwall", "firewall"), ("firewll", "firewall"), ("firewal", "firewall"),
    ("crontba", "crontab"), ("corntab", "crontab"), ("crontb", "crontab"),
    // Hardware
    ("grahpics", "graphics"), ("grpahics", "graphics"), ("graphcis", "graphics"),
    ("processer", "processor"), ("procesor", "processor"), ("proccessor", "processor"),
    ("baterry", "battery"), ("battrey", "battery"), ("batery", "battery"),
    // Services
    ("servcie", "service"), ("serivce", "service"), ("sevice", "service"),
    ("deamon", "daemon"), ("dameon", "daemon"),
    // Actions
    ("instal", "install"), ("intall", "install"), ("isntall", "install"),
    ("uninstal", "uninstall"), ("unintall", "uninstall"),
    ("updte", "update"), ("udpate", "update"), ("upate", "update"),
    ("upgarde", "upgrade"), ("upgrad", "upgrade"), ("upgade", "upgrade"),
    ("rebbot", "reboot"), ("reobot", "reboot"), ("reeboot", "reboot"),
    ("shutdwon", "shutdown"), ("shudown", "shutdown"), ("shutodwn", "shutdown"),
    // File system
    ("partiton", "partition"), ("parttion", "partition"), ("parition", "partition"),
    ("formating", "formatting"), ("fomratting", "formatting"),
    ("mountig", "mounting"), ("moutning", "mounting"),
    // Audio
    ("pipewrie", "pipewire"), ("pipewie", "pipewire"), ("pipwire", "pipewire"),
    ("pulsaudio", "pulseaudio"), ("pusleaudio", "pulseaudio"), ("pulseadio", "pulseaudio"),
    ("headpohnes", "headphones"), ("headhpones", "headphones"), ("headphons", "headphones"),
    ("spekaers", "speakers"), ("spaekers", "speakers"), ("spekers", "speakers"),
    // Printing
    ("pritner", "printer"), ("printr", "printer"), ("prniter", "printer"),
    // SSH
    ("shh", "ssh"), ("shs", "ssh"),
    // Time
    ("timezoen", "timezone"), ("timezon", "timezone"), ("tiemzone", "timezone"),
    // Users
    ("passwrod", "password"), ("pasword", "password"), ("passowrd", "password"),
    ("usernmae", "username"), ("usernam", "username"), ("usrname", "username"),
    // Backup
    ("rsynce", "rsync"), ("rsynv", "rsync"),
    ("bakup", "backup"), ("bakcup", "backup"), ("backpu", "backup"),
    // Locale
    ("keyboad", "keyboard"), ("keybord", "keyboard"), ("keybaord", "keyboard"),
    ("langauge", "language"), ("languge", "language"), ("langage", "language"),
    // Swap
    ("swpa", "swap"), ("sawp", "swap"),
    ("swappines", "swappiness"), ("swapiness", "swappiness"),
    // Process
    ("proceses", "processes"), ("porcess", "process"), ("proccess", "process"),
    ("zombi", "zombie"), ("zombies", "zombie"), ("zombei", "zombie"),
];

/// v0.0.956: Apply typo corrections to query
fn fix_typos(q: &str) -> String {
    let mut result = q.to_string();
    for (typo, correction) in TYPO_CORRECTIONS {
        if result.contains(typo) {
            result = result.replace(typo, correction);
        }
    }
    result
}

/// v0.0.956: Calculate simple edit distance (Levenshtein) for short strings
/// Returns the minimum number of single-character edits needed
fn edit_distance(a: &str, b: &str) -> usize {
    let a_chars: Vec<char> = a.chars().collect();
    let b_chars: Vec<char> = b.chars().collect();
    let len_a = a_chars.len();
    let len_b = b_chars.len();

    // Quick exit for empty strings
    if len_a == 0 { return len_b; }
    if len_b == 0 { return len_a; }

    // Don't compute for long strings (performance)
    if len_a > 15 || len_b > 15 { return usize::MAX; }

    let mut matrix = vec![vec![0usize; len_b + 1]; len_a + 1];

    for i in 0..=len_a { matrix[i][0] = i; }
    for j in 0..=len_b { matrix[0][j] = j; }

    for i in 1..=len_a {
        for j in 1..=len_b {
            let cost = if a_chars[i-1] == b_chars[j-1] { 0 } else { 1 };
            matrix[i][j] = (matrix[i-1][j] + 1)
                .min(matrix[i][j-1] + 1)
                .min(matrix[i-1][j-1] + cost);
        }
    }

    matrix[len_a][len_b]
}

/// v0.0.956: Key terms that patterns commonly check for
/// If user types something close to these, we can fuzzy-match
const FUZZY_TARGETS: &[&str] = &[
    "disk", "memory", "cpu", "gpu", "ram", "storage", "network", "wifi",
    "bluetooth", "battery", "temperature", "kernel", "services", "processes",
    "packages", "installed", "running", "failed", "errors", "logs", "boot",
    "grub", "partition", "mount", "firewall", "ports", "ssh", "docker",
    "steam", "wine", "proton", "vulkan", "opengl", "audio", "sound",
    "display", "monitor", "resolution", "wayland", "gnome", "kde", "plasma",
];

/// v0.0.956: Try to fuzzy-match query words to known terms
fn fuzzy_correct_query(q: &str) -> Option<String> {
    let words: Vec<&str> = q.split_whitespace().collect();
    let mut corrected = false;
    let mut result_words = Vec::new();

    for word in words {
        let mut best_match = word.to_string();
        let mut best_distance = 3; // Track best match (allow up to 2 edits)

        // Only try to correct words that are 4+ characters
        if word.len() >= 4 {
            for target in FUZZY_TARGETS {
                let dist = edit_distance(word, target);
                if dist > 0 && dist <= 2 && dist < best_distance {
                    // Allow at most 2 edits, prefer closer matches
                    best_match = target.to_string();
                    best_distance = dist;
                    corrected = true;
                }
            }
        }
        result_words.push(best_match);
    }

    if corrected {
        Some(result_words.join(" "))
    } else {
        None
    }
}

/// Check if a question matches a common pattern that has a known solution.
/// Returns Some(DeepUnderstanding) with high confidence if matched.
pub fn match_common_pattern(question: &str) -> Option<DeepUnderstanding> {
    let q = question.to_lowercase();

    // Try direct match first
    if let Some(result) = match_patterns_internal(&q) {
        return Some(result);
    }

    // v0.0.952: Try with synonym expansion
    let expanded = expand_with_synonyms(&q);
    if expanded != q {
        debug!("Pattern: trying synonym expansion: {} -> {}", q, expanded);
        if let Some(result) = match_patterns_internal(&expanded) {
            return Some(result);
        }
    }

    // v0.0.955: Try with normalized query
    let normalized = normalize_query(&q);
    if normalized != q {
        debug!("Pattern: trying normalized query: {} -> {}", q, normalized);
        if let Some(result) = match_patterns_internal(&normalized) {
            return Some(result);
        }
        // Try normalized + synonyms
        let norm_expanded = expand_with_synonyms(&normalized);
        if norm_expanded != normalized {
            if let Some(result) = match_patterns_internal(&norm_expanded) {
                return Some(result);
            }
        }
    }

    // v0.0.956: Try with known typo corrections
    let typo_fixed = fix_typos(&q);
    if typo_fixed != q {
        debug!("Pattern: trying typo correction: {} -> {}", q, typo_fixed);
        if let Some(result) = match_patterns_internal(&typo_fixed) {
            return Some(result);
        }
        // Try typo-fixed + synonyms
        let typo_expanded = expand_with_synonyms(&typo_fixed);
        if let Some(result) = match_patterns_internal(&typo_expanded) {
            return Some(result);
        }
    }

    // v0.0.956: Try fuzzy matching (edit distance) as last resort
    if let Some(fuzzy_corrected) = fuzzy_correct_query(&q) {
        debug!("Pattern: trying fuzzy correction: {} -> {}", q, fuzzy_corrected);
        if let Some(result) = match_patterns_internal(&fuzzy_corrected) {
            return Some(result);
        }
        // Try fuzzy + synonyms
        let fuzzy_expanded = expand_with_synonyms(&fuzzy_corrected);
        if let Some(result) = match_patterns_internal(&fuzzy_expanded) {
            return Some(result);
        }
    }

    None
}

/// Internal pattern matching (called with original and expanded queries)
/// v0.0.954: Now tracks which category matched for statistics
fn match_patterns_internal(q: &str) -> Option<DeepUnderstanding> {
    // Check each pattern category (order matters - more specific first)
    // Track which category matched for statistics

    if let Some(r) = factual::match_patterns(q) {
        record_pattern_hit("factual");
        return Some(r);
    }
    if let Some(r) = hardware::match_patterns(q) {
        record_pattern_hit("hardware");
        return Some(r);
    }
    if let Some(r) = network::match_patterns(q) {
        record_pattern_hit("network");
        return Some(r);
    }
    if let Some(r) = gaming::match_patterns(q) {
        record_pattern_hit("gaming");
        return Some(r);
    }
    if let Some(r) = boot::match_patterns(q) {
        record_pattern_hit("boot");
        return Some(r);
    }
    if let Some(r) = container::match_patterns(q) {
        record_pattern_hit("container");
        return Some(r);
    }
    if let Some(r) = logs::match_patterns(q) {
        record_pattern_hit("logs");
        return Some(r);
    }
    if let Some(r) = audio::match_patterns(q) {
        record_pattern_hit("audio");
        return Some(r);
    }
    if let Some(r) = power::match_patterns(q) {
        record_pattern_hit("power");
        return Some(r);
    }
    if let Some(r) = systemd::match_patterns(q) {
        record_pattern_hit("systemd");
        return Some(r);
    }
    if let Some(r) = filesystem::match_patterns(q) {
        record_pattern_hit("filesystem");
        return Some(r);
    }
    if let Some(r) = process::match_patterns(q) {
        record_pattern_hit("process");
        return Some(r);
    }
    if let Some(r) = cron::match_patterns(q) {
        record_pattern_hit("cron");
        return Some(r);
    }
    if let Some(r) = users::match_patterns(q) {
        record_pattern_hit("users");
        return Some(r);
    }
    if let Some(r) = time::match_patterns(q) {
        record_pattern_hit("time");
        return Some(r);
    }
    if let Some(r) = printing::match_patterns(q) {
        record_pattern_hit("printing");
        return Some(r);
    }
    if let Some(r) = backup::match_patterns(q) {
        record_pattern_hit("backup");
        return Some(r);
    }
    if let Some(r) = locale::match_patterns(q) {
        record_pattern_hit("locale");
        return Some(r);
    }
    if let Some(r) = ssh::match_patterns(q) {
        record_pattern_hit("ssh");
        return Some(r);
    }
    if let Some(r) = memory::match_patterns(q) {
        record_pattern_hit("memory");
        return Some(r);
    }
    if let Some(r) = bluetooth::match_patterns(q) {
        record_pattern_hit("bluetooth");
        return Some(r);
    }
    if let Some(r) = virtualization::match_patterns(q) {
        record_pattern_hit("virtualization");
        return Some(r);
    }
    if let Some(r) = display::match_patterns(q) {
        record_pattern_hit("display");
        return Some(r);
    }
    if let Some(r) = encryption::match_patterns(q) {
        record_pattern_hit("encryption");
        return Some(r);
    }
    if let Some(r) = nvidia::match_patterns(q) {
        record_pattern_hit("nvidia");
        return Some(r);
    }
    if let Some(r) = aur::match_patterns(q) {
        record_pattern_hit("aur");
        return Some(r);
    }
    if let Some(r) = appimage::match_patterns(q) {
        record_pattern_hit("appimage");
        return Some(r);
    }
    if let Some(r) = sysinfo::match_patterns(q) {
        record_pattern_hit("sysinfo");
        return Some(r);
    }
    if let Some(r) = wm::match_patterns(q) {
        record_pattern_hit("wm");
        return Some(r);
    }
    if let Some(r) = kernel::match_patterns(q) {
        record_pattern_hit("kernel");
        return Some(r);
    }
    if let Some(r) = zfs::match_patterns(q) {
        record_pattern_hit("zfs");
        return Some(r);
    }
    if let Some(r) = smart::match_patterns(q) {
        record_pattern_hit("smart");
        return Some(r);
    }
    if let Some(r) = development::match_patterns(q) {
        record_pattern_hit("development");
        return Some(r);
    }
    if let Some(r) = security::match_patterns(q) {
        record_pattern_hit("security");
        return Some(r);
    }
    if let Some(r) = desktop::match_patterns(q) {
        record_pattern_hit("desktop");
        return Some(r);
    }
    if let Some(r) = pacman::match_patterns(q) {
        record_pattern_hit("pacman");
        return Some(r);
    }
    if let Some(r) = recovery::match_patterns(q) {
        record_pattern_hit("recovery");
        return Some(r);
    }
    if let Some(r) = errors::match_patterns(q) {
        record_pattern_hit("errors");
        return Some(r);
    }
    if let Some(r) = howto::match_patterns(q) {
        record_pattern_hit("howto");
        return Some(r);
    }
    if let Some(r) = performance::match_patterns(q) {
        record_pattern_hit("performance");
        return Some(r);
    }

    None
}

/// v0.0.926: Result of pattern pre-execution
pub struct PatternPreExecResult {
    pub understanding: DeepUnderstanding,
    pub command_outputs: Vec<(String, String)>,
}

/// v0.0.926: Match pattern and pre-execute suggested commands for grounded answers
/// This provides fresh command output to the LLM without needing an extra round-trip
pub fn match_and_preexec(question: &str) -> Option<PatternPreExecResult> {
    use crate::core_loop::execute_command;

    let understanding = match_common_pattern(question)?;

    // Only pre-execute for high-confidence factual queries
    if understanding.confidence < 0.85 || understanding.suggested_commands.is_empty() {
        return Some(PatternPreExecResult {
            understanding,
            command_outputs: vec![],
        });
    }

    // Execute up to 3 suggested commands
    let mut outputs = Vec::new();
    for cmd in understanding.suggested_commands.iter().take(3) {
        // Skip dangerous commands (pre-execution should be read-only)
        let cmd_lower = cmd.to_lowercase();
        if cmd_lower.contains("rm ")
            || cmd_lower.contains("dd ")
            || cmd_lower.contains("mkfs")
            || cmd_lower.contains("> /")
            || cmd_lower.contains("sudo ")
        {
            debug!("Pattern pre-exec: skipping potentially dangerous command: {}", cmd);
            continue;
        }

        match execute_command(cmd) {
            Ok(output) if !output.trim().is_empty() => {
                debug!("Pattern pre-exec: got output for '{}'", cmd);
                outputs.push((cmd.clone(), output));
            }
            Ok(_) => {
                debug!("Pattern pre-exec: empty output for '{}'", cmd);
            }
            Err(e) => {
                debug!("Pattern pre-exec: failed '{}': {}", cmd, e);
            }
        }
    }

    Some(PatternPreExecResult {
        understanding,
        command_outputs: outputs,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pacman_database_locked() {
        let result = match_common_pattern("pacman says database is locked");
        assert!(result.is_some());
        let u = result.unwrap();
        assert_eq!(u.confidence, 0.95);
        assert!(!u.needs_confirmation);
    }

    #[test]
    fn test_deleted_usr_bin() {
        let result = match_common_pattern("I accidentally deleted /usr/bin");
        assert!(result.is_some());
        assert!(!result.unwrap().needs_confirmation);
    }

    #[test]
    fn test_fan_idle() {
        let result = match_common_pattern("why does my fan spin up when the system is idle");
        assert!(result.is_some());
    }

    #[test]
    fn test_no_match() {
        let result = match_common_pattern("what is the meaning of life");
        assert!(result.is_none());
    }

    #[test]
    fn test_contains_word() {
        // Short words need word boundaries
        assert!(contains_word("my id", "id"));
        assert!(contains_word("show id", "id"));
        assert!(!contains_word("bandwidth", "id")); // id is substring, not word
        assert!(!contains_word("idle system", "id")); // id is substring of idle
        assert!(contains_word("what at jobs", "at"));
        assert!(!contains_word("what jobs", "at")); // at is substring of what
        // Longer words use simple contains
        assert!(contains_word("show kernel version", "kernel"));
        assert!(contains_word("kernel", "kernel"));
    }

    // Factual pattern tests
    #[test]
    fn test_factual_disk_usage() {
        assert!(match_common_pattern("what is my disk usage").is_some());
        assert!(match_common_pattern("show disk space").is_some());
    }

    #[test]
    fn test_factual_ram() {
        assert!(match_common_pattern("how much ram do I have").is_some());
        assert!(match_common_pattern("total memory").is_some());
    }

    #[test]
    fn test_factual_gpu() {
        assert!(match_common_pattern("what gpu do I have").is_some());
        assert!(match_common_pattern("which graphics card").is_some());
    }

    #[test]
    fn test_factual_ip() {
        assert!(match_common_pattern("what is my ip address").is_some());
        assert!(match_common_pattern("show my ip").is_some());
    }

    #[test]
    fn test_factual_kernel() {
        assert!(match_common_pattern("what kernel am I running").is_some());
        assert!(match_common_pattern("kernel version").is_some());
    }

    #[test]
    fn test_factual_services() {
        assert!(match_common_pattern("list failed services").is_some());
        assert!(match_common_pattern("show running services").is_some());
    }

    // Development pattern tests
    #[test]
    fn test_dev_git() {
        assert!(match_common_pattern("git status").is_some());
        assert!(match_common_pattern("show git log").is_some());
    }

    #[test]
    fn test_dev_docker() {
        assert!(match_common_pattern("list docker containers").is_some());
        assert!(match_common_pattern("docker images").is_some());
    }

    #[test]
    fn test_dev_build_tools() {
        assert!(match_common_pattern("cargo version").is_some());
        assert!(match_common_pattern("node version").is_some());
    }

    // Security pattern tests
    #[test]
    fn test_sec_firewall() {
        assert!(match_common_pattern("firewall status").is_some());
        assert!(match_common_pattern("ufw status").is_some());
    }

    #[test]
    fn test_sec_users() {
        assert!(match_common_pattern("list all users").is_some());
        assert!(match_common_pattern("who has sudo access").is_some());
    }

    #[test]
    fn test_sec_ssh() {
        assert!(match_common_pattern("ssh key").is_some());
        assert!(match_common_pattern("ssh status").is_some());
    }

    // Desktop pattern tests
    #[test]
    fn test_desktop_display_server() {
        assert!(match_common_pattern("wayland or x11").is_some());
        assert!(match_common_pattern("which desktop am I running").is_some());
    }

    #[test]
    fn test_desktop_gnome() {
        assert!(match_common_pattern("gnome version").is_some());
        assert!(match_common_pattern("gnome extensions").is_some());
    }

    #[test]
    fn test_desktop_kde() {
        assert!(match_common_pattern("plasma version").is_some());
        assert!(match_common_pattern("kde settings").is_some());
    }

    #[test]
    fn test_desktop_monitors() {
        assert!(match_common_pattern("list connected monitors").is_some());
        assert!(match_common_pattern("screen resolution").is_some());
    }

    // HowTo pattern tests (v0.0.947)
    #[test]
    fn test_howto_install_package() {
        assert!(match_common_pattern("how do I install a package").is_some());
        assert!(match_common_pattern("install package").is_some());
    }

    #[test]
    fn test_howto_update_system() {
        assert!(match_common_pattern("how to update system").is_some());
        assert!(match_common_pattern("upgrade system").is_some());
    }

    #[test]
    fn test_howto_enable_service() {
        assert!(match_common_pattern("how to enable a service").is_some());
        assert!(match_common_pattern("how to restart service").is_some());
    }

    #[test]
    fn test_howto_add_user() {
        assert!(match_common_pattern("how to add a user").is_some());
        assert!(match_common_pattern("give sudo access").is_some());
    }

    #[test]
    fn test_howto_file_permissions() {
        assert!(match_common_pattern("how to change permissions").is_some());
        assert!(match_common_pattern("make file executable").is_some());
    }

    #[test]
    fn test_howto_system_config() {
        assert!(match_common_pattern("how to change hostname").is_some());
        assert!(match_common_pattern("how to reboot").is_some());
    }

    // Network pattern tests (v0.0.948)
    #[test]
    fn test_network_connection() {
        assert!(match_common_pattern("am i connected").is_some());
        assert!(match_common_pattern("wifi status").is_some());
    }

    #[test]
    fn test_network_ip() {
        assert!(match_common_pattern("what is my ip").is_some());
        assert!(match_common_pattern("public ip").is_some());
    }

    #[test]
    fn test_network_dns() {
        assert!(match_common_pattern("dns servers").is_some());
        assert!(match_common_pattern("flush dns cache").is_some());
    }

    #[test]
    fn test_network_ports() {
        assert!(match_common_pattern("open ports").is_some());
        assert!(match_common_pattern("listening ports").is_some());
    }

    // Hardware pattern tests (v0.0.949)
    #[test]
    fn test_hardware_temperature() {
        assert!(match_common_pattern("cpu temperature").is_some());
        assert!(match_common_pattern("gpu temp").is_some());
    }

    #[test]
    fn test_hardware_battery() {
        assert!(match_common_pattern("battery status").is_some());
        assert!(match_common_pattern("battery level").is_some());
    }

    #[test]
    fn test_hardware_cpu() {
        assert!(match_common_pattern("cpu frequency").is_some());
        assert!(match_common_pattern("cpu usage").is_some());
    }

    #[test]
    fn test_hardware_devices() {
        assert!(match_common_pattern("usb devices").is_some());
        assert!(match_common_pattern("pci devices").is_some());
    }

    // Gaming pattern tests (v0.0.950)
    #[test]
    fn test_gaming_steam() {
        assert!(match_common_pattern("steam installation").is_some());
        assert!(match_common_pattern("steam games").is_some());
    }

    #[test]
    fn test_gaming_wine_proton() {
        assert!(match_common_pattern("wine version").is_some());
        assert!(match_common_pattern("proton version").is_some());
    }

    #[test]
    fn test_gaming_controllers() {
        assert!(match_common_pattern("controller detect").is_some());
        assert!(match_common_pattern("xbox controller").is_some());
    }

    #[test]
    fn test_gaming_graphics() {
        assert!(match_common_pattern("vulkan support").is_some());
        assert!(match_common_pattern("opengl version").is_some());
    }

    // Boot pattern tests (v0.0.951)
    #[test]
    fn test_boot_grub() {
        assert!(match_common_pattern("grub config").is_some());
        assert!(match_common_pattern("update grub").is_some());
    }

    #[test]
    fn test_boot_efi() {
        assert!(match_common_pattern("efi boot entry").is_some());
        assert!(match_common_pattern("boot order").is_some());
    }

    #[test]
    fn test_boot_kernel() {
        assert!(match_common_pattern("kernel version").is_some());
        assert!(match_common_pattern("kernel parameters").is_some());
    }

    #[test]
    fn test_boot_issues() {
        assert!(match_common_pattern("boot time").is_some());
        assert!(match_common_pattern("boot errors").is_some());
    }

    // Synonym expansion tests (v0.0.952)
    #[test]
    fn test_synonym_expansion() {
        // "ram" should match patterns with "memory"
        assert!(match_common_pattern("how much ram").is_some());
        // "processor" should match patterns with "cpu"
        assert!(match_common_pattern("processor temperature").is_some());
        // "graphics" should match patterns with "gpu"
        assert!(match_common_pattern("graphics temp").is_some());
        // "wireless" should match patterns with "wifi"
        assert!(match_common_pattern("wireless status").is_some());
    }

    #[test]
    fn test_expanded_synonyms() {
        // Audio synonyms
        assert!(match_common_pattern("sound status").is_some() ||
                match_common_pattern("audio status").is_some());
        // Display synonyms
        assert!(match_common_pattern("screen resolution").is_some() ||
                match_common_pattern("display resolution").is_some());
        // Process synonyms
        assert!(match_common_pattern("task manager").is_some() ||
                match_common_pattern("running processes").is_some());
    }

    #[test]
    fn test_expand_with_synonyms() {
        let expanded = expand_with_synonyms("check my ram usage");
        assert!(expanded.contains("memory"));

        let expanded2 = expand_with_synonyms("processor info");
        assert!(expanded2.contains("cpu"));
    }

    // Query normalization tests (v0.0.955)
    #[test]
    fn test_normalize_query() {
        // Removes punctuation
        let norm = normalize_query("what is my disk usage?");
        assert!(!norm.contains("?"));

        // Removes filler words
        let norm2 = normalize_query("please show me disk usage");
        assert!(!norm2.contains("please"));

        // Collapses spaces
        let norm3 = normalize_query("disk    usage");
        assert_eq!(norm3, "disk usage");
    }

    #[test]
    fn test_normalized_pattern_matching() {
        // "Please check my disk usage?" should match disk patterns
        assert!(match_common_pattern("Please check my disk usage?").is_some());
        // "Can you show me the cpu temperature?" should match
        assert!(match_common_pattern("Can you show me the cpu temperature?").is_some());
        // Filler-heavy queries should still match
        assert!(match_common_pattern("Help me, I need to check battery status!").is_some());
    }

    // Fuzzy matching tests (v0.0.956)
    #[test]
    fn test_edit_distance() {
        // Same strings
        assert_eq!(edit_distance("disk", "disk"), 0);
        // One character different
        assert_eq!(edit_distance("disk", "dsk"), 1);
        assert_eq!(edit_distance("memory", "memroy"), 2); // swap
        // Two characters different
        assert_eq!(edit_distance("kernel", "kernal"), 1);
    }

    #[test]
    fn test_fix_typos() {
        // Common typos should be fixed
        assert!(fix_typos("pacaman").contains("pacman"));
        assert!(fix_typos("kernal version").contains("kernel"));
        assert!(fix_typos("systemclt status").contains("systemctl"));
        assert!(fix_typos("memroy usage").contains("memory"));
    }

    #[test]
    fn test_fuzzy_correct_query() {
        // Should correct "diks" to "disk"
        let corrected = fuzzy_correct_query("diks usage");
        assert!(corrected.is_some());
        assert!(corrected.unwrap().contains("disk"));

        // Should correct "memry" to "memory"
        let corrected2 = fuzzy_correct_query("memry usage");
        assert!(corrected2.is_some());
        assert!(corrected2.unwrap().contains("memory"));

        // Should not correct correct words
        let corrected3 = fuzzy_correct_query("disk usage");
        assert!(corrected3.is_none());
    }

    #[test]
    fn test_fuzzy_pattern_matching() {
        // Typos in common terms should still match
        assert!(match_common_pattern("kernal version").is_some()); // kernel typo
        assert!(match_common_pattern("what is my diks usage").is_some()); // disk typo
        assert!(match_common_pattern("memry usage").is_some()); // memory typo
        assert!(match_common_pattern("packman database locked").is_some()); // pacman typo
    }

    #[test]
    fn test_typo_pattern_matching() {
        // Pre-defined typos should match
        assert!(match_common_pattern("baterry status").is_some()); // battery
        assert!(match_common_pattern("temperture check").is_some()); // temperature
        assert!(match_common_pattern("firwall status").is_some()); // firewall
        assert!(match_common_pattern("netwrok connection").is_some()); // network
    }

    // Container pattern tests (v0.0.957)
    #[test]
    fn test_container_docker() {
        assert!(match_common_pattern("docker containers").is_some());
        assert!(match_common_pattern("docker images").is_some());
        assert!(match_common_pattern("docker version").is_some());
    }

    #[test]
    fn test_container_podman() {
        assert!(match_common_pattern("podman containers").is_some());
        assert!(match_common_pattern("podman images").is_some());
        assert!(match_common_pattern("podman pods").is_some());
    }

    #[test]
    fn test_container_vms() {
        assert!(match_common_pattern("list vms").is_some());
        assert!(match_common_pattern("running vms").is_some());
        assert!(match_common_pattern("virtualization support").is_some());
    }

    // Log pattern tests (v0.0.958)
    #[test]
    fn test_logs_journalctl() {
        assert!(match_common_pattern("recent logs").is_some());
        assert!(match_common_pattern("boot logs").is_some());
        assert!(match_common_pattern("error logs").is_some());
        assert!(match_common_pattern("kernel logs").is_some());
    }

    #[test]
    fn test_logs_dmesg() {
        assert!(match_common_pattern("dmesg").is_some());
        assert!(match_common_pattern("dmesg errors").is_some());
    }

    #[test]
    fn test_logs_analysis() {
        assert!(match_common_pattern("crash logs").is_some());
        assert!(match_common_pattern("what happened").is_some());
        assert!(match_common_pattern("sudo logs").is_some());
    }

    // Audio pattern tests (v0.0.959)
    #[test]
    fn test_audio_general() {
        assert!(match_common_pattern("no sound").is_some());
        assert!(match_common_pattern("audio devices").is_some());
        assert!(match_common_pattern("volume level").is_some());
    }

    #[test]
    fn test_audio_pipewire() {
        assert!(match_common_pattern("pipewire status").is_some());
        assert!(match_common_pattern("pipewire version").is_some());
    }

    #[test]
    fn test_audio_alsa() {
        assert!(match_common_pattern("alsa devices").is_some());
        assert!(match_common_pattern("alsa mixer").is_some());
    }

    // Power pattern tests (v0.0.960)
    #[test]
    fn test_power_battery() {
        assert!(match_common_pattern("battery status").is_some());
        assert!(match_common_pattern("battery level").is_some());
        assert!(match_common_pattern("charging status").is_some());
    }

    #[test]
    fn test_power_suspend() {
        assert!(match_common_pattern("suspend mode").is_some());
        assert!(match_common_pattern("sleep modes").is_some());
    }

    #[test]
    fn test_power_laptop() {
        assert!(match_common_pattern("screen brightness").is_some());
        assert!(match_common_pattern("fan speed").is_some());
        assert!(match_common_pattern("cpu governor").is_some());
    }

    // Systemd pattern tests (v0.0.961)
    #[test]
    fn test_systemd_services() {
        assert!(match_common_pattern("failed services").is_some());
        assert!(match_common_pattern("running services").is_some());
        assert!(match_common_pattern("list services").is_some());
    }

    #[test]
    fn test_systemd_units() {
        assert!(match_common_pattern("list units").is_some());
        assert!(match_common_pattern("list timers").is_some());
        assert!(match_common_pattern("default target").is_some());
    }

    #[test]
    fn test_systemd_boot() {
        assert!(match_common_pattern("boot time").is_some());
        assert!(match_common_pattern("boot blame").is_some());
        assert!(match_common_pattern("slow boot").is_some());
    }

    // Filesystem pattern tests (v0.0.962)
    #[test]
    fn test_filesystem_mounts() {
        assert!(match_common_pattern("list mounts").is_some());
        assert!(match_common_pattern("fstab").is_some());
        assert!(match_common_pattern("disk uuid").is_some());
    }

    #[test]
    fn test_filesystem_lvm() {
        assert!(match_common_pattern("lvm status").is_some());
        assert!(match_common_pattern("logical volumes").is_some());
    }

    #[test]
    fn test_filesystem_btrfs() {
        assert!(match_common_pattern("btrfs status").is_some());
        assert!(match_common_pattern("btrfs subvolumes").is_some());
    }

    #[test]
    fn test_filesystem_general() {
        assert!(match_common_pattern("inode usage").is_some());
        assert!(match_common_pattern("large files").is_some());
        assert!(match_common_pattern("directory sizes").is_some());
    }

    // Process pattern tests (v0.0.963)
    #[test]
    fn test_process_list() {
        assert!(match_common_pattern("all processes").is_some());
        assert!(match_common_pattern("process tree").is_some());
        assert!(match_common_pattern("my processes").is_some());
    }

    #[test]
    fn test_process_resources() {
        assert!(match_common_pattern("cpu hogs").is_some());
        assert!(match_common_pattern("memory hogs").is_some());
        assert!(match_common_pattern("system load").is_some());
    }

    #[test]
    fn test_process_zombie() {
        assert!(match_common_pattern("zombie processes").is_some());
        assert!(match_common_pattern("stuck processes").is_some());
        assert!(match_common_pattern("background jobs").is_some());
    }

    // Cron pattern tests (v0.0.964)
    #[test]
    fn test_cron_crontab() {
        assert!(match_common_pattern("my crontab").is_some());
        assert!(match_common_pattern("crontab list").is_some());
        assert!(match_common_pattern("crontab syntax").is_some());
    }

    #[test]
    fn test_cron_system() {
        assert!(match_common_pattern("system cron").is_some());
        assert!(match_common_pattern("cron daily").is_some());
        assert!(match_common_pattern("cron logs").is_some());
    }

    #[test]
    fn test_cron_at() {
        assert!(match_common_pattern("atq").is_some());
        assert!(match_common_pattern("atd jobs").is_some());
        assert!(match_common_pattern("scheduled jobs").is_some());
    }

    #[test]
    fn test_users_list() {
        assert!(match_common_pattern("all users").is_some());
        assert!(match_common_pattern("list users").is_some());
        assert!(match_common_pattern("logged in users").is_some());
    }

    #[test]
    fn test_users_info() {
        assert!(match_common_pattern("current user").is_some());
        assert!(match_common_pattern("my groups").is_some());
        assert!(match_common_pattern("my shell").is_some());
    }

    #[test]
    fn test_users_login() {
        assert!(match_common_pattern("last logins").is_some());
        assert!(match_common_pattern("failed logins").is_some());
        assert!(match_common_pattern("login history").is_some());
    }

    #[test]
    fn test_time_current() {
        assert!(match_common_pattern("current time").is_some());
        assert!(match_common_pattern("system uptime").is_some());
        assert!(match_common_pattern("unix timestamp").is_some());
    }

    #[test]
    fn test_time_timezone() {
        assert!(match_common_pattern("current timezone").is_some());
        assert!(match_common_pattern("list timezones").is_some());
        assert!(match_common_pattern("time utc").is_some());
    }

    #[test]
    fn test_time_ntp() {
        assert!(match_common_pattern("ntp status").is_some());
        assert!(match_common_pattern("time sync").is_some());
        assert!(match_common_pattern("chrony status").is_some());
    }

    #[test]
    fn test_printing_printers() {
        assert!(match_common_pattern("list printers").is_some());
        assert!(match_common_pattern("default printer").is_some());
        assert!(match_common_pattern("printer status").is_some());
    }

    #[test]
    fn test_printing_jobs() {
        assert!(match_common_pattern("print queue").is_some());
        assert!(match_common_pattern("print jobs").is_some());
    }

    #[test]
    fn test_printing_cups() {
        assert!(match_common_pattern("cups status").is_some());
        assert!(match_common_pattern("cups logs").is_some());
        assert!(match_common_pattern("cups config").is_some());
    }

    #[test]
    fn test_backup_rsync() {
        assert!(match_common_pattern("rsync version").is_some());
        assert!(match_common_pattern("rsync syntax").is_some());
    }

    #[test]
    fn test_backup_borg_restic() {
        assert!(match_common_pattern("borg version").is_some());
        assert!(match_common_pattern("restic snapshots").is_some());
    }

    #[test]
    fn test_backup_tar() {
        assert!(match_common_pattern("tar syntax").is_some());
        assert!(match_common_pattern("tar extract").is_some());
    }

    #[test]
    fn test_locale() {
        assert!(match_common_pattern("current locale").is_some());
        assert!(match_common_pattern("available locales").is_some());
    }

    #[test]
    fn test_keyboard() {
        assert!(match_common_pattern("keyboard layout").is_some());
        assert!(match_common_pattern("console keymap").is_some());
    }

    #[test]
    fn test_fonts() {
        assert!(match_common_pattern("installed fonts").is_some());
        assert!(match_common_pattern("font families").is_some());
    }

    #[test]
    fn test_ssh_service() {
        assert!(match_common_pattern("sshd status").is_some());
        assert!(match_common_pattern("ssh version").is_some());
    }

    #[test]
    fn test_ssh_connections() {
        assert!(match_common_pattern("ssh connections").is_some());
        assert!(match_common_pattern("ssh agent").is_some());
    }

    #[test]
    fn test_ssh_config() {
        assert!(match_common_pattern("ssh config").is_some());
        assert!(match_common_pattern("sshd config").is_some());
    }

    #[test]
    fn test_memory_ram() {
        assert!(match_common_pattern("memory usage").is_some());
        assert!(match_common_pattern("free memory").is_some());
    }

    #[test]
    fn test_memory_swap() {
        assert!(match_common_pattern("swap usage").is_some());
        assert!(match_common_pattern("swappiness").is_some());
    }

    #[test]
    fn test_memory_oom() {
        assert!(match_common_pattern("oom killer").is_some());
        assert!(match_common_pattern("memory pressure").is_some());
    }

    #[test]
    fn test_bluetooth_status() {
        assert!(match_common_pattern("bluetooth status").is_some());
        assert!(match_common_pattern("bluetooth adapter").is_some());
    }

    #[test]
    fn test_bluetooth_devices() {
        assert!(match_common_pattern("paired devices").is_some());
        assert!(match_common_pattern("bluetooth devices").is_some());
    }

    #[test]
    fn test_bluetooth_audio() {
        assert!(match_common_pattern("bluetooth headphones").is_some());
        assert!(match_common_pattern("bluetooth audio").is_some());
    }

    #[test]
    fn test_virtualization_kvm() {
        assert!(match_common_pattern("kvm support").is_some());
        assert!(match_common_pattern("kvm enabled").is_some());
    }

    #[test]
    fn test_virtualization_libvirt() {
        assert!(match_common_pattern("libvirt status").is_some());
        assert!(match_common_pattern("list vms").is_some());
    }

    #[test]
    fn test_virtualization_qemu() {
        assert!(match_common_pattern("qemu version").is_some());
        assert!(match_common_pattern("virtual machines").is_some());
    }

    #[test]
    fn test_display_resolution() {
        assert!(match_common_pattern("current resolution").is_some());
        assert!(match_common_pattern("screen resolution").is_some());
    }

    #[test]
    fn test_display_monitors() {
        assert!(match_common_pattern("connected monitors").is_some());
        assert!(match_common_pattern("primary monitor").is_some());
    }

    #[test]
    fn test_display_scaling() {
        assert!(match_common_pattern("display dpi").is_some());
        assert!(match_common_pattern("refresh rate").is_some());
    }

    #[test]
    fn test_encryption_luks() {
        assert!(match_common_pattern("luks status").is_some());
        assert!(match_common_pattern("encrypted partitions").is_some());
    }

    #[test]
    fn test_encryption_gpg() {
        assert!(match_common_pattern("gpg keys").is_some());
        assert!(match_common_pattern("pacman keys").is_some());
    }

    #[test]
    fn test_encryption_disk() {
        assert!(match_common_pattern("disk encryption").is_some());
        assert!(match_common_pattern("crypttab").is_some());
    }

    #[test]
    fn test_nvidia_smi() {
        assert!(match_common_pattern("nvidia smi").is_some());
        assert!(match_common_pattern("gpu usage").is_some());
    }

    #[test]
    fn test_nvidia_driver() {
        assert!(match_common_pattern("nvidia driver version").is_some());
        assert!(match_common_pattern("cuda version").is_some());
    }

    #[test]
    fn test_nvidia_optimus() {
        assert!(match_common_pattern("prime status").is_some());
        assert!(match_common_pattern("hybrid graphics").is_some());
    }

    #[test]
    fn test_aur_helpers() {
        assert!(match_common_pattern("yay version").is_some());
        assert!(match_common_pattern("paru version").is_some());
        assert!(match_common_pattern("aur helper").is_some());
    }

    #[test]
    fn test_aur_packages() {
        assert!(match_common_pattern("aur packages").is_some());
        assert!(match_common_pattern("foreign packages").is_some());
    }

    #[test]
    fn test_aur_makepkg() {
        assert!(match_common_pattern("makepkg build").is_some());
        assert!(match_common_pattern("pkgbuild").is_some());
    }

    #[test]
    fn test_flatpak() {
        assert!(match_common_pattern("flatpak list").is_some());
        assert!(match_common_pattern("flatpak remotes").is_some());
    }

    #[test]
    fn test_snap() {
        assert!(match_common_pattern("snap list").is_some());
        assert!(match_common_pattern("snapd status").is_some());
    }

    #[test]
    fn test_appimage() {
        assert!(match_common_pattern("appimage list").is_some());
        assert!(match_common_pattern("appimages").is_some());
    }

    #[test]
    fn test_sysinfo_fetch() {
        assert!(match_common_pattern("neofetch").is_some());
        assert!(match_common_pattern("inxi").is_some());
    }

    #[test]
    fn test_sysinfo_hardware() {
        assert!(match_common_pattern("lshw").is_some());
        assert!(match_common_pattern("lspci").is_some());
    }

    #[test]
    fn test_sysinfo_summary() {
        assert!(match_common_pattern("system info").is_some());
        assert!(match_common_pattern("my specs").is_some());
    }

    #[test]
    fn test_wm_hyprland() {
        assert!(match_common_pattern("hyprland config").is_some());
        assert!(match_common_pattern("hyprland monitors").is_some());
        assert!(match_common_pattern("hyprctl").is_some());
    }

    #[test]
    fn test_wm_sway() {
        assert!(match_common_pattern("sway config").is_some());
        assert!(match_common_pattern("sway workspaces").is_some());
        assert!(match_common_pattern("swaymsg").is_some());
    }

    #[test]
    fn test_wm_i3() {
        assert!(match_common_pattern("i3 config").is_some());
        assert!(match_common_pattern("i3 workspaces").is_some());
        assert!(match_common_pattern("i3-msg").is_some());
    }

    #[test]
    fn test_wm_general() {
        assert!(match_common_pattern("which window manager").is_some());
        assert!(match_common_pattern("waybar config").is_some());
        assert!(match_common_pattern("polybar config").is_some());
    }

    #[test]
    fn test_wm_compositor() {
        assert!(match_common_pattern("picom config").is_some());
        assert!(match_common_pattern("screen tearing").is_some());
        assert!(match_common_pattern("compositor status").is_some());
    }

    #[test]
    fn test_kernel_info() {
        assert!(match_common_pattern("kernel version").is_some());
        assert!(match_common_pattern("installed kernels").is_some());
        assert!(match_common_pattern("running kernel").is_some());
    }

    #[test]
    fn test_kernel_modules() {
        assert!(match_common_pattern("loaded modules").is_some());
        assert!(match_common_pattern("lsmod").is_some());
        assert!(match_common_pattern("blacklisted modules").is_some());
    }

    #[test]
    fn test_kernel_params() {
        assert!(match_common_pattern("kernel parameters").is_some());
        assert!(match_common_pattern("sysctl").is_some());
        assert!(match_common_pattern("swappiness").is_some());
    }

    #[test]
    fn test_kernel_dkms() {
        assert!(match_common_pattern("dkms status").is_some());
        assert!(match_common_pattern("dkms modules").is_some());
    }

    #[test]
    fn test_kernel_debug() {
        assert!(match_common_pattern("kernel errors").is_some());
        assert!(match_common_pattern("kernel panic").is_some());
        assert!(match_common_pattern("tainted kernel").is_some());
    }

    #[test]
    fn test_zfs_pool() {
        assert!(match_common_pattern("zpool status").is_some());
        assert!(match_common_pattern("zpool list").is_some());
        assert!(match_common_pattern("zpool iostat").is_some());
    }

    #[test]
    fn test_zfs_dataset() {
        assert!(match_common_pattern("zfs list").is_some());
        assert!(match_common_pattern("zfs compression").is_some());
        assert!(match_common_pattern("zfs mount").is_some());
    }

    #[test]
    fn test_zfs_snapshot() {
        assert!(match_common_pattern("zfs snapshots").is_some());
        assert!(match_common_pattern("list snapshots").is_some());
    }

    #[test]
    fn test_zfs_health() {
        assert!(match_common_pattern("zfs scrub").is_some());
        assert!(match_common_pattern("zfs errors").is_some());
        assert!(match_common_pattern("zfs arc").is_some());
    }

    #[test]
    fn test_smart_status() {
        assert!(match_common_pattern("smart status").is_some());
        assert!(match_common_pattern("disk health").is_some());
        assert!(match_common_pattern("ssd health").is_some());
    }

    #[test]
    fn test_smart_attributes() {
        assert!(match_common_pattern("smart attributes").is_some());
        assert!(match_common_pattern("reallocated sectors").is_some());
    }

    #[test]
    fn test_smart_nvme() {
        assert!(match_common_pattern("nvme health").is_some());
        assert!(match_common_pattern("nvme list").is_some());
    }
}
