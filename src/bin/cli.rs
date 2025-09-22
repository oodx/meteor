//! Meteor RSB CLI
//!
//! Native RSB-compliant command-line interface for meteor token processing.
//! Implements RSB patterns with bootstrap!, dispatch!, and options! macros.

use rsb::prelude::*;

fn main() {
    // RSB bootstrap pattern - gets Args from environment
    let args = bootstrap!();

    // RSB options pattern - parse flags into global context
    options!(&args);

    // RSB dispatch pattern - handle subcommands
    dispatch!(&args, {
        "parse" => parse_command, desc: "Parse meteor token streams",
        "validate" => validate_command, desc: "Validate meteor token format"
    });
}

/// Handle the parse command using RSB patterns
fn parse_command(args: Args) -> i32 {
    // Get input from positional args, skipping flags
    // RSB Args: get non-flag arguments starting from position 1
    let mut input = String::new();

    // Find first non-flag argument as input
    for i in 1..=args.len() {
        let arg = args.get(i);
        if !arg.is_empty() && !arg.starts_with('-') {
            input = arg;
            break;
        }
    }

    if input.is_empty() {
        eprintln!("Error: No input provided");
        eprintln!("Usage: meteor parse [--verbose] [--format=FORMAT] <token_string>");
        eprintln!("Example: meteor parse --verbose --format=json \"app:ui:button=click\"");
        eprintln!("Example: meteor parse \"ctx:ns:key=value;other=data\"");
        return 1;
    }

    // Get options from RSB global context (set by options! macro)
    // Check both --verbose and -v (short flag)
    let verbose = has_var("opt_verbose") || has_var("opt_v");
    let format = get_var("opt_format");
    let format = if format.is_empty() { "text" } else { &format };

    if verbose {
        eprintln!("Parsing input: {}", input);
        eprintln!("Output format: {}", format);
    }

    // Use existing meteor parsing logic
    match meteor::parse(&input) {
        Ok(bucket) => {
            print_output(&bucket, &input, verbose, format);
            0
        }
        Err(e) => {
            eprintln!("Parse error: {}", e);
            1
        }
    }
}

/// Print output in the specified format using existing output logic
fn print_output(bucket: &meteor::TokenBucket, input: &str, verbose: bool, format: &str) {
    match format {
        "json" => print_json_output(bucket, input, verbose),
        "debug" => print_debug_output(bucket, input),
        _ => print_text_output(bucket, input, verbose),
    }
}

/// Print text format output
fn print_text_output(bucket: &meteor::TokenBucket, input: &str, verbose: bool) {
    if verbose {
        println!("=== Meteor Token Parse Results ===");
        println!("Input: {}", input);
        println!("Tokens found: {}", bucket.len());
        println!();
    }

    let namespaces = bucket.namespaces();
    for namespace in &namespaces {
        if namespace.is_empty() {
            println!("Root namespace:");
        } else {
            println!("Namespace '{}':", namespace);
        }

        let keys = bucket.keys_in_namespace(namespace);
        for key in &keys {
            if let Some(value) = bucket.get(namespace, key) {
                println!("  {} = {}", key, value);
            }
        }

        if !namespace.is_empty() || namespaces.len() > 1 {
            println!();
        }
    }
}

/// Show advanced help for parse command
fn show_parse_help() {
    // Simple colored output using ANSI codes directly
    fn colorize(text: &str, color: &str) -> String {
        match color {
            "blue" => format!("\x1b[34m{}\x1b[0m", text),
            "cyan" => format!("\x1b[36m{}\x1b[0m", text),
            "green" => format!("\x1b[32m{}\x1b[0m", text),
            "white" => format!("\x1b[37m{}\x1b[0m", text),
            "yellow" => format!("\x1b[33m{}\x1b[0m", text),
            "bright_black" => format!("\x1b[90m{}\x1b[0m", text),
            _ => text.to_string(),
        }
    }

    println!("{}\n", colorize("meteor parse", "blue"));
    println!("{}\n", colorize("Parse meteor token streams with analysis options", "cyan"));

    println!("{}", colorize("USAGE:", "green"));
    println!("  {} {} {}",
        colorize("meteor parse", "white"),
        colorize("[FLAGS]", "yellow"),
        colorize("<PATTERN>", "white"));
    println!();

    println!("{}", colorize("FLAGS:", "green"));
    println!("  {}          {}",
        colorize("--explain", "white"),
        colorize("Show step-by-step parsing process", "bright_black"));
    println!("  {}         {}",
        colorize("--validate", "white"),
        colorize("Validation-only mode (no parsing output)", "bright_black"));
    println!("  {}          {}",
        colorize("--inspect", "white"),
        colorize("Show internal data structures", "bright_black"));
    println!("  {}      {}",
        colorize("--format=FORMAT", "white"),
        colorize("Output format: text, json, debug [default: text]", "bright_black"));
    println!("  {}        {}",
        colorize("-v, --verbose", "white"),
        colorize("Verbose output with parsing details", "bright_black"));
    println!("  {}         {}",
        colorize("-h, --help", "white"),
        colorize("Show this help message", "bright_black"));
    println!();

    println!("{}", colorize("EXAMPLES:", "green"));
    println!("  {}", colorize("# Basic parsing", "bright_black"));
    println!("  {} {}",
        colorize("meteor parse", "white"),
        colorize("\"app:ui:button=click\"", "cyan"));
    println!();
    println!("  {}", colorize("# Show parsing steps", "bright_black"));
    println!("  {} {}",
        colorize("meteor parse --explain", "white"),
        colorize("\"list[0,1]=matrix\"", "cyan"));
    println!();
    println!("  {}", colorize("# JSON output with validation", "bright_black"));
    println!("  {} {}",
        colorize("meteor parse --format=json --verbose", "white"),
        colorize("\"ctx:ns:key=value;other=data\"", "cyan"));
    println!();
    println!("  {}", colorize("# Debug internal structures", "bright_black"));
    println!("  {} {}",
        colorize("meteor parse --inspect --format=debug", "white"),
        colorize("\"complex:pattern[0,1]=data\"", "cyan"));
    println!();
    println!("  {}", colorize("# Validate pattern without parsing", "bright_black"));
    println!("  {} {}",
        colorize("meteor parse --validate", "white"),
        colorize("\"potentially:malformed=input\"", "cyan"));
    println!();

    println!("{}", colorize("PATTERN SYNTAX:", "green"));
    println!("  {}       {}",
        colorize("key=value", "white"),
        colorize("Basic key-value pair", "bright_black"));
    println!("  {}  {}",
        colorize("ns:key=value", "white"),
        colorize("Namespaced key-value", "bright_black"));
    println!("  {} {}",
        colorize("ctx:ns:key=value", "white"),
        colorize("Full context addressing", "bright_black"));
    println!("  {}    {}",
        colorize("list[0]=item", "white"),
        colorize("Bracket notation (transforms to list__i_0)", "bright_black"));
    println!("  {} {}",
        colorize("matrix[0,1]=cell", "white"),
        colorize("Multi-dimensional indexing", "bright_black"));
    println!();
}

