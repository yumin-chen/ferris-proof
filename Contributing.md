# Contributing to FerrisProof

Thank you for your interest in contributing to FerrisProof! This document provides guidelines for contributing to the project.

## License

By contributing to FerrisProof, you agree that your contributions will be dedicated to the public domain under the CC0 1.0 Universal license, the same as the project itself.

This means:
- You waive all copyright and related rights to your contributions
- Your contributions become part of the public domain
- No attribution is required (though it's appreciated)
- Anyone can use, modify, and distribute your contributions freely

## Code of Conduct

This project adheres to the Rust Code of Conduct. By participating, you are expected to uphold this code.

## Getting Started

1. Fork the repository
2. Clone your fork: `git clone https://github.com/your-username/ferris-proof.git`
3. Create a feature branch: `git checkout -b feature/your-feature-name`
4. Make your changes
5. Run tests: `cargo test --all-features`
6. Run formatting: `cargo fmt --all`
7. Run clippy: `cargo clippy --all-targets --all-features -- -D warnings`
8. Commit your changes: `git commit -am 'Add some feature'`
9. Push to the branch: `git push origin feature/your-feature-name`
10. Create a Pull Request

## Development Setup

### Prerequisites

- Rust 1.70+ (latest stable recommended)
- Git

### Building

```bash
# Clone the repository
git clone https://github.com/ferris-proof/ferris-proof.git
cd ferris-proof

# Build all crates
cargo build --all-features

# Run tests
cargo test --all-features

# Run property-based tests
cargo test --all-features -- --ignored
```

### Project Structure

```
ferris-proof/
├── ferris-proof-cli/     # Command-line interface
├── ferris-proof-core/    # Core verification engine
├── ferris-proof-config/  # Configuration management
├── ferris-proof-plugins/ # Plugin system and tool integrations
├── docs/                 # Documentation
├── .github/              # GitHub Actions workflows
└── examples/             # Example projects
```

## Contribution Guidelines

### Code Style

- Follow standard Rust formatting (`cargo fmt`)
- Use meaningful variable and function names
- Add documentation comments for public APIs
- Keep functions focused and reasonably sized

### Testing

- Write unit tests for new functionality
- Add property-based tests for core logic
- Ensure all tests pass before submitting PR
- Aim for good test coverage (we track coverage in CI)

### Documentation

- Update documentation for user-facing changes
- Add inline code documentation
- Update CHANGELOG.md for notable changes
- Consider adding examples for new features

### Commit Messages

Use conventional commit format:

```
type(scope): description

[optional body]

[optional footer]
```

Types:
- `feat`: New feature
- `fix`: Bug fix
- `docs`: Documentation changes
- `style`: Code style changes
- `refactor`: Code refactoring
- `test`: Test additions or changes
- `chore`: Maintenance tasks

Examples:
- `feat(cli): add --dry-run flag to upgrade command`
- `fix(config): handle missing configuration files gracefully`
- `docs: update getting started guide`

## Types of Contributions

### Bug Reports

When filing a bug report, please include:

- FerrisProof version
- Operating system and version
- Rust version
- Minimal reproduction case
- Expected vs actual behavior
- Relevant log output

### Feature Requests

For feature requests, please:

- Check existing issues first
- Describe the use case clearly
- Explain why the feature would be valuable
- Consider implementation complexity
- Be open to discussion about alternatives

### Code Contributions

We welcome contributions in these areas:

- **Bug fixes**: Always appreciated
- **New verification techniques**: Expand our verification capabilities
- **Tool integrations**: Add support for new verification tools
- **Performance improvements**: Make verification faster
- **Documentation**: Improve user experience
- **Tests**: Increase coverage and reliability

### Plugin Development

FerrisProof has an extensible plugin system. To add a new verification tool:

1. Implement the `VerificationPlugin` trait
2. Add configuration support
3. Include comprehensive tests
4. Document the integration
5. Update tool compatibility matrix

## Review Process

1. **Automated Checks**: All PRs run through CI (tests, formatting, clippy)
2. **Code Review**: Maintainers will review your code
3. **Testing**: Ensure changes don't break existing functionality
4. **Documentation**: Verify documentation is updated appropriately
5. **Merge**: Once approved, maintainers will merge the PR

## Release Process

FerrisProof follows semantic versioning:

- **Major** (x.0.0): Breaking changes
- **Minor** (0.x.0): New features, backward compatible
- **Patch** (0.0.x): Bug fixes, backward compatible

## Getting Help

- **Questions**: Use GitHub Discussions
- **Issues**: File GitHub Issues for bugs
- **Chat**: Join our community discussions
- **Email**: Contact maintainers for sensitive issues

## Recognition

Contributors are recognized in:

- CHANGELOG.md for notable contributions
- GitHub contributors page
- Release notes for significant features

Thank you for contributing to FerrisProof!