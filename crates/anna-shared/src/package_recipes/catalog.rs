//! Package catalog with built-in recipes (v0.0.230).

use super::types::{PackageCategory, PackageManager, PackageRecipe};

/// Get common package recipes
pub fn common_packages() -> Vec<PackageRecipe> {
    vec![
        // Editors
        PackageRecipe::new(
            "vim",
            "Vim",
            PackageCategory::Editor,
            "Vi Improved text editor",
        )
        .with_package(PackageManager::Pacman, "vim")
        .with_package(PackageManager::Apt, "vim")
        .with_package(PackageManager::Dnf, "vim-enhanced"),
        PackageRecipe::new(
            "neovim",
            "Neovim",
            PackageCategory::Editor,
            "Modern Vim fork",
        )
        .with_package(PackageManager::Pacman, "neovim")
        .with_package(PackageManager::Apt, "neovim")
        .with_package(PackageManager::Dnf, "neovim"),
        PackageRecipe::new(
            "nano",
            "nano",
            PackageCategory::Editor,
            "Simple text editor",
        )
        .with_package(PackageManager::Pacman, "nano")
        .with_package(PackageManager::Apt, "nano")
        .with_package(PackageManager::Dnf, "nano"),
        PackageRecipe::new(
            "helix",
            "Helix",
            PackageCategory::Editor,
            "Post-modern text editor",
        )
        .with_package(PackageManager::Pacman, "helix")
        .with_package(PackageManager::Apt, "helix")
        .with_package(PackageManager::Dnf, "helix"),
        // Development
        PackageRecipe::new(
            "git",
            "Git",
            PackageCategory::Development,
            "Version control system",
        )
        .with_package(PackageManager::Pacman, "git")
        .with_package(PackageManager::Apt, "git")
        .with_package(PackageManager::Dnf, "git"),
        PackageRecipe::new(
            "make",
            "Make",
            PackageCategory::Development,
            "Build automation tool",
        )
        .with_package(PackageManager::Pacman, "make")
        .with_package(PackageManager::Apt, "make")
        .with_package(PackageManager::Dnf, "make"),
        PackageRecipe::new(
            "gcc",
            "GCC",
            PackageCategory::Development,
            "GNU Compiler Collection",
        )
        .with_package(PackageManager::Pacman, "gcc")
        .with_package(PackageManager::Apt, "gcc")
        .with_package(PackageManager::Dnf, "gcc"),
        // System
        PackageRecipe::new(
            "htop",
            "htop",
            PackageCategory::System,
            "Interactive process viewer",
        )
        .with_package(PackageManager::Pacman, "htop")
        .with_package(PackageManager::Apt, "htop")
        .with_package(PackageManager::Dnf, "htop"),
        PackageRecipe::new(
            "btop",
            "btop",
            PackageCategory::System,
            "Modern resource monitor",
        )
        .with_package(PackageManager::Pacman, "btop")
        .with_package(PackageManager::Apt, "btop")
        .with_package(PackageManager::Dnf, "btop"),
        // Network
        PackageRecipe::new(
            "curl",
            "curl",
            PackageCategory::Network,
            "Data transfer tool",
        )
        .with_package(PackageManager::Pacman, "curl")
        .with_package(PackageManager::Apt, "curl")
        .with_package(PackageManager::Dnf, "curl"),
        PackageRecipe::new("wget", "wget", PackageCategory::Network, "File downloader")
            .with_package(PackageManager::Pacman, "wget")
            .with_package(PackageManager::Apt, "wget")
            .with_package(PackageManager::Dnf, "wget"),
        // Compression
        PackageRecipe::new(
            "unzip",
            "unzip",
            PackageCategory::Compression,
            "ZIP archive extractor",
        )
        .with_package(PackageManager::Pacman, "unzip")
        .with_package(PackageManager::Apt, "unzip")
        .with_package(PackageManager::Dnf, "unzip"),
        PackageRecipe::new(
            "p7zip",
            "7zip",
            PackageCategory::Compression,
            "7-Zip archive manager",
        )
        .with_package(PackageManager::Pacman, "p7zip")
        .with_package(PackageManager::Apt, "p7zip-full")
        .with_package(PackageManager::Dnf, "p7zip"),
    ]
}
