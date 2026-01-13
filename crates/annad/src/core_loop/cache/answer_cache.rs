//! Answer caching for repeated questions.

use std::collections::HashMap;
use std::time::Instant;
use tracing::{debug, info};

use super::config_cache::get_perf_config;
use super::types::{CachedAnswer, ANSWER_CACHE, MAX_ANSWER_CACHE_SIZE, MIN_CACHE_CONFIDENCE};

/// Normalize question for cache key (lowercase, trim, canonicalize).
pub fn normalize_question(question: &str) -> String {
    const STOP_WORDS: &[&str] = &[
        "what", "how", "can", "do", "does", "is", "are", "the", "a", "an", "my", "i",
        "to", "in", "on", "for", "with", "and", "or", "of", "that", "this", "it",
        "be", "been", "being", "have", "has", "had", "will", "would", "could", "should",
        "please", "help", "me", "tell", "show", "get", "find", "check", "see",
        "about", "much", "many", "some", "any", "using", "use", "currently",
    ];

    fn canonicalize(word: &str) -> &str {
        match word {
            "storage" | "space" | "drive" | "drives" | "hdd" | "ssd" | "nvme" | "partition" | "partitions" | "filesystem" | "fs" => "disk",
            "mounted" | "mount" | "mounts" | "mounting" => "mount",
            "free" | "available" | "remaining" | "left" => "free",
            "used" | "usage" | "using" | "consumed" => "usage",
            "ram" | "mem" | "swap" | "cache" | "buffer" | "buffers" => "memory",
            "cpu" | "processor" | "processors" | "core" | "cores" | "thread" | "threads" => "cpu",
            "load" | "loads" | "utilization" => "load",
            "net" | "wifi" | "wlan" | "ethernet" | "eth" | "internet" | "lan" | "wan" | "interface" | "interfaces" => "network",
            "ip" | "ipv4" | "ipv6" | "address" | "addr" => "ip",
            "port" | "ports" | "socket" | "sockets" => "port",
            "connection" | "connections" | "conn" | "conns" => "connection",
            "bandwidth" | "throughput" | "speed" => "bandwidth",
            "pkg" | "package" | "packages" | "pacman" | "yay" | "paru" | "aur" => "package",
            "installed" | "install" | "installing" => "install",
            "remove" | "removing" | "uninstall" | "uninstalling" | "delete" | "deleting" => "remove",
            "svc" | "service" | "services" | "daemon" | "daemons" | "systemd" | "unit" | "units" => "service",
            "enabled" | "enable" | "enabling" => "enable",
            "disabled" | "disable" | "disabling" => "disable",
            "running" | "active" | "started" | "start" | "starting" => "running",
            "stopped" | "inactive" | "dead" | "stop" | "stopping" => "stopped",
            "restart" | "restarting" | "restarted" | "reload" | "reloading" => "restart",
            "proc" | "process" | "processes" | "pid" | "pids" | "task" | "tasks" => "process",
            "kill" | "killing" | "killed" | "terminate" | "terminating" => "kill",
            "failing" | "failed" | "broken" | "error" | "errors" | "issue" | "issues" | "problem" | "problems" => "failed",
            "crash" | "crashed" | "crashing" | "hang" | "hanging" | "hung" | "freeze" | "frozen" | "stuck" => "crash",
            "slow" | "sluggish" | "lag" | "lagging" | "laggy" | "latency" => "slow",
            "version" | "ver" | "release" => "version",
            "kernel" | "linux" | "uname" => "kernel",
            "update" | "updates" | "upgrade" | "upgrades" | "upgrading" | "updating" => "update",
            "reboot" | "rebooting" | "rebooted" | "poweroff" | "shutdown" => "reboot",
            "boot" | "booting" | "booted" | "startup" | "grub" => "boot",
            "gpu" | "graphics" | "video" | "nvidia" | "amd" | "radeon" | "intel" => "gpu",
            "audio" | "sound" | "speaker" | "speakers" | "mic" | "microphone" | "headphone" | "headphones" => "audio",
            "display" | "monitor" | "monitors" | "screen" | "screens" | "resolution" => "display",
            "bluetooth" | "bt" => "bluetooth",
            "usb" | "device" | "devices" | "peripheral" | "peripherals" => "device",
            "fan" | "fans" | "cooling" | "temperature" | "temp" | "temps" | "thermal" => "thermal",
            "battery" | "power" | "charging" | "acpi" => "power",
            "file" | "files" | "folder" | "folders" | "directory" | "directories" | "dir" | "dirs" => "file",
            "permission" | "permissions" | "perm" | "perms" | "chmod" | "chown" => "permission",
            "owner" | "ownership" | "group" | "groups" => "owner",
            "log" | "logs" | "journal" | "journalctl" | "dmesg" | "syslog" => "log",
            "user" | "users" | "account" | "accounts" | "sudo" | "root" => "user",
            "password" | "passwd" | "pwd" | "credentials" => "password",
            "desktop" | "de" | "gnome" | "kde" | "plasma" | "xfce" | "i3" | "sway" => "desktop",
            "wayland" | "x11" | "xorg" | "xserver" => "display_server",
            "window" | "windows" | "wm" | "compositor" => "window",
            _ => word,
        }
    }

    let expanded = question.to_lowercase()
        .replace("what's", "what is").replace("how's", "how is")
        .replace("where's", "where is").replace("who's", "who is")
        .replace("it's", "it is").replace("that's", "that is")
        .replace("there's", "there is").replace("here's", "here is")
        .replace("i'm", "i am").replace("i've", "i have")
        .replace("i'll", "i will").replace("i'd", "i would")
        .replace("you're", "you are").replace("you've", "you have")
        .replace("you'll", "you will").replace("don't", "do not")
        .replace("doesn't", "does not").replace("didn't", "did not")
        .replace("won't", "will not").replace("wouldn't", "would not")
        .replace("can't", "cannot").replace("couldn't", "could not")
        .replace("shouldn't", "should not").replace("isn't", "is not")
        .replace("aren't", "are not").replace("wasn't", "was not")
        .replace("weren't", "were not").replace("haven't", "have not")
        .replace("hasn't", "has not").replace("hadn't", "had not");

    let normalized: String = expanded.chars()
        .filter(|c| c.is_alphanumeric() || c.is_whitespace())
        .collect();

    let words: Vec<&str> = normalized.split_whitespace()
        .filter(|w| !STOP_WORDS.contains(w))
        .map(canonicalize)
        .collect();

    let mut sorted_words = words.clone();
    sorted_words.sort();
    sorted_words.join(" ")
}

