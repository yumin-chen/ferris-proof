#!/bin/bash
# Local CI test script to verify the pipeline will work
# This script mimics the GitLab CI pipeline steps

set -e

echo "🔧 Setting up Rust toolchain..."
rustup show

echo "📝 Running format check..."
cargo fmt --all -- --check

echo "🔍 Running clippy lints..."
cargo clippy --all-targets --all-features -- -D warnings

echo "🏗️ Building project..."
cargo build --all-features --workspace

echo "🧪 Running unit tests..."
cargo test --lib --all-features --workspace --verbose

echo "🔬 Running integration tests..."
cargo test --tests --all-features --workspace --verbose

echo "📚 Running doc tests..."
cargo test --doc --all-features

echo "✅ All CI checks passed locally!"