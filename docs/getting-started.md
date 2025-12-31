# Getting Started with FerrisProof

This guide will help you get started with FerrisProof, a multi-layer correctness pipeline for Rust applications.

## Installation

### From Crates.io (Recommended)

```bash
cargo install ferris-proof
```

### From Source

```bash
git clone https://github.com/ferris-proof/ferris-proof.git
cd ferris-proof
cargo install --path ferris-proof-cli
```

### Docker

```bash
docker pull ghcr.io/ferris-proof/ferris-proof:latest
docker run --rm -v $(pwd):/workspace ghcr.io/ferris-proof/ferris-proof:latest --help
```

## Your First Project

### 1. Initialize FerrisProof

Navigate to your Rust project and initialize FerrisProof:

```bash
cd my-rust-project
ferris-proof init --level standard
```

This creates a `ferrisproof.toml` configuration file with standard verification settings.

### 2. Run Initial Verification

```bash
ferris-proof check
```

FerrisProof will analyze your code and report any verification issues.

### 3. View Configuration

```bash
ferris-proof config
```

This shows the effective configuration for your project.

## Understanding Verification Levels

FerrisProof supports four progressive verification levels:

### Minimal Level
- **Focus**: Type safety only
- **Good for**: Legacy projects, initial adoption
- **Techniques**: Basic Rust type checking

### Standard Level (Recommended)
- **Focus**: Type safety + property testing
- **Good for**: Most projects, balanced approach
- **Techniques**: Type checking + property-based tests

### Strict Level
- **Focus**: Advanced type-level verification
- **Good for**: Critical systems, high-reliability requirements
- **Techniques**: + Session types, refinement types, concurrency testing

### Formal Level
- **Focus**: Mathematical correctness proofs
- **Good for**: Safety-critical systems, distributed protocols
- **Techniques**: + TLA+/Alloy specifications, model checking

## Next Steps

1. **Explore Configuration**: Learn about [configuration options](configuration.md)
2. **Upgrade Verification**: Try `ferris-proof upgrade --to strict`
3. **Generate Artifacts**: Use `ferris-proof generate --target property-tests`
4. **CI Integration**: Add FerrisProof to your GitHub Actions workflow

## Common Commands

```bash
# Show help
ferris-proof --help

# Initialize project
ferris-proof init --level standard --interactive

# Run verification
ferris-proof check

# Check specific module
ferris-proof check --module src/core

# Upgrade verification level
ferris-proof upgrade --to strict --dry-run

# Generate verification artifacts
ferris-proof generate --target property-tests

# Explain error codes
ferris-proof explain FP-VR-001

# Show configuration
ferris-proof config --validate
```

## Getting Help

- **Documentation**: [docs.rs/ferris-proof](https://docs.rs/ferris-proof)
- **Issues**: [GitHub Issues](https://github.com/ferris-proof/ferris-proof/issues)
- **Discussions**: [GitHub Discussions](https://github.com/ferris-proof/ferris-proof/discussions)
- **Security**: See [SECURITY.md](../SECURITY.md) for security issues