/// Print JSON format output
fn print_json_output(bucket: &meteor::TokenBucket, _input: &str, _verbose: bool) {
    // TODO: Implement JSON serialization
    // For now, use debug representation
    println!("{{");
    let namespaces = bucket.namespaces();
    for (i, namespace) in namespaces.iter().enumerate() {
        let ns_name = if namespace.is_empty() { "root" } else { namespace };
        println!("  \"{}\": {{", ns_name);

        let keys = bucket.keys_in_namespace(namespace);
        for (j, key) in keys.iter().enumerate() {
            if let Some(value) = bucket.get(namespace, key) {
                print!("    \"{}\": \"{}\"", key, value);
                if j < keys.len() - 1 {
                    println!(",");
                } else {
                    println!();
                }
            }
        }

        print!("  }}");
        if i < namespaces.len() - 1 {
            println!(",");
        } else {
            println!();
        }
    }
    println!("}}");
}

/// Handle the validate command using RSB patterns
fn validate_command(args: Args) -> i32 {
    // Get input from positional args, skipping flags
    let mut input = String::new();

    // Find first non-flag argument as input
    for i in 1..=args.len() {
        let arg = args.get(i);
        if !arg.is_empty() && !arg.starts_with('-') {
            input = arg;
            break;
        }
    }

    if input.is_empty() {
        eprintln!("Error: No input provided");
        eprintln!("Usage: meteor validate [--verbose] <token_string>");
        eprintln!("Example: meteor validate \"app:ui:button=click\"");
        eprintln!("Example: meteor validate \"ctx:ns:key=value;list[0]=item\"");
        return 1;
    }

    // Get options from RSB global context
    let verbose = has_var("opt_verbose") || has_var("opt_v");

    if verbose {
        eprintln!("Validating input: {}", input);
    }

    // Use existing meteor parsing + validation logic
    match meteor::parse(&input) {
        Ok(_bucket) => {
            if verbose {
                println!("✅ Valid meteor token format");
                println!("Input: {}", input);
                println!("Status: All tokens parsed successfully");
            } else {
                println!("✅ Valid meteor format");
            }
            0
        }
        Err(e) => {
            if verbose {
                println!("❌ Invalid meteor token format");
                println!("Input: {}", input);
                println!("Error: {}", e);
                println!();
                show_validation_help();
            } else {
                println!("❌ Invalid meteor format: {}", e);
            }
            1
        }
    }
}

/// Show validation help with correct meteor format patterns
fn show_validation_help() {
    println!("Valid meteor format patterns:");
    println!("  key=value                    Basic key-value pair");
    println!("  ns:key=value                 Namespaced key-value");
    println!("  ctx:ns:key=value             Full context addressing");
    println!("  list[0]=item                 Bracket notation");
    println!("  matrix[0,1]=cell             Multi-dimensional indexing");
    println!("  key1=val1;key2=val2         Multiple tokens");
    println!("  ctx=app;ui:button=click      Context switch with data");
    println!();
    println!("Use 'meteor help validate' for more detailed information.");
}

/// Print debug format output
fn print_debug_output(bucket: &meteor::TokenBucket, input: &str) {
    println!("=== DEBUG: Meteor Token Analysis ===");
    println!("Raw input: {:?}", input);
    println!("Bucket structure: {:#?}", bucket);
    println!("Total tokens: {}", bucket.len());
    println!("Namespaces: {:?}", bucket.namespaces());
}