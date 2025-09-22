# RSB CLI Implementation Plan

## Overview

Replace the current hub::cli_ext-based CLI with a native RSB implementation using actual RSB 0.5.0 features.

**Current State**: CLI uses `hub::cli_ext::clap` (works but not RSB-native)
**Target State**: CLI uses `rsb::prelude::*` with `bootstrap!` + `dispatch!` + `options!` pattern

## Implementation Phases

### Phase 1: RSB Feature Validation ✅ COMPLETED (6 hours)

**Goal**: Validate that required RSB features actually work in 0.5.0

#### Step 1.1: Create RSB Feature Tests ✅ COMPLETED
Created comprehensive sanity test files to validate each RSB feature:

```bash
tests/sanity/rsb_sanity_global.rs    ✅ COMPLETED
tests/sanity/rsb_sanity_cli.rs       ✅ COMPLETED
tests/sanity/rsb_sanity_options.rs   ✅ COMPLETED
tests/sanity/rsb_sanity_strings.rs   ✅ COMPLETED (module accessibility)
tests/sanity/rsb_sanity_visuals.rs   ✅ COMPLETED
```

#### Step 1.2: RSB Features Confirmed Available ✅ COMPLETED
- ✅ **GLOBAL**: set_var, get_var, has_var, unset_var, expand_vars
- ✅ **CLI**: Args type, bash-like patterns (1-indexed, skips argv[0])
- ✅ **OPTIONS**: options! macro, flag parsing, global integration
- ✅ **STRINGS**: rsb::string module accessible
- ✅ **Integration**: 11 total sanity tests passing via `test.sh sanity`

**Validation**: ✅ All RSB feature tests pass, APIs confirmed available and working.

### Phase 2: RSB CLI Core Implementation ✅ COMPLETED (2 hours)

**Goal**: Replace hub-based CLI with RSB-native implementation

#### Step 2.1: Implement RSB Main Function ✅ COMPLETED
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

#### Step 2.2: Implement Parse Command ✅ COMPLETED
✅ Successfully implemented parse_command using RSB Args and global state
✅ Bash-like argument processing with args.get_or(1, "") pattern
✅ RSB global context integration for options (has_var, get_var)
✅ Error handling with proper exit codes

#### Step 2.3: Migrate Output Functions ✅ COMPLETED
✅ Implemented print_output with multiple format support (text, json, debug)
✅ Preserved existing output logic and functionality
✅ RSB-compatible verbose mode handling

#### Step 2.4: RSB Features Successfully Demonstrated ✅ COMPLETED
- **Bootstrap Pattern**: `bootstrap!()` gets Args from environment
- **Dispatch Pattern**: `dispatch!()` with command descriptions
- **Built-in Commands**: help, inspect, stack automatically available
- **Global State**: Ready for options! macro integration
- **Error Handling**: Proper RSB-style error messages and exit codes

**Validation**: ✅ CLI fully operational with RSB patterns

**Working Commands**:
```bash
./target/debug/meteor help                           # ✅ Shows help with built-ins
./target/debug/meteor inspect                        # ✅ Lists registered commands
./target/debug/meteor parse "app:ui:button=click"    # ✅ Parses tokens correctly
```

### Phase 3: Feature Parity & Enhancement (2-4 hours) → **READY TO BEGIN**

**Goal**: Ensure all current CLI functionality is preserved and enhanced

**Current Status**: RSB CLI core working, need to validate options processing

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

- [x] All RSB feature sanity tests pass ✅ COMPLETED
- [x] CLI implements parse command with RSB patterns ✅ COMPLETED
- [x] CLI follows RSB best practices (bootstrap!, dispatch!, options!) ✅ COMPLETED
- [x] Built-in RSB commands work (help, inspect, stack) ✅ COMPLETED
- [ ] All current CLI functionality preserved → **IN PROGRESS (PHASE 3)**
- [ ] No hub dependency required → **PENDING (PHASE 4)**
- [ ] All existing meteor tests continue passing → **PENDING (VALIDATION)**

## Timeline

**Total Estimated Effort**: 13-20 hours
- ✅ Phase 1 (RSB Validation): 6 hours **COMPLETED**
- ✅ Phase 2 (Core Implementation): 2 hours **COMPLETED** (faster than estimated!)
- 🔄 Phase 3 (Feature Parity): 2-4 hours **READY TO BEGIN**
- 🔄 Phase 4 (Cleanup): 1-2 hours

**Remaining Effort**: 3-6 hours (Phases 3-4) - Ahead of schedule!

---

**Current Status**: ✅ Phase 2 COMPLETED - RSB CLI core implementation working
**Next Action**: Begin Phase 3 by validating CLI options processing and feature parity

**Major Achievement**: RSB CLI implementation completed **4-5 hours ahead of schedule!**