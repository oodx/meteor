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

    # Core meteor tests
    cargo test --test sanity_meteor
    cargo test --test sanity_legacy
    cargo test --test sanity_types
    cargo test --test sanity_utils
    cargo test --test sanity_sup

    echo
    echo "🔍 Running hub integration tests..."
    echo "=================================="
    echo

    # Hub integration tests
    cargo test --test sanity_hub_integration
    cargo test --test sanity_hub_lite_performance

    echo
    echo "🔍 Running RSB preparatory sanity tests..."
    echo "=========================================="
    echo

    # RSB feature tests (preparatory)
    cargo test --test sanity_rsb_global
    cargo test --test sanity_rsb_options
    cargo test --test sanity_rsb_fs
    cargo test --test sanity_rsb_strings
    cargo test --test sanity_rsb_host
    cargo test --test sanity_rsb_params
    cargo test --test sanity_rsb_dev
    cargo test --test sanity_rsb_colors
    cargo test --test sanity_rsb_integration

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
        meteor_legacy)
            cargo test --test sanity_legacy
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
        hub_integration)
            cargo test --test sanity_hub_integration
            ;;
        hub_lite_performance)
            cargo test --test sanity_hub_lite_performance
            ;;
        hub)
            echo "🔍 Running all hub integration tests..."
            cargo test --test sanity_hub_integration
            cargo test --test sanity_hub_lite_performance
            ;;
        rsb_global)
            cargo test --test sanity_rsb_global
            ;;
        rsb_options)
            cargo test --test sanity_rsb_options
            ;;
        rsb_fs)
            cargo test --test sanity_rsb_fs
            ;;
        rsb_strings)
            cargo test --test sanity_rsb_strings
            ;;
        rsb_host)
            cargo test --test sanity_rsb_host
            ;;
        rsb_params)
            cargo test --test sanity_rsb_params
            ;;
        rsb_dev)
            cargo test --test sanity_rsb_dev
            ;;
        rsb_colors)
            cargo test --test sanity_rsb_colors
            ;;
        rsb_integration)
            cargo test --test sanity_rsb_integration
            ;;
        rsb)
            echo "🔍 Running all RSB preparatory sanity tests..."
            cargo test --test sanity_rsb_global
            cargo test --test sanity_rsb_options
            cargo test --test sanity_rsb_fs
            cargo test --test sanity_rsb_strings
            cargo test --test sanity_rsb_host
            cargo test --test sanity_rsb_params
            cargo test --test sanity_rsb_dev
            cargo test --test sanity_rsb_colors
            cargo test --test sanity_rsb_integration
            ;;
        *)
            echo "❌ Unknown module: $module"
            echo "Available modules:"
            echo "  Core: meteor, meteor_legacy, types, utils, sup"
            echo "  Hub:  hub_integration, hub_lite_performance"
            echo "  RSB:  rsb_global, rsb_options, rsb_fs, rsb_strings, rsb_host, rsb_params, rsb_dev, rsb_colors, rsb_integration"
            echo "  Groups: hub (all hub tests), rsb (all RSB tests)"
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