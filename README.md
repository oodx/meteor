# Meteor

**Shooting Star Token Data Transport Library**

A Rust library implementing the TOKEN_NAMESPACE_CONCEPT with meteor data format, bracket notation, and RSB architecture compliance. **Meteor** is a DATA TYPE representing structured token data.

## Current Status (2025-09-22)

✅ **Architecture Complete** - Modular type system with extensible bracket notation
🚧 **RSB Integration** - Ready for global state and enhanced CLI features
🎯 **Next Phase** - RSB feature implementation and advanced functionality

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
- **`MeteorShower`** - Primary storage with cross-context indexing and object-oriented meteor access
- **`StorageData`** - Serialized/flattened interchange format for JSON/string export (from MeteorShower)

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
# Parse into the engine
cargo run --bin meteor -- parse "app:ui:button=click :;: user:settings:theme=dark"

# Alternate output formats
cargo run --bin meteor -- parse --format=json "app:ui:button=click"
cargo run --bin meteor -- parse --format=debug --verbose "app:ui:button=click"

# Validate without storing
cargo run --bin meteor -- validate "app:ui:button=click :;: user:settings:theme=dark"

# Parse token streams
cargo run --bin meteor -- token "profile=name;role=admin"

# Inspect registered handlers (RSB built-in)
cargo run --bin meteor -- inspect
```

### REPL Usage

```bash
cargo run --bin meteor-repl

# Sample session
meteor> parse app:ui:button=click
meteor> get app:ui:button
meteor> mem set  hello world
meteor> mem edit 
meteor> dump
meteor> exit
```
