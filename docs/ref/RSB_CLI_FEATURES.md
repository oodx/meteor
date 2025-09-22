# Meteor CLI Strategy & Implementation Plan

## Overview

This document outlines the meteor CLI ecosystem strategy, including the core meteor CLI for production use and separate specialized CLI tools for learning and experimentation.

**Current Status**: ✅ Native RSB CLI implemented and operational
**Updated Goal**: Design comprehensive CLI command surface for meteor string testing and API experimentation

## Actual RSB Features Available (RSB 0.5.0)

### 1. GLOBAL (String Store + Expansion) 🌍

**What it actually does**: Simple bash-like global store for strings with variable expansion
**Real API** (from FEATURES_GLOBAL.md):
```rust
use rsb::global::{set_var, get_var, has_var, unset_var, expand_vars};

// String store operations
set_var("METEOR_CONTEXT", "app");
let context = get_var("METEOR_CONTEXT"); // returns "app" or ""
let path = expand_vars("$METEOR_CONTEXT/config"); // "app/config"

// Config file operations
load_config_file("$XDG_CONFIG_HOME/meteor.conf");
save_config_file("/path/to/config", &["METEOR_CONTEXT", "VERBOSE"]);
```

**What we can use it for**:
- CLI configuration storage (verbose mode, output format, etc.)
- Variable expansion in file paths and templates
- Session state between commands

### 2. CLI (Args + Bootstrap + Dispatch) 🖥️

**What it actually does**: Bash-style CLI utilities with Args wrapper and dispatch system
**Real API** (from FEATURES_CLI.md):
```rust
use rsb::prelude::*; // includes Args and bootstrap! macro

// Bootstrap with environment setup
let args = bootstrap!(); // gets Args from env

// Args operations (1-indexed, skips argv[0])
let input = args.get(1); // first positional arg
let verbose = args.has("--verbose");
let config = args.has_val("--config"); // extracts from --config=value

// Dispatch system
dispatch!(&args, {
    "parse" => parse_command, desc: "Parse meteor tokens",
    "help"  => help_command
});
```

**What we can use it for**:
- Replace hub::cli_ext::clap with native RSB Args
- Command dispatching with built-in help
- Bootstrap integration with global state

### 3. OPTIONS (Declarative Option Parsing) 📝

**What it actually does**: Declarative options parsing that writes to global context
**Real API** (from FEATURES_OPTIONS.md):
```rust
use rsb::prelude::*;

let args = bootstrap!();
options!(&args); // parses options into global context

// Check parsed options from global context
if has_var("opt_verbose") { /* verbose mode */ }
let format = get_var("opt_format"); // from --format=json

// Supported patterns:
// --verbose → opt_verbose = "true"
// --format=json → opt_format = "json"
// --not-verbose → opt_verbose = "false"
```

**What we can use it for**:
- Replace manual flag parsing with declarative options
- Automatic global context integration
- Standard option patterns (--flag, --key=value)

### 4. STRINGS (String Processing Utilities) 📝

**What it actually does**: String utilities and case conversions
**Real API** (from FEATURES_STRINGS.md):
```rust
use rsb::string::{str_sub, str_replace, str_upper, str_lower};
use rsb::string::case::{to_snake_case, to_kebab_case, to_camel_case};

// String operations
let substr = str_sub("hello world", 0, Some(5)); // "hello"
let replaced = str_replace("test_file", "_", "-", true); // "test-file"

// Case conversions
let snake = to_snake_case("parseTokenStream"); // "parse_token_stream"
let kebab = to_kebab_case("ParseTokenStream"); // "parse-token-stream"
```

**What we can use it for**:
- Token key transformation and normalization
- Output formatting for different CLI contexts
- String processing for user input

### 5. VISUALS (Terminal Output) 🎨

**What it actually does**: Terminal color and formatting (feature-gated)
**Available in**: RSB with visuals feature (compiled in)
**Current status**: Feature flag available, specific APIs may be limited in 0.5.0

**What we can use it for**:
- Colorized CLI output (✓ marks, error messages)
- Status indicators and progress display
- Terminal capability detection

## Implementation Plan: Hub CLI → Native RSB CLI

### Current CLI Structure (hub-based)
```rust
// src/bin/cli.rs (current)
use hub::cli_ext::clap::{Command, Arg};

fn main() {
    let matches = build_cli().get_matches();
    match matches.subcommand() {
        Some(("parse", sub_matches)) => {
            let input = sub_matches.get_one::<String>("input").unwrap();
            let verbose = sub_matches.get_flag("verbose");
            handle_parse(input, verbose, format);
        }
        _ => { /* help */ }
    }
}
```

