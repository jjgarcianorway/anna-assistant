//! v0.3.250: Update channel — controls which GitHub releases trigger auto-update.
//!
//! Set in /etc/anna/config.toml:
//!   update_channel = "stable"          # default — latest non-pre-release
//!   update_channel = "beta"            # includes pre-releases
//!   update_channel = "pinned:0.3.249"  # never auto-update away from this version

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(try_from = "String", into = "String")]
pub enum UpdateChannel {
    /// Track the latest stable (non-pre-release) GitHub release.
    Stable,
    /// Track the latest release including pre-releases.
    Beta,
    /// Pin to a specific version — auto-update is disabled unless pinned version != current.
    Pinned(String),
}

impl Default for UpdateChannel {
    fn default() -> Self {
        UpdateChannel::Stable
    }
}

impl TryFrom<String> for UpdateChannel {
    type Error = String;
    fn try_from(s: String) -> Result<Self, String> {
        match s.as_str() {
            "stable" => Ok(UpdateChannel::Stable),
            "beta" => Ok(UpdateChannel::Beta),
            _ if s.starts_with("pinned:") => {
                let v = s["pinned:".len()..].trim().to_string();
                if v.is_empty() {
                    Err("pinned: requires a version string, e.g. pinned:0.3.249".to_string())
                } else {
                    Ok(UpdateChannel::Pinned(v))
                }
            }
            _ => Err(format!(
                "unknown update_channel '{}'; valid values: stable, beta, pinned:<version>",
                s
            )),
        }
    }
}

impl From<UpdateChannel> for String {
    fn from(c: UpdateChannel) -> String {
        match c {
            UpdateChannel::Stable => "stable".to_string(),
            UpdateChannel::Beta => "beta".to_string(),
            UpdateChannel::Pinned(v) => format!("pinned:{}", v),
        }
    }
}
