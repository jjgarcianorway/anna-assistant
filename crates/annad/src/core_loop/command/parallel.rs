//! Parallel command execution.

use std::collections::HashMap;

use super::execute::execute_command;

/// Execute multiple commands in parallel
pub fn execute_commands_parallel(commands: &[&str]) -> HashMap<String, String> {
    let results: Vec<_> = std::thread::scope(|s| {
        let handles: Vec<_> = commands
            .iter()
            .map(|cmd| {
                let cmd = *cmd;
                s.spawn(move || (cmd.to_string(), execute_command(cmd).ok()))
            })
            .collect();
        handles.into_iter().map(|h| h.join().ok()).collect()
    });

    let mut output = HashMap::new();
    for result in results.into_iter().flatten() {
        if let (cmd, Some(out)) = result {
            output.insert(cmd, out);
        }
    }
    output
}
