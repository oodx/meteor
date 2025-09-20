# Meteor Module Plan & Architecture

## Overview

Meteor is the "shooting star" token data transport library - a foundational component for structured key-value data streams with context-aware namespacing and bracket notation extensions.

## Design Philosophy

Following RSB (Rebel String-Biased) principles:
- **String-first interfaces** - Simple, composable operations
- **Unix-pipe philosophy** - Data flows through processing stages
- **Minimal complexity** - Essential functionality only, no feature flags
- **Clear separation** - Data transport vs semantic validation

### RSB Documentation References

This design follows established RSB patterns documented in:

- **RSB Architecture** (`rsb/bin/test.sh docs rsb`) - String-biased philosophy, Unix pipe metaphors
- **Module Specification** (`rsb/bin/test.sh docs spec`) - Module organization, prelude policy, cross-module patterns
- **Prelude Policy** (`rsb/bin/test.sh docs PRELUDE_POLICY`) - What goes in prelude vs explicit imports
- **Test Organization** (`rsb/bin/test.sh docs org`) - Required test structure, naming conventions, ceremony standards
- **Testing HOWTO** (`rsb/bin/test.sh docs howto`) - Test categories, enforcement modes, pattern validation
- **Token Namespace Concept** (`docs/ref/TOKEN_NAMESPACE_CONCEPT.md`) - Complete specification of meteor's addressing scheme

### Key Pattern Sources

**String-Biased Interface Design** (from RSB Architecture):
```rust
// RSB Pattern: Simple string signatures
pub fn read_config(path: &str) -> String;
pub fn process_logs(input: &str, pattern: &str) -> String;
```

**Module Organization** (from MODULE_SPEC + Opinionated Pattern):
```rust
// Meteor Pattern: Directory-based modules, everything through lib.rs
src/lib/
├── lib.rs          # Module orchestrator and re-exports
├── types/          # Core data structures (multiple files)
├── utils/          # Essential helper functions (multiple files)
└── sup/            # Support/internal implementations (multiple files)
```

**Prelude Design** (from PRELUDE_POLICY):
```rust
// RSB Pattern: Essential items only in prelude
pub mod prelude {
    pub use crate::types::{Core, Types, Only};
    pub use crate::utils::{essential_functions_only};
    // Note: Advanced features require explicit import
}
```

**Ordinality-Based Organization** (from RSB Architecture):
```rust
// RSB Pattern: Function ordinality - organize by responsibility hierarchy
// Primary → Secondary → Support (dependency order)
// 1st → 2nd → 3rd → 4th (data flow order)
// Most complex → Least complex → Compatibility (complexity order)
```

## Directory Structure

```
meteor/
├── src/
│   ├── lib.rs              # Main crate entry point & re-exports
│   └── lib/                # Core implementation modules
│       ├── lib.rs          # Module orchestrator & prelude
│       ├── types/          # Core data structures
│       ├── utils/          # Essential helper functions
│       └── sup/            # Support/internal implementations
├── tests/
│   ├── sanity.rs           # Core functionality wrapper
│   ├── uat.rs              # Visual demonstrations wrapper
│   ├── sanity/
│   │   └── meteor.rs       # Core functionality tests
│   └── uat/
│       └── meteor.rs       # Visual demonstration tests
├── docs/
│   └── ref/
│       └── TOKEN_NAMESPACE_CONCEPT.md  # Design reference
└── Cargo.toml
```

## Module Responsibilities

### `/src/lib.rs` - Crate Entry Point
**Purpose**: Main library interface and public API surface
```rust
// Re-export essential types and functions for users
pub use lib::prelude::*;

// Module declaration
pub mod lib;
```

### `/src/lib/lib.rs` - Module Orchestrator
**Purpose**: Central coordinator - all modules load through this single entry point
```rust
// Directory-based modules (each has its own mod.rs)
pub mod types;      // → types/mod.rs orchestrates types/ directory
pub mod utils;      // → utils/mod.rs orchestrates utils/ directory
pub mod sup;        // → sup/mod.rs orchestrates sup/ directory

// Public prelude for essential items only
pub mod prelude {
    pub use super::types::{Context, Namespace, Key, TokenBucket, BucketMode};
    pub use super::utils::{parse_token_stream, create_bucket, parse_token};
    // Note: Advanced operations require explicit import
}

// Re-export prelude at lib level
pub use prelude::*;
```

