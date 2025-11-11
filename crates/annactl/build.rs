// Build script for annactl - embeds version at compile time

fn main() {
    // Get version from environment (set by GitHub Actions) or Cargo.toml
    let version =
        std::env::var("ANNA_VERSION").unwrap_or_else(|_| env!("CARGO_PKG_VERSION").to_string());

    // Embed as environment variable for runtime access
    println!("cargo:rustc-env=ANNA_VERSION={}", version);

    // Also rerun if Cargo.toml changes
    println!("cargo:rerun-if-changed=Cargo.toml");
    println!("cargo:rerun-if-env-changed=ANNA_VERSION");
}
