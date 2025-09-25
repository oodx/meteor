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

    # Main sanity test file (flattened tests)
    cargo test --test sanity

    echo
    echo "🔍 Running detailed sanity tests..."
    echo "=================================="
    echo

    # Core meteor tests (using actual file names)
    cargo test --test sanity_meteor
    cargo test --test sanity_meteor_legacy
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
    cargo test --test sanity_hub_deps_baseline

    echo
    echo "🔍 Running RSB baseline tests..."
    echo "==============================="
    echo

    # RSB feature tests (actual existing files)
    cargo test --test sanity_rsb_baseline
    cargo test --test sanity_rsb_sanity_cli
    cargo test --test sanity_rsb_sanity_global
    cargo test --test sanity_rsb_sanity_options
    cargo test --test sanity_rsb_sanity_visuals

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
            cargo test --test sanity_meteor_legacy
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
        hub_deps_baseline)
            cargo test --test sanity_hub_deps_baseline
            ;;
        hub)
            echo "🔍 Running all hub tests..."
            cargo test --test sanity_hub_integration
            cargo test --test sanity_hub_lite_performance
            cargo test --test sanity_hub_deps_baseline
            ;;
        rsb_baseline)
            cargo test --test sanity_rsb_baseline
            ;;
        rsb_sanity_cli)
            cargo test --test sanity_rsb_sanity_cli
            ;;
        rsb_sanity_global)
            cargo test --test sanity_rsb_sanity_global
            ;;
        rsb_sanity_options)
            cargo test --test sanity_rsb_sanity_options
            ;;
        rsb_sanity_visuals)
            cargo test --test sanity_rsb_sanity_visuals
            ;;
        rsb)
            echo "🔍 Running all RSB tests..."
            cargo test --test sanity_rsb_baseline
            cargo test --test sanity_rsb_sanity_cli
            cargo test --test sanity_rsb_sanity_global
            cargo test --test sanity_rsb_sanity_options
            cargo test --test sanity_rsb_sanity_visuals
            ;;
        main)
            echo "🔍 Running main sanity test file..."
            cargo test --test sanity
            ;;
        *)
            echo "❌ Unknown module: $module"
            echo "Available modules:"
            echo "  Core: meteor, meteor_legacy, types, utils, sup"
            echo "  Hub:  hub_integration, hub_lite_performance, hub_deps_baseline"
            echo "  RSB:  rsb_baseline, rsb_sanity_cli, rsb_sanity_global, rsb_sanity_options, rsb_sanity_visuals"
            echo "  Groups: hub (all hub tests), rsb (all RSB tests)"
            echo "  Special: main (main sanity.rs flattened test file)"
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
