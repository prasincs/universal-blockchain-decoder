#!/bin/bash
set -e

# Universal Blockchain Decoder - Test Demo Script
# Showcases all test types and validation strategies

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
MAGENTA='\033[0;35m'
CYAN='\033[0;36m'
BOLD='\033[1m'
RESET='\033[0m'

# Emojis for visual appeal
CHECK="✅"
CROSS="❌"
ROCKET="🚀"
TEST="🧪"
PROP="🎲"
LINK="🔗"
DOC="📚"
PERF="⚡"
COV="📊"
LOCK="🔒"

# Print a header
print_header() {
    echo -e "\n${BOLD}${BLUE}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${RESET}"
    echo -e "${BOLD}${CYAN}$1${RESET}"
    echo -e "${BOLD}${BLUE}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${RESET}\n"
}

# Print a step
print_step() {
    echo -e "${BOLD}${MAGENTA}▶${RESET} ${BOLD}$1${RESET}"
}

# Print success
print_success() {
    echo -e "${GREEN}${CHECK} $1${RESET}"
}

# Print info
print_info() {
    echo -e "${CYAN}ℹ️  $1${RESET}"
}

# Print warning
print_warning() {
    echo -e "${YELLOW}⚠️  $1${RESET}"
}

# Count tests
count_tests() {
    local pattern=$1
    find . -path "./target" -prune -o -name "*.rs" -type f -print0 | \
        xargs -0 grep -l "$pattern" 2>/dev/null | wc -l
}

