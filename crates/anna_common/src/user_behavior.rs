use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::Path;
use std::process::Command;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserBehaviorPatterns {
    pub command_patterns: Option<CommandPatterns>,
    pub resource_patterns: Option<ResourcePatterns>,
    pub disk_patterns: Option<DiskPatterns>,
    pub network_patterns: Option<NetworkPatterns>,
    pub application_patterns: Option<ApplicationPatterns>,
    pub gaming_patterns: Option<GamingPatterns>,
    pub development_patterns: Option<DevelopmentPatterns>,
    pub security_patterns: Option<SecurityPatterns>,
    pub user_profile: UserProfile,
    pub recommendations: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommandPatterns {
    pub top_commands: Vec<(String, u32)>,
    pub total_commands: u32,
    pub unique_commands: u32,
    pub shell_type: Option<String>,
    pub most_active_hours: Vec<u8>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourcePatterns {
    pub typical_cpu_usage: f64,
    pub typical_memory_usage: f64,
    pub peak_usage_times: Vec<String>,
    pub resource_intensive_apps: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiskPatterns {
    pub file_type_distribution: HashMap<String, u64>,
    pub largest_directories: Vec<(String, u64)>,
    pub total_files: u64,
    pub storage_growth_rate: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkPatterns {
    pub connection_count: u32,
    pub bandwidth_usage: Option<BandwidthUsage>,
    pub frequently_accessed_hosts: Vec<String>,
    pub network_activity_level: NetworkActivityLevel,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BandwidthUsage {
    pub bytes_sent: u64,
    pub bytes_received: u64,
    pub packets_sent: u64,
    pub packets_received: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum NetworkActivityLevel {
    Low,
    Medium,
    High,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApplicationPatterns {
    pub frequently_used: Vec<(String, u32)>,
    pub recently_used: Vec<String>,
    pub running_applications: Vec<String>,
    pub application_categories: HashMap<String, u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GamingPatterns {
    pub steam_installed: bool,
    pub steam_games_count: Option<u32>,
    pub gaming_processes: Vec<String>,
    pub gpu_gaming_hours: Option<f64>,
    pub is_gaming_system: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DevelopmentPatterns {
    pub git_repositories: u32,
    pub programming_languages: Vec<String>,
    pub development_tools: Vec<String>,
    pub build_tool_usage: Vec<String>,
    pub is_development_system: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityPatterns {
    pub sudo_usage_count: u32,
    pub ssh_connection_count: u32,
    pub failed_login_attempts: u32,
    pub security_awareness_level: SecurityLevel,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SecurityLevel {
    Low,
    Medium,
    High,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserProfile {
    pub primary_use_case: UseCase,
    pub secondary_use_cases: Vec<UseCase>,
    pub experience_level: ExperienceLevel,
    pub activity_level: ActivityLevel,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum UseCase {
    Gaming,
    Development,
    ServerAdmin,
    Workstation,
    MediaProduction,
    GeneralUse,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ExperienceLevel {
    Beginner,
    Intermediate,
    Advanced,
    Expert,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ActivityLevel {
    Light,
    Moderate,
    Heavy,
}

impl UserBehaviorPatterns {
    pub fn detect() -> Self {
        let command_patterns = detect_command_patterns();
        let resource_patterns = detect_resource_patterns();
        let disk_patterns = detect_disk_patterns();
        let network_patterns = detect_network_patterns();
        let application_patterns = detect_application_patterns();
        let gaming_patterns = detect_gaming_patterns();
        let development_patterns = detect_development_patterns();
        let security_patterns = detect_security_patterns();

        let user_profile = infer_user_profile(
            &command_patterns,
            &application_patterns,
            &gaming_patterns,
            &development_patterns,
            &security_patterns,
        );

        let recommendations =
            generate_recommendations(&user_profile, &command_patterns, &security_patterns);

        UserBehaviorPatterns {
            command_patterns,
            resource_patterns,
            disk_patterns,
            network_patterns,
            application_patterns,
            gaming_patterns,
            development_patterns,
            security_patterns,
            user_profile,
            recommendations,
        }
    }
}

fn detect_command_patterns() -> Option<CommandPatterns> {
    // Try to read bash_history or zsh_history
    let home = std::env::var("HOME").ok()?;
    let history_paths = vec![
        format!("{}/.bash_history", home),
        format!("{}/.zsh_history", home),
        format!("{}/.local/share/fish/fish_history", home),
    ];

    let mut shell_type = None;
    let mut commands: HashMap<String, u32> = HashMap::new();

    for path in history_paths {
        if Path::new(&path).exists() {
            if path.contains("bash") {
                shell_type = Some("bash".to_string());
            } else if path.contains("zsh") {
                shell_type = Some("zsh".to_string());
            } else if path.contains("fish") {
                shell_type = Some("fish".to_string());
            }

            if let Ok(history) = std::fs::read_to_string(&path) {
                for line in history.lines().take(10000) {
                    // Extract first command (before pipes, args, etc.)
                    let cmd = line.split_whitespace().next().unwrap_or("");
                    if !cmd.is_empty() {
                        *commands.entry(cmd.to_string()).or_insert(0) += 1;
                    }
                }
                break;
            }
        }
    }

    if commands.is_empty() {
        return None;
    }

    let total_commands = commands.values().sum();
    let unique_commands = commands.len() as u32;

    let mut top_commands: Vec<(String, u32)> = commands.into_iter().collect();
    top_commands.sort_by(|a, b| b.1.cmp(&a.1));
    top_commands.truncate(20);

    Some(CommandPatterns {
        top_commands,
        total_commands,
        unique_commands,
        shell_type,
        most_active_hours: Vec::new(), // Would need timestamp analysis
    })
}

fn detect_resource_patterns() -> Option<ResourcePatterns> {
    // This would ideally use historical data
    // For now, provide current snapshot
    None
}

fn detect_disk_patterns() -> Option<DiskPatterns> {
    let home = std::env::var("HOME").ok()?;
    let mut file_type_distribution: HashMap<String, u64> = HashMap::new();
    let mut total_files = 0u64;

    // Analyze home directory file types
    if let Ok(output) = Command::new("find")
        .args([&home, "-type", "f", "-name", "*.*"])
        .output()
    {
        if let Ok(files) = String::from_utf8(output.stdout) {
            for file in files.lines().take(10000) {
                total_files += 1;
                if let Some(ext) = Path::new(file).extension() {
                    if let Some(ext_str) = ext.to_str() {
                        *file_type_distribution
                            .entry(ext_str.to_string())
                            .or_insert(0) += 1;
                    }
                }
            }
        }
    }

    // Get largest directories
    let largest_directories = get_largest_directories(&home);

    Some(DiskPatterns {
        file_type_distribution,
        largest_directories,
        total_files,
        storage_growth_rate: None, // Would need historical tracking
    })
}

fn get_largest_directories(base_path: &str) -> Vec<(String, u64)> {
    let mut dirs = Vec::new();

    if let Ok(output) = Command::new("du")
        .args(["-d", "1", "-h", base_path])
        .output()
    {
        if let Ok(du_output) = String::from_utf8(output.stdout) {
            for line in du_output.lines().take(20) {
                let parts: Vec<&str> = line.split_whitespace().collect();
                if parts.len() >= 2 {
                    dirs.push((parts[1].to_string(), 0)); // Size parsing would need improvement
                }
            }
        }
    }

    dirs
}

fn detect_network_patterns() -> Option<NetworkPatterns> {
    let mut connection_count = 0u32;
    let mut frequently_accessed_hosts = Vec::new();

    // Get active connections
    if let Ok(output) = Command::new("ss").args(["-tunaH"]).output() {
        if let Ok(ss_output) = String::from_utf8(output.stdout) {
            connection_count = ss_output.lines().count() as u32;
        }
    }

    // Get bandwidth stats from /proc/net/dev
    let bandwidth_usage = get_bandwidth_usage();

    let network_activity_level = if connection_count > 100 {
        NetworkActivityLevel::High
    } else if connection_count > 20 {
        NetworkActivityLevel::Medium
    } else {
        NetworkActivityLevel::Low
    };

    Some(NetworkPatterns {
        connection_count,
        bandwidth_usage,
        frequently_accessed_hosts,
        network_activity_level,
    })
}

fn get_bandwidth_usage() -> Option<BandwidthUsage> {
    if let Ok(dev_stats) = std::fs::read_to_string("/proc/net/dev") {
        let mut bytes_received = 0u64;
        let mut bytes_sent = 0u64;
        let mut packets_received = 0u64;
        let mut packets_sent = 0u64;

        for line in dev_stats.lines().skip(2) {
            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.len() >= 10 {
                bytes_received += parts[1].parse::<u64>().unwrap_or(0);
                packets_received += parts[2].parse::<u64>().unwrap_or(0);
                bytes_sent += parts[9].parse::<u64>().unwrap_or(0);
                packets_sent += parts[10].parse::<u64>().unwrap_or(0);
            }
        }

        return Some(BandwidthUsage {
            bytes_sent,
            bytes_received,
            packets_sent,
            packets_received,
        });
    }

    None
}

fn detect_application_patterns() -> Option<ApplicationPatterns> {
    let mut running_applications = Vec::new();
    let mut application_categories: HashMap<String, u32> = HashMap::new();

    // Get running processes
    if let Ok(output) = Command::new("ps")
        .args(["-eo", "comm", "--no-headers"])
        .output()
    {
        if let Ok(ps_output) = String::from_utf8(output.stdout) {
            for line in ps_output.lines() {
                let app = line.trim().to_string();
                if !app.is_empty() {
                    running_applications.push(app.clone());

                    // Categorize
                    let category = categorize_application(&app);
                    *application_categories.entry(category).or_insert(0) += 1;
                }
            }
        }
    }

    running_applications.sort();
    running_applications.dedup();

    Some(ApplicationPatterns {
        frequently_used: Vec::new(), // Would need usage tracking
        recently_used: Vec::new(),
        running_applications,
        application_categories,
    })
}

fn categorize_application(app: &str) -> String {
    let app_lower = app.to_lowercase();

    if app_lower.contains("firefox")
        || app_lower.contains("chrome")
        || app_lower.contains("browser")
    {
        "Browser".to_string()
    } else if app_lower.contains("code") || app_lower.contains("vim") || app_lower.contains("emacs")
    {
        "Editor".to_string()
    } else if app_lower.contains("docker") || app_lower.contains("podman") {
        "Container".to_string()
    } else if app_lower.contains("ssh") || app_lower.contains("sshd") {
        "Network".to_string()
    } else {
        "Other".to_string()
    }
}

fn detect_gaming_patterns() -> Option<GamingPatterns> {
    let steam_installed = is_command_available("steam");
    let mut steam_games_count = None;
    let mut gaming_processes = Vec::new();

    if steam_installed {
        // Check Steam library
        let home = std::env::var("HOME").ok()?;
        let steam_apps = format!("{}/.local/share/Steam/steamapps", home);

        if let Ok(entries) = std::fs::read_dir(&steam_apps) {
            steam_games_count = Some(entries.count() as u32);
        }
    }

    // Check for gaming processes
    if let Ok(output) = Command::new("ps").args(["-eo", "comm"]).output() {
        if let Ok(ps_output) = String::from_utf8(output.stdout) {
            for line in ps_output.lines() {
                let proc = line.trim();
                if is_gaming_process(proc) {
                    gaming_processes.push(proc.to_string());
                }
            }
        }
    }

    let is_gaming_system = steam_installed || !gaming_processes.is_empty();

    Some(GamingPatterns {
        steam_installed,
        steam_games_count,
        gaming_processes,
        gpu_gaming_hours: None,
        is_gaming_system,
    })
}

fn is_gaming_process(proc: &str) -> bool {
    let gaming_keywords = vec!["steam", "wine", "proton", "gamemode", "lutris"];
    gaming_keywords
        .iter()
        .any(|kw| proc.to_lowercase().contains(kw))
}

fn detect_development_patterns() -> Option<DevelopmentPatterns> {
    let home = std::env::var("HOME").ok()?;
    let mut git_repositories = 0u32;
    let mut programming_languages = Vec::new();
    let mut development_tools = Vec::new();
    let mut build_tool_usage = Vec::new();

    // Count git repos
    if let Ok(output) = Command::new("find")
        .args([&home, "-name", ".git", "-type", "d"])
        .output()
    {
        if let Ok(git_output) = String::from_utf8(output.stdout) {
            git_repositories = git_output.lines().count() as u32;
        }
    }

    // Detect programming languages by file extensions
    let lang_extensions = vec![
        ("rs", "Rust"),
        ("py", "Python"),
        ("js", "JavaScript"),
        ("ts", "TypeScript"),
        ("go", "Go"),
        ("java", "Java"),
        ("c", "C"),
        ("cpp", "C++"),
    ];

    for (ext, lang) in lang_extensions {
        if let Ok(output) = Command::new("find")
            .args([&home, "-name", &format!("*.{}", ext), "-type", "f"])
            .output()
        {
            if let Ok(find_output) = String::from_utf8(output.stdout) {
                if find_output.lines().count() > 5 {
                    programming_languages.push(lang.to_string());
                }
            }
        }
    }

    // Check for development tools
    let dev_tools = vec!["cargo", "npm", "pip", "maven", "gradle", "make"];
    for tool in dev_tools {
        if is_command_available(tool) {
            development_tools.push(tool.to_string());
        }
    }

    // Check for build tool usage in history
    if let Ok(home) = std::env::var("HOME") {
        if let Ok(history) = std::fs::read_to_string(format!("{}/.bash_history", home)) {
            for build_tool in &["cargo", "npm", "make", "mvn", "gradle"] {
                if history.contains(build_tool) {
                    build_tool_usage.push(build_tool.to_string());
                }
            }
        }
    }

    let is_development_system = git_repositories > 0 || !programming_languages.is_empty();

    Some(DevelopmentPatterns {
        git_repositories,
        programming_languages,
        development_tools,
        build_tool_usage,
        is_development_system,
    })
}

fn detect_security_patterns() -> Option<SecurityPatterns> {
    let mut sudo_usage_count = 0u32;
    let mut ssh_connection_count = 0u32;
    let mut failed_login_attempts = 0u32;

    // Count sudo usage from bash history
    if let Ok(home) = std::env::var("HOME") {
        if let Ok(history) = std::fs::read_to_string(format!("{}/.bash_history", home)) {
            sudo_usage_count = history.matches("sudo ").count() as u32;
        }
    }

    // Count SSH connections
    if let Ok(output) = Command::new("ss").args(["-tnaH"]).output() {
        if let Ok(ss_output) = String::from_utf8(output.stdout) {
            ssh_connection_count = ss_output
                .lines()
                .filter(|line| line.contains(":22"))
                .count() as u32;
        }
    }

    // Check failed login attempts from journal
    if let Ok(output) = Command::new("journalctl")
        .args(["_COMM=sshd", "-g", "Failed password", "-n", "1000"])
        .output()
    {
        if let Ok(journal_output) = String::from_utf8(output.stdout) {
            failed_login_attempts = journal_output.lines().count() as u32;
        }
    }

    let security_awareness_level = if sudo_usage_count > 100 && failed_login_attempts == 0 {
        SecurityLevel::High
    } else if sudo_usage_count > 20 {
        SecurityLevel::Medium
    } else {
        SecurityLevel::Low
    };

    Some(SecurityPatterns {
        sudo_usage_count,
        ssh_connection_count,
        failed_login_attempts,
        security_awareness_level,
    })
}

fn infer_user_profile(
    command_patterns: &Option<CommandPatterns>,
    _application_patterns: &Option<ApplicationPatterns>,
    gaming_patterns: &Option<GamingPatterns>,
    development_patterns: &Option<DevelopmentPatterns>,
    security_patterns: &Option<SecurityPatterns>,
) -> UserProfile {
    let mut use_cases = Vec::new();

    // Determine primary use case
    if let Some(dev) = development_patterns {
        if dev.is_development_system {
            use_cases.push((UseCase::Development, dev.git_repositories as u32));
        }
    }

    if let Some(gaming) = gaming_patterns {
        if gaming.is_gaming_system {
            use_cases.push((UseCase::Gaming, gaming.steam_games_count.unwrap_or(10)));
        }
    }

    if let Some(sec) = security_patterns {
        if sec.sudo_usage_count > 50 || sec.ssh_connection_count > 10 {
            use_cases.push((UseCase::ServerAdmin, sec.sudo_usage_count));
        }
    }

    if use_cases.is_empty() {
        use_cases.push((UseCase::GeneralUse, 1));
    }

    use_cases.sort_by(|a, b| b.1.cmp(&a.1));

    let primary_use_case = use_cases[0].0.clone();
    let secondary_use_cases: Vec<UseCase> =
        use_cases.iter().skip(1).map(|(uc, _)| uc.clone()).collect();

    let experience_level = if let Some(cmd) = command_patterns {
        if cmd.unique_commands > 200 {
            ExperienceLevel::Expert
        } else if cmd.unique_commands > 100 {
            ExperienceLevel::Advanced
        } else if cmd.unique_commands > 50 {
            ExperienceLevel::Intermediate
        } else {
            ExperienceLevel::Beginner
        }
    } else {
        ExperienceLevel::Intermediate
    };

    let activity_level = if let Some(cmd) = command_patterns {
        if cmd.total_commands > 10000 {
            ActivityLevel::Heavy
        } else if cmd.total_commands > 1000 {
            ActivityLevel::Moderate
        } else {
            ActivityLevel::Light
        }
    } else {
        ActivityLevel::Moderate
    };

    UserProfile {
        primary_use_case,
        secondary_use_cases,
        experience_level,
        activity_level,
    }
}

fn generate_recommendations(
    profile: &UserProfile,
    _command_patterns: &Option<CommandPatterns>,
    security_patterns: &Option<SecurityPatterns>,
) -> Vec<String> {
    let mut recommendations = Vec::new();

    match profile.primary_use_case {
        UseCase::Development => {
            recommendations.push(
                "Development-focused system detected - consider git hooks for automation"
                    .to_string(),
            );
        }
        UseCase::Gaming => {
            recommendations.push(
                "Gaming system detected - ensure gamemode and CPU governor are optimized"
                    .to_string(),
            );
        }
        UseCase::ServerAdmin => {
            recommendations.push(
                "Server admin activity detected - consider fail2ban for SSH protection".to_string(),
            );
        }
        _ => {}
    }

    if let Some(sec) = security_patterns {
        if sec.failed_login_attempts > 10 {
            recommendations
                .push("Multiple failed login attempts detected - enable fail2ban".to_string());
        }
    }

    recommendations
}

fn is_command_available(cmd: &str) -> bool {
    Command::new("which")
        .arg(cmd)
        .output()
        .map(|output| output.status.success())
        .unwrap_or(false)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_behavior_detection() {
        let patterns = UserBehaviorPatterns::detect();
        assert!(matches!(
            patterns.user_profile.primary_use_case,
            UseCase::Gaming
                | UseCase::Development
                | UseCase::ServerAdmin
                | UseCase::Workstation
                | UseCase::MediaProduction
                | UseCase::GeneralUse
        ));
    }
}
