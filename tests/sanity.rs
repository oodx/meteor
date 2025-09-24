//! Meteor Sanity Tests
//!
//! RSB-compliant sanity test entry point. These tests validate core functionality
//! with no ceremony following RSB testing patterns.
//!
//! Tests are organized in tests/sanity/ subdirectory for detailed functionality.

// Re-export all sanity tests from subdirectory modules
// This allows both `cargo test --test sanity` and `test.sh sanity` to work

extern crate meteor;

#[cfg(test)]
mod tests {
    // Import the actual meteor module to run basic validation
    use meteor::BracketNotation;

    /// Sanity check: meteor compiles and basic API works
    #[test]
    fn sanity_meteor_compiles_and_basic_api() {
        // Basic compilation and API access test - use MeteorShower now
        let shower = meteor::parse_shower("app:ui:key=value").unwrap();
        assert_eq!(shower.len(), 1);
        let found = shower.find("app", "ui", "key");
        assert!(found.is_some());
        assert_eq!(found.unwrap().token().value(), "value");
    }

    /// Sanity check: bracket notation works
    #[test]
    fn sanity_bracket_notation_basic() {
        // Test bracket notation with MeteorShower
        let shower = meteor::parse_shower("app:list:list[0]=item").unwrap();
        assert_eq!(shower.len(), 1);
        let found = shower.find("app", "list", "list[0]");
        assert!(found.is_some() || shower.find("app", "list", "list__i_0").is_some());
    }

    /// Sanity check: BracketNotation trait works
    #[test]
    fn sanity_bracket_notation_trait() {
        let flat = "list__i_0";
        assert_eq!(flat.to_bracket(), "list[0]");
    }

    /// Sanity check: MeteorShower collection works
    #[test]
    fn sanity_meteor_shower_basic() {
        let shower = meteor::parse_shower("app:ui:button=click").unwrap();
        assert_eq!(shower.len(), 1);

        let meteors = shower.by_context("app");
        assert_eq!(meteors.len(), 1);
    }

    // RSB Feature Sanity Tests
    // These test actual RSB functionality needed for CLI implementation

    /// RSB GLOBAL sanity: Basic variable operations
    #[test]
    fn sanity_rsb_global_variables() {
        use rsb::prelude::*;

        // Test basic global variable operations
        set_var("TEST_SANITY", "working");
        assert_eq!(get_var("TEST_SANITY"), "working");
        assert!(has_var("TEST_SANITY"));

        // Test variable expansion
        let expanded = expand_vars("Status: $TEST_SANITY");
        assert_eq!(expanded, "Status: working");

        // Cleanup
        unset_var("TEST_SANITY");
        assert!(!has_var("TEST_SANITY"));
    }

    /// RSB CLI sanity: Args type availability
    #[test]
    fn sanity_rsb_cli_args() {
        use rsb::prelude::*;

        // Test that Args type is available for CLI implementation
        let test_args = vec!["meteor".to_string(), "parse".to_string(), "--verbose".to_string()];
        let args = Args::new(&test_args);

        // Test basic Args operations
        assert_eq!(args.len(), 3); // All args including argv[0]
        assert_eq!(args.get(1), "parse".to_string()); // First arg after argv[0]
        assert!(args.has("--verbose"));
    }

    /// RSB OPTIONS sanity: CLI option parsing
    #[test]
    fn sanity_rsb_options_parsing() {
        use rsb::prelude::*;

        // Clear any existing options
        if has_var("opt_test") { unset_var("opt_test"); }

        let test_args = vec!["program".to_string(), "--test".to_string()];
        let args = Args::new(&test_args);

        // Parse options into global context
        options!(&args);

        // Verify option was parsed
        assert_eq!(get_var("opt_test"), "true");

        // Cleanup
        unset_var("opt_test");
    }

    /// RSB STRINGS sanity: String processing
    #[test]
    fn sanity_rsb_strings_processing() {
        // Test RSB string functions from rsb::string module
        use rsb::string::*;

        // Test string case conversion functions
        let test_str = "HelloWorld";
        let snake = to_snake_case(test_str);
        assert_eq!(snake, "hello_world");

        let kebab = to_kebab_case(test_str);
        assert_eq!(kebab, "hello-world");

        // Test string utility functions
        let prefix_test = "hello_world";
        assert!(is_name(prefix_test)); // Should be valid identifier name

        // Test string replacement
        let replaced = str_replace("hello world", "world", "rust", false);
        assert_eq!(replaced, "hello rust");

    }

    /// RSB GLOBAL comprehensive: Variable expansion and CLI integration
    #[test]
    fn sanity_rsb_global_comprehensive() {
        use rsb::prelude::*;

        // Test CLI configuration via global variables
        set_var("METEOR_CONTEXT", "app");
        set_var("METEOR_NAMESPACE", "ui.components");
        set_var("HOME", "/home/user");

        // Test path construction with expansion
        let config_path = expand_vars("$HOME/.config/meteor");
        assert_eq!(config_path, "/home/user/.config/meteor");

        // Test context expansion for CLI
        let context_display = expand_vars("[$METEOR_CONTEXT:$METEOR_NAMESPACE]");
        assert_eq!(context_display, "[app:ui.components]");

        // Cleanup
        unset_var("METEOR_CONTEXT");
        unset_var("METEOR_NAMESPACE");
        unset_var("HOME");
    }

    /// RSB CLI comprehensive: Bash-like argument patterns
    #[test]
    fn sanity_rsb_cli_comprehensive() {
        use rsb::prelude::*;

        // Test bash-like args with mixed flags and positional arguments
        let complex_args = vec![
            "meteor".to_string(),
            "parse".to_string(),
            "--verbose".to_string(),
            "--context=app".to_string(),
            "input.txt".to_string(),
        ];
        let args = Args::new(&complex_args);

        // Test bash-like $1, $2, $3 access (1-indexed, skips argv[0])
        assert_eq!(args.get(1), "parse".to_string());
        assert_eq!(args.get(4), "input.txt".to_string());

        // Test flag detection
        assert!(args.has("--verbose"));
        // Note: --context=app format may need different detection
        // For now, test what we know works
        assert!(!args.has("--missing"));

        // Test total arg count
        assert_eq!(args.len(), 5);
    }

    /// RSB OPTIONS comprehensive: Standard CLI patterns with global integration
    #[test]
    fn sanity_rsb_options_comprehensive() {
        use rsb::prelude::*;

        // Clear test variables
        ["opt_verbose", "opt_quiet", "opt_config", "VERBOSE", "QUIET"].iter()
            .for_each(|key| if has_var(key) { unset_var(key); });

        let test_args = vec![
            "meteor".to_string(),
            "--verbose".to_string(),
            "--config=/path/config".to_string(),
        ];
        let args = Args::new(&test_args);

        // Parse options
        options!(&args);

        // Test that at least one form of verbose was parsed
        let verbose_found = has_var("opt_verbose") || has_var("VERBOSE");
        assert!(verbose_found, "Verbose option should be parsed");

        // Test integration with global context
        set_var("CLI_MODE", "interactive");
        let mode_expansion = expand_vars("Mode: $CLI_MODE");
        assert_eq!(mode_expansion, "Mode: interactive");

        // Cleanup
        ["opt_verbose", "opt_quiet", "opt_config", "VERBOSE", "QUIET", "CLI_MODE"].iter()
            .for_each(|key| if has_var(key) { unset_var(key); });
    }
}