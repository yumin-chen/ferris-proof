#!/bin/bash
set -euo pipefail

# FerrisProof CI Setup Script
# Installs necessary tools for local development and CI

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

log_info() {
    echo -e "${BLUE}[INFO]${NC} $1"
}

log_success() {
    echo -e "${GREEN}[SUCCESS]${NC} $1"
}

log_warning() {
    echo -e "${YELLOW}[WARNING]${NC} $1"
}

log_error() {
    echo -e "${RED}[ERROR]${NC} $1"
}

install_rust_components() {
    log_info "Installing Rust components..."
    
    # Essential components
    rustup component add rustfmt clippy
    
    # Optional but recommended components
    if rustup component add llvm-tools-preview 2>/dev/null; then
        log_success "llvm-tools-preview installed"
    else
        log_warning "llvm-tools-preview not available for this toolchain"
    fi
    
    log_success "Rust components installed"
}

install_cargo_tools() {
    log_info "Installing useful Cargo tools..."
    
    local tools=(
        "cargo-audit:Security auditing"
        "cargo-llvm-cov:Code coverage"
        "cargo-udeps:Unused dependencies detection"
        "cargo-watch:File watching for development"
        "cargo-nextest:Faster test runner"
        "cargo-deny:Dependency management"
    )
    
    for tool_info in "${tools[@]}"; do
        local tool_name="${tool_info%%:*}"
        local tool_desc="${tool_info##*:}"
        
        if cargo install --list | grep -q "^$tool_name "; then
            log_info "$tool_name already installed"
        else
            log_info "Installing $tool_name ($tool_desc)..."
            if cargo install "$tool_name"; then
                log_success "$tool_name installed"
            else
                log_warning "Failed to install $tool_name"
            fi
        fi
    done
}

setup_git_hooks() {
    log_info "Setting up Git hooks..."
    
    # Create hooks directory if it doesn't exist
    mkdir -p .git/hooks
    
    # Pre-commit hook
    cat > .git/hooks/pre-commit << 'EOF'
#!/bin/bash
# FerrisProof pre-commit hook

set -e

echo "Running pre-commit checks..."

# Quick format and lint check
if ! cargo fmt --all -- --check; then
    echo "❌ Code formatting issues found. Run 'cargo fmt' to fix."
    exit 1
fi

if ! cargo clippy --all-targets --all-features -- -D warnings; then
    echo "❌ Clippy warnings found. Please fix them."
    exit 1
fi

# Quick build check
if ! cargo check --all-features; then
    echo "❌ Build check failed."
    exit 1
fi

echo "✅ Pre-commit checks passed!"
EOF

    # Pre-push hook
    cat > .git/hooks/pre-push << 'EOF'
#!/bin/bash
# FerrisProof pre-push hook

set -e

echo "Running pre-push checks..."

# Run the quick CI pipeline
if [ -f "scripts/ci-local.sh" ]; then
    ./scripts/ci-local.sh --quick
else
    echo "⚠️  Local CI script not found, running basic checks..."
    cargo test --all-features
fi

echo "✅ Pre-push checks passed!"
EOF

    # Make hooks executable
    chmod +x .git/hooks/pre-commit .git/hooks/pre-push
    
    log_success "Git hooks installed"
}

setup_vscode_settings() {
    log_info "Setting up VS Code settings..."
    
    mkdir -p .vscode
    
    # VS Code settings
    cat > .vscode/settings.json << 'EOF'
{
    "rust-analyzer.cargo.features": "all",
    "rust-analyzer.checkOnSave.command": "clippy",
    "rust-analyzer.checkOnSave.allFeatures": true,
    "rust-analyzer.cargo.buildScripts.enable": true,
    "editor.formatOnSave": true,
    "editor.codeActionsOnSave": {
        "source.fixAll": true
    },
    "[rust]": {
        "editor.defaultFormatter": "rust-lang.rust-analyzer",
        "editor.formatOnSave": true
    },
    "files.watcherExclude": {
        "**/target/**": true
    },
    "search.exclude": {
        "**/target": true,
        "**/.git": true
    }
}
EOF

    # VS Code tasks
    cat > .vscode/tasks.json << 'EOF'
{
    "version": "2.0.0",
    "tasks": [
        {
            "label": "cargo check",
            "type": "cargo",
            "command": "check",
            "args": ["--all-features"],
            "group": "build",
            "presentation": {
                "clear": true
            }
        },
        {
            "label": "cargo test",
            "type": "cargo",
            "command": "test",
            "args": ["--all-features"],
            "group": "test",
            "presentation": {
                "clear": true
            }
        },
        {
            "label": "cargo clippy",
            "type": "cargo",
            "command": "clippy",
            "args": ["--all-targets", "--all-features", "--", "-D", "warnings"],
            "group": "build",
            "presentation": {
                "clear": true
            }
        },
        {
            "label": "Local CI",
            "type": "shell",
            "command": "./scripts/ci-local.sh",
            "group": "test",
            "presentation": {
                "clear": true
            }
        }
    ]
}
EOF

    log_success "VS Code settings configured"
}

