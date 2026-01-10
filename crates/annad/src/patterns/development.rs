//! Development and programming patterns
//! v0.0.916: Git, Docker, and common dev tool queries

use anna_shared::rpc::{DeepUnderstanding, IntentCategory};

/// Match development-related queries
pub fn match_patterns(q: &str) -> Option<DeepUnderstanding> {
    if let Some(u) = match_git(q) {
        return Some(u);
    }
    if let Some(u) = match_docker(q) {
        return Some(u);
    }
    if let Some(u) = match_build_tools(q) {
        return Some(u);
    }
    if let Some(u) = match_languages(q) {
        return Some(u);
    }
    None
}

/// Pattern with keywords, description, topic, and commands
type DevPattern = (&'static [&'static str], &'static str, &'static str, &'static [&'static str]);

fn match_git(q: &str) -> Option<DeepUnderstanding> {
    let patterns: &[DevPattern] = &[
        // Git status/info
        (&["git", "status"], "git repository status", "git", &["git status"]),
        (&["git", "log"], "git commit history", "git", &["git log --oneline -10"]),
        (&["recent", "commit"], "recent commits", "git", &["git log --oneline -10"]),
        (&["last", "commit"], "last commit info", "git", &["git log -1"]),
        (&["git", "branch"], "git branches", "git", &["git branch -a"]),
        (&["current", "branch"], "current git branch", "git", &["git branch --show-current"]),
        (&["git", "diff"], "git differences", "git", &["git diff --stat"]),
        (&["uncommitted", "changes"], "uncommitted changes", "git", &["git status --short", "git diff --stat"]),
        (&["git", "remote"], "git remotes", "git", &["git remote -v"]),
        // Git troubleshooting
        (&["git", "conflict"], "git merge conflict", "git",
            &["git status", "echo 'FIX: Edit conflicted files, then: git add <file> && git commit'"]),
        (&["git", "undo", "commit"], "undo last commit", "git",
            &["echo 'To undo last commit (keep changes): git reset --soft HEAD~1'",
              "echo 'To undo and discard changes: git reset --hard HEAD~1'"]),
        (&["git", "stash"], "git stash operations", "git",
            &["git stash list", "echo 'Stash: git stash | Pop: git stash pop'"]),
        (&["git", "not", "push"], "git push issues", "git",
            &["git status", "git remote -v",
              "echo 'Check: git push --set-upstream origin <branch>'"]),
        (&["git", "detached", "head"], "git detached HEAD state", "git",
            &["git status", "echo 'FIX: git checkout <branch> or git checkout -b <new-branch>'"]),
        (&["git", "clean"], "git clean working directory", "git",
            &["git status", "echo 'Preview: git clean -n | Execute: git clean -fd'"]),
        (&["git", "large", "file"], "git large file issues", "git",
            &["git rev-list --objects --all | git cat-file --batch-check | sort -k3 -n | tail -10",
              "echo 'Consider git-lfs for large files'"]),
    ];

    for (keywords, interpreted, topic, commands) in patterns {
        if keywords.iter().all(|kw| q.contains(kw)) {
            return Some(DeepUnderstanding {
                interpreted_as: interpreted.to_string(),
                category: IntentCategory::Factual,
                confidence: 0.9,
                topic: Some(topic.to_string()),
                needs_confirmation: false,
                suggested_commands: commands.iter().map(|s| s.to_string()).collect(),
                ..Default::default()
            });
        }
    }
    None
}

fn match_docker(q: &str) -> Option<DeepUnderstanding> {
    let patterns: &[DevPattern] = &[
        // Docker info
        (&["docker", "container"], "docker containers", "docker", &["docker ps -a"]),
        (&["docker", "running"], "running containers", "docker", &["docker ps"]),
        (&["docker", "image"], "docker images", "docker", &["docker images"]),
        (&["docker", "volume"], "docker volumes", "docker", &["docker volume ls"]),
        (&["docker", "network"], "docker networks", "docker", &["docker network ls"]),
        (&["docker", "log"], "docker logs", "docker", &["docker ps -q | head -1 | xargs -I{} docker logs --tail 20 {}"]),
        (&["docker", "stats"], "docker resource usage", "docker", &["docker stats --no-stream"]),
        // Docker troubleshooting
        (&["docker", "not", "start"], "docker container not starting", "docker",
            &["docker ps -a | head -5", "echo 'Check logs: docker logs <container>'"]),
        (&["docker", "disk", "space"], "docker disk usage", "docker",
            &["docker system df", "echo 'Cleanup: docker system prune -a'"]),
        (&["docker", "prune"], "docker cleanup", "docker",
            &["docker system df",
              "echo 'Prune: docker system prune -a (removes unused images/containers)'"]),
        (&["docker", "compose"], "docker compose status", "docker",
            &["docker compose ps 2>/dev/null || docker-compose ps 2>/dev/null"]),
        (&["podman"], "podman info", "containers", &["podman info 2>/dev/null | head -20"]),
    ];

    for (keywords, interpreted, topic, commands) in patterns {
        if keywords.iter().all(|kw| q.contains(kw)) {
            return Some(DeepUnderstanding {
                interpreted_as: interpreted.to_string(),
                category: IntentCategory::Factual,
                confidence: 0.9,
                topic: Some(topic.to_string()),
                needs_confirmation: false,
                suggested_commands: commands.iter().map(|s| s.to_string()).collect(),
                ..Default::default()
            });
        }
    }
    None
}