### `/src/lib/types/` - Core Data Structures
**Purpose**: Essential data types organized by ordinality (responsibility hierarchy)
```rust
// Ordinality-based organization:
types/
├── mod.rs          # Type orchestrator
├── primary.rs      # Core types: Context, Namespace, Key (foundation)
├── bucket.rs       # Secondary: TokenBucket (depends on primary types)
└── error.rs        # Support: Error types (used by all)
```

**Design Patterns**:
- **Primary types** - Foundation that everything builds on
- **Secondary types** - Depend on primary, provide structure
- **Support types** - Used across the hierarchy
- String-biased constructors: `Context::from_str("app")`
- Simple validation: Return `Result<T, MeteorError>`
- Context isolation: Each context gets separate bucket

### `/src/lib/utils/` - Essential Helper Functions
**Purpose**: Data flow operations organized by processing ordinality
```rust
// Data flow ordinality organization:
utils/
├── mod.rs          # Utils orchestrator
├── parse.rs        # 1st: String → Tokens (input processing)
├── transform.rs    # 2nd: Transform tokens (bracket→dunder, validation)
├── organize.rs     # 3rd: Tokens → TokenBucket (structure building)
└── access.rs       # 4th: Query/retrieve from bucket (output operations)
```

**Design Patterns**:
- **Data flow order** - Each stage processes output of previous stage
- **Single responsibility** - Each file handles one transformation step
- **Composable pipeline** - `parse → transform → organize → access`
- String input/output: `parse_token_stream(input: &str) -> Result<TokenBucket, MeteorError>`
- Error conversion: Convert parse errors to `MeteorError`
- No complex algorithms: Push complexity to `sup/` if needed

### `/src/lib/sup/` - Support & Internal Implementation
**Purpose**: Internal complexity organized by implementation ordinality
```rust
// Internal complexity ordinality organization:
sup/
├── mod.rs          # Support orchestrator
├── bracket.rs      # Bracket notation parsing internals (complex algorithms)
├── validation.rs   # Complex validation logic (multi-step validation)
└── compat.rs       # RSB compatibility helpers (migration support)
```

**Design Patterns**:
- **Complexity isolation** - Keep complex algorithms away from public API
- **Internal ordinality** - Most complex → least complex → compatibility
- Not part of public API: `pub(crate)` visibility only
- Performance optimizations: Caching, efficient data structures
- Support functions: Help utils/ and types/ without exposing complexity

## Data Flow Architecture

### Token Stream Processing Pipeline
```
Input String → Parse → Transform → Organize → TokenBucket
     ↓           ↓         ↓          ↓            ↓
"ctx=app;    Context    Bracket    Namespace   Structured
ui:btn[0]=   +Token     →Dunder    Routing     Storage
submit"      Objects    Transform   Logic      Access
```

### Context-Namespace-Key Addressing
```rust
// Full addressing: ctx:namespace:key=value
"app:ui.widgets:button[0]=submit"

// Resolves to:
Context("app") + Namespace("ui.widgets") + Key("button__i_0") = "submit"

// Stored in TokenBucket as:
TokenBucket {
    context: "app",
    data: {
        "ui.widgets" → { "button__i_0" → "submit" }
    }
}
```

## API Design Principles

### 1. String-Biased Interface
```rust
// ✅ Good: Simple string interface
let bucket = meteor::parse_token_stream("ctx=app; ui:button[0]=click")?;
let value = bucket.get("ui", "button__i_0");

// ❌ Avoid: Complex type constructors
let bucket = TokenBucket::new(Context::App, vec![Token::new(...)]);
```

### 2. Composable Operations
```rust
// ✅ Good: Chainable operations (if implementing)
let result = parse_token_stream(input)
    .and_then(|bucket| bucket.get_context("app"))
    .and_then(|ctx| ctx.get_namespace("ui.widgets"));

// ✅ Good: Simple function composition
let bucket = create_bucket_from_stream(input)?;
let widgets = get_namespace_data(&bucket, "ui.widgets")?;
```

### 3. Error Transparency
```rust
// ✅ Good: Clear error types
pub enum MeteorError {
    ParseError(String),
    InvalidContext(String),
    InvalidNamespace(String),
    InvalidKey(String),
}

// ❌ Avoid: Generic errors
pub enum Error { Something(String) }
```