create_makefile() {
    log_info "Creating Makefile for common tasks..."
    
    cat > Makefile << 'EOF'
# FerrisProof Makefile
# Common development tasks

.PHONY: help check test build clean fmt lint audit coverage doc ci-local container-build

# Default target
help:
	@echo "FerrisProof Development Tasks"
	@echo ""
	@echo "Available targets:"
	@echo "  check        - Quick check (format, lint, build)"
	@echo "  test         - Run all tests"
	@echo "  build        - Build in release mode"
	@echo "  clean        - Clean build artifacts"
	@echo "  fmt          - Format code"
	@echo "  lint         - Run clippy"
	@echo "  audit        - Security audit"
	@echo "  coverage     - Generate coverage report"
	@echo "  doc          - Generate documentation"
	@echo "  ci-local     - Run local CI pipeline"
	@echo "  container    - Build container image"

# Quick development checks
check:
	cargo fmt --all -- --check
	cargo clippy --all-targets --all-features -- -D warnings
	cargo build --all-features

# Run tests
test:
	cargo test --all-features
	cargo test --all-features -- --ignored

# Build release
build:
	cargo build --release --all-features

# Clean artifacts
clean:
	cargo clean

# Format code
fmt:
	cargo fmt --all

# Lint code
lint:
	cargo clippy --all-targets --all-features -- -D warnings

# Security audit
audit:
	cargo audit

# Generate coverage report
coverage:
	cargo llvm-cov --all-features --workspace --html

# Generate documentation
doc:
	cargo doc --all-features --no-deps --open

# Run local CI
ci-local:
	./scripts/ci-local.sh

# Build container
container:
	./scripts/container-build.sh build

# Install development dependencies
setup:
	./scripts/ci-setup.sh
EOF

    log_success "Makefile created"
}

main() {
    log_info "Setting up FerrisProof development environment..."
    
    # Check if Rust is installed
    if ! command -v cargo &> /dev/null; then
        log_error "Rust/Cargo not found. Please install Rust first: https://rustup.rs/"
        exit 1
    fi
    
    install_rust_components
    install_cargo_tools
    setup_git_hooks
    setup_vscode_settings
    create_makefile
    
    log_success "Development environment setup complete! 🎉"
    echo
    log_info "Next steps:"
    echo "  • Run 'make check' to verify everything works"
    echo "  • Run 'make ci-local' to run the full CI pipeline"
    echo "  • Run 'make help' to see all available commands"
    echo "  • Check .vscode/settings.json for VS Code configuration"
}

case "${1:-}" in
    --help|-h)
        echo "FerrisProof CI Setup Script"
        echo
        echo "Usage: $0 [OPTIONS]"
        echo
        echo "This script sets up the development environment with:"
        echo "  • Rust components (rustfmt, clippy, etc.)"
        echo "  • Useful Cargo tools"
        echo "  • Git hooks for pre-commit/pre-push checks"
        echo "  • VS Code settings and tasks"
        echo "  • Makefile for common tasks"
        echo
        echo "Options:"
        echo "  --help, -h    Show this help message"
        exit 0
        ;;
    "")
        main
        ;;
    *)
        log_error "Unknown option: $1"
        echo "Use --help for usage information"
        exit 1
        ;;
esac