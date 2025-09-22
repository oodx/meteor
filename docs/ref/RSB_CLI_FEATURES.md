# RSB CLI Implementation Plan for Meteor Admin CLI

## Overview

This document outlines the implementation plan for replacing the current hub::cli_ext-based CLI with native RSB features, based on actual RSB 0.5.0 capabilities documented in docs/ref/features/.

**Current Status**: CLI uses hub::cli_ext::clap (works but not RSB-native)
**Goal**: Replace with native RSB CLI features for proper RSB compliance

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
        Ok(bucket) => {
            print_output(&bucket, input, verbose, &format);
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

## Success Criteria

- [ ] RSB feature sanity tests all pass
- [ ] CLI works with RSB-native implementation
- [ ] All existing meteor functionality preserved
- [ ] No hub dependency needed
- [ ] CLI follows RSB patterns (bootstrap!, dispatch!, options!)

---

**Next Steps**:
1. Start with TASK-RSB-002: Create RSB feature sanity tests
2. Validate actual RSB 0.5.0 API availability
3. Implement RSB-native CLI based on validated features