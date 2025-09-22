//! Meteor Admin CLI
//!
//! RSB-compliant command-line interface for meteor token processing.
//! Provides experimental access to meteor's token parsing and processing capabilities.
//! Enhanced with hub::cli_ext for robust argument parsing and validation.

use hub::cli_ext::clap::{Arg, ArgAction, Command};
use std::process;

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

fn build_cli() -> Command {
    Command::new("meteor")
        .about("Meteor Admin CLI - RSB-compliant token processing tool")
        .version(env!("CARGO_PKG_VERSION"))
        .author("oodx contributors")
        .long_about("Advanced token processing with context isolation, bracket notation, and value escaping")
        .subcommand(
            Command::new("parse")
                .about("Parse and display token stream structure")
                .long_about("Parse token strings with context-namespace-key pattern support")
                .arg(
                    Arg::new("input")
                        .help("Token string to parse")
                        .long_help("Token string using meteor syntax: 'ctx=app; ui:button=click'")
                        .required(true)
                        .value_name("TOKEN_STRING")
                )
                .arg(
                    Arg::new("verbose")
                        .short('v')
                        .long("verbose")
                        .help("Show detailed parsing information")
                        .action(ArgAction::SetTrue)
                )
                .arg(
                    Arg::new("format")
                        .short('f')
                        .long("format")
                        .help("Output format")
                        .value_parser(["text", "json", "debug"])
                        .default_value("text")
                        .value_name("FORMAT")
                )
        )
}

fn handle_parse(input: &str, verbose: bool, format: &str) {
    match meteor::parse_token_stream(input) {
        Ok(bucket) => {
            match format {
                "json" => print_json_output(&bucket, input, verbose),
                "debug" => print_debug_output(&bucket, input),
                _ => print_text_output(&bucket, input, verbose),
            }
        }
        Err(e) => {
            eprintln!("Parse error: {}", e);
            process::exit(1);
        }
    }
}

fn print_text_output(bucket: &meteor::TokenBucket, input: &str, verbose: bool) {
    println!("✓ Successfully parsed token stream");

    if verbose {
        println!("Input: {}", input);
    }

    println!("Tokens: {}", bucket.len());
    println!("Context: {}", bucket.context().name());

    let namespaces = bucket.namespaces();
    println!("Namespaces: {}", namespaces.len());

    for namespace in namespaces {
        println!("  • {}", namespace);

        if verbose {
            // Show keys in each namespace
            for (ns, key, value) in bucket.iter() {
                if ns == *namespace {
                    println!("    ├─ {} = {}", key, value);
                }
            }
        }
    }
}

fn print_json_output(bucket: &meteor::TokenBucket, input: &str, verbose: bool) {
    println!("{{");
    println!("  \"status\": \"success\",");
    if verbose {
        println!("  \"input\": \"{}\",", escape_json(input));
    }
    println!("  \"tokens\": {},", bucket.len());
    println!("  \"context\": \"{}\",", bucket.context().name());
    println!("  \"namespaces\": [");

    let namespaces = bucket.namespaces();
    for (i, namespace) in namespaces.iter().enumerate() {
        print!("    \"{}\"", namespace);
        if i < namespaces.len() - 1 {
            println!(",");
        } else {
            println!();
        }
    }
    println!("  ]");
    println!("}}");
}

fn print_debug_output(bucket: &meteor::TokenBucket, input: &str) {
    println!("=== DEBUG OUTPUT ===");
    println!("Input: {:?}", input);
    println!("Parsed tokens: {}", bucket.len());
    println!("Context: {} (type: {:?})", bucket.context().name(), bucket.context());

    println!("\n--- Token Details ---");
    for (i, (namespace, key, value)) in bucket.iter().enumerate() {
        println!("Token {}: {}::{} = {:?}", i, namespace, key, value);
        println!("  Context: {}", bucket.context().name());
        println!("  Namespace: {}", namespace);
        println!("  Key: {}", key);
        println!("  Value: {:?}", value);
    }
}

fn escape_json(s: &str) -> String {
    s.replace('\\', "\\\\").replace('"', "\\\"")
}

