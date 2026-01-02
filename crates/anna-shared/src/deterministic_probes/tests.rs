//! Tests for deterministic probe rules.

#[cfg(test)]
mod tests {
    use crate::deterministic_probes::registry::DeterministicProbeRegistry;

    #[test]
    fn test_cpu_most_maps_to_top_cpu() {
        let registry = DeterministicProbeRegistry::new();
        let probes = registry.get_probes("which service is using the most CPU?");
        assert!(probes.is_some());
        let probes = probes.unwrap();
        assert!(
            probes.contains(&"top_cpu"),
            "Should contain top_cpu, got {:?}",
            probes
        );
        assert!(!probes.contains(&"cpu_info"), "Should NOT contain cpu_info");
    }

    #[test]
    fn test_what_cpu_maps_to_cpu_info() {
        let registry = DeterministicProbeRegistry::new();
        let probes = registry.get_probes("what CPU do I have?");
        assert!(probes.is_some());
        let probes = probes.unwrap();
        assert!(probes.contains(&"cpu_info"));
    }

    #[test]
    fn test_swap_maps_to_swap_files_not_package() {
        let registry = DeterministicProbeRegistry::new();
        let probes = registry.get_probes("do I have swap?");
        assert!(probes.is_some());
        let probes = probes.unwrap();
        assert!(probes.contains(&"swap_files"), "Should contain swap_files");
        assert!(!probes.iter().any(|p| p.contains("pacman")));
    }

    #[test]
    fn test_vim_setup_maps_to_config_probes() {
        let registry = DeterministicProbeRegistry::new();
        let probes = registry.get_probes("what is my vim setup?");
        assert!(probes.is_some());
        let probes = probes.unwrap();
        assert!(probes.contains(&"vimrc_content") || probes.contains(&"nvim_config"));
    }

    #[test]
    fn test_bluetooth_maps_to_bluetooth_probes() {
        let registry = DeterministicProbeRegistry::new();
        let probes = registry.get_probes("is bluetooth enabled and working?");
        assert!(probes.is_some());
        let probes = probes.unwrap();
        assert!(probes.contains(&"bluetooth_service"));
    }

    #[test]
    fn test_boot_slow_maps_to_boot_probes() {
        let registry = DeterministicProbeRegistry::new();
        let probes = registry.get_probes("why is my boot slow?");
        assert!(probes.is_some());
        let probes = probes.unwrap();
        assert!(probes.contains(&"boot_time"));
        assert!(probes.contains(&"boot_blame"));
    }

    #[test]
    fn test_free_ram_maps_to_memory_info() {
        let registry = DeterministicProbeRegistry::new();
        let probes = registry.get_probes("how much free ram do I have?");
        assert!(probes.is_some());
        let probes = probes.unwrap();
        assert!(probes.contains(&"memory_info"));
    }

    #[test]
    fn test_wallpaper_maps_to_desktop_probes() {
        let registry = DeterministicProbeRegistry::new();
        let probes = registry.get_probes("where are my wallpapers?");
        assert!(probes.is_some());
        let probes = probes.unwrap();
        assert!(probes.contains(&"desktop_wallpaper"));
    }

    #[test]
    fn test_concept_not_package() {
        let registry = DeterministicProbeRegistry::new();

        // Concepts that should NOT be package queries
        assert!(registry.is_concept_not_package("do I have swap?"));
        assert!(registry.is_concept_not_package("is bluetooth working?"));
        assert!(registry.is_concept_not_package("how is my audio?"));

        // Actual package queries
        assert!(!registry.is_concept_not_package("install firefox"));
        assert!(!registry.is_concept_not_package("pacman -S vim"));
    }

    #[test]
    fn test_package_install_not_blocked() {
        let registry = DeterministicProbeRegistry::new();

        // "install vim" should NOT match vim.setup (has negative keyword "install")
        let probes = registry.get_probes("install vim");
        // Either no match (goes to package flow) or different intent
        if let Some(probes) = probes {
            assert!(!probes.contains(&"vimrc_content"));
        }
    }
}
