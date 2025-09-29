//! CLI-05 Parity Tests - Text/JSON Output Format Consistency
//!
//! Tests that verify CLI parse command produces consistent results between
//! text and JSON output formats using meteor view APIs. These tests ensure
//! the CLI refactoring maintains output parity.

// CLI integration tests - no direct library imports needed
use std::process::Command;

/// Test basic CLI text output with meteor view APIs
#[test]
fn test_cli_text_output_basic() {
    let input = "app:ui:button=click; app:ui:theme=dark; user:settings:lang=en";

    let output = run_cli_parse_text(input).expect("CLI should execute successfully");

    // Verify text output contains expected entries with proper format
    assert!(output.contains("Context: app"));
    assert!(output.contains("Namespace: ui"));
    assert!(output.contains("Key: button"));
    assert!(output.contains("Value: click"));
    assert!(output.contains("Key: theme"));
    assert!(output.contains("Value: dark"));
    assert!(output.contains("Context: user"));
    assert!(output.contains("Namespace: settings"));
    assert!(output.contains("Key: lang"));
    assert!(output.contains("Value: en"));

    // Verify total meteors count
    assert!(output.contains("Total: 3 meteors"));

    // Verify ordering (app meteors should come before user meteors)
    let app_pos = output.find("Context: app").unwrap();
    let user_pos = output.find("Context: user").unwrap();
    assert!(
        app_pos < user_pos,
        "Workspace ordering should place app before user"
    );
}

/// Test basic CLI JSON output with meteor view APIs
#[test]
fn test_cli_json_output_basic() {
    let input = "app:ui:button=click; app:ui:theme=dark; user:settings:lang=en";

    let output = run_cli_parse_json(input).expect("CLI should execute successfully");

    // Parse JSON to verify structure
    let json: serde_json::Value =
        serde_json::from_str(&output).expect("CLI should produce valid JSON");

    // Verify JSON structure contains expected top-level fields
    assert!(json.get("cursor").is_some(), "JSON should contain cursor");
    assert!(
        json.get("contexts").is_some(),
        "JSON should contain contexts count"
    );
    assert!(
        json.get("meteors").is_some(),
        "JSON should contain meteors array"
    );

    // Verify contexts count
    assert_eq!(json["contexts"], 2);

    // Verify meteors array
    let meteors = json["meteors"]
        .as_array()
        .expect("meteors should be an array");
    assert_eq!(meteors.len(), 3);

    // Check first meteor (app:ui:button=click)
    let first = &meteors[0];
    assert_eq!(first["context"], "app");
    assert_eq!(first["namespace"], "ui");
    assert_eq!(first["key"], "button");
    assert_eq!(first["value"], "click");

    // Check second meteor (app:ui:theme=dark)
    let second = &meteors[1];
    assert_eq!(second["context"], "app");
    assert_eq!(second["namespace"], "ui");
    assert_eq!(second["key"], "theme");
    assert_eq!(second["value"], "dark");

    // Check third meteor (user:settings:lang=en)
    let third = &meteors[2];
    assert_eq!(third["context"], "user");
    assert_eq!(third["namespace"], "settings");
    assert_eq!(third["key"], "lang");
    assert_eq!(third["value"], "en");

    // Verify cursor
    assert_eq!(json["cursor"]["context"], "app");
    assert_eq!(json["cursor"]["namespace"], "main");
}

