.PHONY: build install uninstall clean test fmt lint smoke help

help:
	@echo "Anna v0.10.1 Build Targets"
	@echo ""
	@echo "  build      - Build release binaries"
	@echo "  install    - Install Anna system-wide (requires root)"
	@echo "  uninstall  - Remove Anna (data preserved)"
	@echo "  clean      - Clean build artifacts"
	@echo "  test       - Run unit tests"
	@echo "  fmt        - Format code"
	@echo "  lint       - Run clippy"
	@echo "  smoke      - Run smoke tests (requires install)"
	@echo ""

build:
	@echo "Building release binaries..."
	cargo build --release
	@echo "✓ Build complete"

install: build
	@echo "Installing Anna v0.10.1..."
	sudo ./scripts/install_v10.sh

uninstall:
	@echo "Uninstalling Anna v0.10.1..."
	sudo ./scripts/uninstall_v10.sh

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
