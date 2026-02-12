//! Telegram bot setup wizard for annactl
//! Guides users through creating a Telegram bot and configuring Anna

use std::io::{self, Write};
use std::process::Command;

/// ANSI color codes
const CYAN: &str = "\x1b[38;2;120;200;255m";
const GREEN: &str = "\x1b[38;2;120;255;120m";
const YELLOW: &str = "\x1b[38;2;255;210;120m";
const RED: &str = "\x1b[38;2;255;100;100m";
const BOLD: &str = "\x1b[1m";
const RESET: &str = "\x1b[0m";

/// Run the interactive Telegram setup wizard
pub fn run_telegram_setup() {
    println!();
    println!("{}{}Telegram Bot Setup{}", BOLD, CYAN, RESET);
    println!("{}", "─".repeat(60));
    println!();
    println!("Anna can be controlled remotely via Telegram!");
    println!("You'll receive:");
    println!("  • Morning briefings with system health charts");
    println!("  • Real-time alerts for critical issues");
    println!("  • Full annactl access from your phone");
    println!();
    println!("{}", "─".repeat(60));
    println!();

    // Step 1: Create bot with BotFather
    println!("{}Step 1: Create a Telegram Bot{}", YELLOW, RESET);
    println!();
    println!("1. Open Telegram and search for {}@BotFather{}", BOLD, RESET);
    println!("2. Send: {}/newbot{}", BOLD, RESET);
    println!("3. Choose a name (e.g., {}'My Anna Bot'{} or {}'Home Server Assistant'{})",
             BOLD, RESET, BOLD, RESET);
    println!("4. Choose a username (must end in 'bot', e.g., {}my_anna_bot{})", BOLD, RESET);
    println!("5. BotFather will reply with your bot token");
    println!();
    println!("{}Example token:{} 123456789:ABCdefGHIjklMNOpqrsTUVwxyz", CYAN, RESET);
    println!();

    // Get bot token
    let token = loop {
        print!("Paste your bot token: ");
        io::stdout().flush().unwrap();

        let mut input = String::new();
        io::stdin().read_line(&mut input).unwrap();
        let token = input.trim().to_string();

        if token.is_empty() {
            println!("{}✗{} No token provided. Try again or press Ctrl+C to cancel.", RED, RESET);
            continue;
        }

        if !token.contains(':') || token.len() < 20 {
            println!("{}✗{} Token doesn't look valid (should be like: 123456789:ABC...)", RED, RESET);
            print!("Continue anyway? [y/N] ");
            io::stdout().flush().unwrap();
            let mut confirm = String::new();
            io::stdin().read_line(&mut confirm).unwrap();
            if !confirm.trim().eq_ignore_ascii_case("y") {
                continue;
            }
        }

        break token;
    };

    println!("{}✓{} Token received", GREEN, RESET);
    println!();

    // Step 2: Get user ID
    println!("{}Step 2: Get Your Telegram User ID{}", YELLOW, RESET);
    println!();
    println!("1. Open Telegram and search for {}@userinfobot{}", BOLD, RESET);
    println!("2. Send any message (e.g., {}/start{})", BOLD, RESET);
    println!("3. The bot will reply with your user ID (a number)");
    println!();
    println!("{}Example ID:{} 123456789", CYAN, RESET);
    println!();

    let user_id = loop {
        print!("Paste your Telegram user ID: ");
        io::stdout().flush().unwrap();

        let mut input = String::new();
        io::stdin().read_line(&mut input).unwrap();
        let id = input.trim().to_string();

        if id.is_empty() {
            println!("{}✗{} No user ID provided. Try again or press Ctrl+C to cancel.", RED, RESET);
            continue;
        }

        if !id.chars().all(|c| c.is_ascii_digit()) {
            println!("{}✗{} User ID should be a number (digits only)", RED, RESET);
            print!("Continue anyway? [y/N] ");
            io::stdout().flush().unwrap();
            let mut confirm = String::new();
            io::stdin().read_line(&mut confirm).unwrap();
            if !confirm.trim().eq_ignore_ascii_case("y") {
                continue;
            }
        }

        break id;
    };

    println!("{}✓{} User ID received", GREEN, RESET);
    println!();

    // Step 3: Save configuration
    println!("{}Step 3: Save Configuration{}", YELLOW, RESET);
    println!();
    println!("Creating {}/etc/anna/telegram.env{}...", BOLD, RESET);
    println!();

    let config_content = format!(
        "ANNA_TELEGRAM_TOKEN={}\nANNA_TELEGRAM_USERS={}\n",
        token, user_id
    );

    // Write to temp file first, then move with sudo
    let temp_path = "/tmp/anna_telegram.env";
    if let Err(e) = std::fs::write(temp_path, &config_content) {
        println!("{}✗{} Failed to write config: {}", RED, RESET, e);
        return;
    }

    // Move to /etc/anna/ with sudo
    let status = Command::new("pkexec")
        .args([
            "sh", "-c",
            &format!(
                "mv {} /etc/anna/telegram.env && chmod 640 /etc/anna/telegram.env && chown root:anna /etc/anna/telegram.env",
                temp_path
            )
        ])
        .status();

    match status {
        Ok(s) if s.success() => {
            println!("{}✓{} Configuration saved to /etc/anna/telegram.env", GREEN, RESET);
            println!();
        }
        _ => {
            println!("{}✗{} Failed to save configuration (permission denied)", RED, RESET);
            println!();
            println!("Manual setup:");
            println!("  sudo tee /etc/anna/telegram.env >/dev/null <<EOF");
            println!("ANNA_TELEGRAM_TOKEN={}", token);
            println!("ANNA_TELEGRAM_USERS={}", user_id);
            println!("EOF");
            println!("  sudo chmod 640 /etc/anna/telegram.env");
            println!("  sudo chown root:anna /etc/anna/telegram.env");
            println!();
            return;
        }
    }

    // Step 4: Restart daemon
    println!("{}Step 4: Restart Anna Daemon{}", YELLOW, RESET);
    println!();
    println!("Restarting annad to load Telegram configuration...");
    println!();

    let restart = Command::new("sudo")
        .args(["systemctl", "restart", "annad"])
        .status();

    match restart {
        Ok(s) if s.success() => {
            println!("{}✓{} Daemon restarted successfully", GREEN, RESET);
            println!();
        }
        _ => {
            println!("{}⚠{} Could not restart daemon automatically", YELLOW, RESET);
            println!("  Run: {}sudo systemctl restart annad{}", BOLD, RESET);
            println!();
        }
    }

    // Final instructions
    println!("{}", "─".repeat(60));
    println!();
    println!("{}{}Setup Complete!{}", BOLD, GREEN, RESET);
    println!();
    println!("{}Next steps:{}", BOLD, RESET);
    println!("  1. Open Telegram and find your bot (search for the username you chose)");
    println!("  2. Send: {}/start{}", BOLD, RESET);
    println!("  3. Try asking: {}\"what's my disk usage?\"{}", BOLD, RESET);
    println!();
    println!("Anna will send you:");
    println!("  • Daily morning briefings at 8:00 AM with system health charts");
    println!("  • Instant answers to your questions");
    println!("  • Critical alerts if something needs attention");
    println!();
    println!("{}", "─".repeat(60));
    println!();
}

