//! Build script for anna-shared.
//! v0.0.73: Sets build-time environment variables for version info.
//!
//! This script sets:
//! - ANNA_GIT_SHA: Short git commit hash
//! - ANNA_BUILD_DATE: UTC ISO timestamp

use std::process::Command;

fn main() {
    // Get git SHA
    let git_sha = Command::new("git")
        .args(["rev-parse", "--short", "HEAD"])
        .output()
        .ok()
        .and_then(|output| {
            if output.status.success() {
                String::from_utf8(output.stdout).ok()
            } else {
                None
            }
        })
        .map(|s| s.trim().to_string())
        .unwrap_or_else(|| "unknown".to_string());

    // Get build date in UTC
    let build_date = chrono::Utc::now().format("%Y-%m-%d").to_string();

    // Set environment variables for compilation
    println!("cargo:rustc-env=ANNA_GIT_SHA={}", git_sha);
    println!("cargo:rustc-env=ANNA_BUILD_DATE={}", build_date);

    // Rebuild if git HEAD changes
    println!("cargo:rerun-if-changed=.git/HEAD");
    println!("cargo:rerun-if-changed=.git/refs/heads/");
}
