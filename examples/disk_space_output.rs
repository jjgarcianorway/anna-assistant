//! Example: What disk space output SHOULD look like
//!
//! This demonstrates the difference between:
//! - OLD: "disk-space: Issue detected"
//! - NEW: Actual analysis with recommendations
//!
//! Run with: cargo run --example disk_space_output

use anna_common::display::*;

fn main() {
    let use_color = should_use_color();

    println!("\n==== OLD OUTPUT (Useless) ====\n");
    println!("Health: ❌ (2 ok, 0 warn, 1 fail)");
    println!("  ❌ disk-space: Issue detected");
    println!();
    println!("Report: /var/lib/anna/reports/daily-20251113-113727.json");
    println!("Next: annactl repair (to fix issues)");

    println!("\n\n==== NEW OUTPUT (Actually Useful) ====\n");

    // Create a critical section showing the problem
    let mut section = Section::new(
        "Disk Almost Full (3% free)",
        StatusLevel::Critical,
        use_color,
    );
    section.add_blank();
    section.add_line("Your root partition has only 24GB free out of 802GB.");
    section.add_line("This needs immediate attention to prevent system issues.");

    print!("{}", section.render());

    // Show analysis
    println!("\n📊 Disk Usage Analysis:\n");
    let items = vec![
        ("🐳 Containers", "450GB", "~/.local/share/containers"),
        ("🐳 Docker", "200GB", "/var/lib/docker"),
        ("🏗️  Builds", "80GB", "~/builds"),
        ("📦 Packages", "40GB", "/var/cache/pacman"),
        ("📝 Logs", "8GB", "/var/log"),
    ];

    for (label, size, path) in items {
        println!("  {:<20} {:>8}  {}", label, size, path);
    }

    // Build recommendations
    let mut rec = Recommendation::new(use_color);

    rec.add_step(RecommendationStep {
        number: 1,
        title: "Clean container images (safest, saves ~300GB)".to_string(),
        command: Some("podman system prune -a".to_string()),
        explanation: "Removes unused images and stopped containers. This is the safest option and will free up the most space.".to_string(),
        warning: Some("This removes ALL unused container data".to_string()),
        wiki_link: Some(WikiLink {
            title: "Podman".to_string(),
            url: "https://wiki.archlinux.org/title/Podman#Pruning".to_string(),
            section: Some("Pruning unused data".to_string()),
        }),
        estimated_impact: Some("Frees ~300GB".to_string()),
    });

    rec.add_step(RecommendationStep {
        number: 2,
        title: "Clean Docker data (saves ~150GB)".to_string(),
        command: Some("docker system prune -a --volumes".to_string()),
        explanation: "Removes all unused Docker data including volumes. Only do this if you're sure you don't need old containers.".to_string(),
        warning: Some("This removes ALL unused data AND volumes - cannot be undone!".to_string()),
        wiki_link: Some(WikiLink {
            title: "Docker".to_string(),
            url: "https://wiki.archlinux.org/title/Docker#Pruning".to_string(),
            section: Some("Removing unused data".to_string()),
        }),
        estimated_impact: Some("Frees ~150GB".to_string()),
    });

    rec.add_step(RecommendationStep {
        number: 3,
        title: "Review build artifacts".to_string(),
        command: Some("ncdu ~/builds".to_string()),
        explanation: "Use ncdu (NCurses Disk Usage) to interactively explore and delete old build files. You can navigate with arrow keys and delete with 'd'.".to_string(),
        warning: None,
        wiki_link: Some(WikiLink {
            title: "List of applications/Utilities".to_string(),
            url: "https://wiki.archlinux.org/title/List_of_applications/Utilities#Disk_usage_display".to_string(),
            section: Some("Disk usage display".to_string()),
        }),
        estimated_impact: Some("Varies - you decide what to keep".to_string()),
    });

    rec.add_step(RecommendationStep {
        number: 4,
        title: "Auto-clean package cache (saves ~20GB)".to_string(),
        command: Some("sudo paccache -rk1".to_string()),
        explanation: "Keeps only the latest version of each installed package. This is safe and recommended for regular maintenance.".to_string(),
        warning: None,
        wiki_link: Some(WikiLink {
            title: "Pacman".to_string(),
            url: "https://wiki.archlinux.org/title/Pacman#Cleaning_the_package_cache".to_string(),
            section: Some("Cleaning the package cache".to_string()),
        }),
        estimated_impact: Some("Frees ~20GB".to_string()),
    });

    println!("\n{}", rec.render());

    // Final advice
    println!("💡 Recommendation: Start with step #1 (podman prune). It's the safest");
    println!("   and will free up the most space. Then run 'annactl daily' again");
    println!("   to verify the disk space is back to healthy levels.\n");
}
