//! Ollama Installer - Automatic Local LLM Bootstrap
//!
//! Anna orchestrates the installation and configuration of Ollama
//! so she can have her own local LLM brain without user intervention.

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::process::Command;
use tracing::{debug, info, warn};

/// Default Ollama endpoint
pub const DEFAULT_OLLAMA_URL: &str = "http://127.0.0.1:11434/v1";

/// Default model to use
pub const DEFAULT_MODEL: &str = "llama3.2:1b"; // Small, fast model

/// Ollama installation state
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum OllamaState {
    /// Not installed
    NotInstalled,

    /// Installed but not running
    Installed,

    /// Running but no models available
    RunningNoModels,

    /// Fully configured and ready
    Ready,
}

/// Ollama installer
pub struct OllamaInstaller {
    /// Whether to use sudo for operations
    use_sudo: bool,
}

impl OllamaInstaller {
    /// Create a new installer
    pub fn new() -> Self {
        Self {
            use_sudo: !nix::unistd::getuid().is_root(),
        }
    }

    /// Check current Ollama state
    pub fn check_state(&self) -> Result<OllamaState> {
        // Check if Ollama is installed
        if !self.is_ollama_installed()? {
            return Ok(OllamaState::NotInstalled);
        }

        // Check if service is running
        if !self.is_service_running()? {
            return Ok(OllamaState::Installed);
        }

        // Check if models are available
        if !self.has_models()? {
            return Ok(OllamaState::RunningNoModels);
        }

        Ok(OllamaState::Ready)
    }

    /// Check if Ollama is installed
    fn is_ollama_installed(&self) -> Result<bool> {
        let output = Command::new("which")
            .arg("ollama")
            .output()
            .context("Failed to check for ollama binary")?;

        Ok(output.status.success())
    }

    /// Check if Ollama service is running
    fn is_service_running(&self) -> Result<bool> {
        // Try to connect to the API
        let client = reqwest::blocking::Client::new();
        let result = client
            .get("http://127.0.0.1:11434/api/tags")
            .timeout(std::time::Duration::from_secs(2))
            .send();

        Ok(result.is_ok())
    }

    /// Check if any models are available
    fn has_models(&self) -> Result<bool> {
        let client = reqwest::blocking::Client::new();
        let response = client
            .get("http://127.0.0.1:11434/api/tags")
            .timeout(std::time::Duration::from_secs(5))
            .send()
            .context("Failed to check Ollama models")?;

        if !response.status().is_success() {
            return Ok(false);
        }

        #[derive(Deserialize)]
        struct TagsResponse {
            models: Vec<serde_json::Value>,
        }

        let tags: TagsResponse = response.json().context("Failed to parse models list")?;
        Ok(!tags.models.is_empty())
    }

    /// Install Ollama via package manager
    ///
    /// Returns the commands that were executed for transparency
    pub fn install_ollama(&self) -> Result<Vec<String>> {
        let mut commands = Vec::new();

        info!("Installing Ollama via pacman...");

        // Try official repo first
        let install_cmd = if self.use_sudo {
            vec!["sudo", "pacman", "-S", "--noconfirm", "ollama"]
        } else {
            vec!["pacman", "-S", "--noconfirm", "ollama"]
        };

        let output = Command::new(install_cmd[0])
            .args(&install_cmd[1..])
            .output()
            .context("Failed to execute pacman")?;

        commands.push(install_cmd.join(" "));

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            warn!("Pacman install failed: {}", stderr);

            // Try AUR with yay if available
            if self.is_yay_installed()? {
                info!("Trying AUR install with yay...");
                let yay_cmd = vec!["yay", "-S", "--noconfirm", "ollama"];

                let output = Command::new("yay")
                    .args(&yay_cmd[1..])
                    .output()
                    .context("Failed to execute yay")?;

                commands.push(yay_cmd.join(" "));

                if !output.status.success() {
                    anyhow::bail!("Failed to install Ollama via pacman or yay");
                }
            } else {
                anyhow::bail!("Ollama not in official repos and yay not available");
            }
        }

