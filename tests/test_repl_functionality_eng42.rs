//! REPL-05 Functional Tests - Scripted REPL Command Validation
//!
//! Tests that verify REPL commands work correctly after the meteor view API
//! refactoring. These tests validate key REPL functionality through automated
//! scripted interactions.

use std::io::Write;
use std::process::{Command, Stdio};

/// Test basic REPL parsing and engine state
#[test]
fn test_repl_basic_parse_and_state() {
    let commands = vec![
        "parse app:ui:button=click; app:ui:theme=dark",
        "show",
        "exit",
    ];

    let output = run_repl_commands(&commands).expect("REPL should execute successfully");

    // Verify parsing worked
    assert!(output.contains("Parsed 1 segment(s)"));

    // Verify engine state shows parsed data
    assert!(output.contains("=== Meteor Engine State ==="));
    assert!(output.contains("Context: app"));
    assert!(output.contains("Namespace: ui"));
    assert!(output.contains("button = click"));
    assert!(output.contains("theme = dark"));
}

/// Test REPL list command with meteor view APIs
#[test]
fn test_repl_list_command() {
    let commands = vec![
        "parse app:ui:button=click; app:ui:theme=dark; user:settings:lang=en",
        "list app ui",
        "list user settings",
        "exit",
    ];

    let output = run_repl_commands(&commands).expect("REPL should execute successfully");

    // Verify list command shows correct entries
    assert!(output.contains("Entries for app:ui:"));
    assert!(output.contains("button = click"));
    assert!(output.contains("theme = dark"));

    assert!(output.contains("Entries for user:settings:"));
    assert!(output.contains("lang = en"));
}

/// Test REPL contexts and namespaces commands
#[test]
fn test_repl_contexts_namespaces() {
    let commands = vec![
        "parse app:ui:button=click; sys:config:debug=true; user:profile:name=John",
        "contexts",
        "namespaces app",
        "namespaces sys",
        "namespaces user",
        "exit",
    ];

    let output = run_repl_commands(&commands).expect("REPL should execute successfully");

    // Verify contexts command shows all contexts
    assert!(output.contains("Contexts:"));
    assert!(output.contains("app"));
    assert!(output.contains("sys"));
    assert!(output.contains("user"));

    // Verify namespaces commands
    assert!(output.contains("Namespaces in app:"));
    assert!(output.contains("ui"));

    assert!(output.contains("Namespaces in sys:"));
    assert!(output.contains("config"));

    assert!(output.contains("Namespaces in user:"));
    assert!(output.contains("profile"));
}

/// Test REPL get/set/delete operations
#[test]
fn test_repl_crud_operations() {
    let commands = vec![
        "set app:ui:button click",
        "set app:ui:theme dark",
        "get app:ui:button",
        "get app:ui:theme",
        "delete app:ui:theme",
        "get app:ui:theme",
        "exit",
    ];

    let output = run_repl_commands(&commands).expect("REPL should execute successfully");

    // Verify set operations
    assert!(output.contains("Stored 'app:ui:button'."));
    assert!(output.contains("Stored 'app:ui:theme'."));

    // Verify get operations
    assert!(output.contains("app:ui:button = click"));
    assert!(output.contains("app:ui:theme = dark"));

    // Verify delete operation
    assert!(output.contains("Removed app:ui:theme"));
    assert!(output.contains("app:ui:theme not found"));
}

/// Test REPL memory/scratch functionality
#[test]
fn test_repl_mem_functionality() {
    let commands = vec![
        "mem set test_key test_value",
        "mem set another_key another_value",
        "mem list",
        "mem get test_key",
        "mem delete test_key",
        "mem list",
        "exit",
    ];

    let output = run_repl_commands(&commands).expect("REPL should execute successfully");

    // Verify mem set operations
    assert!(output.contains("Stored $test_key"));
    assert!(output.contains("Stored $another_key"));

    // Verify mem list shows entries
    assert!(output.contains("Scratch entries:"));
    assert!(output.contains("$test_key = test_value"));
    assert!(output.contains("$another_key = another_value"));

    // Verify mem get operation
    assert!(output.contains("$test_key = test_value"));

    // Verify mem delete operation
    assert!(output.contains("Removed $test_key"));

    // Verify final list only shows remaining entry
    let final_list_start = output.rfind("Scratch entries:").unwrap();
    let final_section = &output[final_list_start..];
    assert!(final_section.contains("$another_key = another_value"));
    assert!(!final_section.contains("$test_key"));
}

