# CI/CD Pipeline Documentation

This document describes the comprehensive CI/CD pipeline for FerrisProof, following a three-tier approach: local development, containerized builds, and vendor-specific platforms.

## Pipeline Architecture


FerrisProof includes a comprehensive CI/CD pipeline that can be run locally, in containers, or on various CI platforms.

### 1. Local Pipeline (`scripts/ci-local.sh`)

The local pipeline is designed to run the complete CI process on developer machines before pushing code.

**Features:**
- Format checking with `cargo fmt`
- Linting with `cargo clippy`
- Debug and release builds
- Unit tests and property-based tests
- Documentation tests
- Security auditing (if `cargo-audit` is available)
- Unused dependency detection (if `cargo-udeps` is available)
- Documentation generation

**Usage:**
```bash
# Run full pipeline
./scripts/ci-local.sh

# Quick checks only
./scripts/ci-local.sh --quick

# Generate coverage report
./scripts/ci-local.sh --coverage

# Show help
./scripts/ci-local.sh --help
```

**Environment Variables:**
- `RUST_VERSION`: Rust version to use (default: stable)
- `PROPTEST_CASES`: Number of property test cases (default: 1000)
- `CARGO_TERM_COLOR`: Terminal color output (default: always)
- `RUST_BACKTRACE`: Rust backtrace level (default: 1)

### 2. Containerized Pipeline

#### Alpine-based Containerfile (`Containerfile.alpine`)

Optimized for minimal size and security:
- Multi-stage build with Rust 1.75 on Alpine Linux
- Static linking for maximum portability
- Non-root user execution with proper UID/GID mapping for Podman
- Health checks included
- Comprehensive OCI labels
- Version-pinned dependencies for reproducible builds

**Image size:** ~15-20MB (compared to ~100MB+ with standard Debian images)

#### Container Build Script (`scripts/container-build.sh`)

Podman-optimized build script with Docker fallback:

**Features:**
- Automatic container engine detection (Podman preferred)
- Multi-platform builds
- Image testing
- Registry push capabilities
- Comprehensive logging

**Usage:**
```bash
# Build image
./scripts/container-build.sh build

# Build and test
./scripts/container-build.sh --tag v1.0.0 all

# Push to registry
./scripts/container-build.sh --registry ghcr.io/ferris-proof push
```

### 3. GitLab CI/CD (`.gitlab-ci.yml`)

Comprehensive GitLab CI pipeline with the following stages:

#### Validate Stage
- **format_check**: Code formatting validation
- **clippy_lint**: Linting with Clippy

#### Test Stage
- **test_debug**: Debug build and unit tests
- **test_property**: Property-based tests (allowed to fail)
- **test_docs**: Documentation tests
- **coverage**: Code coverage reporting

#### Security Stage
- **security_audit**: Dependency security audit
- **dependency_check**: Unused dependency detection

#### Build Stage
- **build_linux_gnu**: Linux GNU target
- **build_linux_musl**: Linux MUSL target (static)
- **build_macos**: macOS target (tags only)

#### Container Stage
- **container_build**: Multi-arch container build with Podman
- **container_test**: Container functionality testing
- **container_security**: Security scanning with Trivy

#### Deploy Stage
- **pages**: Documentation deployment
- **release**: Automated releases for tags

## Development Setup

### Quick Setup

Run the setup script to configure your development environment:

```bash
./scripts/ci-setup.sh
```

This installs:
- Rust components (rustfmt, clippy, llvm-tools-preview)
- Useful Cargo tools (audit, llvm-cov, udeps, etc.)
- Git hooks for pre-commit/pre-push checks
- VS Code settings and tasks
- Makefile for common operations

### Manual Setup

1. **Install Rust components:**
   ```bash
   rustup component add rustfmt clippy llvm-tools-preview
   ```

2. **Install development tools:**
   ```bash
   cargo install cargo-audit cargo-llvm-cov cargo-udeps cargo-nextest
   ```

3. **Set up Git hooks:**
   ```bash
   # The setup script creates these automatically
   chmod +x .git/hooks/pre-commit .git/hooks/pre-push
   ```

## Usage Examples

### Local Development

```bash
# Quick development check
make check

# Run all tests
make test

# Full local CI pipeline
make ci-local

# Generate coverage report
make coverage

# Build container image
make container
```

### Container Operations

```bash
# Build with Podman
./scripts/container-build.sh --engine podman build

# Build multi-platform image
./scripts/container-build.sh --platform linux/amd64,linux/arm64 build

# Build and push to registry
./scripts/container-build.sh --registry ghcr.io/ferris-proof all
```

### GitLab CI

The GitLab CI pipeline runs automatically on:
- Merge requests
- Pushes to main branch
- Git tags
- Manual pipeline triggers

**Key features:**
- Caching for faster builds
- Parallel job execution
- Multi-platform container builds
- Automated releases
- Security scanning
- Documentation deployment

## Performance Optimizations

### Build Caching
- **Local**: Cargo cache in `~/.cargo` and `target/`
- **GitLab**: Project-level cache based on `Cargo.lock`
- **Container**: Multi-stage builds with dependency caching

### Parallel Execution
- GitLab jobs run in parallel where possible
- Multi-platform container builds
- Separate validation, test, and build stages

### Resource Efficiency
- Alpine-based containers for minimal size
- Static linking for portability
- Stripped binaries for smaller artifacts

## Security Considerations

### Container Security
- Non-root user execution with proper UID/GID mapping
- Minimal base image (Alpine)
- Security scanning with Trivy
- Signed container images
- Version-pinned dependencies for reproducible builds

### Dependency Security
- Regular security audits with `cargo-audit`
- Unused dependency detection
- Automated vulnerability scanning

### Access Control
- GitLab CI uses project-specific registry credentials
- Container images pushed to project registry
- Release artifacts signed and checksummed

## Troubleshooting

### Common Issues

1. **Rust toolchain issues:**
   ```bash
   rustup update
   rustup component add rustfmt clippy
   ```

2. **Container build failures:**
   ```bash
   # Check container engine
   podman --version
   # or
   docker --version
   
   # Clean build cache
   podman system prune -a
   ```

3. **GitLab CI failures:**
   - Check runner availability
   - Verify cache configuration
   - Review job logs for specific errors

### Performance Issues

1. **Slow builds:**
   - Ensure caching is working
   - Use `cargo-nextest` for faster testing
   - Consider incremental compilation settings

2. **Large container images:**
   - Use Alpine-based Dockerfile
   - Enable static linking
   - Strip debug symbols

## Monitoring and Metrics

### GitLab CI Metrics
- Build duration tracking
- Test coverage reporting
- Security vulnerability counts
- Container image sizes

### Local Metrics
- Build times with `cargo build --timings`
- Test execution times
- Coverage percentages

## Future Enhancements

### Planned Improvements
- [ ] Benchmark regression testing
- [ ] Automated dependency updates
- [ ] Performance profiling integration
- [ ] Cross-compilation for more targets
- [ ] Integration with external security scanners

### Platform Extensions
- [ ] GitHub Actions workflow
- [ ] Azure DevOps pipeline
- [ ] Jenkins pipeline
- [ ] CircleCI configuration

## Contributing

When contributing to the CI pipeline:

1. Test changes locally first with `./scripts/ci-local.sh`
2. Ensure container builds work with `./scripts/container-build.sh`
3. Verify GitLab CI changes in a merge request
4. Update documentation for any new features
5. Follow the existing patterns and conventions

For questions or issues, please open an issue in the project repository.