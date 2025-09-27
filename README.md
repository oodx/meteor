# Meteor

**Shooting Star Token Data Transport Library**

A Rust library implementing the TOKEN_NAMESPACE_CONCEPT with meteor data format, bracket notation, and RSB architecture compliance. **Meteor** is a DATA TYPE representing structured token data.

## Current Status (2025-10-03)

✅ **Architecture Complete** - Modular type system with extensible bracket notation
✅ **RSB Integration** - Full CLI command suite with global state
✅ **CLI Enhancement Suite** - Query, manipulation, history, and reset commands
✅ **ME-1 Complete (8/8 SP)** - Workspace foundations delivered
  - ✅ ENG-01: EngineWorkspace with ordering, caching, scratch slots
  - ✅ ENG-02: Mutation hooks with atomic cache invalidation
  - ✅ ENG-03: Debug workspace inspection & instrumentation
  - ✅ REGR-03: Namespace depth regression guard across all profiles
🎯 **Production Ready** - 234 tests passing, ME-2 (Iterators) ready to start

## Key Features

### Meteor Data Format
```rust
use meteor::{parse_shower, Meteor, TokenKey, BracketNotation};

// Complete meteor data format: context:namespace:key=value
let shower = parse_shower("app:ui.widgets:button[0]=submit")?;
let meteors = shower.by_context("app");

// Bracket notation with caching + inverse parsing
let key = TokenKey::new("list[0]");
assert_eq!(key.to_string(), "list__i_0");        // Flat (Display)
assert_eq!(key.to_bracket(), "list[0]");         // Original (BracketNotation)
assert_eq!("list__i_0".to_bracket(), "list[0]"); // Inverse parsing
```

### Primary Storage Architecture
- **`MeteorEngine`** - Stateful stream processor with cursor state, command history, and workspace management
- **`MeteorShower`** - Static collection storage with cross-context indexing and object-oriented meteor access
- **`StorageData`** - Hybrid flat+tree storage with O(1) access and hierarchical queries
- **`EngineWorkspace`** - Internal workspace layer for ordering metadata, query caches, and scratch operations

### Extensible Design
- **`BracketNotation`** trait for custom bracket notation
- **MODULE_SPEC** compliant organization
- **RSB** integration ready

## Quick Start

```rust
use meteor::{parse_shower, parse_meteor, TokenKey};

// Primary API: Cross-context parsing (MeteorShower)
let shower = parse_shower("app:ui:button=click; user:settings:theme=dark")?;
let app_meteors = shower.by_context("app");
assert_eq!(shower.get("app", "ui", "button"), Some("click"));

// Single context parsing (Meteor)
let meteor = parse_meteor("app:ui:button=click,theme=dark")?;
assert_eq!(meteor.context().name(), "app");

// Bracket notation
let key = TokenKey::new("grid[2,3]");
assert_eq!(key.to_string(), "grid__i_2_3");
```

## Engine Workspace Features (ME-1)

Meteor includes an internal **EngineWorkspace** layer that supports future iteration and aggregation APIs:

- **Per-Namespace Ordering**: Deterministic key iteration via `key_order` tracking
- **Query Caching**: Cached results for glob/prefix queries with automatic invalidation
- **Scratch Slots**: Isolated temporary storage for REPL multi-step operations
- **Cache Invalidation**: Atomic updates on all mutations (set, delete, reset)
- **Instrumentation** (optional): Cache hit/miss tracking via `workspace-instrumentation` feature flag

```rust
use meteor::types::MeteorEngine;

let mut engine = MeteorEngine::new();
engine.set("app:ui:button", "click").unwrap();
engine.set("app:ui:theme", "dark").unwrap();

// Workspace automatically maintains key ordering for iteration
#[cfg(debug_assertions)]
{
    let status = engine.workspace_status();
    println!("Namespaces: {}", status.namespace_count);
    println!("Ordered keys: {}", status.total_ordered_keys);
}
```

## Configuration

Meteor uses **build-time configuration** via `meteor.toml` for security and deployment flexibility:

```toml
[profile]
active = "default"  # or "enterprise", "embedded", "strict"

[limits.default]
max_meteors_per_shower = 1000
max_command_history = 1000
max_contexts = 100
```

**Deployment Profiles:**
- **Default**: Balanced limits for general use (1k meteors, 128 char keys)
- **Enterprise**: High-performance (10k meteors, 256 char keys, 8k values)
- **Embedded**: Memory-constrained (100 meteors, 32 char keys, 256 values)
- **Strict**: Security-focused (50 meteors, 16 char keys, 128 values)

```bash
# Build with different profiles
METEOR_PROFILE=enterprise cargo build --release
METEOR_PROFILE=strict cargo build --release

# Check current configuration
cargo run --bin meteor-config
```

**Security:** Limits are **compiled into the binary** - no runtime tampering possible.

📖 **[Full Configuration Guide](docs/CONFIGURATION.md)**

## Documentation

- **[Configuration System](docs/CONFIGURATION.md)** - Build-time configuration and deployment profiles
- **[Architecture Status](docs/procs/ARCHITECTURE_STATUS.md)** - Current type system and capabilities
- **[Tasks](docs/procs/TASKS.txt)** - Development progress and roadmap
- **[Token Namespace Concept](docs/ref/TOKEN_NAMESPACE_CONCEPT.md)** - Specification

## Development

```bash
# Build and test
cargo build
cargo test

# Run CLI
cargo run -- --help

# Configuration inspection
cargo run --bin meteor-config
`

### CLI Usage

```bash
# Stream Parsing Commands
meteor parse "app:ui:button=click :;: user:settings:theme=dark"
meteor validate "app:ui:button=click :;: user:settings:theme=dark"
meteor token "profile=name;role=admin"

# Query Commands (one-shot stateless)
meteor get app:ui:button                    # Get value by path
meteor list app ui                          # List keys in namespace
meteor contexts                             # List all contexts
meteor namespaces app                       # List namespaces in context

# Data Manipulation Commands
meteor set app:ui:button click              # Set key-value pair
meteor set --dry-run app:ui:button click    # Preview without executing
meteor delete app:ui:button                 # Delete key by path

# History & Audit Commands
meteor history                              # Show command audit trail
meteor history --limit=10                   # Last 10 commands
meteor history --format=json                # JSON output

# Reset Commands
meteor reset cursor                         # Reset cursor to default (app:main)
meteor reset storage                        # Clear all stored data
meteor reset all                            # Reset cursor and storage
meteor reset app                            # Delete specific context

# All commands support --format=json|text for scripting
meteor get app:ui:button --format=json

# RSB Built-in Commands
meteor inspect                              # List all registered commands
meteor help                                 # Show help
```

### REPL Usage

The REPL provides **interactive, stateful** processing:

```bash
meteor-repl

# Sample session - stateful data manipulation
meteor> parse app:ui:button=click
meteor> set app:ui:theme dark
meteor> get app:ui:button
meteor> list app ui
meteor> contexts
meteor> history
meteor> reset cursor
meteor> dump
meteor> exit
```

**When to use CLI vs REPL:**
- **CLI**: One-shot scripting, automation, pipelines (`meteor get app:ui:button`)
- **REPL**: Interactive exploration, continuous processing, stateful workflows