### 4. Consumer Responsibility Model
```rust
// Meteor provides data transport only
let bucket = meteor::parse_token_stream("grid[2,3]=cell")?;

// Consumers handle semantic validation
let grid_manager = GridLayoutManager::new();
grid_manager.validate_bucket(&bucket)?;  // Consumer enforces grid semantics
grid_manager.apply_updates(&bucket)?;     // Consumer applies changes
```

## Testing Strategy

**Following RSB Test Organization** (from `rsb/bin/test.sh docs org`):
- **Sanity tests** - Required for every module, core functionality validation
- **UAT tests** - Required for every module, visual demonstrations
- **Naming convention** - `sanity.rs`, `uat.rs` wrapper files
- **Directory structure** - `tests/sanity/meteor.rs`, `tests/uat/meteor.rs`
- **Function naming** - `sanity_meteor_basic()`, `uat_meteor_demo()`

### Sanity Tests (`tests/sanity/meteor.rs`)
**Purpose**: Core functionality validation - no visual ceremony
```rust
#[test]
fn test_basic_token_parsing() {
    let bucket = parse_token_stream("key=value").unwrap();
    assert_eq!(bucket.get("global", "key"), Some("value"));
}

#[test]
fn test_bracket_notation() {
    let bucket = parse_token_stream("list[0]=item").unwrap();
    assert_eq!(bucket.get("global", "list__i_0"), Some("item"));
}

#[test]
fn test_context_isolation() {
    let app_bucket = parse_token_stream("ctx=app; data=secret").unwrap();
    let user_bucket = parse_token_stream("ctx=user; data=public").unwrap();
    // Verify contexts don't cross-contaminate
}
```

### UAT Tests (`tests/uat/meteor.rs`)
**Purpose**: Visual demonstrations of capabilities
```rust
#[test]
fn demonstrate_token_stream_processing() {
    println!("🌠 Meteor Token Stream Demo");

    let input = "ctx=app; ui.widgets:button[0]=Submit; ui.widgets:list[]=item1";
    let bucket = parse_token_stream(input).unwrap();

    println!("Input: {}", input);
    println!("Parsed context: {}", bucket.context);
    println!("Namespaces: {:?}", bucket.get_namespaces());
    println!("Bracket transformations:");
    for (key, value) in bucket.get_namespace("ui.widgets").unwrap() {
        println!("  {} = {}", key, value);
    }
}
```

## Migration Strategy from RSB

### Phase 1: Structure Setup
1. Create basic module structure following this plan
2. Set up Cargo.toml with minimal dependencies
3. Create empty modules with proper visibility

### Phase 2: Core Types Migration
1. Copy Token, Namespace, TokenBucket from RSB
2. Add Context type for origin tracking
3. Implement bracket notation parsing in Key

### Phase 3: Utilities & Integration
1. Copy parsing utilities from RSB token module
2. Add bracket transformation logic
3. Create simplified API surface

### Phase 4: Testing & Validation
1. Create sanity tests for all core functionality
2. Add UAT tests demonstrating capabilities
3. Validate against TOKEN_NAMESPACE_CONCEPT.md requirements

## External Dependencies

**Minimal approach** - avoid heavy dependencies:
```toml
[dependencies]
# Only if absolutely necessary for parsing
# regex = "1.0"  # For complex bracket parsing if needed
```

**Standard library preferred**:
- `std::collections::HashMap` for storage
- `std::str` for string parsing
- `std::fmt` for display implementations
- `std::error` for error traits

## Performance Considerations

### String Operations
- Use `&str` for parsing, `String` for storage
- Avoid unnecessary allocations during parsing
- Consider `Cow<str>` for optional optimizations

### Memory Layout
- TokenBucket uses nested HashMap - acceptable for flexibility
- Consider arena allocation if profiling shows issues
- Keep Context small (single String)

### Parsing Efficiency
- Single-pass parsing where possible
- Early validation to fail fast
- Avoid regex unless complexity demands it

## Future Extensions (Not in MVP)

### Advanced Features (Later)
- Stream processing traits for large inputs
- Serialization support (serde feature)
- Advanced bracket operations (`key[++]`, `key[?]`)
- Cross-context relationship support (backlog)

### Integration Points
- Layout engine consumer integration
- Boxy output formatting support
- Configuration file parsing utilities

---

This plan provides a focused, extensible foundation for meteor while following RSB principles and supporting the layout engine use case. The structure separates concerns clearly while maintaining simplicity and composability.