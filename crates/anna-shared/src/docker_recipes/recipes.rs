//! Built-in Docker Compose recipes (v0.0.235).

use super::types::{DockerFeature, DockerRecipe};

/// Get built-in Docker Compose recipes
pub fn builtin_recipes() -> Vec<DockerRecipe> {
    vec![
        // Create compose file
        DockerRecipe::new(
            DockerFeature::CreateCompose,
            "Create a docker-compose.yml file",
        )
        .with_answer(
            r#"To create a docker-compose.yml:

**Basic template:**
```yaml
version: '3.8'

services:
  app:
    image: nginx:latest
    ports:
      - "8080:80"
    volumes:
      - ./html:/usr/share/nginx/html
    restart: unless-stopped

  db:
    image: postgres:15
    environment:
      POSTGRES_USER: myuser
      POSTGRES_PASSWORD: mypassword
      POSTGRES_DB: mydb
    volumes:
      - db_data:/var/lib/postgresql/data
    restart: unless-stopped

volumes:
  db_data:
```

**With build from Dockerfile:**
```yaml
services:
  app:
    build:
      context: .
      dockerfile: Dockerfile
    ports:
      - "3000:3000"
```

**Common options:**
- `image:` - Docker image to use
- `build:` - Build from Dockerfile
- `ports:` - Port mapping (host:container)
- `volumes:` - Mount volumes
- `environment:` - Environment variables
- `depends_on:` - Service dependencies
- `restart:` - Restart policy (no, always, on-failure, unless-stopped)"#,
        ),
        // Start services
        DockerRecipe::new(
            DockerFeature::StartServices,
            "Start Docker Compose services",
        )
        .with_command("docker compose up -d")
        .with_answer(
            r#"To start Docker Compose services:

**Start in background:**
```bash
docker compose up -d
```

**Start with logs visible:**
```bash
docker compose up
```

**Start specific service:**
```bash
docker compose up -d servicename
```

**Rebuild and start:**
```bash
docker compose up -d --build
```

**Force recreate containers:**
```bash
docker compose up -d --force-recreate
```

**Scale a service:**
```bash
docker compose up -d --scale web=3
```

**Useful flags:**
- `-d` - Detached mode (background)
- `--build` - Build images before starting
- `--force-recreate` - Recreate containers
- `--no-deps` - Don't start linked services
- `--remove-orphans` - Remove old containers"#,
        ),
        // Stop services
        DockerRecipe::new(DockerFeature::StopServices, "Stop Docker Compose services")
            .with_command("docker compose down")
            .with_answer(
                r#"To stop Docker Compose services:

**Stop and remove containers:**
```bash
docker compose down
```

**Stop without removing:**
```bash
docker compose stop
```

**Stop and remove volumes:**
```bash
docker compose down -v
```

**Stop and remove images:**
```bash
docker compose down --rmi all
```

**Stop specific service:**
```bash
docker compose stop servicename
```

**Remove everything (containers, volumes, images, networks):**
```bash
docker compose down -v --rmi all --remove-orphans
```"#,
            ),
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
        // Build images
        DockerRecipe::new(DockerFeature::BuildImages, "Build Docker images")
            .with_command("docker compose build")
            .with_answer(
                r#"To build Docker images:

**Build all services:**
```bash
docker compose build
```

**Build specific service:**
```bash
docker compose build servicename
```

**Build without cache:**
```bash
docker compose build --no-cache
```

**Build with progress output:**
```bash
docker compose build --progress=plain
```

**Build with build args:**
```bash
docker compose build --build-arg VERSION=1.0
```

**In docker-compose.yml:**
```yaml
services:
  app:
    build:
      context: .
      dockerfile: Dockerfile
      args:
        - VERSION=1.0
```"#,
            ),
        // Pull images
        DockerRecipe::new(DockerFeature::PullImages, "Pull Docker images")
            .with_command("docker compose pull")
            .with_answer(
                r#"To pull/update Docker images:

**Pull all images:**
```bash
docker compose pull
```

**Pull specific service:**
```bash
docker compose pull servicename
```

**Pull and start:**
```bash
docker compose pull && docker compose up -d
```

**Check for updates:**
```bash
docker compose pull --dry-run
```

**For individual images:**
```bash
docker pull nginx:latest
docker pull postgres:15
```"#,
            ),
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
