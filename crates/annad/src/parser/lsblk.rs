//! Parser for lsblk JSON output

use anyhow::Result;
use serde_json::{json, Value};

/// Parse lsblk -J output
pub fn parse(raw: &str) -> Result<Value> {
    // lsblk -J outputs valid JSON directly
    let parsed: Value = serde_json::from_str(raw)?;

    let mut disks: Vec<Value> = Vec::new();
    let mut total_size: u64 = 0;
    let mut total_used: u64 = 0;

    if let Some(blockdevices) = parsed.get("blockdevices").and_then(|v| v.as_array()) {
        for device in blockdevices {
            let name = device.get("name").and_then(|v| v.as_str()).unwrap_or("");
            let dev_type = device.get("type").and_then(|v| v.as_str()).unwrap_or("");
            let size_str = device.get("size").and_then(|v| v.as_str()).unwrap_or("0");
            let mountpoint = device.get("mountpoint").and_then(|v| v.as_str());

            // Parse size (can be "500G", "1T", etc.)
            let size_bytes = parse_size(size_str);

            if dev_type == "disk" {
                total_size += size_bytes;

                let mut partitions: Vec<Value> = Vec::new();
                if let Some(children) = device.get("children").and_then(|v| v.as_array()) {
                    for child in children {
                        let part_name =
                            child.get("name").and_then(|v| v.as_str()).unwrap_or("");
                        let part_size_str =
                            child.get("size").and_then(|v| v.as_str()).unwrap_or("0");
                        let part_mountpoint = child.get("mountpoint").and_then(|v| v.as_str());
                        let fstype = child.get("fstype").and_then(|v| v.as_str());

                        let part_size = parse_size(part_size_str);

                        // Estimate used space (would need df for accurate info)
                        if part_mountpoint.is_some() {
                            total_used += part_size / 2; // Rough estimate
                        }

                        partitions.push(json!({
                            "name": part_name,
                            "size_bytes": part_size,
                            "mountpoint": part_mountpoint,
                            "fstype": fstype,
                        }));
                    }
                }

                disks.push(json!({
                    "name": name,
                    "size_bytes": size_bytes,
                    "size_gb": (size_bytes as f64 / 1073741824.0 * 10.0).round() / 10.0,
                    "mountpoint": mountpoint,
                    "partitions": partitions,
                }));
            }
        }
    }

    Ok(json!({
        "disks": disks,
        "total_size_bytes": total_size,
        "total_size_gb": (total_size as f64 / 1073741824.0 * 10.0).round() / 10.0,
        "disk_count": disks.len(),
    }))
}

/// Parse size string like "500G", "1T", "256M" to bytes
fn parse_size(s: &str) -> u64 {
    let s = s.trim();
    if s.is_empty() {
        return 0;
    }

    let (num_str, suffix) = if s.chars().last().map_or(false, |c| c.is_alphabetic()) {
        let idx = s.len() - 1;
        (&s[..idx], &s[idx..])
    } else {
        (s, "")
    };

    let num: f64 = num_str.parse().unwrap_or(0.0);
    let multiplier: u64 = match suffix.to_uppercase().as_str() {
        "K" => 1024,
        "M" => 1024 * 1024,
        "G" => 1024 * 1024 * 1024,
        "T" => 1024 * 1024 * 1024 * 1024,
        _ => 1,
    };

    (num * multiplier as f64) as u64
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_size() {
        assert_eq!(parse_size("500G"), 536870912000);
        assert_eq!(parse_size("1T"), 1099511627776);
        assert_eq!(parse_size("256M"), 268435456);
    }

    #[test]
    fn test_parse_lsblk() {
        let sample = r#"{
            "blockdevices": [
                {
                    "name": "sda",
                    "type": "disk",
                    "size": "500G",
                    "children": [
                        {"name": "sda1", "size": "512M", "mountpoint": "/boot", "fstype": "vfat"},
                        {"name": "sda2", "size": "499G", "mountpoint": "/", "fstype": "ext4"}
                    ]
                }
            ]
        }"#;
        let result = parse(sample).unwrap();
        assert_eq!(result["disk_count"], 1);
        assert!(result["total_size_gb"].as_f64().unwrap() > 400.0);
    }
}
