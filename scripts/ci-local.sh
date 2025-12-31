#!/bin/bash
set -euo pipefail

# FerrisProof Local CI Pipeline
# This script runs the complete CI pipeline locally before pushing to remote

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Configuration
RUST_VERSION=${RUST_VERSION:-"stable"}
PROPTEST_CASES=${PROPTEST_CASES:-"1000"}
CARGO_TERM_COLOR=${CARGO_TERM_COLOR:-"always"}
RUST_BACKTRACE=${RUST_BACKTRACE:-"1"}

# Export environment variables
export CARGO_TERM_COLOR RUST_BACKTRACE PROPTEST_CASES

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

run_step() {
    local step_name="$1"
    local step_command="$2"
    
    log_info "Running: $step_name"
    if eval "$step_command"; then
        log_success "$step_name completed"
    else
        log_error "$step_name failed"
        exit 1
    fi
    echo
}

check_prerequisites() {
    log_info "Checking prerequisites..."
    
    # Check if Rust is installed
    if ! command -v cargo &> /dev/null; then
        log_error "Cargo not found. Please install Rust: https://rustup.rs/"
        exit 1
    fi
    
    # Check Rust version
    local rust_version=$(rustc --version | cut -d' ' -f2)
    log_info "Using Rust version: $rust_version"
    
    # Check if required components are installed
    if ! rustup component list --installed | grep -q "rustfmt"; then
        log_warning "rustfmt not installed, installing..."
        rustup component add rustfmt
    fi
    
    if ! rustup component list --installed | grep -q "clippy"; then
        log_warning "clippy not installed, installing..."
        rustup component add clippy
    fi
    
    log_success "Prerequisites check completed"
    echo
}

main() {
    local start_time=$(date +%s)
    
    log_info "Starting FerrisProof Local CI Pipeline"
    log_info "Working directory: $(pwd)"
    echo
    
    # Check prerequisites
    check_prerequisites
    
    # Format check
    run_step "Format Check" "cargo fmt --all -- --check"
    
    # Clippy linting
    run_step "Clippy Linting" "cargo clippy --all-targets --all-features -- -D warnings"
    
    # Build (debug)
    run_step "Debug Build" "cargo build --all-features"
    
    # Build (release)
    run_step "Release Build" "cargo build --release --all-features"
    
    # Unit tests
    run_step "Unit Tests" "cargo test --all-features"
    
    # Property-based tests
    run_step "Property-based Tests" "cargo test --all-features -- --ignored"
    
    # Documentation tests
    run_step "Documentation Tests" "cargo test --doc --all-features"
    
    # Security audit (if cargo-audit is available)
    if command -v cargo-audit &> /dev/null; then
        run_step "Security Audit" "cargo audit"
    else
        log_warning "cargo-audit not found, skipping security audit"
        log_info "Install with: cargo install cargo-audit"
    fi
    
    # Check for unused dependencies (if cargo-udeps is available)
    if command -v cargo-udeps &> /dev/null; then
        run_step "Unused Dependencies Check" "cargo +nightly udeps --all-targets"
    else
        log_warning "cargo-udeps not found, skipping unused dependencies check"
        log_info "Install with: cargo install cargo-udeps"
    fi
    
    # Generate documentation
    run_step "Documentation Generation" "cargo doc --all-features --no-deps"
    
    local end_time=$(date +%s)
    local duration=$((end_time - start_time))
    
    echo
    log_success "All CI checks passed! 🎉"
    log_info "Total time: ${duration}s"
    
    # Optional: Run benchmarks if available
    if [ -d "benches" ] || grep -q "\[\[bench\]\]" Cargo.toml; then
        echo
        log_info "Benchmarks available. Run with: cargo bench"
    fi
    
    # Optional: Coverage report if cargo-llvm-cov is available
    if command -v cargo-llvm-cov &> /dev/null; then
        echo
        log_info "Generate coverage report with: cargo llvm-cov --all-features --workspace --html"
    fi
}

# Handle script arguments
case "${1:-}" in
    --help|-h)
        echo "FerrisProof Local CI Pipeline"
        echo
        echo "Usage: $0 [OPTIONS]"
        echo
        echo "Options:"
        echo "  --help, -h     Show this help message"
        echo "  --quick, -q    Run quick checks only (format, clippy, build)"
        echo "  --coverage, -c Generate coverage report (requires cargo-llvm-cov)"
        echo
        echo "Environment Variables:"
        echo "  RUST_VERSION     Rust version to use (default: stable)"
        echo "  PROPTEST_CASES   Number of property test cases (default: 1000)"
        echo "  CARGO_TERM_COLOR Terminal color output (default: always)"
        echo "  RUST_BACKTRACE   Rust backtrace level (default: 1)"
        exit 0
        ;;
    --quick|-q)
        log_info "Running quick checks only"
        check_prerequisites
        run_step "Format Check" "cargo fmt --all -- --check"
        run_step "Clippy Linting" "cargo clippy --all-targets --all-features -- -D warnings"
        run_step "Build Check" "cargo build --all-features"
        log_success "Quick checks completed! 🚀"
        exit 0
        ;;
    --coverage|-c)
        if ! command -v cargo-llvm-cov &> /dev/null; then
            log_error "cargo-llvm-cov not found. Install with: cargo install cargo-llvm-cov"
            exit 1
        fi
        log_info "Generating coverage report..."
        cargo llvm-cov --all-features --workspace --html
        log_success "Coverage report generated in target/llvm-cov/html/"
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