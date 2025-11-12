.PHONY: help build build-release test validate-1.6 consensus-poc clean install setup-system

# Default target
help:
	@echo "Anna Assistant - Makefile targets"
	@echo ""
	@echo "Build targets:"
	@echo "  make build          - Build debug binaries"
	@echo "  make build-release  - Build release binaries"
	@echo "  make test           - Run all tests"
	@echo ""
	@echo "Validation:"
	@echo "  make validate-1.6   - Run Phase 1.6 validation harness"
	@echo "  make consensus-poc  - Build Phase 1.8 consensus PoC (simulator + CLI)"
	@echo ""
	@echo "Deployment:"
	@echo "  make setup-system   - Create system user, dirs (requires sudo)"
	@echo "  make install        - Install binaries to /usr/bin (requires sudo)"
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
	cargo test --all

# Run Phase 1.6 validation harness
validate-1.6: build
	@echo "Running Phase 1.6 validation..."
	@bash scripts/validate_phase_1_6.sh

# Build Phase 1.8 consensus PoC
consensus-poc:
	@echo "=== Building Phase 1.8 Consensus PoC ==="
	@echo ""
	@echo "Building consensus simulator..."
	@cargo build --package consensus_sim
	@echo ""
	@echo "Building annactl with consensus commands..."
	@cargo build --package annactl
	@echo ""
	@echo "Creating artifacts directory..."
	@mkdir -p ./artifacts/simulations
	@echo ""
	@echo "✓ Consensus PoC built successfully!"
	@echo ""
	@echo "Available commands:"
	@echo "  ./target/debug/consensus_sim --nodes 5 --scenario healthy"
	@echo "  ./target/debug/consensus_sim --nodes 5 --scenario slow-node"
	@echo "  ./target/debug/consensus_sim --nodes 5 --scenario byzantine"
	@echo "  ./target/debug/annactl consensus init-keys"
	@echo "  ./target/debug/annactl consensus status --json"
	@echo ""
	@echo "See docs/consensus_poc_user_guide.md for full documentation."

# Clean build artifacts
clean:
	cargo clean
	rm -f validation_report_*.txt

# Setup system (requires sudo)
setup-system:
	@echo "Setting up system user and directories (requires sudo)..."
	sudo bash scripts/setup-anna-system.sh

# Install binaries (requires sudo)
install: build-release
	@echo "Installing binaries to /usr/bin (requires sudo)..."
	sudo install -m 0755 target/release/annad /usr/bin/annad
	sudo install -m 0755 target/release/annactl /usr/bin/annactl
	@echo "Installed: /usr/bin/annad /usr/bin/annactl"
	@annactl --version
