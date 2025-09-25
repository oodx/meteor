#!/bin/bash
# Meteor Test Entry Point
# RSB-compliant unified interface for running all meteor tests

set -e

# Configuration
ROOT_DIR="$(cd "$(dirname "$0")/.." && pwd)"
TEST_DIR="$ROOT_DIR/tests"

# Meteor binary (when implemented)
METEOR="./target/release/meteor"


# Parse optional flags (can be anywhere in arguments)
TEST_SLEEP=""
NO_SLEEP="false"
QUICK_MODE="true"  # Default to quick mode
COMPREHENSIVE_MODE="false"
BENCHMARK_MODE="false"
SNAP_BENCHMARKS="false"
ARGS=()

while [[ $# -gt 0 ]]; do
    case "$1" in
        --sleep)
            TEST_SLEEP="$2"
            shift 2
            ;;
        --no-sleep)
            NO_SLEEP="true"
            shift 1
            ;;
        --quick)
            QUICK_MODE="true"
            COMPREHENSIVE_MODE="false"
            shift 1
            ;;
        --comprehensive|--full)
            QUICK_MODE="false"
            COMPREHENSIVE_MODE="true"
            shift 1
            ;;
        --benchmark)
            BENCHMARK_MODE="true"
            shift 1
            ;;
        --snap-benchmarks)
            BENCHMARK_MODE="true"
            SNAP_BENCHMARKS="true"
            shift 1
            ;;
        *)
            ARGS+=("$1")
            shift 1
            ;;
    esac
done

# Restore non-flag arguments
set -- "${ARGS[@]}"

# Available tests (RSB-compliant structure)
declare -A TESTS=(
    # Core RSB test categories
    ["sanity"]="sanity.rs"
    ["uat"]="uat.rs"
    ["hybrid"]="test_engine_hybrid.rs"

    # Future tests (when implemented - commented out to prevent failures)
    # ["foundation"]="foundation.rs"     # TODO: Fix quote parsing issue first
    # ["validation"]="validation.rs"
    # ["comprehensive"]="comprehensive/meteor.rs"
    # ["integration"]="integration/meteor.rs"
    # ["performance"]="performance/meteor.rs"

    # Aliases for RSB compliance
    ["smoke"]="sanity.rs"
    ["demo"]="uat.rs"
    # ["all"]="all.sh"
)

show_help() {
    echo "🌠 METEOR TEST RUNNER (RSB-Compliant)"
    echo "====================================="
    echo
    echo "Available Commands:"
    echo "  test.sh [options] sanity              Run core functionality tests"
    echo "  test.sh [options] uat                 Run user acceptance tests with demonstrations"
    echo "  test.sh [options] hybrid              Run hybrid storage regression tests"
    echo "  test.sh list                          List available tests"
    echo "  test.sh help                          Show this help"
    echo "  test.sh docs [topic]                  Show documentation for topic"
    echo ""
    echo "Options:"
    echo "  --comprehensive        Run full validation"
    echo "  --quick                Force quick mode (default)"
    echo "  --sleep N              Add sleep/timeout of N seconds between demo steps"
    echo "  --no-sleep             Disable all sleeps (default behavior)"
    echo "  --benchmark            Run performance benchmarks"
    echo ""
    echo "RSB-Compliant Test Categories:"
    echo "  sanity                 Core functionality validation (no ceremony)"
    echo "  uat                    User acceptance tests with visual demonstrations"
    echo "  validation             Architecture validation (MeteorShower storage)"
    echo "  comprehensive          Complete feature coverage (when implemented)"
    echo ""
    echo "Current Implementation Status:"
    echo "  🔄 Foundation phase - basic test structure being built"
    echo "  📋 Next: Implement core parsing tests in tests/sanity/meteor.rs"
    echo "  🎯 Goal: cargo test && test.sh sanity passes"
}

