//! Staff personality traits and dialogue (v0.0.243).
//!
//! Each staff member has unique personality quirks that show up in
//! internal communications, making the IT department feel more alive.

/// Personality trait for a staff member
#[derive(Debug, Clone, Copy)]
pub struct Personality {
    /// Opening phrases when starting work
    pub greetings: &'static [&'static str],
    /// Phrases when finding an answer
    pub success: &'static [&'static str],
    /// Phrases when uncertain
    pub uncertain: &'static [&'static str],
    /// Personality quirk/signature
    pub quirk: &'static str,
}

/// Get personality for a staff member by their ID
pub fn personality_for(person_id: &str) -> Personality {
    match person_id {
        // Network team
        "network_jr" => Personality {
            // Michael - enthusiastic newbie who loves TCP/IP trivia
            greetings: &["Let me check the packets!", "Network time!", "I'll trace this."],
            success: &["Found it in the routing table!", "The packets don't lie.", "Connection established!"],
            uncertain: &["The network is... complex.", "Might need Ana on this one."],
            quirk: "Loves TCP/IP trivia",
        },
        "network_sr" => Personality {
            // Ana - calm, experienced, speaks in networking metaphors
            greetings: &["Routing this request...", "Let's see where this goes.", "Tracing the path."],
            success: &["Clear as a well-configured subnet.", "No dropped packets here.", "Connection secure."],
            uncertain: &["This needs deeper inspection.", "The topology is unusual."],
            quirk: "Speaks in networking metaphors",
        },

        // Desktop team
        "desktop_jr" => Personality {
            // Sofia - vim enthusiast, always mentions keyboard shortcuts
            greetings: &["Just :wq'd my last task!", "Let me hjkl through this.", "Config time!"],
            success: &["That's a clean config.", "Just like editing a buffer.", ":w success!"],
            uncertain: &["Might need to check the docs.", "Erik would know the DE side."],
            quirk: "Everything reminds her of vim",
        },
        "desktop_sr" => Personality {
            // Erik - DE expert, slightly grumpy about Wayland
            greetings: &["Another day, another config.", "Let's see what's broken.", "Checking the display stack."],
            success: &["Fixed. X11 strikes again.", "Compositor sorted.", "Clean render."],
            uncertain: &["Wayland probably.", "This might need a restart."],
            quirk: "Grumpy about Wayland",
        },

        // Hardware team
        "hardware_jr" => Personality {
            // Nora - excited about hardware, uses sound effects
            greetings: &["*checks lspci* Ooh!", "Hardware scan initiated!", "Let me probe this."],
            success: &["Beep boop! Found it!", "Driver detected!", "Hardware happy!"],
            uncertain: &["This might be a driver thing.", "Jon handles firmware."],
            quirk: "Makes beep boop sounds",
        },
        "hardware_sr" => Personality {
            // Jon - firmware wizard, very methodical
            greetings: &["Running diagnostics...", "Let's check the metal.", "Inspecting the hardware layer."],
            success: &["Firmware check complete.", "Hardware verified.", "All pins connected."],
            uncertain: &["Might need a BIOS update.", "Could be a chipset issue."],
            quirk: "Calls hardware 'the metal'",
        },

        // Storage team
        "storage_jr" => Personality {
            // Lars - obsessed with disk space, always worrying
            greetings: &["How much free space...?", "Let me check the mounts!", "Storage audit time!"],
            success: &["Plenty of space!", "Clean filesystem.", "Blocks accounted for."],
            uncertain: &["We might need to clean up.", "Ines handles the RAID."],
            quirk: "Always worried about disk space",
        },
        "storage_sr" => Personality {
            // Ines - calm storage architect, ZFS enthusiast
            greetings: &["Checking pool status...", "Let's see the storage layout.", "Analyzing the arrays."],
            success: &["Data integrity verified.", "Pool healthy.", "Redundancy confirmed."],
            uncertain: &["Might need a scrub.", "This array is interesting."],
            quirk: "Will recommend ZFS for everything",
        },

        // Performance team
        "perf_jr" => Personality {
            // Kari - loves htop, always has it running
            greetings: &["*glances at htop*", "Let me check the load!", "Monitoring engaged!"],
            success: &["Numbers look good!", "Performance nominal.", "No bottlenecks!"],
            uncertain: &["Something's eating cycles.", "Mateo should profile this."],
            quirk: "Always has htop running",
        },
        "perf_sr" => Personality {
            // Mateo - optimization obsessed, speaks in percentages
            greetings: &["Profiling initiated.", "Let's optimize.", "Analyzing the metrics."],
            success: &["That's 100% efficiency.", "Optimal performance achieved.", "Zero wasted cycles."],
            uncertain: &["Need more data points.", "This requires deeper analysis."],
            quirk: "Speaks in percentages",
        },

        // Security team
        "security_jr" => Personality {
            // Priya - paranoid (in a good way), checks everything twice
            greetings: &["Let me verify that...", "Security check!", "Auditing permissions."],
            success: &["Verified and secure.", "Permissions correct.", "No vulnerabilities detected."],
            uncertain: &["We should double-check.", "Oskar needs to review this."],
            quirk: "Checks everything twice",
        },
        "security_sr" => Personality {
            // Oskar - night owl security expert, cryptography nerd
            greetings: &["Analyzing the vectors...", "Let me check the hashes.", "Reviewing security posture."],
            success: &["Cryptographically sound.", "Attack surface minimized.", "System hardened."],
            uncertain: &["This needs encryption review.", "Potential vulnerability."],
            quirk: "Night owl, loves cryptography",
        },

        // Services team
        "services_jr" => Personality {
            // Hugo - systemd fan, loves unit files
            greetings: &["Checking systemd status!", "Let me inspect the units.", "Service scan running."],
            success: &["All units active!", "Service healthy.", "systemctl approves."],
            uncertain: &["Unit might need restart.", "Mina handles containers."],
            quirk: "Enthusiastic about systemd",
        },
        "services_sr" => Personality {
            // Mina - container expert, slightly cynical
            greetings: &["Let's see what's containerized...", "Checking the orchestration.", "Pod status incoming."],
            success: &["Container healthy.", "Orchestration optimal.", "All replicas running."],
            uncertain: &["Container networking, probably.", "This needs a restart."],
            quirk: "Slightly cynical about microservices",
        },

        // Logs team
        "logs_jr" => Personality {
            // Daniel - night shift, drinks lots of coffee
            greetings: &["*sips coffee* Let me check...", "Diving into journalctl.", "Log analysis time!"],
            success: &["Found it in the logs!", "Clear log trail.", "Evidence documented."],
            uncertain: &["Logs are... interesting.", "Need more caffeine for this."],
            quirk: "Drinks way too much coffee",
        },
        "logs_sr" => Personality {
            // Lea - log aggregation expert, very organized
            greetings: &["Querying the indices...", "Let me correlate the logs.", "Pattern matching initiated."],
            success: &["Pattern identified.", "Logs correlated.", "Root cause found."],
            uncertain: &["Need to aggregate more data.", "This requires correlation."],
            quirk: "Extremely organized log dashboards",
        },

        // General team
        "general_jr" => Personality {
            // Tomas - helpful, always takes notes
            greetings: &["Let me document this!", "Taking notes...", "I'll look into it!"],
            success: &["Documented and resolved!", "Added to the knowledge base.", "Case closed!"],
            uncertain: &["Let me find the right team.", "I'll escalate this."],
            quirk: "Takes detailed notes on everything",
        },
        "general_sr" => Personality {
            // Sara - coordination expert, knows everyone
            greetings: &["Let me coordinate.", "I know who can help.", "Routing to the experts."],
            success: &["Team effort pays off!", "Coordination successful.", "All resolved."],
            uncertain: &["This needs cross-team work.", "Let me pull in specialists."],
            quirk: "Knows everyone's schedule by heart",
        },

        // Default for unknown
        _ => Personality {
            greetings: &["Looking into this.", "Let me check.", "On it."],
            success: &["Found it.", "Done.", "Resolved."],
            uncertain: &["Need to investigate.", "Checking further."],
            quirk: "Reliable team member",
        },
    }
}

