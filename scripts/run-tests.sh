#!/bin/bash

# FerrisProof Test Runner Script
# This script runs all types of tests: unit, integration, and property-based tests

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Default configuration
TEST_TYPE="all"
VERBOSE=false
PROPERTY_CASES=1000
PROPERTY_SHRINK=10000
TIMEOUT_MINUTES=30
PARALLEL=true
COVERAGE=false

# Function to print colored output
print_status() {
    echo -e "${BLUE}[INFO]${NC} $1"
}

print_success() {
    echo -e "${GREEN}[SUCCESS]${NC} $1"
}

print_warning() {
    echo -e "${YELLOW}[WARNING]${NC} $1"
}

print_error() {
    echo -e "${RED}[ERROR]${NC} $1"
}

# Function to show usage
show_usage() {
    cat << EOF
Usage: $0 [OPTIONS]

Run FerrisProof test suite with various configurations.

OPTIONS:
    -t, --type TYPE         Test type: unit, integration, property, all (default: all)
    -v, --verbose           Enable verbose output
    -c, --cases NUM         Number of property test cases (default: 1000)
    -s, --shrink NUM        Max shrink iterations for property tests (default: 10000)
    -T, --timeout MIN       Timeout in minutes (default: 30)
    -j, --parallel          Run tests in parallel (default: true)
    -C, --coverage          Generate code coverage report
    -h, --help              Show this help message

EXAMPLES:
    $0                      # Run all tests with default settings
    $0 -t unit              # Run only unit tests
    $0 -t property -c 5000  # Run property tests with 5000 cases
    $0 -C                   # Run all tests with coverage
    $0 -v -t all            # Run all tests with verbose output

ENVIRONMENT VARIABLES:
    PROPTEST_CASES          Override property test case count
    PROPTEST_MAX_SHRINK_ITERS Override max shrink iterations
    PROPTEST_TIMEOUT        Override property test timeout (ms)
    RUST_BACKTRACE          Enable Rust backtraces (0, 1, full)
    CARGO_TERM_COLOR        Control colored output (auto, always, never)
EOF
}

# Parse command line arguments
while [[ $# -gt 0 ]]; do
    case $1 in
        -t|--type)
            TEST_TYPE="$2"
            shift 2
            ;;
        -v|--verbose)
            VERBOSE=true
            shift
            ;;
        -c|--cases)
            PROPERTY_CASES="$2"
            shift 2
            ;;
        -s|--shrink)
            PROPERTY_SHRINK="$2"
            shift 2
            ;;
        -T|--timeout)
            TIMEOUT_MINUTES="$2"
            shift 2
            ;;
        -j|--parallel)
            PARALLEL=true
            shift
            ;;
        -C|--coverage)
            COVERAGE=true
            shift
            ;;
        -h|--help)
            show_usage
            exit 0
            ;;
        *)
            print_error "Unknown option: $1"
            show_usage
            exit 1
            ;;
    esac
done

# Validate test type
case $TEST_TYPE in
    unit|integration|property|all)
        ;;
    *)
        print_error "Invalid test type: $TEST_TYPE"
        print_error "Valid types: unit, integration, property, all"
        exit 1
        ;;
esac

# Set up environment
export CARGO_TERM_COLOR=${CARGO_TERM_COLOR:-always}
export RUST_BACKTRACE=${RUST_BACKTRACE:-1}
export PROPTEST_CASES=${PROPTEST_CASES:-$PROPERTY_CASES}
export PROPTEST_MAX_SHRINK_ITERS=${PROPTEST_MAX_SHRINK_ITERS:-$PROPERTY_SHRINK}
export PROPTEST_TIMEOUT=${PROPTEST_TIMEOUT:-$((TIMEOUT_MINUTES * 60 * 1000))}

# Build verbose flag
VERBOSE_FLAG=""
if [ "$VERBOSE" = true ]; then
    VERBOSE_FLAG="--verbose"
fi

# Build parallel flag
PARALLEL_FLAG=""
if [ "$PARALLEL" = true ]; then
    PARALLEL_FLAG="--"
else
    PARALLEL_FLAG="-- --test-threads=1"
fi

print_status "Starting FerrisProof test suite"
print_status "Test type: $TEST_TYPE"
print_status "Property test cases: $PROPTEST_CASES"
print_status "Max shrink iterations: $PROPTEST_MAX_SHRINK_ITERS"
print_status "Timeout: ${TIMEOUT_MINUTES} minutes"
print_status "Verbose: $VERBOSE"
print_status "Coverage: $COVERAGE"

# Function to run unit tests
run_unit_tests() {
    print_status "Running unit tests..."
    
    if [ "$COVERAGE" = true ]; then
        cargo llvm-cov --lib --all-features --workspace $VERBOSE_FLAG --lcov --output-path lcov-unit.info
    else
        timeout "${TIMEOUT_MINUTES}m" cargo test --lib --all-features --workspace $VERBOSE_FLAG $PARALLEL_FLAG
    fi
    
    if [ $? -eq 0 ]; then
        print_success "Unit tests passed"
    else
        print_error "Unit tests failed"
        return 1
    fi
}