/// Simple edit distance for fuzzy matching (Levenshtein).
pub fn edit_distance(a: &str, b: &str) -> usize {
    let a_chars: Vec<char> = a.chars().collect();
    let b_chars: Vec<char> = b.chars().collect();
    let a_len = a_chars.len();
    let b_len = b_chars.len();

    if a == b { return 0; }
    if a_len == 0 { return b_len; }
    if b_len == 0 { return a_len; }

    let len_diff = if a_len > b_len { a_len - b_len } else { b_len - a_len };
    if len_diff > 3 { return len_diff; }

    let mut prev_row: Vec<usize> = (0..=b_len).collect();
    let mut curr_row: Vec<usize> = vec![0; b_len + 1];

    for (i, a_char) in a_chars.iter().enumerate() {
        curr_row[0] = i + 1;
        for (j, b_char) in b_chars.iter().enumerate() {
            let cost = if a_char == b_char { 0 } else { 1 };
            curr_row[j + 1] = (prev_row[j + 1] + 1)
                .min(curr_row[j] + 1)
                .min(prev_row[j] + cost);
        }
        std::mem::swap(&mut prev_row, &mut curr_row);
    }
    prev_row[b_len]
}

/// Get cached answer for a question.
pub fn get_cached_answer(question: &str) -> Option<(String, f32)> {
    let perf = get_perf_config();
    let ttl = perf.answer_cache_ttl_secs;
    if ttl == 0 { return None; }

    if let Ok(guard) = ANSWER_CACHE.read() {
        if let Some(ref cache) = *guard {
            let key = normalize_question(question);

            if let Some(cached) = cache.get(&key) {
                if cached.cached_at.elapsed().as_secs() < ttl {
                    info!("Answer cache HIT for: {}", &question[..question.len().min(50)]);
                    return Some((cached.answer.clone(), cached.confidence));
                }
            }

            if key.len() >= 8 {
                for (cached_key, cached) in cache.iter() {
                    if cached.cached_at.elapsed().as_secs() >= ttl { continue; }
                    let distance = edit_distance(&key, cached_key);
                    if distance > 0 && distance <= 2 {
                        info!("Answer cache FUZZY HIT (dist={}) for: {}", distance, &question[..question.len().min(50)]);
                        return Some((cached.answer.clone(), cached.confidence * 0.9));
                    }
                }
            }
        }
    }
    None
}

/// Cache an answer for a question.
pub fn cache_answer(question: &str, answer: &str, confidence: f32) {
    let perf = get_perf_config();
    if perf.answer_cache_ttl_secs == 0 || confidence < MIN_CACHE_CONFIDENCE { return; }
    if answer.len() < 20 { return; }

    if let Ok(mut guard) = ANSWER_CACHE.write() {
        let cache = guard.get_or_insert_with(HashMap::new);
        let key = normalize_question(question);

        cache.insert(key, CachedAnswer {
            answer: answer.to_string(),
            cached_at: Instant::now(),
            confidence,
        });

        if cache.len() > MAX_ANSWER_CACHE_SIZE {
            let ttl = perf.answer_cache_ttl_secs;
            cache.retain(|_, v| v.cached_at.elapsed().as_secs() < ttl);

            if cache.len() > MAX_ANSWER_CACHE_SIZE {
                let mut entries: Vec<_> = cache.iter().collect();
                entries.sort_by(|a, b| b.1.cached_at.cmp(&a.1.cached_at));
                let keys_to_remove: Vec<String> = entries.iter()
                    .skip(MAX_ANSWER_CACHE_SIZE / 2)
                    .map(|(k, _)| (*k).clone())
                    .collect();
                for key in keys_to_remove { cache.remove(&key); }
            }
        }

        debug!("Cached answer for: {} (confidence: {:.2})", &question[..question.len().min(50)], confidence);
    }
}

/// Clear the answer cache.
pub fn clear_answer_cache() {
    if let Ok(mut guard) = ANSWER_CACHE.write() {
        *guard = Some(HashMap::new());
        info!("Answer cache cleared");
    }
}