fn match_build_tools(q: &str) -> Option<DeepUnderstanding> {
    let patterns: &[DevPattern] = &[
        // Make/CMake
        (&["makefile", "error"], "makefile error", "build",
            &["make -n 2>&1 | head -20", "echo 'Check: missing dependencies or syntax errors'"]),
        (&["cmake", "error"], "cmake error", "build",
            &["cat CMakeLists.txt 2>/dev/null | head -20",
              "echo 'Try: rm -rf build && mkdir build && cd build && cmake ..'"]),
        // Cargo/Rust
        (&["cargo", "build", "fail"], "cargo build failure", "rust",
            &["cargo check 2>&1 | head -30"]),
        (&["rust", "compile", "error"], "rust compilation error", "rust",
            &["cargo check 2>&1 | head -30"]),
        (&["cargo", "version"], "cargo/rust version", "rust",
            &["cargo --version", "rustc --version"]),
        // Node/npm
        (&["npm", "error"], "npm error", "nodejs",
            &["npm --version", "node --version", "cat package.json 2>/dev/null | head -10"]),
        (&["npm", "install", "fail"], "npm install failure", "nodejs",
            &["rm -rf node_modules && npm install 2>&1 | tail -20"]),
        (&["node", "version"], "node version", "nodejs",
            &["node --version", "npm --version"]),
        (&["yarn", "error"], "yarn error", "nodejs",
            &["yarn --version", "cat package.json 2>/dev/null | head -10"]),
        // Python
        (&["pip", "install", "fail"], "pip install failure", "python",
            &["pip --version", "python --version",
              "echo 'Try: pip install --user <package> or use venv'"]),
        (&["python", "version"], "python version", "python",
            &["python --version 2>/dev/null || python3 --version", "pip --version 2>/dev/null"]),
        (&["virtualenv", "activate"], "virtualenv activation", "python",
            &["echo 'Create: python -m venv venv'", "echo 'Activate: source venv/bin/activate'"]),
        // Go
        (&["go", "mod"], "go modules", "go",
            &["go mod tidy 2>&1 | head -10", "cat go.mod 2>/dev/null | head -10"]),
        (&["go", "version"], "go version", "go", &["go version"]),
    ];

    for (keywords, interpreted, topic, commands) in patterns {
        if keywords.iter().all(|kw| q.contains(kw)) {
            return Some(DeepUnderstanding {
                interpreted_as: interpreted.to_string(),
                category: IntentCategory::Factual,
                confidence: 0.9,
                topic: Some(topic.to_string()),
                needs_confirmation: false,
                suggested_commands: commands.iter().map(|s| s.to_string()).collect(),
                ..Default::default()
            });
        }
    }
    None
}

fn match_languages(q: &str) -> Option<DeepUnderstanding> {
    let patterns: &[DevPattern] = &[
        // Installed compilers/interpreters
        (&["installed", "compiler"], "installed compilers", "development",
            &["which gcc g++ clang rustc go 2>/dev/null"]),
        (&["gcc", "version"], "gcc version", "development", &["gcc --version | head -1"]),
        (&["clang", "version"], "clang version", "development", &["clang --version | head -1"]),
        // Environment
        (&["path", "variable"], "PATH environment variable", "shell",
            &["echo $PATH | tr ':' '\\n'"]),
        (&["environment", "variable"], "environment variables", "shell",
            &["env | head -30"]),
        (&["which", "command"], "command location", "shell",
            &["echo 'Usage: which <command>'"]),
    ];

    for (keywords, interpreted, topic, commands) in patterns {
        if keywords.iter().all(|kw| q.contains(kw)) {
            return Some(DeepUnderstanding {
                interpreted_as: interpreted.to_string(),
                category: IntentCategory::Factual,
                confidence: 0.9,
                topic: Some(topic.to_string()),
                needs_confirmation: false,
                suggested_commands: commands.iter().map(|s| s.to_string()).collect(),
                ..Default::default()
            });
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_git_status() {
        let result = match_patterns("git status");
        assert!(result.is_some());
        let u = result.unwrap();
        assert!(u.suggested_commands.iter().any(|c| c.contains("git status")));
    }

    #[test]
    fn test_docker_containers() {
        let result = match_patterns("list docker containers");
        assert!(result.is_some());
        let u = result.unwrap();
        assert!(u.suggested_commands.iter().any(|c| c.contains("docker ps")));
    }

    #[test]
    fn test_cargo_version() {
        let result = match_patterns("cargo version");
        assert!(result.is_some());
        let u = result.unwrap();
        assert!(u.suggested_commands.iter().any(|c| c.contains("cargo")));
    }

    #[test]
    fn test_node_version() {
        let result = match_patterns("node version");
        assert!(result.is_some());
        let u = result.unwrap();
        assert!(u.suggested_commands.iter().any(|c| c.contains("node")));
    }
}
