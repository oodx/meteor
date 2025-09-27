# TICKET-013 Breakdown Analysis

## Current State

### REPL Already Has (Interactive Mode)
The REPL (`src/bin/repl.rs`) already implements most desired functionality:
- ✅ `set <path> <value>` - Direct data manipulation
- ✅ `get <path>` - Query stored data
- ✅ `delete <path>` - Delete data
- ✅ `contexts` - List all contexts
- ✅ `namespaces [context]` - List namespaces
- ✅ `list [path]` - List keys/values
- ✅ `dump` / `show` - Display engine state
- ✅ `parse <stream>` - Parse and process meteor streams
- ✅ `validate <stream>` - Validate format
- ✅ `token <stream>` - Parse token streams

### CLI Currently Has (One-Shot Mode)
The CLI (`src/bin/cli.rs`) has limited commands:
- ✅ `meteor parse <stream>` - Parse meteor streams
- ✅ `meteor validate <stream>` - Validate format
- ✅ `meteor token <stream>` - Parse token streams
- ❌ No query commands (get, list, contexts, namespaces)
- ❌ No manipulation commands (set, delete)
- ❌ No history/audit commands
- ❌ No reset commands
- ❌ No interactive/continuous mode

### Engine Capabilities
From `src/lib/types/meteor/engine.rs`:
- ✅ `set(path, value)` - Set key-value
- ✅ `get(path)` - Get value by path
- ✅ `delete(path)` - Delete key
- ✅ `contexts()` - List contexts
- ✅ `namespaces_in_context(context)` - List namespaces
- ✅ `command_history: Vec<ControlCommand>` - Audit trail (private)
- ✅ `execute_control_command(cmd_type, target)` - Control commands
- ❌ No public accessor for `command_history` (needs `pub fn history()`)

## Proposed Breakdown

### TICKET-013A: CLI Query Commands (P1) - 1-2 hours ⭐ **START HERE**
**Goal**: Port read-only REPL commands to CLI for scripting/automation

**New CLI Commands**:
```bash
meteor get <path>              # Get value by path
meteor list [path]             # List keys/values (optionally filtered)
meteor contexts                # List all contexts
meteor namespaces [context]    # List namespaces in context
```

**Implementation**:
- Port logic from `handle_get`, `handle_list`, `handle_contexts`, `handle_namespaces` in repl.rs
- Adapt for one-shot CLI execution (no interactive loop)
- Add proper RSB dispatch entries
- Support `--format=json|text|yaml` for output

**Value**: Enables scripting and automation with Meteor data
**Dependencies**: None - engine methods already exist
**Risk**: Low - read-only operations

---

### TICKET-013B: CLI Data Manipulation Commands (P2) - 1-2 hours
**Goal**: Port write operations to CLI for automation

**New CLI Commands**:
```bash
meteor set <path> <value>      # Set key-value pair
meteor delete <path>           # Delete key by path
```

**Implementation**:
- Port logic from `handle_set`, `handle_delete` in repl.rs
- Adapt for one-shot CLI execution
- Add proper RSB dispatch entries
- Support `--dry-run` flag for safety

**Value**: Enables automated data manipulation
**Dependencies**: TICKET-013A (for consistency)
**Risk**: Medium - write operations, need good error handling

---

### TICKET-013C: CLI History/Audit Commands (P2) - 1 hour
**Goal**: Surface command audit trail for debugging/monitoring

**New CLI Command**:
```bash
meteor history                 # Show command audit trail
meteor history --json          # JSON format for tooling
meteor history --limit=10      # Last N commands
```

**Engine Changes**:
```rust
// Add to engine.rs
pub fn history(&self) -> &[ControlCommand] {
    &self.command_history
}

pub fn last_command(&self) -> Option<&ControlCommand> {
    self.command_history.last()
}
```

**Implementation**:
- Add public accessor methods to MeteorEngine
- Create CLI handler to format and display history
- Support multiple output formats

**Value**: Debugging and audit trail visibility
**Dependencies**: Engine method addition (5 minutes)
**Risk**: Low - read-only

---

### TICKET-013D: CLI Reset Commands (P2) - 30 minutes
**Goal**: Add reset functionality to CLI

**New CLI Commands**:
```bash
meteor reset cursor            # Reset cursor to default (app:main)
meteor reset [context]         # Clear context data
```

**Implementation**:
- Port logic from engine's `execute_control_command`
- Add proper CLI handlers
- Add confirmation prompts for data-destructive operations

**Value**: Cleanup and state management
**Dependencies**: None
**Risk**: Medium - data destructive

---

### TICKET-013E: Stream Processing Mode (P1) - 2-3 hours ⚠️ **COMPLEX**
**Goal**: Add continuous/interactive stream processing mode

**New CLI Command**:
```bash
meteor process-stream          # Enter continuous processing mode
meteor process-stream < file   # Process from file
echo "stream" | meteor process-stream  # Process from stdin
```

**Two Possible Approaches**:

**Option 1: Stateful Session Mode** (More complex)
- CLI maintains persistent MeteorEngine
- Reads lines from stdin continuously
- Processes each line as a meteor stream
- Outputs results after each line
- Maintains state across lines

**Option 2: Just Use REPL** (Simpler)
- User runs `meteor-repl` for interactive mode
- `process-stream` could be alias to repl
- Or: `meteor repl` launches interactive shell

**Implementation Considerations**:
- State persistence across inputs
- Output buffering/flushing
- Signal handling (Ctrl+C, EOF)
- Error recovery (continue on error vs fail fast)

**Value**: Continuous processing workflows
**Dependencies**: TICKET-013A/B for consistency
**Risk**: High - fundamentally different mode, state management complexity

**Recommendation**: Consider if REPL already solves this. If so, mark as "use REPL instead".

---

## Recommended Execution Order

1. **TICKET-013A** (Query Commands) - Immediate value, low risk ⭐
2. **TICKET-013C** (History Commands) - Quick win, debugging value
3. **TICKET-013B** (Data Manipulation) - Builds on query pattern
4. **TICKET-013D** (Reset Commands) - Simple, completes CRUD operations
5. **TICKET-013E** (Stream Processing) - Evaluate if needed vs REPL

## Effort Estimate Summary

Original: "3-4 hours" as one ticket
Broken down:
- 013A: 1-2 hours (query)
- 013B: 1-2 hours (manipulation)
- 013C: 1 hour (history)
- 013D: 0.5 hours (reset)
- 013E: 2-3 hours (stream mode) OR 0 hours (use REPL)

**Total: 5.5-8.5 hours** (if doing all sub-tickets)
**Recommended Start: 013A only** (1-2 hours for immediate value)

## Key Decision Points

1. **Do we need stream processing mode?**
   - REPL already provides interactive continuous processing
   - Could document "use `meteor-repl` for interactive mode" instead
   - Could add `meteor repl` as alias to `meteor-repl` binary

2. **Output format consistency**
   - Should all commands support `--format=json|text|yaml`?
   - How to handle structured data (meteors) vs simple values?

3. **State management**
   - CLI commands are currently stateless (create engine, process, exit)
   - Stream mode would need stateful engine across inputs
   - This is a significant architectural change

## Recommendation

**Split TICKET-013 into sub-tickets 013A-E** and start with **TICKET-013A (Query Commands)**.

This provides:
- Immediate scripting value
- Clear incremental progress
- Easier to review/test
- Natural stopping points

**Defer TICKET-013E** (Stream Processing) pending decision on whether REPL already satisfies that use case.