### Target RSB CLI Structure
```rust
// src/bin/cli.rs (RSB-native)
use rsb::prelude::*;

fn main() {
    let args = bootstrap!();
    options!(&args);

    dispatch!(&args, {
        "parse" => parse_command, desc: "Parse meteor token streams",
        "help"  => help_command
    });
}

fn parse_command(args: Args) -> i32 {
    let input = args.get_or(1, "");
    let verbose = has_var("opt_verbose");
    let format = get_var("opt_format");

    // Use meteor parsing with RSB context
    handle_parse(input, verbose, &format)
}

## Implementation Steps

### Step 1: Create RSB Feature Sanity Tests (First Priority)
**Before** implementing the CLI, we need to validate RSB features work:

```bash
# Create these test files to validate actual RSB APIs:
tests/sanity/rsb_sanity_global.rs   # Test set_var, get_var, expand_vars
tests/sanity/rsb_sanity_cli.rs      # Test Args, bootstrap!, dispatch!
tests/sanity/rsb_sanity_options.rs  # Test options! macro
tests/sanity/rsb_sanity_strings.rs  # Test string utilities
tests/sanity/rsb_sanity_visuals.rs  # Test visual features
```

### Step 2: Implement RSB-Native CLI
Once sanity tests pass, replace `src/bin/cli.rs`:

```rust
// New RSB-native CLI implementation
use rsb::prelude::*;

fn main() {
    let args = bootstrap!();
    options!(&args);

    dispatch!(&args, {
        "parse" => parse_command, desc: "Parse meteor token streams"
    });
}

fn parse_command(args: Args) -> i32 {
    let input = args.get_or(1, "");
    if input.is_empty() {
        eprintln!("Error: No input provided");
        return 1;
    }

    let verbose = has_var("opt_verbose");
    let format = get_var("opt_format");

    // Existing meteor functionality
    match meteor::parse_token_stream(input) {
        Ok(shower) => {
            print_output(&shower, input, verbose, &format);
            0
        }
        Err(e) => {
            eprintln!("Parse error: {}", e);
            1
        }
    }
}
```

### Step 3: Update Cargo.toml Dependencies
```toml
[dependencies]
# Remove hub dependency, use only RSB
rsb = { git = "https://github.com/oodx/rsb.git", features = ["global", "cli", "stdopts", "strings", "visuals"] }
```

### Step 4: Integration Benefits
- **Simpler code**: dispatch! macro handles subcommands automatically
- **Global state**: CLI options stored in RSB global context
- **Better help**: Built-in help, inspect, stack commands
- **RSB compliance**: Native RSB patterns throughout

## Required RSB Features Update

**Current Cargo.toml**: `["visuals", "stdopts"]`
**Needed**: `["global", "cli", "stdopts", "strings", "visuals"]`

**Note**: Need to verify these features exist in RSB 0.5.0 through sanity tests first.

## Validation Strategy

1. **Create RSB feature sanity tests** (TASK-RSB-002 from TASKS.txt)
2. **Validate each RSB feature** works as documented
3. **Implement CLI incrementally** (parse command first)
4. **Migrate functionality** from hub-based to RSB-native
5. **Ensure all existing tests pass**

## CLI Ecosystem Strategy

### Core CLI: `meteor`
**Purpose**: Production-ready tool for parsing and processing meteor strings
**Target Users**: Developers, scripts, automation
**Current Commands**:
- ✅ `meteor parse <input>` - Parse meteor token streams
- ✅ `meteor help` - Built-in RSB help
- ✅ `meteor inspect` - List registered functions
- ✅ `meteor stack` - Show call stack

### Enhanced Commands (Next Phase)
**Complete REPL-like Interface for String Experimentation**:

#### 1. Enhanced Parsing Commands
```bash
# Current (works)
meteor parse "app:ui:button=click"

# Enhanced parsing with analysis
meteor parse --explain "ctx:ns:key[0]=value"    # Show parsing steps
meteor parse --validate "malformed input"       # Validation-only mode
meteor parse --inspect "list[0,1]=matrix"       # Show internal structure
meteor parse --format=json "input"             # JSON output format
meteor parse --format=debug "input"            # Debug output format
```

#### 2. API Surface Exploration
```bash
# Test different meteor APIs
meteor data "key=value;ns:item=data"           # StorageData API demo (via MeteorShower::to_data())
meteor shower "app:ui:x=1;user:cfg:y=2"       # MeteorShower API demo
meteor transform "list[0]=item"               # Show bracket transformation
meteor organize "ns1:a=1;ns2:b=2"            # Show organization patterns
meteor access "ns:key=val" --query "ns.key"  # Test access patterns
```

#### 3. Interactive Exploration
```bash
# REPL-style interface
meteor repl                                   # Start interactive mode
meteor explore "ctx:ns:key=value"              # Guided exploration
meteor playground                             # Sandbox mode for experimentation
```

#### 4. Validation & Debugging
```bash
# Debugging tools
meteor debug "complex:pattern[0,1]=value"      # Debug mode with step-by-step
meteor lint "input patterns"                   # Pattern validation and suggestions
meteor diff "pattern1" "pattern2"             # Compare patterns
meteor stats "token stream"                   # Show parsing statistics
```

#### 5. Interactive REPL Commands
**Inside `meteor repl` interactive shell**:
```
🌟 Meteor Interactive Shell
Type 'help' for commands, 'exit' to quit

