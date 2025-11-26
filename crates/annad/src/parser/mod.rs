//! Parser registry - parse probe output into structured JSON

mod cpuinfo;
mod lsblk;
mod meminfo;

use anyhow::{bail, Result};
use serde_json::Value;

/// Parse raw output using the specified parser
pub fn parse(parser_name: &str, raw_output: &str) -> Result<Value> {
    match parser_name {
        "raw_v1" => Ok(Value::String(raw_output.to_string())),
        "cpuinfo_v1" => cpuinfo::parse(raw_output),
        "meminfo_v1" => meminfo::parse(raw_output),
        "lsblk_v1" => lsblk::parse(raw_output),
        _ => bail!("Unknown parser: {}", parser_name),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_raw_parser() {
        let result = parse("raw_v1", "hello world").unwrap();
        assert_eq!(result.as_str().unwrap(), "hello world");
    }

    #[test]
    fn test_unknown_parser() {
        let result = parse("unknown_v1", "test");
        assert!(result.is_err());
    }
}
