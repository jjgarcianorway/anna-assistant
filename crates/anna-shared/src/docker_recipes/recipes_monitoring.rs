//! Docker monitoring and logging recipes.

use super::types::{DockerFeature, DockerRecipe};

/// Get Docker monitoring and logging recipes
pub fn monitoring_recipes() -> Vec<DockerRecipe> {
    vec![
        // View logs
        DockerRecipe::new(DockerFeature::ViewLogs, "View Docker Compose logs")
            .with_command("docker compose logs")
            .with_answer(
                r#"To view Docker Compose logs:

**All services:**
```bash
docker compose logs
```

**Follow logs (like tail -f):**
```bash
docker compose logs -f
```

**Specific service:**
```bash
docker compose logs -f servicename
```

**Last N lines:**
```bash
docker compose logs --tail=100
```

**With timestamps:**
```bash
docker compose logs -t
```

**Since a time:**
```bash
docker compose logs --since 1h
docker compose logs --since "2024-01-01T00:00:00"
```

**Combine options:**
```bash
docker compose logs -f --tail=50 servicename
```"#,
            ),
        // List containers
        DockerRecipe::new(DockerFeature::ListContainers, "List Docker containers")
            .with_command("docker compose ps")
            .with_answer(
                r#"To list Docker containers:

**Compose services:**
```bash
docker compose ps
```

**All containers (including stopped):**
```bash
docker compose ps -a
```

**All Docker containers (not just compose):**
```bash
docker ps -a
```

**Format output:**
```bash
docker ps --format "table {{.Names}}\t{{.Status}}\t{{.Ports}}"
```

**Filter by status:**
```bash
docker ps -f "status=running"
docker ps -f "status=exited"
```

**Show resource usage:**
```bash
docker stats
```"#,
            ),
    ]
}
