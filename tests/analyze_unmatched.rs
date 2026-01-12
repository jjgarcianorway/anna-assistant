// Quick script to analyze unmatched questions
// Run with: cargo test -p annad test_show_unmatched -- --nocapture

#[cfg(test)]
mod test {
    use annad::patterns::match_common_pattern;

    const UNMATCHED_QUESTIONS: &[&str] = &[
        // Audio gaps
        "audio latency", "sample rate", "audio routing", "speaker test",
        "headphone detection", "midi devices", "audio mixing", "jack audio status",
        "sound server info", "audio codecs", "equalizer settings", "audio profiles",
        "default audio device", "audio troubleshoot",

        // Backup gaps
        "borg backups", "tar archives", "backup schedule", "incremental backup",
        "restore backup howto", "backup verification",

        // Boot gaps
        "initramfs info", "boot order", "grub theme", "boot menu timeout",
        "show initrd contents", "check boot loader", "plymouth status",
        "silent boot setup", "boot splash config", "grub password setup",

        // Recovery gaps
        "how to enter single user mode", "boot into rescue mode", "recover from failed update",
        "chroot into broken system", "reinstall bootloader", "fix fstab mistake",
        "recover deleted files", "fix broken initramfs",

        // Performance gaps
        "reduce boot time", "optimize for gaming", "improve battery life",
        "reduce swap usage", "tune for ssd", "profile application performance",
        "find memory leaks",

        // Process gaps
        "process threads", "process io", "process open files", "process environment",
        "process limits", "process cpu time", "process memory map", "process signals",
        "process user",
    ];

    #[test]
    fn test_verify_gaps() {
        for q in UNMATCHED_QUESTIONS {
            let result = match_common_pattern(q);
            if result.is_some() {
                println!("NOW MATCHED: {}", q);
            } else {
                println!("STILL UNMATCHED: {}", q);
            }
        }
    }
}
