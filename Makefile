.PHONY: help build build-release test clean install

# Default target
help:
	@echo "Anna Assistant - Makefile targets"
	@echo ""
	@echo "Build targets:"
	@echo "  make build          - Build debug binaries"
	@echo "  make build-release  - Build release binaries"
	@echo "  make test           - Run all tests"
	@echo ""
	@echo "Deployment:"
	@echo "  make install        - Install binaries to /usr/local/bin (requires sudo)"
	@echo ""
	@echo "Cleanup:"
	@echo "  make clean          - Remove build artifacts"

# Build debug binaries
build:
	cargo build --bins

# Build release binaries
build-release:
	cargo build --release --bins

# Run all tests
test:
	cargo test --workspace

# Clean build artifacts
clean:
	cargo clean

# Install binaries (requires sudo)
install: build-release
	@echo "Installing binaries to /usr/local/bin (requires sudo)..."
	sudo install -m 0755 target/release/annad /usr/local/bin/annad
	sudo install -m 0755 target/release/annactl /usr/local/bin/annactl
	@echo "Installed: /usr/local/bin/annad /usr/local/bin/annactl"
	@annactl --version
