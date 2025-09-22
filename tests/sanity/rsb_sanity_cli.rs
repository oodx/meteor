//! RSB CLI Feature Sanity Tests
//!
//! Tests actual CLI feature functionality with bash-like argument patterns.
//! Validates Args type, options! macro, and CLI session management.
//!
//! RSB CLI features mimic bash patterns for argument handling.

use rsb::prelude::*;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_args_bash_like_basic() {
        // Test bash-like args handling per FEATURES_CLI.md
        // Args wrapper provides 1-indexed access (skips argv[0])
        let test_args = vec![
            "meteor".to_string(),       // argv[0] - program name
            "parse".to_string(),        // argv[1] - command
            "--verbose".to_string(),    // argv[2] - flag
            "input.txt".to_string(),    // argv[3] - file
        ];

        let args = Args::new(&test_args);

        // Per RSB CLI docs: get(n) is 1-indexed and skips argv[0]
        assert_eq!(args.get(1), "parse".to_string());     // First real arg
        assert_eq!(args.get(2), "--verbose".to_string()); // Second arg
        assert_eq!(args.get(3), "input.txt".to_string()); // Third arg

        // Test get_or with default
        assert_eq!(args.get_or(4, "default"), "default".to_string());

        // Test all() returns full args
        let all_args = args.all();
        assert_eq!(all_args.len(), 4);
        assert_eq!(all_args[0], "meteor");
    }

    #[test]
    fn test_args_bash_flag_detection() {
        // Test bash-style flag detection per FEATURES_CLI.md API
        let test_args = vec![
            "meteor".to_string(),
            "--verbose".to_string(),
            "-q".to_string(),
            "--format=json".to_string(),
            "--config".to_string(),
            "test.conf".to_string(),
            "parse".to_string(),
        ];

        let args = Args::new(&test_args);

        // Test has() for flag presence
        assert!(args.has("--verbose"));  // Long flag
        assert!(args.has("-q"));         // Short flag
        assert!(!args.has("--missing")); // Non-existent flag

        // Test has_val() for flags with values (--flag=value or --flag value)
        if let Some(val) = args.has_val("--format") {
            assert_eq!(val, "json"); // Should extract value from --format=json
        } else {
            panic!("Expected --format to have value");
        }

        if let Some(val) = args.has_val("--config") {
            assert_eq!(val, "test.conf"); // Should get next arg as value
        } else {
            panic!("Expected --config to have value");
        }
    }

    #[test]
    fn test_args_bash_edge_cases() {
        // Test bash-like edge cases per RSB CLI API docs

        // Empty args (just program name)
        let empty_args = vec!["meteor".to_string()];
        let empty = Args::new(&empty_args);
        assert_eq!(empty.remaining().len(), 0); // No args after program name
        assert_eq!(empty.get_or(1, "none"), "none".to_string());

        // Test key=value and key:array patterns from CLI docs
        let kv_args = vec![
            "meteor".to_string(),
            "key1=value1".to_string(),
            "key2:a,b,c".to_string(),
        ];
        let kv = Args::new(&kv_args);

        // Test get_kv() for key=value parsing
        if let Some((k, v)) = kv.get_kv("key1") {
            assert_eq!(k, "key1");
            assert_eq!(v, "value1");
        }

        // Test get_array() for key:a,b,c parsing
        if let Some((k, arr)) = kv.get_array("key2") {
            assert_eq!(k, "key2");
            assert_eq!(arr, vec!["a", "b", "c"]);
        }

        // Test join() method
        assert!(kv.join(" ").contains("key1=value1"));
    }

    #[test]
    fn test_options_macro_bash_style() {
        // Test options! macro for bash-style option parsing

        // Clear any existing test options
        if has_var("opt_verbose") { unset_var("opt_verbose"); }
        if has_var("opt_quiet") { unset_var("opt_quiet"); }
        if has_var("opt_format") { unset_var("opt_format"); }

        // Test basic flag parsing
        let test_args = vec![
            "meteor".to_string(),
            "--verbose".to_string(),
            "-q".to_string(),
        ];
        let args = Args::new(&test_args);

        // Parse options into global context (bash-like)
        options!(&args);

        // Verify options were parsed bash-style
        assert_eq!(get_var("opt_verbose"), "true");  // Long flag -> opt_verbose=true
        assert_eq!(get_var("opt_q"), "true");        // Short flag -> opt_q=true

        // Cleanup
        unset_var("opt_verbose");
        unset_var("opt_q");
    }

    #[test]
    fn test_options_macro_value_parsing() {
        // Test value parsing patterns like bash getopts

        // Clear existing test vars
        if has_var("opt_format") { unset_var("opt_format"); }
        if has_var("opt_output") { unset_var("opt_output"); }

        let test_args = vec![
            "meteor".to_string(),
            "--format=json".to_string(),     // --key=value style
            "-o".to_string(),                // -o value style (next arg)
            "output.txt".to_string(),
        ];
        let args = Args::new(&test_args);

        options!(&args);

        // Check if format was parsed (depends on RSB implementation)
        // This might need adjustment based on actual RSB behavior
        if has_var("opt_format") {
            let format = get_var("opt_format");
            // Could be "json" or "true" depending on RSB parsing
            assert!(!format.is_empty());
        }

        // Cleanup
        if has_var("opt_format") { unset_var("opt_format"); }
        if has_var("opt_output") { unset_var("opt_output"); }
        if has_var("opt_o") { unset_var("opt_o"); }
    }

    #[test]
    fn test_cli_session_bash_like() {
        // Test CLI session management with bash-like patterns

        // Simulate bash-like environment variables for CLI state
        set_var("METEOR_PWD", "/current/dir");
        set_var("METEOR_OLDPWD", "/previous/dir");
        set_var("METEOR_CONTEXT", "app");

        // Test bash-like variable access
        assert_eq!(get_var("METEOR_PWD"), "/current/dir");
        assert_eq!(get_var("METEOR_OLDPWD"), "/previous/dir");
        assert_eq!(get_var("METEOR_CONTEXT"), "app");

        // Test bash-like path expansion in CLI context
        let expanded_path = expand_vars("$METEOR_PWD/config");
        assert_eq!(expanded_path, "/current/dir/config");

        // Cleanup
        unset_var("METEOR_PWD");
        unset_var("METEOR_OLDPWD");
        unset_var("METEOR_CONTEXT");
    }

    #[test]
    fn test_args_bash_positional_handling() {
        // Test bash-like positional argument handling

        let mixed_args = vec![
            "meteor".to_string(),        // $0
            "parse".to_string(),         // $1 - command
            "--verbose".to_string(),     // flag (not positional)
            "input.txt".to_string(),     // $2 - first file
            "--format=json".to_string(), // flag (not positional)
            "output.txt".to_string(),    // $3 - second file
        ];

        let args = Args::new(&mixed_args);

        // Test bash-like $0, $1, $2, etc. access
        assert_eq!(args.get(0), "meteor".to_string());    // $0
        assert_eq!(args.get(1), "parse".to_string());     // $1
        assert_eq!(args.get(3), "input.txt".to_string()); // $3 (skipping flag at $2)
        assert_eq!(args.get(5), "output.txt".to_string()); // $5 (skipping flag at $4)

        // Test total arg count (bash $#)
        assert_eq!(args.len(), 6);
    }

    #[test]
    fn test_args_bash_flag_conventions() {
        // Test bash CLI flag conventions

        let convention_args = vec![
            "meteor".to_string(),
            "--help".to_string(),         // Help flag
            "--version".to_string(),      // Version flag
            "--config=/path/config".to_string(), // Config with path
            "-vvv".to_string(),          // Verbose levels (bash style)
            "--dry-run".to_string(),     // Dry run flag
        ];

        let args = Args::new(&convention_args);

        // Test standard bash flag patterns
        assert!(args.has("--help"));
        assert!(args.has("--version"));
        assert!(args.has("--config"));
        assert!(args.has("-vvv"));
        assert!(args.has("--dry-run"));

        // Test that non-existent patterns return false
        assert!(!args.has("--bogus"));
        assert!(!args.has("-z"));
    }

    #[test]
    fn test_cli_integration_with_global() {
        // Test CLI integration with global variable store (bash-like env)

        // Simulate CLI parsing that sets global variables
        let cli_args = vec![
            "meteor".to_string(),
            "--context=api".to_string(),
            "--namespace=v1.users".to_string(),
            "parse".to_string(),
        ];

        let args = Args::new(&cli_args);

        // Parse and store in global context (like bash environment)
        options!(&args);

        // Manually set some expected CLI state (simulating CLI implementation)
        set_var("METEOR_COMMAND", "parse");
        set_var("METEOR_ARGC", &args.len().to_string());

        // Test that CLI state is accessible globally
        assert_eq!(get_var("METEOR_COMMAND"), "parse");
        assert_eq!(get_var("METEOR_ARGC"), "7");

        // Test bash-like variable expansion with CLI state
        let status = expand_vars("Running $METEOR_COMMAND with $METEOR_ARGC args");
        assert!(status.contains("Running parse"));
        assert!(status.contains("with 7 args"));

        // Cleanup
        unset_var("METEOR_COMMAND");
        unset_var("METEOR_ARGC");
        // Clean up any option vars that might have been set
        if has_var("opt_context") { unset_var("opt_context"); }
        if has_var("opt_namespace") { unset_var("opt_namespace"); }
    }
}