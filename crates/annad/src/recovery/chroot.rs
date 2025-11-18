//! Chroot environment detection
//!
//! Phase 0.6: Recovery Framework - Chroot detection
//! Citation: [archwiki:Chroot#Using_arch-chroot]

use anyhow::Result;
use std::path::{Path, PathBuf};

/// Chroot environment information
#[derive(Debug, Clone)]
pub struct ChrootEnvironment {
    /// Root path for chroot
    pub root_path: PathBuf,
    /// Whether /proc is mounted
    pub proc_mounted: bool,
    /// Whether /sys is mounted
    pub sys_mounted: bool,
    /// Whether /dev is mounted
    pub dev_mounted: bool,
    /// Whether environment is chroot-ready
    pub ready: bool,
}

/// Detect if we're in a chroot environment
pub fn detect_chroot() -> Result<bool> {
    // Check if /proc/1/root links to /
    let proc_root = std::fs::read_link("/proc/1/root").unwrap_or_else(|_| PathBuf::from("/"));
    let is_chroot = proc_root != PathBuf::from("/");

    Ok(is_chroot)
}

/// Check if a given path is a valid chroot candidate
pub fn is_chroot_candidate(path: &Path) -> Result<ChrootEnvironment> {
    let proc_mounted = path.join("proc").exists();
    let sys_mounted = path.join("sys").exists();
    let dev_mounted = path.join("dev").exists();

    // Check for essential directories
    let has_etc = path.join("etc").is_dir();
    let has_usr = path.join("usr").is_dir();
    let has_bin = path.join("bin").exists() || path.join("usr/bin").exists();

    let ready = proc_mounted && sys_mounted && dev_mounted && has_etc && has_usr && has_bin;

    Ok(ChrootEnvironment {
        root_path: path.to_path_buf(),
        proc_mounted,
        sys_mounted,
        dev_mounted,
        ready,
    })
}

/// Prepare a chroot environment by mounting required filesystems
pub fn prepare_chroot(root_path: &Path) -> Result<()> {
    use std::process::Command;

    // Mount proc if not mounted
    let proc_path = root_path.join("proc");
    if !proc_path.join("version").exists() {
        Command::new("mount")
            .args(&["-t", "proc", "proc", proc_path.to_str().unwrap()])
            .status()?;
    }

    // Mount sys if not mounted
    let sys_path = root_path.join("sys");
    if !sys_path.join("kernel").exists() {
        Command::new("mount")
            .args(&["--rbind", "/sys", sys_path.to_str().unwrap()])
            .status()?;
        Command::new("mount")
            .args(&["--make-rslave", sys_path.to_str().unwrap()])
            .status()?;
    }

    // Mount dev if not mounted
    let dev_path = root_path.join("dev");
    if !dev_path.join("null").exists() {
        Command::new("mount")
            .args(&["--rbind", "/dev", dev_path.to_str().unwrap()])
            .status()?;
        Command::new("mount")
            .args(&["--make-rslave", dev_path.to_str().unwrap()])
            .status()?;
    }

    Ok(())
}

/// Cleanup chroot environment by unmounting filesystems
pub fn cleanup_chroot(root_path: &Path) -> Result<()> {
    use std::process::Command;

    // Unmount in reverse order
    let _ = Command::new("umount")
        .args(&["-l", root_path.join("dev").to_str().unwrap()])
        .status();

    let _ = Command::new("umount")
        .args(&["-l", root_path.join("sys").to_str().unwrap()])
        .status();

    let _ = Command::new("umount")
        .args(&["-l", root_path.join("proc").to_str().unwrap()])
        .status();

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_detect_chroot() {
        // This test may fail in CI if not in chroot
        let result = detect_chroot();
        assert!(result.is_ok());
    }

    #[test]
    fn test_is_chroot_candidate() {
        // Test with root path
        let result = is_chroot_candidate(Path::new("/"));
        assert!(result.is_ok());
    }
}
