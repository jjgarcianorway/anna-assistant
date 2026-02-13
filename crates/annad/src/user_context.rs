//! User Context Detection - Find the REAL user, not the daemon user.
//!
//! Problem: Daemon runs as root, but we need to know the actual calling user.
//! Solution: Check multiple sources in priority order.

use anyhow::Result;
use std::process::Command;
use tracing::{debug, warn};

/// Get the real user who is using Anna (not the daemon user).
pub fn get_real_user() -> Result<String> {
    // Priority 1: SUDO_USER (if user ran via sudo)
    if let Ok(sudo_user) = std::env::var("SUDO_USER") {
        if !sudo_user.is_empty() && sudo_user != "root" {
            debug!("Real user from SUDO_USER: {}", sudo_user);
            return Ok(sudo_user);
        }
    }

    // Priority 2: Check who owns /run/user/* directories
    // When a user logs in, their session creates /run/user/<uid>
    // We can map this back to the username
    if let Ok(username) = get_user_from_runtime_dir() {
        debug!("Real user from runtime dir: {}", username);
        return Ok(username);
    }

    // Priority 3: Check who is logged in (if only one user)
    if let Ok(username) = get_single_logged_in_user() {
        debug!("Real user from w command: {}", username);
        return Ok(username);
    }

    // Priority 4: USER environment variable (might be root)
    if let Ok(user) = std::env::var("USER") {
        if user != "root" {
            debug!("Real user from USER env: {}", user);
            return Ok(user);
        }
    }

    // Priority 5: Check socket ownership (if available)
    if let Ok(username) = get_user_from_socket() {
        debug!("Real user from socket: {}", username);
        return Ok(username);
    }

    // Fallback: Use root if all else fails (daemon mode)
    warn!("Could not determine real user, falling back to root");
    Ok("root".to_string())
}

/// Get user from /run/user/* directory ownership.
fn get_user_from_runtime_dir() -> Result<String> {
    let output = Command::new("ls")
        .arg("-l")
        .arg("/run/user/")
        .output()?;

    if !output.status.success() {
        return Err(anyhow::anyhow!("ls failed"));
    }

    let stdout = String::from_utf8_lossy(&output.stdout);

    // Parse ls output to find user directories
    // drwx------ 15 lhoqvso lhoqvso 480 Feb 13 06:00 1000
    for line in stdout.lines() {
        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.len() >= 3 {
            let owner = parts[2];
            if owner != "root" {
                return Ok(owner.to_string());
            }
        }
    }

    Err(anyhow::anyhow!("No user runtime directories found"))
}

/// Get single logged-in user if there's only one.
fn get_single_logged_in_user() -> Result<String> {
    let output = Command::new("w")
        .arg("-h")
        .output()?;

    if !output.status.success() {
        return Err(anyhow::anyhow!("w command failed"));
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    let mut users: Vec<String> = Vec::new();

    for line in stdout.lines() {
        let parts: Vec<&str> = line.split_whitespace().collect();
        if let Some(&username) = parts.first() {
            if username != "root" && !users.contains(&username.to_string()) {
                users.push(username.to_string());
            }
        }
    }

    // Only return if there's exactly one non-root user
    if users.len() == 1 {
        Ok(users[0].clone())
    } else if users.is_empty() {
        Err(anyhow::anyhow!("No users logged in"))
    } else {
        Err(anyhow::anyhow!("Multiple users logged in"))
    }
}

/// Get user from socket ownership (check /run/anna/anna.sock).
fn get_user_from_socket() -> Result<String> {
    let output = Command::new("ls")
        .arg("-l")
        .arg("/run/anna/anna.sock")
        .output()?;

    if !output.status.success() {
        return Err(anyhow::anyhow!("Socket not found"));
    }

    let stdout = String::from_utf8_lossy(&output.stdout);

    // srw-rw---- 1 root anna 0 Feb 13 06:00 /run/anna/anna.sock
    // We want the group (anna) or check who's in the anna group

    // For now, if socket exists and is accessible, check who's in anna group
    get_users_in_anna_group()
}

/// Get users in the 'anna' group.
fn get_users_in_anna_group() -> Result<String> {
    let output = Command::new("getent")
        .arg("group")
        .arg("anna")
        .output()?;

    if !output.status.success() {
        return Err(anyhow::anyhow!("anna group not found"));
    }

    let stdout = String::from_utf8_lossy(&output.stdout);

    // anna:x:999:lhoqvso,otheruser
    if let Some(line) = stdout.lines().next() {
        let parts: Vec<&str> = line.split(':').collect();
        if parts.len() >= 4 {
            let members = parts[3];
            let users: Vec<&str> = members.split(',').collect();

            // If only one user in anna group, that's probably our user
            if users.len() == 1 && !users[0].is_empty() {
                return Ok(users[0].to_string());
            }
        }
    }

    Err(anyhow::anyhow!("Could not determine user from anna group"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_real_user() {
        // This will vary by system, just ensure it doesn't panic
        let user = get_real_user();
        assert!(user.is_ok());
        let username = user.unwrap();
        assert!(!username.is_empty());
        println!("Detected user: {}", username);
    }
}
