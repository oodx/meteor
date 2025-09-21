#!/bin/bash
# RSB Sanity Tests Runner
# Wraps cargo tests for clean integration with test.sh

set -e

ROOT_DIR="$(cd "$(dirname "$0")/../.." && pwd)"
cd "$ROOT_DIR"

# Function to run all sanity tests
run_all_sanity() {
    echo "🔍 Running all sanity tests..."
    echo "============================="
    echo

    cargo test --test sanity_meteor
    cargo test --test sanity_types
    cargo test --test sanity_utils
    cargo test --test sanity_sup

    echo
    echo "✅ All sanity tests completed!"
}

# Function to run specific module sanity tests
run_module_sanity() {
    local module="$1"

    echo "🔍 Running sanity tests for module: $module"
    echo "==========================================="
    echo

    case "$module" in
        meteor)
            cargo test --test sanity_meteor
            ;;
        types)
            cargo test --test sanity_types
            ;;
        utils)
            cargo test --test sanity_utils
            ;;
        sup)
            cargo test --test sanity_sup
            ;;
        *)
            echo "❌ Unknown module: $module"
            echo "Available modules: meteor, types, utils, sup"
            exit 1
            ;;
    esac

    echo
    echo "✅ Sanity tests for $module completed!"
}

# Main execution
if [[ $# -eq 0 ]]; then
    run_all_sanity
else
    run_module_sanity "$1"
fi