# Function to run integration tests
run_integration_tests() {
    print_status "Running integration tests..."
    
    if [ "$COVERAGE" = true ]; then
        cargo llvm-cov --tests --all-features --workspace $VERBOSE_FLAG --lcov --output-path lcov-integration.info
    else
        timeout "${TIMEOUT_MINUTES}m" cargo test --tests --all-features --workspace $VERBOSE_FLAG $PARALLEL_FLAG
    fi
    
    if [ $? -eq 0 ]; then
        print_success "Integration tests passed"
    else
        print_error "Integration tests failed"
        return 1
    fi
}

# Function to run property tests
run_property_tests() {
    print_status "Running property-based tests..."
    print_status "Configuration: $PROPTEST_CASES cases, $PROPTEST_MAX_SHRINK_ITERS max shrink iterations"
    
    # Run property tests in each crate separately for better reporting
    local failed=false
    
    print_status "Running configuration property tests..."
    if ! timeout "${TIMEOUT_MINUTES}m" cargo test --manifest-path ferris-proof-config/Cargo.toml property_tests $VERBOSE_FLAG $PARALLEL_FLAG; then
        print_error "Configuration property tests failed"
        failed=true
    fi
    
    if ! timeout "${TIMEOUT_MINUTES}m" cargo test --manifest-path ferris-proof-config/Cargo.toml standalone_property_test $VERBOSE_FLAG $PARALLEL_FLAG; then
        print_error "Configuration standalone property tests failed"
        failed=true
    fi
    
    print_status "Running core property tests..."
    if ! timeout "${TIMEOUT_MINUTES}m" cargo test --manifest-path ferris-proof-core/Cargo.toml cache_property_tests $VERBOSE_FLAG $PARALLEL_FLAG; then
        print_error "Core cache property tests failed"
        failed=true
    fi
    
    if ! timeout "${TIMEOUT_MINUTES}m" cargo test --manifest-path ferris-proof-core/Cargo.toml project_structure $VERBOSE_FLAG $PARALLEL_FLAG; then
        print_error "Core project structure property tests failed"
        failed=true
    fi
    
    if ! timeout "${TIMEOUT_MINUTES}m" cargo test --manifest-path ferris-proof-core/Cargo.toml project_setup_test $VERBOSE_FLAG $PARALLEL_FLAG; then
        print_error "Core project setup property tests failed"
        failed=true
    fi
    
    print_status "Running plugin property tests..."
    if ! timeout "${TIMEOUT_MINUTES}m" cargo test --manifest-path ferris-proof-plugins/Cargo.toml network_isolation_tests $VERBOSE_FLAG $PARALLEL_FLAG; then
        print_error "Plugin network isolation property tests failed"
        failed=true
    fi
    
    if [ "$failed" = true ]; then
        print_error "Some property tests failed"
        return 1
    else
        print_success "All property tests passed"
    fi
}

# Function to generate coverage report
generate_coverage_report() {
    if [ "$COVERAGE" = true ]; then
        print_status "Generating combined coverage report..."
        
        # Check if lcov is available
        if command -v lcov >/dev/null 2>&1; then
            lcov --add-tracefile lcov-unit.info --add-tracefile lcov-integration.info --output-file lcov.info
            print_success "Coverage report generated: lcov.info"
        else
            print_warning "lcov not found, skipping coverage merge"
        fi
    fi
}

# Main execution
main() {
    local exit_code=0
    
    # Ensure we're in the project root
    if [ ! -f "Cargo.toml" ]; then
        print_error "Must be run from project root (Cargo.toml not found)"
        exit 1
    fi
    
    # Check if cargo is available
    if ! command -v cargo >/dev/null 2>&1; then
        print_error "cargo not found. Please install Rust and Cargo."
        exit 1
    fi
    
    # Install coverage tools if needed
    if [ "$COVERAGE" = true ]; then
        if ! cargo llvm-cov --version >/dev/null 2>&1; then
            print_status "Installing cargo-llvm-cov..."
            cargo install cargo-llvm-cov
        fi
    fi
    
    # Run tests based on type
    case $TEST_TYPE in
        unit)
            run_unit_tests || exit_code=1
            ;;
        integration)
            run_integration_tests || exit_code=1
            ;;
        property)
            run_property_tests || exit_code=1
            ;;
        all)
            run_unit_tests || exit_code=1
            run_integration_tests || exit_code=1
            run_property_tests || exit_code=1
            ;;
    esac
    
    # Generate coverage report if requested
    generate_coverage_report
    
    # Final status
    if [ $exit_code -eq 0 ]; then
        print_success "All tests completed successfully!"
    else
        print_error "Some tests failed!"
    fi
    
    exit $exit_code
}

# Run main function
main "$@"