list_tests() {
    echo "🗂️ METEOR AVAILABLE TESTS"
    echo "========================="
    echo
    for test_name in $(printf "%s\n" "${!TESTS[@]}" | sort); do
        test_file="${TESTS[$test_name]}"
        if [[ -f "$TEST_DIR/$test_file" ]]; then
            echo "✅ $test_name → $test_file"
        else
            echo "❌ $test_name → $test_file (missing - foundation phase)"
        fi
    done
    echo
    echo "🔄 Implementation Status:"
    echo "   Foundation phase - test infrastructure being built"
    echo "   Use 'cargo test' for basic Rust tests"
    echo "   Use 'test.sh docs' for RSB documentation"
}

run_test() {
    local test_name="$1"

    if [[ -z "$test_name" ]]; then
        echo "❌ Error: Test name required"
        echo "Use: test.sh <test>"
        echo "Available tests: ${!TESTS[*]}"
        exit 1
    fi

    if [[ ! "${TESTS[$test_name]+exists}" ]]; then
        echo "❌ Error: Unknown test '$test_name'"
        echo "Available tests: ${!TESTS[*]}"
        exit 1
    fi

    local test_file="${TESTS[$test_name]}"
    local test_path="$TEST_DIR/$test_file"

    echo "🚀 Running Meteor test: $test_name"
    echo "=================================="
    echo

    # Change to project root
    cd "$ROOT_DIR"

    # For Rust tests, use cargo test
    if [[ "$test_file" == *.rs ]]; then
        if [[ ! -f "$test_path" ]]; then
            echo "❌ Test file not found: $test_path"
            echo "🔄 Foundation phase - tests are being implemented"
            echo "📋 Use 'cargo test' for available Rust tests"
            exit 1
        fi

        echo "🦀 Running Rust test: $test_file"
        if [[ "$test_name" == "sanity" ]]; then
            cargo test --test sanity
        elif [[ "$test_name" == "uat" ]]; then
            cargo test --test uat
        elif [[ "$test_name" == "hybrid" ]]; then
            cargo test --test test_engine_hybrid
        else
            cargo test --test "$test_name"
        fi
    else
        # For shell scripts
        if [[ ! -f "$test_path" ]]; then
            echo "❌ Test file not found: $test_path"
            exit 1
        fi
        exec bash "$test_path"
    fi
}

show_docs() {
    local topic="${1:-meteor}"

    echo "📚 METEOR DOCUMENTATION"
    echo "======================="
    echo

    case "$topic" in
        "meteor"|"architecture")
            echo "🌠 Meteor Architecture:"
            echo "  - Token data transport library"
            echo "  - Context-namespace-key addressing"
            echo "  - String-biased API design"
            echo "  - RSB-compliant ordinality organization"
            echo
            echo "📁 Key Files:"
            echo "  - docs/procs/PROCESS.txt     ← Master workflow"
            echo "  - docs/procs/QUICK_REF.txt   ← 30-second context"
            echo "  - .analysis/consolidated_wisdom.txt ← Architectural wisdom"
            echo
            ;;
        "rsb")
            echo "🏗️ RSB Compliance Patterns:"
            echo "  - String-biased interfaces"
            echo "  - Ordinality-based organization"
            echo "  - Unix pipe processing philosophy"
            echo "  - test.sh as unified test entry point"
            echo
            ;;
        "tests")
            echo "🧪 Test Organization:"
            echo "  - tests/sanity.rs    ← Core functionality"
            echo "  - tests/uat.rs       ← User demonstrations"
            echo "  - tests/sanity/      ← Detailed sanity tests"
            echo "  - tests/uat/         ← Detailed UAT tests"
            echo
            ;;
        *)
            echo "Available topics: meteor, rsb, tests, architecture"
            ;;
    esac
}

# Main command dispatch
case "${1:-help}" in
    "sanity"|"uat"|"hybrid"|"smoke"|"demo")
        run_test "$1"
        ;;
    "list")
        list_tests
        ;;
    "docs")
        show_docs "$2"
        ;;
    "help"|"--help"|"-h")
        show_help
        ;;
    *)
        echo "❌ Unknown command: $1"
        echo "Use: test.sh help"
        exit 1
        ;;
esac
