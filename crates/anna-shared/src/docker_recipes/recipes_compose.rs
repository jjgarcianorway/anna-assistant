//! Docker Compose file creation and service management recipes.

use super::types::{DockerFeature, DockerRecipe};

/// Get Docker Compose file creation and service management recipes
pub fn compose_recipes() -> Vec<DockerRecipe> {
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
    ]
}
