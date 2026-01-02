//! Docker container operations, cleanup, and debugging recipes.

use super::types::{DockerFeature, DockerRecipe};

/// Get Docker container operations, cleanup, and debugging recipes
pub fn operation_recipes() -> Vec<DockerRecipe> {
    vec![
        // Exec in container
        DockerRecipe::new(DockerFeature::ExecContainer, "Execute command in container")
            .with_command("docker compose exec servicename bash")
            .with_answer(
                r#"To execute commands in containers:

**Interactive shell:**
```bash
docker compose exec servicename bash
docker compose exec servicename sh    # If bash not available
```

**Run a command:**
```bash
docker compose exec servicename ls -la
docker compose exec db psql -U myuser mydb
```

**As different user:**
```bash
docker compose exec -u root servicename bash
```

**Without TTY (for scripts):**
```bash
docker compose exec -T servicename command
```

**For non-running containers (run instead of exec):**
```bash
docker compose run servicename bash
```

**Attach to running container:**
```bash
docker attach containername
# Detach with Ctrl+P Ctrl+Q
```"#,
            ),
        // Cleanup
        DockerRecipe::new(DockerFeature::Cleanup, "Cleanup Docker resources").with_answer(
            r#"To cleanup Docker resources:

**Remove stopped containers:**
```bash
docker container prune
```

**Remove unused images:**
```bash
docker image prune
docker image prune -a  # Remove all unused images
```

**Remove unused volumes:**
```bash
docker volume prune
```

**Remove unused networks:**
```bash
docker network prune
```

**Remove everything unused:**
```bash
docker system prune
docker system prune -a --volumes  # Everything including volumes
```

**Check disk usage:**
```bash
docker system df
```

**For Compose project:**
```bash
docker compose down -v --rmi all --remove-orphans
```"#,
        ),
        // Debug
        DockerRecipe::new(DockerFeature::Debug, "Debug Docker issues").with_answer(
            r#"To debug Docker issues:

1. **Check container status:**
```bash
docker compose ps
docker inspect containername
```

2. **View logs:**
```bash
docker compose logs servicename
docker logs containername --tail=100
```

3. **Check resource usage:**
```bash
docker stats
```

4. **Inspect networking:**
```bash
docker network ls
docker network inspect networkname
```

5. **Check if ports are in use:**
```bash
ss -tlnp | grep :8080
```

6. **Interactive debug:**
```bash
docker compose exec servicename sh
docker run -it --entrypoint sh imagename
```

7. **Common issues:**
- **Port conflict**: Another process using the port
- **Volume permissions**: Check file ownership
- **Network issues**: Verify service names for DNS
- **OOM killed**: Check memory limits and `docker events`

8. **View events:**
```bash
docker events --since 1h
```"#,
        ),
    ]
}