        info!("Ollama installed successfully");
        Ok(commands)
    }

    /// Check if yay is installed
    fn is_yay_installed(&self) -> Result<bool> {
        let output = Command::new("which")
            .arg("yay")
            .output()
            .context("Failed to check for yay")?;

        Ok(output.status.success())
    }

    /// Start Ollama service
    pub fn start_service(&self) -> Result<Vec<String>> {
        let mut commands = Vec::new();

        info!("Starting Ollama service...");

        // Enable and start systemd service
        let enable_cmd = if self.use_sudo {
            vec!["sudo", "systemctl", "enable", "ollama"]
        } else {
            vec!["systemctl", "enable", "ollama"]
        };

        Command::new(enable_cmd[0])
            .args(&enable_cmd[1..])
            .output()
            .context("Failed to enable Ollama service")?;

        commands.push(enable_cmd.join(" "));

        let start_cmd = if self.use_sudo {
            vec!["sudo", "systemctl", "start", "ollama"]
        } else {
            vec!["systemctl", "start", "ollama"]
        };

        Command::new(start_cmd[0])
            .args(&start_cmd[1..])
            .output()
            .context("Failed to start Ollama service")?;

        commands.push(start_cmd.join(" "));

        // Wait a moment for service to start
        std::thread::sleep(std::time::Duration::from_secs(2));

        info!("Ollama service started");
        Ok(commands)
    }

    /// Pull default model
    pub fn pull_model(&self, model: &str) -> Result<Vec<String>> {
        let mut commands = Vec::new();

        info!("Pulling model: {}", model);

        let pull_cmd = vec!["ollama", "pull", model];

        let output = Command::new("ollama")
            .args(&["pull", model])
            .output()
            .context("Failed to pull Ollama model")?;

        commands.push(pull_cmd.join(" "));

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            anyhow::bail!("Failed to pull model {}: {}", model, stderr);
        }

        info!("Model {} pulled successfully", model);
        Ok(commands)
    }

    /// Test that Ollama is working with a simple query
    pub fn test_ollama(&self) -> Result<bool> {
        debug!("Testing Ollama with simple query...");

        let client = reqwest::blocking::Client::new();

        let test_prompt = serde_json::json!({
            "model": DEFAULT_MODEL,
            "messages": [
                {
                    "role": "user",
                    "content": "Say 'test' in one word"
                }
            ],
            "stream": false
        });

        let response = client
            .post("http://127.0.0.1:11434/v1/chat/completions")
            .json(&test_prompt)
            .timeout(std::time::Duration::from_secs(30))
            .send()
            .context("Failed to send test query to Ollama")?;

        if !response.status().is_success() {
            warn!("Ollama test query failed with status: {}", response.status());
            return Ok(false);
        }

        debug!("Ollama test successful");
        Ok(true)
    }

    /// Full automatic setup - install, configure, and test
    ///
    /// Returns all commands executed for transparency
    pub fn auto_setup(&self) -> Result<Vec<String>> {
        let mut all_commands = Vec::new();

        let state = self.check_state()?;
        info!("Current Ollama state: {:?}", state);

        match state {
            OllamaState::NotInstalled => {
                info!("Ollama not installed - installing...");
                all_commands.extend(self.install_ollama()?);
                all_commands.extend(self.start_service()?);
                all_commands.extend(self.pull_model(DEFAULT_MODEL)?);
            }
            OllamaState::Installed => {
                info!("Ollama installed but not running - starting...");
                all_commands.extend(self.start_service()?);
                all_commands.extend(self.pull_model(DEFAULT_MODEL)?);
            }
            OllamaState::RunningNoModels => {
                info!("Ollama running but no models - pulling default...");
                all_commands.extend(self.pull_model(DEFAULT_MODEL)?);
            }
            OllamaState::Ready => {
                info!("Ollama already ready");
                return Ok(all_commands);
            }
        }

        // Test that everything works
        if !self.test_ollama()? {
            anyhow::bail!("Ollama setup completed but test failed");
        }

        info!("Ollama auto-setup completed successfully");
        Ok(all_commands)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ollama_installer_creation() {
        let installer = OllamaInstaller::new();
        // Should not panic
        let _ = installer.check_state();
    }

    #[test]
    fn test_default_constants() {
        assert!(DEFAULT_OLLAMA_URL.starts_with("http://"));
        assert!(!DEFAULT_MODEL.is_empty());
    }
}
