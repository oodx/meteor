# RSB CLI Implementation Plan

## Overview

Replace the current hub::cli_ext-based CLI with a native RSB implementation using actual RSB 0.5.0 features.

**Current State**: CLI uses `hub::cli_ext::clap` (works but not RSB-native)
**Target State**: CLI uses `rsb::prelude::*` with `bootstrap!` + `dispatch!` + `options!` pattern

## Implementation Phases

### Phase 1: RSB Feature Validation (6-8 hours)

**Goal**: Validate that required RSB features actually work in 0.5.0

#### Step 1.1: Create RSB Feature Tests
Create individual sanity test files to validate each RSB feature:

```bash
tests/sanity/rsb_sanity_global.rs    # 1-1.5 hours
tests/sanity/rsb_sanity_cli.rs       # 1-1.5 hours
tests/sanity/rsb_sanity_options.rs   # 1-1.5 hours
tests/sanity/rsb_sanity_strings.rs   # 1-1.5 hours
tests/sanity/rsb_sanity_visuals.rs   # 1-1.5 hours
```

#### Step 1.2: Update Cargo.toml Features
```toml
[dependencies]
rsb = { git = "https://github.com/oodx/rsb.git", features = ["global", "cli", "stdopts", "strings", "visuals"] }
```

**Validation**: All RSB feature tests pass, proving APIs are available.

### Phase 2: RSB CLI Core Implementation (4-6 hours)

**Goal**: Replace hub-based CLI with RSB-native implementation

#### Step 2.1: Implement RSB Main Function
```rust
// src/bin/cli.rs (new RSB version)
use rsb::prelude::*;

fn main() {
    let args = bootstrap!();
    options!(&args);

    dispatch!(&args, {
        "parse" => parse_command, desc: "Parse meteor token streams"
    });
}
```

#### Step 2.2: Implement Parse Command
```rust
fn parse_command(args: Args) -> i32 {
    // Get input from positional args
    let input = args.get_or(1, "");
    if input.is_empty() {
        eprintln!("Error: No input provided");
        return 1;
    }

    // Get options from RSB global context
    let verbose = has_var("opt_verbose");
    let format = get_var("opt_format");
    let format = if format.is_empty() { "text" } else { &format };

    // Use existing meteor parsing
    match meteor::parse_token_stream(input) {
        Ok(bucket) => {
            print_output(&bucket, input, verbose, format);
            0
        }
        Err(e) => {
            eprintln!("Parse error: {}", e);
            1
        }
    }
}
```

#### Step 2.3: Migrate Output Functions
Adapt existing output functions to work with RSB global state:

```rust
fn print_output(bucket: &meteor::TokenBucket, input: &str, verbose: bool, format: &str) {
    match format {
        "json" => print_json_output(bucket, input, verbose),
        "debug" => print_debug_output(bucket, input),
        _ => print_text_output(bucket, input, verbose),
    }
}
```

**Validation**: CLI works with basic parse command using RSB patterns.

### Phase 3: Feature Parity & Enhancement (2-4 hours)

**Goal**: Ensure all current CLI functionality is preserved and enhanced

#### Step 3.1: Command Line Options Support
Ensure RSB options! macro handles all current CLI options:

```bash
# Current supported options:
meteor parse --verbose --format=json "app:ui:button=click"
meteor parse -v -f debug "ctx:ns:key=value"

# RSB should automatically parse these into:
# opt_verbose = "true"
# opt_format = "json"/"debug"
```

#### Step 3.2: Help and Built-in Commands
RSB dispatch! provides built-in commands:

```bash
meteor help      # Built-in help command
meteor inspect   # Shows registered commands
meteor stack     # Shows call stack for debugging
```

#### Step 3.3: Error Handling & User Experience
- Improve error messages using RSB patterns
- Add command suggestions for unknown commands
- Validate input before processing

**Validation**: All current CLI functionality works + new RSB features.

### Phase 4: Dependencies & Cleanup (1-2 hours)

**Goal**: Remove hub dependency and clean up codebase

#### Step 4.1: Remove Hub Dependency
```toml
[dependencies]
# Remove this line:
# hub = { git = "https://github.com/oodx/hub.git", features = ["core", "cli-ext", "async-ext", "error-ext"] }

# Keep only:
rsb = { git = "https://github.com/oodx/rsb.git", features = ["global", "cli", "stdopts", "strings", "visuals"] }
```

