.PHONY: build install uninstall clean test fmt lint smoke help bump check-version release

help:
	@echo "Anna Assistant Build Targets"
	@echo ""
	@echo "Development:"
	@echo "  build           - Build release binaries"
	@echo "  test            - Run unit tests"
	@echo "  fmt             - Format code"
	@echo "  lint            - Run clippy"
	@echo "  smoke           - Run smoke tests (requires install)"
	@echo "  check-version   - Verify version consistency"
	@echo ""
	@echo "Release Management:"
	@echo "  bump VERSION=v1.2.3  - Bump VERSION file and update Cargo.toml"
	@echo "  release              - Run transactional release (uses VERSION file)"
	@echo ""
	@echo "Installation:"
	@echo "  install         - Install Anna system-wide (requires root)"
	@echo "  uninstall       - Remove Anna (data preserved)"
	@echo ""
	@echo "Cleanup:"
	@echo "  clean           - Clean build artifacts"
	@echo ""

build:
	@echo "Building release binaries..."
	cargo build --release
	@echo "✓ Build complete"

install: build
	@echo "Installing Anna from local build..."
	sudo ./scripts/install.sh --from-local

uninstall:
	@echo "Uninstalling Anna..."
	sudo ./scripts/uninstall.sh 2>/dev/null || sudo ./scripts/uninstall_v10.sh 2>/dev/null || echo "No uninstall script found"

clean:
	@echo "Cleaning build artifacts..."
	cargo clean
	@echo "✓ Clean complete"

test:
	@echo "Running unit tests..."
	cargo test --all
	@echo "✓ Tests complete"

fmt:
	@echo "Formatting code..."
	cargo fmt --all
	@echo "✓ Format complete"

lint:
	@echo "Running clippy..."
	cargo clippy --all-targets -- -D warnings
	@echo "✓ Lint complete"

smoke:
	@echo "Running smoke tests..."
	@./tests/smoke.sh
	@echo "✓ Smoke tests complete"

# Bump version (updates VERSION and Cargo.toml, prompts for CHANGELOG)
bump:
	@if [ -z "$(VERSION)" ]; then \
		echo "ERROR: VERSION not specified"; \
		echo "Usage: make bump VERSION=v1.2.3"; \
		exit 1; \
	fi
	@echo "→ Bumping version to $(VERSION)..."
	@# Validate semver format
	@if ! echo "$(VERSION)" | grep -qE '^v[0-9]+\.[0-9]+\.[0-9]+(-rc\.[0-9]+)?$$'; then \
		echo "ERROR: Invalid version format: $(VERSION)"; \
		echo "Expected: v1.0.0 or v1.0.0-rc.1"; \
		exit 1; \
	fi
	@# Update VERSION file
	@echo "$(VERSION)" > VERSION
	@echo "✓ Updated VERSION file to $(VERSION)"
	@# Update Cargo.toml
	@sed -i -E 's/^version = ".*"/version = "$(subst v,,$(VERSION))"/' Cargo.toml
	@echo "✓ Updated Cargo.toml to $(subst v,,$(VERSION))"
	@# Check CHANGELOG reminder
	@echo ""
	@echo "⚠  REMINDER: Update CHANGELOG.md with changes for $(VERSION)"
	@echo ""
	@echo "Next steps:"
	@echo "  1. Edit CHANGELOG.md to add release notes"
	@echo "  2. Review changes: git diff"
	@echo "  3. Commit: git add VERSION Cargo.toml CHANGELOG.md && git commit -m 'chore: bump version to $(VERSION)'"
	@echo "  4. Release: make release"
	@echo ""

# Check version consistency
check-version:
	@echo "→ Checking version consistency..."
	@./scripts/check-version-consistency.sh

# Run transactional release
release:
	@echo "→ Running transactional release..."
	@./scripts/release.sh