/// Get a greeting phrase for a staff member (uses deterministic selection)
pub fn get_greeting(person_id: &str, seed: u64) -> &'static str {
    let personality = personality_for(person_id);
    let idx = (seed as usize) % personality.greetings.len();
    personality.greetings[idx]
}

/// Get a success phrase for a staff member (uses deterministic selection)
pub fn get_success(person_id: &str, seed: u64) -> &'static str {
    let personality = personality_for(person_id);
    let idx = (seed as usize) % personality.success.len();
    personality.success[idx]
}

/// Get an uncertain phrase for a staff member (uses deterministic selection)
pub fn get_uncertain(person_id: &str, seed: u64) -> &'static str {
    let personality = personality_for(person_id);
    let idx = (seed as usize) % personality.uncertain.len();
    personality.uncertain[idx]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_personality_for_known_staff() {
        let p = personality_for("network_jr");
        assert!(!p.greetings.is_empty());
        assert_eq!(p.quirk, "Loves TCP/IP trivia");
    }

    #[test]
    fn test_personality_for_unknown() {
        let p = personality_for("unknown_person");
        assert_eq!(p.quirk, "Reliable team member");
    }

    #[test]
    fn test_deterministic_greeting() {
        // Same seed should give same phrase
        let g1 = get_greeting("desktop_jr", 42);
        let g2 = get_greeting("desktop_jr", 42);
        assert_eq!(g1, g2);

        // Different seed may give different phrase
        let g3 = get_greeting("desktop_jr", 1);
        // (might be same or different, but should not panic)
        assert!(!g3.is_empty());
    }
}
