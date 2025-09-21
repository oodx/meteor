#!/bin/bash
# RSB UAT Tests Runner
# Wraps cargo tests for clean integration with test.sh

set -e

ROOT_DIR="$(cd "$(dirname "$0")/../.." && pwd)"
cd "$ROOT_DIR"

# Function to run all UAT tests
run_all_uat() {
    echo "🎭 Running all UAT tests (Visual Ceremony)..."
    echo "=============================================="
    echo

    cargo test --test uat_meteor -- --nocapture

    echo
    echo "✅ All UAT tests completed!"
}

# Function to run specific module UAT tests
run_module_uat() {
    local module="$1"

    echo "🎭 Running UAT tests for module: $module"
    echo "========================================"
    echo

    case "$module" in
        meteor)
            cargo test --test uat_meteor -- --nocapture
            ;;
        *)
            echo "❌ Unknown module: $module"
            echo "Available modules: meteor"
            exit 1
            ;;
    esac

    echo
    echo "✅ UAT tests for $module completed!"
}

# Main execution
if [[ $# -eq 0 ]]; then
    run_all_uat
else
    run_module_uat "$1"
fi