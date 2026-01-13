//! Build script for anna-shared
//! Verifies VERSION file matches Cargo.toml and embeds build info.

use std::env;
use std::fs;
use std::path::Path;

fn main() {
    // Tell cargo to rerun if VERSION file changes
    println!("cargo:rerun-if-changed=../../VERSION");

    // Read VERSION file
    let version_file = Path::new("../../VERSION");
    let version_from_file = fs::read_to_string(version_file)
        .expect("Failed to read VERSION file")
        .trim()
        .to_string();

    // Get version from Cargo.toml (via env var set by cargo)
    let cargo_version = env!("CARGO_PKG_VERSION");

    // Verify they match
    if version_from_file != cargo_version {
        panic!(
            "VERSION file mismatch!\n\
             VERSION file: {}\n\
             Cargo.toml:   {}\n\n\
             Update VERSION file or Cargo.toml to match.",
            version_from_file, cargo_version
        );
    }

    // Embed build timestamp
    let build_time = chrono::Utc::now().to_rfc3339();
    println!("cargo:rustc-env=ANNA_BUILD_TIME={}", build_time);

    // Try to get git info
    let git_sha = std::process::Command::new("git")
        .args(["rev-parse", "--short", "HEAD"])
        .output()
        .ok()
        .and_then(|o| String::from_utf8(o.stdout).ok())
        .map(|s| s.trim().to_string())
        .unwrap_or_else(|| "unknown".to_string());
    println!("cargo:rustc-env=ANNA_GIT_SHA={}", git_sha);

    let git_dirty = std::process::Command::new("git")
        .args(["status", "--porcelain"])
        .output()
        .ok()
        .map(|o| !o.stdout.is_empty())
        .unwrap_or(false);
    println!("cargo:rustc-env=ANNA_GIT_DIRTY={}", git_dirty);
}