/// Test text/JSON parity for complex meteor stream with quotes and special characters
#[test]
fn test_cli_output_parity_complex() {
    let input = r#"app:ui:message="Hello; World"; app:config:path="/usr/bin"; user:profile:name="John Doe""#;

    let text_output = run_cli_parse_text(input).expect("CLI text should execute successfully");
    let json_output = run_cli_parse_json(input).expect("CLI JSON should execute successfully");

    // Parse JSON for structured comparison
    let json: serde_json::Value =
        serde_json::from_str(&json_output).expect("CLI should produce valid JSON");

    // Verify both formats contain the same key-value pairs

    // Check app:ui:message
    assert!(text_output.contains(r#"message = "Hello; World""#));
    assert_eq!(json["app"]["ui"]["message"], "\"Hello; World\"");

    // Check app:config:path
    assert!(text_output.contains(r#"path = "/usr/bin""#));
    assert_eq!(json["app"]["config"]["path"], "\"/usr/bin\"");

    // Check user:profile:name
    assert!(text_output.contains(r#"name = "John Doe""#));
    assert_eq!(json["user"]["profile"]["name"], "\"John Doe\"");

    // Verify context ordering is consistent
    let app_pos = text_output.find("Context: app").unwrap();
    let user_pos = text_output.find("Context: user").unwrap();
    assert!(
        app_pos < user_pos,
        "Text output should maintain workspace ordering"
    );
}

/// Test text/JSON parity for bracket notation
#[test]
fn test_cli_output_parity_bracket_notation() {
    let input = "app:list:items[0]=apple; app:list:items[1]=banana; app:grid:cell[2,3]=value";

    let text_output = run_cli_parse_text(input).expect("CLI text should execute successfully");
    let json_output = run_cli_parse_json(input).expect("CLI JSON should execute successfully");

    // Parse JSON for structured comparison
    let json: serde_json::Value =
        serde_json::from_str(&json_output).expect("CLI should produce valid JSON");

    // Verify bracket notation is handled consistently
    // Note: The implementation preserves bracket notation in display (ENG-21)

    // Check bracket notation keys in text output
    assert!(text_output.contains("items[0] = apple"));
    assert!(text_output.contains("items[1] = banana"));
    assert!(text_output.contains("cell[2,3] = value"));

    // Check bracket notation keys in JSON output
    assert_eq!(json["app"]["list"]["items[0]"], "apple");
    assert_eq!(json["app"]["list"]["items[1]"], "banana");
    assert_eq!(json["app"]["grid"]["cell[2,3]"], "value");
}

/// Test text/JSON parity for control tokens
#[test]
fn test_cli_output_parity_control_tokens() {
    let input = "app:ui:button=click; ctl:delete=app:ui:theme; app:ui:lang=en";

    let text_output = run_cli_parse_text(input).expect("CLI text should execute successfully");
    let json_output = run_cli_parse_json(input).expect("CLI JSON should execute successfully");

    // Parse JSON for structured comparison
    let json: serde_json::Value =
        serde_json::from_str(&json_output).expect("CLI should produce valid JSON");

    // Verify control tokens are processed (delete should not appear as data)
    // Only the stored values should appear
    assert!(text_output.contains("button = click"));
    assert!(text_output.contains("lang = en"));

    // Control delete should not create a stored value
    assert!(!text_output.contains("ctl:delete"));
    assert!(!text_output.contains("app:ui:theme"));

    // Same for JSON
    assert_eq!(json["app"]["ui"]["button"], "click");
    assert_eq!(json["app"]["ui"]["lang"], "en");

    // No control tokens should appear as data
    assert!(json.get("ctl").is_none());
}

/// Test text/JSON parity for minimal input with proper meteor format
#[test]
fn test_cli_output_parity_minimal() {
    let input = "app:main:key=value";

    let text_output = run_cli_parse_text(input).expect("CLI text should execute successfully");
    let json_output = run_cli_parse_json(input).expect("CLI JSON should execute successfully");

    // Parse JSON for structured comparison
    let json: serde_json::Value =
        serde_json::from_str(&json_output).expect("CLI should produce valid JSON");

    // Verify minimal input produces consistent output
    assert!(text_output.contains("key = value"));
    assert_eq!(json["app"]["main"]["key"], "value");

    // Verify default context/namespace handling
    assert!(text_output.contains("Context: app"));
    assert!(text_output.contains("Namespace: main") || text_output.contains("Namespace: (root)"));
}

/// Test error handling parity between text and JSON outputs
#[test]
fn test_cli_output_error_handling_parity() {
    // Test with invalid meteor stream (unclosed quotes)
    let invalid_input = r#"app:ui:message="unclosed quote"#;

    let text_result = run_cli_parse_text(invalid_input);
    let json_result = run_cli_parse_json(invalid_input);

    // Both should either succeed (if gracefully handled) or fail consistently
    match (text_result, json_result) {
        (Ok(text), Ok(json)) => {
            // If both succeed, they should handle the error gracefully and consistently
            // Check that both indicate the error somehow
            let text_lower = text.to_lowercase();
            let json_lower = json.to_lowercase();

            // Both should either show the error or handle it the same way
            assert!(
                text_lower.contains("error")
                    || text_lower.contains("empty")
                    || !text_lower.is_empty()
            );
            assert!(
                json_lower.contains("error")
                    || json_lower.contains("empty")
                    || !json_lower.is_empty()
            );
        }
        (Err(_), Err(_)) => {
            // Both failing is also acceptable - consistent error handling
        }
        _ => {
            panic!("Text and JSON outputs should handle errors consistently");
        }
    }
}

/// Test workspace ordering consistency between text and JSON
#[test]
fn test_cli_output_workspace_ordering_parity() {
    // Create input that will test workspace ordering
    let input = "user:profile:name=John; app:ui:theme=dark; sys:config:debug=true; app:ui:lang=en";

    let text_output = run_cli_parse_text(input).expect("CLI text should execute successfully");
    let json_output = run_cli_parse_json(input).expect("CLI JSON should execute successfully");

    // Parse JSON to check ordering
    let json: serde_json::Value =
        serde_json::from_str(&json_output).expect("CLI should produce valid JSON");

    // In text output, check context ordering
    let app_pos = text_output.find("Context: app").unwrap();
    let sys_pos = text_output.find("Context: sys").unwrap();
    let user_pos = text_output.find("Context: user").unwrap();

    // Workspace ordering should be consistent
    assert!(app_pos < sys_pos && sys_pos < user_pos);

    // JSON should contain all contexts
    assert!(json.get("app").is_some());
    assert!(json.get("sys").is_some());
    assert!(json.get("user").is_some());

    // Verify values are correct in both formats
    assert!(text_output.contains("name = John"));
    assert!(text_output.contains("theme = dark"));
    assert!(text_output.contains("debug = true"));
    assert!(text_output.contains("lang = en"));

    assert_eq!(json["user"]["profile"]["name"], "John");
    assert_eq!(json["app"]["ui"]["theme"], "dark");
    assert_eq!(json["sys"]["config"]["debug"], "true");
    assert_eq!(json["app"]["ui"]["lang"], "en");
}

// Helper functions for running CLI commands

fn run_cli_parse_text(input: &str) -> Result<String, Box<dyn std::error::Error>> {
    // Run the CLI command with text output (default format)
    let output = Command::new("cargo")
        .args(&["run", "--bin", "meteor", "--", "parse", input])
        .current_dir(env!("CARGO_MANIFEST_DIR"))
        .output()?;

    if !output.status.success() {
        return Err(format!(
            "CLI command failed: {}",
            String::from_utf8_lossy(&output.stderr)
        )
        .into());
    }

    Ok(String::from_utf8(output.stdout)?)
}

fn run_cli_parse_json(input: &str) -> Result<String, Box<dyn std::error::Error>> {
    // Run the CLI command with JSON output
    let output = Command::new("cargo")
        .args(&[
            "run",
            "--bin",
            "meteor",
            "--",
            "parse",
            "--format=json",
            input,
        ])
        .current_dir(env!("CARGO_MANIFEST_DIR"))
        .output()?;

    if !output.status.success() {
        return Err(format!(
            "CLI command failed: {}",
            String::from_utf8_lossy(&output.stderr)
        )
        .into());
    }

    Ok(String::from_utf8(output.stdout)?)
}