#### Step 4.2: Update Imports
Remove all `use hub::` imports from codebase.

#### Step 4.3: Test All Functionality
- Run all existing tests (should still pass)
- Test CLI manually with various inputs
- Verify no regressions

**Validation**: No hub dependency, all tests pass, CLI fully functional.

## Technical Implementation Details

### RSB Feature Usage

#### GLOBAL (State Management)
```rust
// Store CLI configuration
set_var("METEOR_LAST_CONTEXT", "app");
set_var("METEOR_OUTPUT_FORMAT", "json");

// Retrieve configuration
let context = get_var("METEOR_LAST_CONTEXT");
let verbose = has_var("opt_verbose");

// Variable expansion
let config_path = expand_vars("$HOME/.meteor/config");
```

#### CLI (Args & Dispatch)
```rust
// Bootstrap gets Args from environment
let args = bootstrap!();

// Dispatch handles subcommands automatically
dispatch!(&args, {
    "parse" => parse_command, desc: "Parse token streams",
    "config" => config_command, desc: "Manage configuration"
});

// Args provides bash-like access (1-indexed)
let input = args.get(1);           // First positional arg
let flag = args.has("--verbose");  // Boolean flag
let value = args.has_val("--format"); // Extract value from --format=value
```

#### OPTIONS (Declarative Parsing)
```rust
// Parse options into global context
options!(&args);

// Options automatically become opt_* variables:
// --verbose       → opt_verbose = "true"
// --format=json   → opt_format = "json"
// --not-verbose   → opt_verbose = "false"
```

#### STRINGS (Text Processing)
```rust
// String utilities for token processing
let normalized_key = to_snake_case("buttonClickHandler"); // "button_click_handler"
let cleaned = str_replace(input, " ", "", true); // Remove all spaces
```

#### VISUALS (Terminal Output)
```rust
// Enhanced output with terminal formatting
// (Specific APIs TBD based on RSB 0.5.0 capabilities)
```

### Migration Strategy

#### Before (Hub-based)
```rust
use hub::cli_ext::clap::{Command, Arg, ArgAction};

fn main() {
    let matches = build_cli().get_matches();
    match matches.subcommand() {
        Some(("parse", sub_matches)) => {
            let input = sub_matches.get_one::<String>("input").unwrap();
            let verbose = sub_matches.get_flag("verbose");
            let format = sub_matches.get_one::<String>("format").map(|s| s.as_str()).unwrap_or("text");
            handle_parse(input, verbose, format);
        }
        _ => {
            build_cli().print_help().unwrap();
            process::exit(1);
        }
    }
}
```

#### After (RSB-native)
```rust
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
    let verbose = has_var("opt_verbose");
    let format = get_var("opt_format");

    handle_parse(input, verbose, &format)
}
```

**Benefits**:
- Simpler, more declarative code
- Built-in help, inspect, stack commands
- Global state management
- Native RSB compliance
- No external CLI framework dependency

## Risk Mitigation

### Risk 1: RSB Features Not Available in 0.5.0
**Mitigation**: Phase 1 validates all required features first
**Fallback**: Keep hub dependency if RSB features insufficient

### Risk 2: Breaking Changes to Existing CLI
**Mitigation**: Thorough testing, gradual migration
**Fallback**: Maintain compatibility with current CLI interface

### Risk 3: Performance Regression
**Mitigation**: Benchmark before/after implementation
**Fallback**: Profile and optimize RSB usage patterns

## Success Criteria

- [ ] All RSB feature sanity tests pass
- [ ] CLI implements parse command with RSB patterns
- [ ] All current CLI functionality preserved
- [ ] No hub dependency required
- [ ] All existing meteor tests continue passing
- [ ] CLI follows RSB best practices (bootstrap!, dispatch!, options!)
- [ ] Built-in RSB commands work (help, inspect, stack)

## Timeline

**Total Estimated Effort**: 13-20 hours
- Phase 1 (RSB Validation): 6-8 hours
- Phase 2 (Core Implementation): 4-6 hours
- Phase 3 (Feature Parity): 2-4 hours
- Phase 4 (Cleanup): 1-2 hours

**Dependencies**: Must complete Phase 1 before proceeding to implementation phases.

---

**Next Action**: Begin Phase 1 by implementing TASK-RSB-002 (RSB feature sanity tests)