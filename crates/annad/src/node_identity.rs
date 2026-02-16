//! Machine identity: stable node_id + Ed25519 seed key generated on first run.
//!
//! Files:
//!   /var/lib/anna/node_id  — UUID4, mode 644
//!   /var/lib/anna/node_key — 64 hex chars (32-byte Ed25519 seed), mode 600
//!
//! Neither file is ever overwritten once created.

use anna_shared::paths::paths;
use anyhow::Result;
use std::io::Read;
use std::os::unix::fs::PermissionsExt;
use tracing::info;
use uuid::Uuid;

pub struct NodeIdentity {
    pub id: String,      // UUID4
    pub key_hex: String, // 64 hex chars = 32-byte Ed25519 seed
}

/// Load or generate the machine identity.
/// Safe to call multiple times — files are never overwritten.
pub fn init_node_identity() -> Result<NodeIdentity> {
    std::fs::create_dir_all(paths().data_dir.as_path())?;

    let id = load_or_generate_id()?;
    let key_hex = load_or_generate_key()?;

    info!("Machine identity: node_id={}", id);
    // Key is never logged.

    Ok(NodeIdentity { id, key_hex })
}

fn load_or_generate_id() -> Result<String> {
    let path = paths().node_id_file();
    if path.exists() {
        let s = std::fs::read_to_string(&path)?;
        return Ok(s.trim().to_string());
    }
    let id = Uuid::new_v4().to_string();
    std::fs::write(&path, &id)?;
    Ok(id)
}

fn load_or_generate_key() -> Result<String> {
    let path = paths().node_key_file();
    if path.exists() {
        let s = std::fs::read_to_string(&path)?;
        return Ok(s.trim().to_string());
    }
    // Read 32 bytes of OS entropy
    let mut seed = [0u8; 32];
    std::fs::File::open("/dev/urandom")?.read_exact(&mut seed)?;
    let hex: String = seed.iter().map(|b| format!("{:02x}", b)).collect();
    std::fs::write(&path, &hex)?;
    // Restrict to owner-read-only
    std::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o600))?;
    Ok(hex)
}