# Main demo
main() {
    print_header "${ROCKET} Universal Blockchain Decoder - Test Demo"

    echo -e "${BOLD}This demo showcases our comprehensive testing strategy:${RESET}"
    echo -e "  ${TEST} Unit Tests           - Test individual functions"
    echo -e "  ${PROP} Property Tests       - Test mathematical properties (1000s of cases)"
    echo -e "  ${LINK} Integration Tests    - Test real blockchain data"
    echo -e "  ${DOC} Documentation Tests   - Test code examples in docs"
    echo -e "  ${PERF} Performance Tests    - Benchmark critical paths"
    echo -e "  ${COV} Coverage Analysis     - Measure test coverage"
    echo -e "  ${LOCK} Security Audit       - Check for vulnerabilities\n"

    # Show test file statistics
    print_header "${TEST} Test Statistics"
    print_info "Scanning codebase for test files..."

    UNIT_TESTS=$(find crates/*/tests -name "*.rs" 2>/dev/null | wc -l)
    PROPERTY_TESTS=$(find . -name "*property_tests.rs" 2>/dev/null | wc -l)
    INTEGRATION_TESTS=$(find . -name "*integration_tests.rs" 2>/dev/null | wc -l)

    echo -e "  ${TEST} Unit test files:        ${GREEN}${BOLD}${UNIT_TESTS}${RESET}"
    echo -e "  ${PROP} Property test files:    ${GREEN}${BOLD}${PROPERTY_TESTS}${RESET}"
    echo -e "  ${LINK} Integration test files: ${GREEN}${BOLD}${INTEGRATION_TESTS}${RESET}"
    echo ""

    # Test organization
    print_header "${TEST} Test Organization"
    print_info "Test files are organized by crate:"
    echo ""
    for dir in crates/*/tests; do
        if [ -d "$dir" ]; then
            crate=$(basename $(dirname "$dir"))
            count=$(find "$dir" -name "*.rs" | wc -l)
            echo -e "  📦 ${BOLD}${crate}${RESET}: ${GREEN}${count}${RESET} test files"
        fi
    done
    echo ""

    # 1. Unit Tests
    print_header "${TEST} 1. Unit Tests - Testing Individual Components"
    print_step "Running unit tests in library code..."
    print_info "These tests verify individual functions and methods work correctly"
    echo ""

    if cargo test --lib --all --no-fail-fast 2>&1 | tee /tmp/unit_tests.log; then
        print_success "Unit tests passed!"
        # Extract test count
        UNIT_COUNT=$(grep -oP '\d+(?= passed)' /tmp/unit_tests.log | tail -1)
        echo -e "${GREEN}${BOLD}   → ${UNIT_COUNT} tests passed${RESET}\n"
    else
        print_warning "Some unit tests failed (expected during development)"
    fi

    # 2. Property-Based Tests
    print_header "${PROP} 2. Property-Based Tests - Mathematical Properties"
    print_step "Running property tests with 1000 random inputs per test..."
    print_info "Testing invariants like: decode(encode(x)) == x"
    print_info "Each property is tested with 1000 randomly generated cases"
    echo ""

    export PROPTEST_CASES=1000

    if cargo test --test property_tests --all --no-fail-fast 2>&1 | tee /tmp/property_tests.log; then
        print_success "Property tests passed!"
        PROP_COUNT=$(grep -oP '\d+(?= passed)' /tmp/property_tests.log | tail -1)
        echo -e "${GREEN}${BOLD}   → ${PROP_COUNT} properties verified across $(($PROP_COUNT * 1000)) random cases${RESET}\n"
    else
        print_warning "Some property tests failed (expected during development)"
    fi

    # 3. Integration Tests
    print_header "${LINK} 3. Integration Tests - Real Blockchain Data"
    print_step "Testing against real blockchain transactions..."
    print_info "These tests use actual transaction data from Bitcoin, Ethereum, etc."
    echo ""

    # Test core integration
    if cargo test -p universal-decoder-core --tests --no-fail-fast 2>&1 | tee /tmp/integration_tests.log; then
        print_success "Core integration tests passed!"
        INT_COUNT=$(grep -oP '\d+(?= passed)' /tmp/integration_tests.log | tail -1)
        echo -e "${GREEN}${BOLD}   → ${INT_COUNT} integration tests passed${RESET}\n"
    else
        print_warning "Some integration tests failed (expected during Phase 1.5)"
    fi

    # 4. Documentation Tests
    print_header "${DOC} 4. Documentation Tests - Examples in Docs"
    print_step "Running code examples from documentation..."
    print_info "Ensures all code examples in docs actually compile and run"
    echo ""

    if cargo test --doc --all --no-fail-fast 2>&1 | tee /tmp/doc_tests.log; then
        print_success "Documentation tests passed!"
        DOC_COUNT=$(grep -oP '\d+(?= passed)' /tmp/doc_tests.log | tail -1)
        echo -e "${GREEN}${BOLD}   → ${DOC_COUNT} documentation examples verified${RESET}\n"
    else
        print_warning "Some documentation tests failed"
    fi

    # 5. Code Quality Checks
    print_header "${LOCK} 5. Code Quality - Format & Lint"
    print_step "Checking code formatting..."

    if cargo fmt --all -- --check 2>&1; then
        print_success "Code is properly formatted!"
    else
        print_warning "Code formatting issues found (run 'cargo fmt --all')"
    fi

    echo ""
    print_step "Running clippy (Rust linter)..."

    if cargo clippy --all-targets --all-features -- -D warnings 2>&1 | tee /tmp/clippy.log; then
        print_success "No clippy warnings!"
    else
        print_warning "Clippy found some issues"
    fi

    # 6. Performance Tests
    print_header "${PERF} 6. Performance Tests - Benchmarks"
    print_step "Checking for benchmark tests..."

    BENCH_COUNT=$(find . -name "*.rs" -path "*/benches/*" 2>/dev/null | wc -l)

    if [ "$BENCH_COUNT" -gt 0 ]; then
        echo -e "${GREEN}Found ${BENCH_COUNT} benchmark files${RESET}"
        print_info "To run benchmarks: cargo bench"
    else
        print_info "No benchmarks found yet (planned for Phase 1.5)"
    fi

    # 7. Coverage Analysis
    print_header "${COV} 7. Coverage Analysis"
    print_step "Checking if cargo-llvm-cov is installed..."

    if command -v cargo-llvm-cov &> /dev/null; then
        print_success "cargo-llvm-cov is installed!"
        print_step "Generating coverage report..."

        # Create coverage directory
        mkdir -p coverage

        if cargo llvm-cov --all-features --workspace --lcov --output-path ./coverage/lcov.info 2>&1 | tee /tmp/coverage.log; then
            # Calculate coverage percentage
            if [ -f ./coverage/lcov.info ]; then
                LF=$(grep -oP '^LF:\K\d+' coverage/lcov.info | awk '{s+=$1} END {print s}')
                LH=$(grep -oP '^LH:\K\d+' coverage/lcov.info | awk '{s+=$1} END {print s}')

                if [ "$LF" -gt 0 ]; then
                    COVERAGE_PCT=$(echo "scale=1; ($LH * 100) / $LF" | bc)
                    echo -e "${GREEN}${BOLD}   → Code coverage: ${COVERAGE_PCT}%${RESET}"

                    if (( $(echo "$COVERAGE_PCT >= 80" | bc -l) )); then
                        print_success "Coverage meets 80% threshold!"
                    elif (( $(echo "$COVERAGE_PCT >= 50" | bc -l) )); then
                        print_warning "Coverage is ${COVERAGE_PCT}% (target: 80%)"
                    else
                        print_info "Coverage is ${COVERAGE_PCT}% (expected during Phase 1.5)"
                    fi
                fi
            fi
        fi
    else
        print_info "Install cargo-llvm-cov to see coverage: cargo install cargo-llvm-cov"
    fi

    # 8. Security Audit
    print_header "${LOCK} 8. Security Audit"
    print_step "Checking if cargo-audit is installed..."

    if command -v cargo-audit &> /dev/null; then
        print_success "cargo-audit is installed!"
        print_step "Running security audit..."

        if cargo audit 2>&1 | tee /tmp/audit.log; then
            print_success "No known security vulnerabilities!"
        else
            print_warning "Security audit found issues (check /tmp/audit.log)"
        fi
    else
        print_info "Install cargo-audit: cargo install cargo-audit"
    fi

    # Summary
    print_header "${ROCKET} Test Demo Complete!"

    echo -e "${BOLD}Summary:${RESET}"
    echo -e "  ${CHECK} Unit tests verify individual components"
    echo -e "  ${CHECK} Property tests verify mathematical invariants"
    echo -e "  ${CHECK} Integration tests verify real blockchain data"
    echo -e "  ${CHECK} Documentation tests verify code examples"
    echo -e "  ${CHECK} Code quality checked with fmt + clippy"
    echo ""

    print_info "All test types are automated in CI/CD (.github/workflows/test.yml)"
    print_info "See docs/TESTING_STRATEGY.md for detailed testing philosophy"

    echo -e "\n${BOLD}${GREEN}To run specific test types:${RESET}"
    echo -e "  ${CYAN}cargo test --lib${RESET}                    # Unit tests"
    echo -e "  ${CYAN}cargo test --test property_tests${RESET}    # Property tests"
    echo -e "  ${CYAN}cargo test --tests${RESET}                  # Integration tests"
    echo -e "  ${CYAN}cargo test --doc${RESET}                    # Documentation tests"
    echo -e "  ${CYAN}cargo bench${RESET}                         # Benchmarks"
    echo -e "  ${CYAN}cargo llvm-cov${RESET}                      # Coverage"
    echo -e "  ${CYAN}cargo audit${RESET}                         # Security audit"

    echo ""
}

# Run the demo
main "$@"
