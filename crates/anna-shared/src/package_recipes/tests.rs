//! Package recipe tests (v0.0.230).

#[cfg(test)]
mod tests {
    use crate::package_recipes::{common_packages, find_recipe, PackageManager};

    #[test]
    fn test_package_manager_display() {
        assert_eq!(PackageManager::Pacman.display_name(), "pacman");
        assert_eq!(PackageManager::Apt.display_name(), "apt");
    }

    #[test]
    fn test_find_recipe() {
        assert!(find_recipe("vim").is_some());
        assert!(find_recipe("git").is_some());
        assert!(find_recipe("nonexistent").is_none());
    }

    #[test]
    fn test_package_for_manager() {
        let vim = find_recipe("vim").unwrap();
        assert_eq!(vim.package_for(&PackageManager::Pacman), Some("vim"));
        assert_eq!(vim.package_for(&PackageManager::Dnf), Some("vim-enhanced"));
    }

    #[test]
    fn test_install_command() {
        let vim = find_recipe("vim").unwrap();
        let cmd = vim.install_command(&PackageManager::Pacman);
        assert!(cmd.unwrap().contains("pacman -S"));
    }

    #[test]
    fn test_common_packages_count() {
        let packages = common_packages();
        assert!(packages.len() >= 10);
    }
}