meteor> parse "app:ui:button=click"
✅ Parsed successfully
📦 MeteorShower { contexts: ["app"], meteors: 1, indexed: true }
🔍 ui:button = "click"

meteor> explain "list[0,1]=matrix"
🔄 Parse Steps:
  1. Key: "list[0,1]" → Transform to "list__i_0_1"
  2. Value: "matrix"
  3. Namespace: "" (root)
  4. Context: "app" (default)
✅ Result: app::list__i_0_1 = "matrix"

Available REPL commands:
  parse <pattern>     - Parse meteor string
  explain <pattern>   - Show parsing steps
  validate <pattern>  - Validate without parsing
  transform <key>     - Show bracket transformations
  data <pattern>      - Demo StorageData API (MeteorShower::to_data())
  shower <pattern>    - Demo MeteorShower API
  organize <pattern>  - Show organization
  access <pattern>    - Test access patterns
  stats <pattern>     - Show parsing statistics
  clear              - Clear screen
  help               - Show this help
  exit               - Exit shell
```

### Learning CLI: `meteor-learn` (Separate Binary)
**Purpose**: Educational tool for learning meteor concepts
**Target Users**: New users, tutorials, documentation
**Planned Commands**:
```bash
# Educational features (NOT in core meteor CLI)
meteor-learn concepts                         # Core concepts guide
meteor-learn examples                         # Pattern library
meteor-learn demo <topic>                     # Guided tutorials
meteor-learn playground                       # Sandbox experimentation
meteor-learn patterns                         # Common usage patterns
meteor-learn notation                         # Bracket notation guide
```

### Debugging CLI: `meteor-debug` (Future)
**Purpose**: Advanced debugging and analysis
**Target Users**: Power users, developers
**Planned Commands**:
```bash
meteor-debug lint "patterns"                  # Pattern validation
meteor-debug diff "pattern1" "pattern2"       # Compare patterns
meteor-debug stats "token stream"             # Parsing statistics
meteor-debug profile "complex patterns"       # Performance analysis
```

## Implementation Phases

### ✅ Phase 1: RSB Native CLI (COMPLETED)
- Native RSB implementation with bootstrap!, dispatch!, options!
- Core parse command with full feature parity
- RSB 0.6 colors/visuals integration
- Hub dependency optimization (removed cli-ext, kept test-ext)
- Advanced help system framework implemented
- CLI strategy documentation completed with full command specification

### 🎯 Phase 2: Enhanced Command Surface (IN PROGRESS)
**Timeline**: 2-3 hours
**Focus**: REPL-like interface for meteor string experimentation
**Status**: Advanced help system framework added, validate command designed

#### Immediate Next Steps:
1. **Add validate command**: `meteor validate <pattern>` - Check if string is valid meteor format
2. **Add help command**: `meteor help <topic>` - Detailed help for specific commands
3. **Enhanced parse flags**: --explain, --validate, --inspect modes
4. **API exploration commands**: data, shower, transform, access
5. **Interactive REPL**: meteor repl with command history

#### Enhanced Parse Commands
```rust
// Add to dispatch! in src/bin/cli.rs
dispatch!(&args, {
    "parse" => parse_command, desc: "Parse meteor token streams",
    "data" => data_command, desc: "Explore StorageData API via MeteorShower::to_data()",
    "shower" => shower_command, desc: "Explore MeteorShower API",
    "transform" => transform_command, desc: "Show bracket transformations",
    "access" => access_command, desc: "Test access patterns",
    "repl" => repl_command, desc: "Interactive meteor shell"
});
```

#### Complete Command Implementation Strategy
1. **parse_command** - Enhanced with --explain, --validate, --inspect, --format flags
2. **data_command** - Demonstrate StorageData API via MeteorShower::to_data() with examples
3. **shower_command** - Show MeteorShower multi-context handling
4. **transform_command** - Live bracket notation transformation
5. **organize_command** - Show token organization patterns
6. **access_command** - Query pattern testing and validation
7. **explore_command** - Guided exploration of meteor patterns
8. **playground_command** - Sandbox experimentation mode
9. **debug_command** - Step-by-step parsing analysis
10. **lint_command** - Pattern validation and suggestions
11. **diff_command** - Compare two meteor patterns
12. **stats_command** - Parsing statistics and performance
13. **repl_command** - Interactive shell with full command set

#### Flag and Option Specifications
**Global Flags** (work with all commands):
```bash
--format=FORMAT     # Output format: text, json, debug
--verbose, -v       # Verbose output
--color=MODE        # Color mode: auto, always, never
--help, -h          # Show command help
```

**Parse Command Flags**:
```bash
meteor parse [FLAGS] <PATTERN>
  --explain         # Show step-by-step parsing process
  --validate        # Validation-only mode (no output)
  --inspect         # Show internal data structures
  --transform       # Show bracket notation transformations
  --format=FORMAT   # Output format (text, json, debug)
