//! Precondition extraction logic.

use crate::recipe_schema::Precondition;
use super::types::{FileEditType, TicketData};

/// Extract preconditions from probe data.
pub fn extract_preconditions(data: &TicketData) -> Vec<Precondition> {
    let mut preconditions = Vec::new();

    // Check for tool existence from probes
    for (probe_name, probe_result) in &data.probes_used {
        if probe_name.contains("which") || probe_name.contains("command") {
            if !probe_result.is_empty() && !probe_result.contains("not found") {
                // Extract tool name
                if let Some(tool) = extract_tool_from_which(probe_result) {
                    preconditions.push(Precondition::ToolExists { tool });
                }
            }
        }
    }

    // Check for file existence from edits
    for edit in &data.file_edits {
        if edit.edit_type != FileEditType::WriteFile {
            // Existing file edit implies file should exist
            preconditions.push(Precondition::FileExists {
                path: edit.path.clone(),
            });
        }
    }

    preconditions
}

fn extract_tool_from_which(output: &str) -> Option<String> {
    let path = output.trim();
    if path.starts_with('/') {
        path.split('/').last().map(String::from)
    } else {
        Some(path.to_string())
    }
}