/// Show current Telegram configuration status
pub fn show_telegram_status() {
    println!();
    println!("{}Telegram Configuration{}", BOLD, RESET);
    println!("{}", "─".repeat(60));
    println!();

    let config_path = "/etc/anna/telegram.env";

    if std::path::Path::new(config_path).exists() {
        // Read configuration
        match std::fs::read_to_string(config_path) {
            Ok(content) => {
                let has_token = content.contains("ANNA_TELEGRAM_TOKEN=");
                let has_user = content.contains("ANNA_TELEGRAM_USERS=");

                if has_token && has_user {
                    println!("{}✓{} Telegram is configured", GREEN, RESET);
                    println!();
                    println!("Configuration: {}", config_path);

                    // Extract user ID (safe to show)
                    if let Some(line) = content.lines().find(|l| l.starts_with("ANNA_TELEGRAM_USERS=")) {
                        let user_id = line.strip_prefix("ANNA_TELEGRAM_USERS=").unwrap_or("");
                        println!("User ID: {}", user_id);
                    }

                    // Don't show token (security)
                    println!("Token: {}[hidden]{}", CYAN, RESET);
                    println!();
                    println!("To reconfigure, run: {}annactl telegram setup{}", BOLD, RESET);
                } else {
                    println!("{}✗{} Configuration incomplete", YELLOW, RESET);
                    println!();
                    println!("Missing: {}",
                        if !has_token { "token" } else { "user ID" });
                    println!();
                    println!("Run: {}annactl telegram setup{}", BOLD, RESET);
                }
            }
            Err(_) => {
                println!("{}✗{} Cannot read configuration (permission denied)", RED, RESET);
                println!();
                println!("File exists but is not readable by current user.");
                println!("This is normal - configuration is protected.");
                println!();
            }
        }
    } else {
        println!("{}○{} Telegram not configured", YELLOW, RESET);
        println!();
        println!("To set up Telegram access:");
        println!("  {}annactl telegram setup{}", BOLD, RESET);
        println!();
        println!("You'll be able to:");
        println!("  • Control Anna from your phone");
        println!("  • Receive morning briefings with charts");
        println!("  • Get critical alerts");
        println!();
    }

    println!("{}", "─".repeat(60));
    println!();
}
