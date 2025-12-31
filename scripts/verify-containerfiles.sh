#!/bin/bash
set -euo pipefail

# Verify Containerfiles are properly configured for Podman

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

check_containerfiles() {
    log_info "Checking Containerfiles..."
    
    # Check if Containerfiles exist
    if [ ! -f "Containerfile" ]; then
        log_error "Containerfile not found"
        return 1
    fi
    
    if [ ! -f "Containerfile.alpine" ]; then
        log_error "Containerfile.alpine not found"
        return 1
    fi
    
    log_success "Containerfiles found"
    
    # Check for Podman-specific optimizations
    local checks=(
        "docker.io/library/:Explicit registry specification"
        "adduser.*-u.*1001:Specific UID for Podman compatibility"
        "HEALTHCHECK:Health check configuration"
        "org.opencontainers.image:OCI labels"
        "chown.*ferrisproof:Proper ownership"
    )
    
    for check_info in "${checks[@]}"; do
        local pattern="${check_info%%:*}"
        local desc="${check_info##*:}"
        
        if grep -q "$pattern" Containerfile.alpine; then
            log_success "✓ $desc found in Containerfile.alpine"
        else
            log_warning "⚠ $desc not found in Containerfile.alpine"
        fi
    done
}

check_scripts() {
    log_info "Checking script references..."
    
    # Check if scripts reference Containerfiles correctly
    if grep -q "Containerfile" scripts/container-build.sh; then
        log_success "✓ container-build.sh references Containerfiles"
    else
        log_error "✗ container-build.sh doesn't reference Containerfiles"
        return 1
    fi
    
    if grep -q "Containerfile" .gitlab-ci.yml; then
        log_success "✓ GitLab CI references Containerfiles"
    else
        log_error "✗ GitLab CI doesn't reference Containerfiles"
        return 1
    fi
}

test_container_build() {
    log_info "Testing container build (dry run)..."
    
    # Check if Podman is available
    if command -v podman &> /dev/null; then
        log_info "Testing with Podman..."
        if podman build --help > /dev/null 2>&1; then
            log_success "✓ Podman build command available"
        else
            log_warning "⚠ Podman build command failed"
        fi
    elif command -v docker &> /dev/null; then
        log_info "Testing with Docker..."
        if docker build --help > /dev/null 2>&1; then
            log_success "✓ Docker build command available"
        else
            log_warning "⚠ Docker build command failed"
        fi
    else
        log_warning "⚠ Neither Podman nor Docker found"
    fi
}

main() {
    log_info "Verifying Containerfile configuration for Podman..."
    echo
    
    check_containerfiles
    echo
    
    check_scripts
    echo
    
    test_container_build
    echo
    
    log_success "Containerfile verification completed! 🎉"
    echo
    log_info "Next steps:"
    echo "  • Test build: ./scripts/container-build.sh build"
    echo "  • Run CI: ./scripts/ci-local.sh"
    echo "  • Check documentation: docs/ci-pipeline.md"
}

main "$@"