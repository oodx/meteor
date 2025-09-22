//! RSB GLOBAL Feature Sanity Tests
//!
//! Tests actual GLOBAL feature functionality (not just availability).
//! Validates global variable store operations, expansion, config files, and introspection.
//!
//! These tests ensure RSB GLOBAL features work correctly for CLI implementation.

use rsb::prelude::*;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_global_variable_operations() {
        // Test basic global variable store operations
        set_var("TEST_KEY", "test_value");

        // Test get_var
        assert_eq!(get_var("TEST_KEY"), "test_value");

        // Test has_var
        assert!(has_var("TEST_KEY"));
        assert!(!has_var("NONEXISTENT_KEY"));

        // Test unset_var
        unset_var("TEST_KEY");
        assert!(!has_var("TEST_KEY"));
        assert_eq!(get_var("TEST_KEY"), ""); // should return empty string
    }

    #[test]
    fn test_variable_expansion() {
        // Test variable expansion functionality
        set_var("PROJECT", "meteor");
        set_var("VERSION", "0.1.0");

        // Test simple expansion
        let expanded = expand_vars("Testing $PROJECT functionality");
        assert!(expanded.contains("meteor"));

        // Test expansion with braces
        let expanded_braces = expand_vars("${PROJECT} version ${VERSION}");
        assert!(expanded_braces.contains("meteor"));
        assert!(expanded_braces.contains("0.1.0"));

        // Cleanup
        unset_var("PROJECT");
        unset_var("VERSION");
    }

    #[test]
    fn test_cli_configuration_storage() {
        // Test using global store for CLI configuration

        // Store CLI options
        set_var("opt_verbose", "true");
        set_var("opt_format", "json");
        set_var("opt_config", "/path/to/config");

        // Retrieve CLI options (as CLI would do)
        let verbose = has_var("opt_verbose");
        let format = get_var("opt_format");
        let config = get_var("opt_config");

        assert!(verbose);
        assert_eq!(format, "json");
        assert_eq!(config, "/path/to/config");

        // Test option negation
        set_var("opt_verbose", "false");
        let verbose_disabled = get_var("opt_verbose") == "false";
        assert!(verbose_disabled);

        // Cleanup
        unset_var("opt_verbose");
        unset_var("opt_format");
        unset_var("opt_config");
    }

    #[test]
    fn test_path_expansion() {
        // Test path expansion for CLI file operations
        set_var("HOME", "/home/user");
        set_var("PROJECT_NAME", "meteor");

        // Test path construction with expansion
        let config_path = expand_vars("$HOME/.config/${PROJECT_NAME}/config.toml");
        assert!(config_path.contains("/home/user/.config/meteor/config.toml"));

        // Test relative path expansion
        let output_path = expand_vars("${PROJECT_NAME}_output.txt");
        assert_eq!(output_path, "meteor_output.txt");

        // Cleanup
        unset_var("HOME");
        unset_var("PROJECT_NAME");
    }

    #[test]
    fn test_context_session_management() {
        // Test using global store for CLI session management

        // Simulate CLI session state
        set_var("METEOR_LAST_CONTEXT", "app");
        set_var("METEOR_LAST_NAMESPACE", "ui.widgets");
        set_var("METEOR_SESSION_ID", "12345");

        // Retrieve session state
        let last_context = get_var("METEOR_LAST_CONTEXT");
        let last_namespace = get_var("METEOR_LAST_NAMESPACE");
        let session_id = get_var("METEOR_SESSION_ID");

        assert_eq!(last_context, "app");
        assert_eq!(last_namespace, "ui.widgets");
        assert_eq!(session_id, "12345");

        // Test session state persistence across "commands"
        set_var("METEOR_COMMAND_COUNT", "1");
        let count = get_var("METEOR_COMMAND_COUNT");
        assert_eq!(count, "1");

        // Increment command count
        set_var("METEOR_COMMAND_COUNT", "2");
        let new_count = get_var("METEOR_COMMAND_COUNT");
        assert_eq!(new_count, "2");

        // Cleanup session
        unset_var("METEOR_LAST_CONTEXT");
        unset_var("METEOR_LAST_NAMESPACE");
        unset_var("METEOR_SESSION_ID");
        unset_var("METEOR_COMMAND_COUNT");
    }

    #[test]
    fn test_global_variable_types() {
        // Test different types of values in global store

        // String values
        set_var("STRING_VAL", "hello world");
        assert_eq!(get_var("STRING_VAL"), "hello world");

        // Boolean-like values (CLI flags)
        set_var("BOOL_TRUE", "true");
        set_var("BOOL_FALSE", "false");
        set_var("BOOL_ONE", "1");
        set_var("BOOL_ZERO", "0");

        // Test boolean interpretation
        assert_eq!(get_var("BOOL_TRUE"), "true");
        assert_eq!(get_var("BOOL_FALSE"), "false");
        assert_eq!(get_var("BOOL_ONE"), "1");
        assert_eq!(get_var("BOOL_ZERO"), "0");

        // Number-like values
        set_var("NUMBER", "42");
        assert_eq!(get_var("NUMBER"), "42");

        // JSON-like values
        set_var("JSON_CONFIG", r#"{"key": "value"}"#);
        assert_eq!(get_var("JSON_CONFIG"), r#"{"key": "value"}"#);

        // Cleanup
        unset_var("STRING_VAL");
        unset_var("BOOL_TRUE");
        unset_var("BOOL_FALSE");
        unset_var("BOOL_ONE");
        unset_var("BOOL_ZERO");
        unset_var("NUMBER");
        unset_var("JSON_CONFIG");
    }

    #[test]
    fn test_expansion_edge_cases() {
        // Test edge cases in variable expansion

        set_var("EMPTY", "");
        set_var("SPACE", " ");
        set_var("SPECIAL", "!@#$%^&*()");

        // Test empty variable expansion
        let empty_expanded = expand_vars("Value: $EMPTY");
        assert_eq!(empty_expanded, "Value: ");

        // Test space variable
        let space_expanded = expand_vars("Before${SPACE}After");
        assert_eq!(space_expanded, "Before After");

        // Test special characters
        let special_expanded = expand_vars("Special: $SPECIAL");
        assert!(special_expanded.contains("!@#$%^&*()"));

        // Test nonexistent variable
        let missing_expanded = expand_vars("Missing: $NONEXISTENT");
        assert_eq!(missing_expanded, "Missing: ");

        // Cleanup
        unset_var("EMPTY");
        unset_var("SPACE");
        unset_var("SPECIAL");
    }

    #[test]
    fn test_global_functions_integration() {
        // Test that all global functions work together

        // Setup test scenario
        set_var("TEST_INTEGRATION", "working");

        // Test all functions in sequence
        assert!(has_var("TEST_INTEGRATION"));
        assert_eq!(get_var("TEST_INTEGRATION"), "working");

        // Test expansion with the variable
        let expanded = expand_vars("Status: $TEST_INTEGRATION");
        assert_eq!(expanded, "Status: working");

        // Test variable update
        set_var("TEST_INTEGRATION", "updated");
        assert_eq!(get_var("TEST_INTEGRATION"), "updated");

        // Test cleanup
        unset_var("TEST_INTEGRATION");
        assert!(!has_var("TEST_INTEGRATION"));
        assert_eq!(get_var("TEST_INTEGRATION"), "");
    }
}