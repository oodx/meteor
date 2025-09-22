# Meteor

**Shooting Star Token Data Transport Library**

A Rust library implementing the TOKEN_NAMESPACE_CONCEPT with full addressing, bracket notation, and RSB architecture compliance.

## Current Status (2025-09-22)

✅ **Architecture Complete** - Modular type system with extensible bracket notation
🚧 **RSB Integration** - Ready for global state and enhanced CLI features
🎯 **Next Phase** - RSB feature implementation and advanced functionality

## Key Features

### Full Token Addressing
```rust
use meteor::{parse_shower, Meteor, TokenKey, BracketNotation};

// Complete addressing: context:namespace:key=value
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

## Documentation

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
```
