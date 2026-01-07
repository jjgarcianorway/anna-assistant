.PHONY: help patch minor major commit build test clean snapshot fmt clippy install-user

help:
	@echo "Anna Assistant - Makefile targets"
	@echo ""
	@echo "Version management:"
	@echo "  patch       Bump patch version (X.Y.Z -> X.Y.Z+1)"
	@echo "  minor       Bump minor version (X.Y.Z -> X.Y+1.0)"
	@echo "  major       Bump major version (X.Y.Z -> X+1.0.0)"
	@echo ""
	@echo "Development:"
	@echo "  build       Build release binaries"
	@echo "  test        Run all tests"
	@echo "  fmt         Format all code"
	@echo "  clippy      Run clippy checks"
	@echo "  clean       Clean build artifacts"
	@echo ""
	@echo "Git workflow:"
	@echo "  commit      Commit with quality gates (requires MSG=\"...\")"
	@echo "  snapshot    Create timestamped snapshot branch"
	@echo ""
	@echo "Installation:"
	@echo "  install-user  Install to user local directories"
	@echo ""
	@echo "Example workflow:"
	@echo "  make patch && MSG=\"Add feature X\" make commit"

patch:
	@tools/bump.sh patch

minor:
	@tools/bump.sh minor

major:
	@tools/bump.sh major

commit:
	@tools/commit-if-green.sh

build:
	cargo build --release

install-user:
	install -Dm755 target/release/assistantd $(HOME)/.local/bin/assistantd
	install -Dm755 target/release/assistantctl $(HOME)/.local/bin/assistantctl
	install -Dm644 systemd/user/assistantd.service $(HOME)/.config/systemd/user/assistantd.service
	install -Dm644 etc/assistant/policy.d/default.yaml $(HOME)/.config/assistant/policy.d/default.yaml
	install -d $(HOME)/.local/share/assistant/skills
	cp -r var/lib/assistant/skills/* $(HOME)/.local/share/assistant/skills/ || true

test:
	cargo test --all

fmt:
	cargo fmt --all

clippy:
	cargo clippy --all -- -D warnings

clean:
	cargo clean

snapshot:
	@tools/snapshot.sh
