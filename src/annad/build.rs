// Build script to embed VERSION file content
use std::env;
use std::fs;
use std::path::PathBuf;

fn main() {
    // Get workspace root (two levels up from src/annad)
    let manifest_dir = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap());
    let workspace_root = manifest_dir.parent().unwrap().parent().unwrap();
    let version_file = workspace_root.join("VERSION");

    // Read VERSION file
    let version = fs::read_to_string(&version_file)
        .unwrap_or_else(|e| panic!("Failed to read VERSION file at {:?}: {}", version_file, e))
        .trim()
        .to_string();

    // Validate version format (must start with 'v')
    if !version.starts_with('v') {
        panic!("VERSION file must start with 'v' (e.g., v1.0.0-rc.1), got: {}", version);
    }

    // Remove 'v' prefix for CARGO_PKG_VERSION compatibility
    let version_no_v = version.strip_prefix('v').unwrap();

    // Set ANNA_VERSION env var (with 'v' prefix for display)
    println!("cargo:rustc-env=ANNA_VERSION={}", version);

    // Override CARGO_PKG_VERSION to match (without 'v' prefix)
    println!("cargo:rustc-env=CARGO_PKG_VERSION={}", version_no_v);

    // Rebuild if VERSION file changes
    println!("cargo:rerun-if-changed={}", version_file.display());

    // Rebuild if build.rs itself changes
    println!("cargo:rerun-if-changed=build.rs");
}
