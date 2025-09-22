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
        "parse" => parse_command, desc: "Parse meteor token streams"
    });
}

/// Handle the parse command using RSB patterns
fn parse_command(args: Args) -> i32 {
    // Get input from positional args (1-indexed, skips argv[0])
    let input = args.get_or(1, "");
    if input.is_empty() {
        eprintln!("Error: No input provided");
        eprintln!("Usage: meteor parse <token_string>");
        eprintln!("Example: meteor parse \"app:ui:button=click\"");
        return 1;
    }

    // Get options from RSB global context (set by options! macro)
    let verbose = has_var("opt_verbose");
    let format = get_var("opt_format");
    let format = if format.is_empty() { "text" } else { &format };

    if verbose {
        eprintln!("Parsing input: {}", input);
        eprintln!("Output format: {}", format);
    }

    // Use existing meteor parsing logic
    match meteor::parse_token_stream(&input) {
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

/// Print debug format output
fn print_debug_output(bucket: &meteor::TokenBucket, input: &str) {
    println!("=== DEBUG: Meteor Token Analysis ===");
    println!("Raw input: {:?}", input);
    println!("Bucket structure: {:#?}", bucket);
    println!("Total tokens: {}", bucket.len());
    println!("Namespaces: {:?}", bucket.namespaces());
}