```

**API Command Patterns**:
```bash
meteor data [--demo] <PATTERN>       # StorageData API demonstration via MeteorShower::to_data()
meteor shower [--contexts] <PATTERN> # MeteorShower multi-context demo
meteor access --query=QUERY <PATTERN> # Access pattern testing
meteor debug [--steps] <PATTERN>     # Debug mode with step tracking
meteor stats [--perf] <PATTERN>      # Statistics and performance metrics
```

### 📚 Phase 3: Learning CLI (FUTURE)
**Timeline**: 4-6 hours
**Scope**: Separate `meteor-learn` binary

#### Educational Commands (Moved to meteor-learn)
**Note**: These commands were removed from core meteor CLI to keep it focused
```bash
# Educational features (NOT in core meteor CLI)
meteor-learn concepts                         # Core concepts guide
meteor-learn examples                         # Curated examples library
meteor-learn patterns                         # Common usage patterns
meteor-learn notation                         # Bracket notation guide
meteor-learn contexts                         # Context system explanation
meteor-learn demo <topic>                     # Guided tutorials
```

#### Separate Binary Benefits
- **Focused Core**: Keep meteor CLI lean and production-focused
- **Rich Learning**: Full educational features without bloat
- **Different Audiences**: Production users vs learners
- **Package Size**: Optional learning tools don't affect core CLI

### 🔧 Phase 4: Advanced Tooling (FUTURE)
**Timeline**: 6-8 hours
**Scope**: `meteor-debug`, `meteor-bench`, etc.

## Technical Implementation

### Current RSB Implementation
```rust
// src/bin/cli.rs - Current structure
use rsb::prelude::*;

fn main() {
    let args = bootstrap!();
    options!(&args);

    dispatch!(&args, {
        "parse" => parse_command, desc: "Parse meteor token streams"
    });
}
```

### Enhanced Implementation Plan
```rust
// Enhanced dispatch with new commands
dispatch!(&args, {
    "parse" => parse_command, desc: "Parse meteor token streams",
    "data" => data_command, desc: "Explore StorageData API via MeteorShower::to_data()",
    "shower" => shower_command, desc: "Explore MeteorShower API",
    "transform" => transform_command, desc: "Show transformations",
    "access" => access_command, desc: "Test access patterns",
    "repl" => repl_command, desc: "Interactive shell"
});

// Command implementations follow RSB patterns
fn data_command(args: Args) -> i32 {
    let input = get_input_or_example(&args);

    match meteor::parse_shower(&input) {
        Ok(shower) => {
            let storage_data = shower.to_data();
            demo_storage_api(&storage_data);
            0
        }
        Err(e) => {
            eprintln!("Error: {}", e);
            1
        }
    }
}
```

## Success Criteria

### ✅ Phase 1 (COMPLETED)
- [x] RSB feature sanity tests all pass
- [x] CLI works with RSB-native implementation
- [x] All existing meteor functionality preserved
- [x] Hub dependency optimized (removed cli-ext)
- [x] CLI follows RSB patterns (bootstrap!, dispatch!, options!)
- [x] RSB 0.6 colors/visuals integration

### 🎯 Phase 2 (CURRENT TARGET)
- [ ] Enhanced parse command with --explain, --validate, --inspect
- [ ] API exploration commands (data, shower, transform, access)
- [ ] Interactive REPL mode for experimentation
- [ ] Rich output with colors and structured display
- [ ] All commands follow RSB patterns

### 📚 Future Phases
- [ ] Separate meteor-learn binary for educational features
- [ ] Comprehensive examples and tutorial system
- [ ] Advanced debugging and profiling tools

---

**Current Focus**: Implement Phase 2 enhanced command surface for meteor string experimentation and API exploration.