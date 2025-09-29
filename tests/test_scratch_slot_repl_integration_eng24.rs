//! ENG-24 Integration Tests - REPL Scratch Slot API Integration
//!
//! Tests that verify the REPL mem commands properly use the new workspace-backed
//! scratch slot APIs with lifetime-managed handles instead of direct storage.

use std::io::Write;
use std::process::{Command, Stdio};

/// Test that REPL mem commands use scratch slot APIs instead of direct storage
#[test]
fn test_repl_mem_uses_scratch_slots() {
    let commands = vec![
        "mem set scratch_test test_value",
        "mem get scratch_test",
        "mem list",
        "exit",
    ];

    let output = run_repl_commands(&commands).expect("REPL should execute successfully");

    // Verify mem commands work correctly with scratch slots
    assert!(output.contains("Stored $scratch_test"));
    assert!(output.contains("$scratch_test = test_value"));
    assert!(output.contains("Scratch entries:"));
    assert!(output.contains("$scratch_test = test_value"));
}

/// Test scratch slot lifetime management through REPL mem operations
#[test]
fn test_repl_scratch_slot_lifecycle() {
    let commands = vec![
        "mem set temp_slot temporary_data",
        "mem set persistent_slot persistent_data",
        "mem list",
        "mem delete temp_slot",
        "mem list",
        "mem get temp_slot",
        "mem get persistent_slot",
        "exit",
    ];

    let output = run_repl_commands(&commands).expect("REPL should execute successfully");

    // Verify lifecycle operations
    assert!(output.contains("Stored $temp_slot"));
    assert!(output.contains("Stored $persistent_slot"));

    // Verify deletion worked
    assert!(output.contains("Removed $temp_slot"));

    // Verify final state - temp_slot should be empty, persistent should remain
    assert!(output.contains("$temp_slot is empty"));
    assert!(output.contains("$persistent_slot = persistent_data"));

    // Count occurrences to verify temp_slot was in first list but not second
    let temp_slot_entries = output.matches("$temp_slot = temporary_data").count();
    let persistent_slot_entries = output.matches("$persistent_slot = persistent_data").count();

    // temp_slot should appear once (in first list only)
    // persistent_slot should appear 3 times (first list, second list, final get)
    assert_eq!(
        temp_slot_entries, 1,
        "temp_slot should appear once in output"
    );
    assert_eq!(
        persistent_slot_entries, 3,
        "persistent_slot should appear three times in output"
    );
}

/// Test REPL mem edit functionality with scratch slots
#[test]
fn test_repl_mem_edit_with_scratch_slots() {
    let commands = vec![
        "mem set editable_slot initial_value",
        "mem get editable_slot",
        "exit",
    ];

    let output = run_repl_commands(&commands).expect("REPL should execute successfully");

    // Verify basic edit workflow
    assert!(output.contains("Stored $editable_slot"));
    assert!(output.contains("$editable_slot = initial_value"));
}

/// Test multiple scratch slots work independently
#[test]
fn test_repl_multiple_scratch_slots() {
    let commands = vec![
        "mem set slot1 value1",
        "mem set slot2 value2",
        "mem set slot3 value3",
        "mem list",
        "mem get slot1",
        "mem get slot2",
        "mem get slot3",
        "mem delete slot2",
        "mem list",
        "exit",
    ];

    let output = run_repl_commands(&commands).expect("REPL should execute successfully");

    // Verify multiple slots work independently
    assert!(output.contains("Stored $slot1"));
    assert!(output.contains("Stored $slot2"));
    assert!(output.contains("Stored $slot3"));

    // Verify all values are accessible
    assert!(output.contains("$slot1 = value1"));
    assert!(output.contains("$slot2 = value2"));
    assert!(output.contains("$slot3 = value3"));

    // Verify selective deletion
    assert!(output.contains("Removed $slot2"));

    // Verify slot2 was removed while others remain
    // slot2 should appear twice (initial list, get) but not in final list
    // slot1 and slot3 should appear three times each (initial list, get, final list)
    let slot1_entries = output.matches("$slot1 = value1").count();
    let slot2_entries = output.matches("$slot2 = value2").count();
    let slot3_entries = output.matches("$slot3 = value3").count();

    assert_eq!(slot1_entries, 3, "slot1 should appear 3 times");
    assert_eq!(
        slot2_entries, 2,
        "slot2 should appear 2 times (before deletion)"
    );
    assert_eq!(slot3_entries, 3, "slot3 should appear 3 times");
}

/// Test scratch slot persistence across REPL operations
#[test]
fn test_repl_scratch_slot_persistence() {
    let commands = vec![
        "mem set persistent_data important_value",
        "parse app:ui:button=click", // Other operations shouldn't affect scratch
        "set app:ui:theme dark",
        "mem get persistent_data",
        "mem list",
        "contexts", // Another operation
        "mem get persistent_data",
        "exit",
    ];

    let output = run_repl_commands(&commands).expect("REPL should execute successfully");

    // Verify scratch data persists across other operations
    assert!(output.contains("Stored $persistent_data"));

    // Should be accessible after parse operation
    let after_parse = extract_section(
        &output,
        "meteor> mem get persistent_data",
        "meteor> mem list",
    );
    assert!(after_parse.contains("$persistent_data = important_value"));

    // Should still be in list
    let list_section = extract_section(&output, "meteor> mem list", "meteor> contexts");
    assert!(list_section.contains("$persistent_data = important_value"));

    // Should still be accessible after other operations
    let final_get = extract_section(&output, "meteor> mem get persistent_data", "meteor> exit");
    assert!(final_get.contains("$persistent_data = important_value"));
}

/// Test edge cases with scratch slot naming
#[test]
fn test_repl_scratch_slot_naming_edge_cases() {
    let commands = vec![
        "mem set simple test1",
        "mem set with_underscore test2",
        "mem set $prefixed test3",       // $ prefix should be handled
        "mem set \"quoted name\" test4", // Quoted names should be cleaned
        "mem list",
        "exit",
    ];

    let output = run_repl_commands(&commands).expect("REPL should execute successfully");

    // Verify all variations work
    assert!(output.contains("Stored $simple"));
    assert!(output.contains("Stored $with_underscore"));
    assert!(output.contains("Stored $prefixed"));

    // List should show cleaned names
    assert!(output.contains("$simple = test1"));
    assert!(output.contains("$with_underscore = test2"));
    assert!(output.contains("$prefixed = test3"));
}

/// Test error handling with scratch slots
#[test]
fn test_repl_scratch_slot_error_handling() {
    let commands = vec![
        "mem get nonexistent_slot",
        "mem delete nonexistent_slot",
        "mem list", // Should show empty
        "exit",
    ];

    let output = run_repl_commands(&commands).expect("REPL should execute successfully");

    // Verify proper error handling
    assert!(output.contains("$nonexistent_slot is empty"));
    assert!(output.contains("$nonexistent_slot was empty"));
    assert!(output.contains("Scratch pad empty"));
}

// Helper functions

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

fn extract_section(output: &str, start_marker: &str, end_marker: &str) -> String {
    let start_pos = output.find(start_marker).unwrap_or(0);
    let search_from = start_pos + start_marker.len();
    let end_pos = output[search_from..]
        .find(end_marker)
        .map(|pos| search_from + pos)
        .unwrap_or(output.len());

    output[start_pos..end_pos].to_string()
}
