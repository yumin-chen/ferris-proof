#!/bin/bash
set -euo pipefail

# FerrisProof Container Build Script
# Optimized for Podman but works with Docker too

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Configuration
CONTAINER_ENGINE=${CONTAINER_ENGINE:-"podman"}
IMAGE_NAME=${IMAGE_NAME:-"ferris-proof"}
IMAGE_TAG=${IMAGE_TAG:-"latest"}
DOCKERFILE=${DOCKERFILE:-"Containerfile.alpine"}
REGISTRY=${REGISTRY:-""}
PLATFORM=${PLATFORM:-"linux/amd64"}

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

check_container_engine() {
    if ! command -v "$CONTAINER_ENGINE" &> /dev/null; then
        log_error "$CONTAINER_ENGINE not found"
        if [ "$CONTAINER_ENGINE" = "podman" ]; then
            log_info "Install Podman: https://podman.io/getting-started/installation"
            log_info "Falling back to Docker..."
            if command -v docker &> /dev/null; then
                CONTAINER_ENGINE="docker"
                log_info "Using Docker instead"
            else
                log_error "Neither Podman nor Docker found"
                exit 1
            fi
        else
            exit 1
        fi
    fi
    
    log_info "Using container engine: $CONTAINER_ENGINE"
}

build_image() {
    local full_image_name="$IMAGE_NAME:$IMAGE_TAG"
    if [ -n "$REGISTRY" ]; then
        full_image_name="$REGISTRY/$full_image_name"
    fi
    
    log_info "Building image: $full_image_name"
    log_info "Using Containerfile: $DOCKERFILE"
    log_info "Platform: $PLATFORM"
    
    local build_args=()
    
    # Podman-specific optimizations
    if [ "$CONTAINER_ENGINE" = "podman" ]; then
        build_args+=(
            "--layers"
            "--platform=$PLATFORM"
        )
    fi
    
    # Docker-specific optimizations
    if [ "$CONTAINER_ENGINE" = "docker" ]; then
        build_args+=(
            "--platform=$PLATFORM"
        )
    fi
    
    # Build the image
    if $CONTAINER_ENGINE build \
        "${build_args[@]}" \
        -f "$DOCKERFILE" \
        -t "$full_image_name" \
        .; then
        log_success "Image built successfully: $full_image_name"
    else
        log_error "Failed to build image"
        exit 1
    fi
    
    # Show image size
    local image_size
    if [ "$CONTAINER_ENGINE" = "podman" ]; then
        image_size=$($CONTAINER_ENGINE images --format "{{.Size}}" "$full_image_name")
    else
        image_size=$($CONTAINER_ENGINE images --format "table {{.Size}}" "$full_image_name" | tail -n1)
    fi
    log_info "Image size: $image_size"
    
    return 0
}

test_image() {
    local full_image_name="$IMAGE_NAME:$IMAGE_TAG"
    if [ -n "$REGISTRY" ]; then
        full_image_name="$REGISTRY/$full_image_name"
    fi
    
    log_info "Testing image: $full_image_name"
    
    # Test basic functionality
    if $CONTAINER_ENGINE run --rm "$full_image_name" --version; then
        log_success "Image test passed"
    else
        log_error "Image test failed"
        exit 1
    fi
    
    # Test help command
    if $CONTAINER_ENGINE run --rm "$full_image_name" --help > /dev/null; then
        log_success "Help command test passed"
    else
        log_error "Help command test failed"
        exit 1
    fi
}

push_image() {
    if [ -z "$REGISTRY" ]; then
        log_warning "No registry specified, skipping push"
        return 0
    fi
    
    local full_image_name="$REGISTRY/$IMAGE_NAME:$IMAGE_TAG"
    
    log_info "Pushing image: $full_image_name"
    
    if $CONTAINER_ENGINE push "$full_image_name"; then
        log_success "Image pushed successfully"
    else
        log_error "Failed to push image"
        exit 1
    fi
}

show_usage() {
    echo "FerrisProof Container Build Script"
    echo
    echo "Usage: $0 [OPTIONS] [COMMAND]"
    echo
    echo "Commands:"
    echo "  build    Build the container image (default)"
    echo "  test     Test the built image"
    echo "  push     Push image to registry"
    echo "  all      Build, test, and push"
    echo
    echo "Options:"
    echo "  --help, -h           Show this help message"
    echo "  --engine ENGINE      Container engine (podman|docker, default: podman)"
    echo "  --name NAME          Image name (default: ferris-proof)"
    echo "  --tag TAG            Image tag (default: latest)"
    echo "  --dockerfile FILE    Containerfile to use (default: Containerfile.alpine)"
    echo "  --registry REGISTRY  Registry to push to (optional)"
    echo "  --platform PLATFORM  Target platform (default: linux/amd64)"
    echo
    echo "Environment Variables:"
    echo "  CONTAINER_ENGINE     Container engine to use"
    echo "  IMAGE_NAME           Image name"
    echo "  IMAGE_TAG            Image tag"
    echo "  DOCKERFILE           Containerfile path"
    echo "  REGISTRY             Registry URL"
    echo "  PLATFORM             Target platform"
    echo
    echo "Examples:"
    echo "  $0 build"
    echo "  $0 --engine docker --tag v1.0.0 build"
    echo "  $0 --registry ghcr.io/ferris-proof --tag latest all"
}

main() {
    local command="build"
    
    # Parse arguments
    while [[ $# -gt 0 ]]; do
        case $1 in
            --help|-h)
                show_usage
                exit 0
                ;;
            --engine)
                CONTAINER_ENGINE="$2"
                shift 2
                ;;
            --name)
                IMAGE_NAME="$2"
                shift 2
                ;;
            --tag)
                IMAGE_TAG="$2"
                shift 2
                ;;
            --dockerfile)
                DOCKERFILE="$2"
                shift 2
                ;;
            --registry)
                REGISTRY="$2"
                shift 2
                ;;
            --platform)
                PLATFORM="$2"
                shift 2
                ;;
            build|test|push|all)
                command="$1"
                shift
                ;;
            *)
                log_error "Unknown option: $1"
                show_usage
                exit 1
                ;;
        esac
    done
    
    check_container_engine
    
    case $command in
        build)
            build_image
            ;;
        test)
            test_image
            ;;
        push)
            push_image
            ;;
        all)
            build_image
            test_image
            push_image
            ;;
        *)
            log_error "Unknown command: $command"
            show_usage
            exit 1
            ;;
    esac
    
    log_success "Container operations completed successfully! 🎉"
}

main "$@"