//! Docker image building and pulling recipes.

use super::types::{DockerFeature, DockerRecipe};

/// Get Docker image building and pulling recipes
pub fn image_recipes() -> Vec<DockerRecipe> {
    vec![
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
    ]
}
