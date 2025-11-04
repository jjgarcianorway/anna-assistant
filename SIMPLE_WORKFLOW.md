# Simple Workflow

## You make changes, then:

```bash
./scripts/release.sh      # Builds, commits, tags, pushes, creates GitHub release
sudo ./scripts/install.sh # Downloads and installs latest GitHub release
```

Done.

- `release.sh` = everything (build, git, release)
- `install.sh` = download from GitHub and install

That's it.
