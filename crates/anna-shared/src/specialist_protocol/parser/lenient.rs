//! Lenient parsing with sensible defaults for partial/malformed JSON.

use crate::specialist_protocol::{
    schema::{ResponseMeta, ResponseStatus, StrictResponse},
    ResponseDetails,
};
use serde_json;

/// Try lenient parsing with sensible defaults
pub fn try_lenient_parse(json_str: &str) -> Option<StrictResponse> {
    // Try parsing as a partial object and fill in defaults
    let value: serde_json::Value = serde_json::from_str(json_str).ok()?;
    let obj = value.as_object()?;

    // Extract fields with defaults
    let status = obj
        .get("status")
        .and_then(|v| v.as_str())
        .and_then(|s| match s {
            "success" => Some(ResponseStatus::Success),
            "partial" => Some(ResponseStatus::Partial),
            "failure" => Some(ResponseStatus::Failure),
            _ => None,
        })
        .unwrap_or(ResponseStatus::Failure);

    let confidence = obj
        .get("confidence")
        .and_then(|v| v.as_f64())
        .map(|f| f as f32)
        .unwrap_or(0.0);

    let domain = obj
        .get("domain")
        .and_then(|v| v.as_str())
        .unwrap_or("unknown")
        .to_string();

    let intent = obj
        .get("intent")
        .and_then(|v| v.as_str())
        .unwrap_or("unknown")
        .to_string();

    let summary = obj
        .get("summary")
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .to_string();

    // Try to get key_facts from details
    let key_facts: Vec<String> = obj
        .get("details")
        .and_then(|d| d.get("key_facts"))
        .and_then(|kf| kf.as_array())
        .map(|arr| {
            arr.iter()
                .filter_map(|v| v.as_str().map(|s| s.to_string()))
                .collect()
        })
        .unwrap_or_default();

    let meta = ResponseMeta {
        handled_by: obj
            .get("meta")
            .and_then(|m| m.get("handled_by"))
            .and_then(|v| v.as_str())
            .unwrap_or("Unknown")
            .to_string(),
        ticket_id: obj
            .get("meta")
            .and_then(|m| m.get("ticket_id"))
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string(),
        version: 1,
    };

    Some(StrictResponse {
        status,
        confidence,
        domain,
        intent,
        summary,
        details: ResponseDetails {
            key_facts,
            diagnosis: None,
            recommendations: vec![],
        },
        actions: Default::default(),
        evidence: Default::default(),
        metrics: Default::default(),
        meta,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_lenient_parsing() {
        let json = r#"{
            "status": "success",
            "summary": "It works",
            "meta": { "ticket_id": "T-1" }
        }"#;

        let result = try_lenient_parse(json);
        assert!(result.is_some());

        let response = result.unwrap();
        assert_eq!(response.status, ResponseStatus::Success);
        assert_eq!(response.summary, "It works");
        assert_eq!(response.domain, "unknown"); // default
        assert_eq!(response.confidence, 0.0); // default
    }
}
