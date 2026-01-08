# PWE Karaoke - Development Makefile
# Convenient commands for common development tasks

.PHONY: help build run release clean fmt lint check test setup-python install-python all installer installer-deps

# Default target
help:
	@echo "PWE Karaoke - Available commands:"
	@echo ""
	@echo "  make setup-python    - Create Python virtual environment"
	@echo "  make install-python  - Install Python dependencies (Spleeter)"
	@echo "  make build          - Build the project in debug mode"
	@echo "  make run            - Build and run the project"
	@echo "  make release        - Build optimized release version"
	@echo "  make installer      - Build installers for distribution"
	@echo "  make installer-deps - Install dependencies for building installers"
	@echo "  make clean          - Clean build artifacts"
	@echo "  make fmt            - Format code with rustfmt"
	@echo "  make lint           - Run clippy lints"
	@echo "  make check          - Run fmt + lint + build checks"
	@echo "  make test           - Run tests"
	@echo "  make all            - Format, lint, and build"
	@echo ""

# Setup Python virtual environment
setup-python:
	@echo "Creating Python virtual environment..."
	python3 -m venv venv
	@echo "Virtual environment created. Activate with: source venv/bin/activate"

# Install Python dependencies
install-python:
	@echo "Installing Python dependencies..."
	@if [ ! -d "venv" ]; then \
		echo "Virtual environment not found. Run 'make setup-python' first."; \
		exit 1; \
	fi
	./venv/bin/pip install --upgrade pip
	./venv/bin/pip install -r requirements.txt
	@echo "Python dependencies installed successfully!"

# Build in debug mode
build:
	cargo build

# Build and run
run:
	cargo run

# Build release version
release:
	cargo build --release

# Clean build artifacts
clean:
	cargo clean
	rm -rf target/

# Format code
fmt:
	@echo "Formatting code with rustfmt..."
	cargo fmt --all

# Run clippy lints
lint:
	@echo "Running clippy lints..."
	cargo clippy --all-targets --all-features -- -D warnings

# Lint with suggestions (doesn't fail on warnings)
lint-suggestions:
	@echo "Running clippy with suggestions..."
	cargo clippy --all-targets --all-features

# Check formatting without modifying files
fmt-check:
	@echo "Checking code formatting..."
	cargo fmt --all -- --check

# Run all checks (format, lint, build)
check: fmt-check lint
	@echo "Running cargo check..."
	cargo check --all-targets --all-features

# Run tests
test:
	cargo test

# Watch mode for development (requires cargo-watch)
watch:
	cargo watch -x 'run'

# Install development tools
install-tools:
	@echo "Installing development tools..."
	cargo install cargo-watch
	cargo install cargo-edit
	@echo "Development tools installed!"

# Format, lint, and build
all: fmt lint build
	@echo "All checks passed and project built successfully!"

# Pre-commit checks (useful for git hooks)
pre-commit: fmt lint test
	@echo "Pre-commit checks passed!"

# Install dependencies for building installers
installer-deps:
	@echo "Checking for OpenSSL development libraries..."
	@if ! pkg-config --exists openssl; then \
		echo "Error: OpenSSL development libraries not found."; \
		echo ""; \
		echo "Please install them:"; \
		echo "  Ubuntu/Debian: sudo apt install -y libssl-dev pkg-config"; \
		echo "  Fedora/RHEL:   sudo dnf install -y openssl-devel pkg-config"; \
		echo "  macOS:         brew install openssl pkg-config"; \
		echo ""; \
		exit 1; \
	fi
	@echo "OpenSSL found. Installing cargo-bundle..."
	cargo install cargo-bundle
	@echo "cargo-bundle installed! You may need to install additional system dependencies."
	@echo "See installer/README.md for platform-specific requirements."

# Build installers for distribution
installer: release
	@echo "Building installers..."
	@bash -c './installer/build-installers.sh'
	@echo "Installers created in target/release/bundle/"