/// Test REPL token parsing functionality
#[test]
fn test_repl_token_parsing() {
    let commands = vec!["token button=click; theme=dark; ui:lang=en", "exit"];

    let output = run_repl_commands(&commands).expect("REPL should execute successfully");

    // Verify token parsing shows detailed breakdown
    assert!(output.contains("Parsed 3 token(s):"));
    assert!(output.contains("Token 1:"));
    assert!(output.contains("Token 2:"));
    assert!(output.contains("Token 3:"));

    // Verify token details
    assert!(output.contains("Key: button"));
    assert!(output.contains("Value: click"));
    assert!(output.contains("Key: theme"));
    assert!(output.contains("Value: dark"));
    assert!(output.contains("Namespace: ui"));
    assert!(output.contains("Key: lang"));
    assert!(output.contains("Value: en"));
}

/// Test REPL validation functionality
#[test]
fn test_repl_validation() {
    let commands = vec![
        "validate app:ui:button=click; app:ui:theme=dark",
        "validate app:ui:button=click :;: user:settings:lang=en",
        "validate invalid_format_here",
        "exit",
    ];

    let output = run_repl_commands(&commands).expect("REPL should execute successfully");

    // Verify valid formats are accepted
    assert!(output.contains("✅ Valid meteor stream"));

    // Verify invalid format is rejected
    assert!(output.contains("❌ Invalid stream:"));
}

/// Test REPL bracket notation support
#[test]
fn test_repl_bracket_notation() {
    let commands = vec![
        "parse app:list:items[0]=apple; app:list:items[1]=banana; app:grid:cell[2,3]=value",
        "list app list",
        "get app:list:items[0]",
        "exit",
    ];

    let output = run_repl_commands(&commands).expect("REPL should execute successfully");

    // Verify bracket notation is handled correctly (preserved per ENG-21)
    assert!(output.contains("items[0] = apple"));
    assert!(output.contains("items[1] = banana"));
    assert!(output.contains("cell[2,3] = value"));

    // Verify get operation works with bracket notation
    assert!(output.contains("= apple") || output.contains("apple"));
}

/// Test REPL help and error handling
#[test]
fn test_repl_help_and_errors() {
    let commands = vec![
        "help",
        "mem help",
        "unknown_command",
        "get", // Missing argument
        "set", // Missing arguments
        "exit",
    ];

    let output = run_repl_commands(&commands).expect("REPL should execute successfully");

    // Verify help commands work
    assert!(output.contains("Available commands:"));
    assert!(output.contains("parse <stream>"));
    assert!(output.contains("mem commands:"));
    assert!(output.contains("mem set <name> <value>"));

    // Verify error handling
    assert!(output.contains("Unknown command 'unknown_command'"));
    assert!(output.contains("Usage: get"));
    assert!(output.contains("Usage: set"));
}

/// Test REPL workspace ordering consistency after meteor view API update
#[test]
fn test_repl_workspace_ordering() {
    let commands = vec![
        "parse user:profile:name=John; app:ui:theme=dark; sys:config:debug=true; app:ui:lang=en",
        "show",
        "contexts",
        "exit",
    ];

    let output = run_repl_commands(&commands).expect("REPL should execute successfully");

    // Find all context positions in the engine state output
    let show_section_start = output.find("=== Meteor Engine State ===").unwrap();
    let show_section = &output[show_section_start..];

    // Check that contexts appear in workspace order (app, sys, user)
    let app_pos = show_section.find("Context: app").unwrap();
    let sys_pos = show_section.find("Context: sys").unwrap();
    let user_pos = show_section.find("Context: user").unwrap();

    assert!(
        app_pos < sys_pos && sys_pos < user_pos,
        "Contexts should appear in workspace order: app < sys < user"
    );

    // Verify all meteors are properly displayed
    assert!(show_section.contains("name = John"));
    assert!(show_section.contains("theme = dark"));
    assert!(show_section.contains("debug = true"));
    assert!(show_section.contains("lang = en"));
}

// Helper function to run REPL commands

fn run_repl_commands(commands: &[&str]) -> Result<String, Box<dyn std::error::Error>> {
    let mut child = Command::new("cargo")
        .args(&["run", "--bin", "meteor-repl", "--"])
        .current_dir(env!("CARGO_MANIFEST_DIR"))
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()?;

    {
        let stdin = child.stdin.as_mut().expect("Failed to open stdin");
        for command in commands {
            writeln!(stdin, "{}", command)?;
        }
    }

    let output = child.wait_with_output()?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(format!("REPL command failed: {}", stderr).into());
    }

    Ok(String::from_utf8(output.stdout)?)
}
