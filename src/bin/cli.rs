use rsb::prelude::*;

fn main() {
    // Set up RSB CLI context
    let args = Args::new(&std::env::args().collect::<Vec<_>>());

    // Parse command-line options
    options!(&args);

    // Get the command (first non-flag argument)
    let command = args.get(1);

    // Dispatch based on command
    let exit_code = match command.as_str() {
        "parse" => parse_command(args),
        "validate" => validate_command(args),
        "help" | "--help" | "-h" | "" => {
            show_help();
            0
        }
        "inspect" => {
            show_inspect();
            0
        }
        "stack" => {
            show_stack();
            0
        }
        _ => {
            eprintln!("Unknown command: {}", command);
            show_help();
            1
        }
    };

    std::process::exit(exit_code);
}

/// Show help message
fn show_help() {
    println!("\x1b[1m\x1b[34mmeteor\x1b[0m");
    println!();
    println!("\x1b[1mUSAGE:\x1b[0m");
    println!("  meteor <command> [options]");
    println!();
    println!("\x1b[1mCOMMANDS:\x1b[0m");
    println!("  \x1b[36mparse          \x1b[0m Parse meteor token streams");
    println!("  \x1b[36mvalidate       \x1b[0m Validate meteor token format");
    println!();
    println!("\x1b[1mBUILT-IN COMMANDS:\x1b[0m");
    println!("  \x1b[32mhelp           \x1b[0m Show this help message");
    println!("  \x1b[32minspect        \x1b[0m List all available functions");
    println!("  \x1b[32mstack          \x1b[0m Show the current call stack");
}

/// Show inspect (RSB built-in)
fn show_inspect() {
    println!("Available meteor functions:");
    println!("  parse          - Parse token streams");
    println!("  validate       - Validate token format");
    println!("  help          - Show help message");
    println!("  inspect       - List available functions");
    println!("  stack         - Show call stack");
}

/// Show stack (RSB built-in)
fn show_stack() {
    println!("Call stack:");
    println!("  meteor (main)");
}

/// Handle the parse command using RSB patterns
fn parse_command(args: Args) -> i32 {
    // Get input from positional args, skipping flags
    let mut input = String::new();
    let mut format = String::from("text");

    // Parse arguments looking for format and input
    for i in 1..=args.len() {
        let arg = args.get(i);
        if arg.starts_with("--format=") {
            format = arg.strip_prefix("--format=").unwrap_or("text").to_string();
        } else if !arg.is_empty() && !arg.starts_with('-') && arg != "parse" {
            input = arg;
        }
    }

    if input.is_empty() {
        eprintln!("Error: No input provided");
        eprintln!("Usage: meteor parse [--format=FORMAT] <token_string>");
        eprintln!("Example: meteor parse \"app:ui:button=click\"");
        eprintln!("Example: meteor parse \"ctx:ns:key=value;list[0]=item\"");
        return 1;
    }

    // Get options from RSB global context
    let verbose = has_var("opt_verbose") || has_var("opt_v");

    if verbose {
        eprintln!("Parsing input: {}", input);
        eprintln!("Output format: {}", format);
    }

    // Use existing meteor parsing logic
    match meteor::parse(&input) {
        Ok(shower) => {
            print_output(&shower, &input, verbose, &format);
            0
        }
        Err(e) => {
            eprintln!("Parse error: {}", e);
            1
        }
    }
}

/// Print output in the specified format using existing output logic
fn print_output(shower: &meteor::MeteorShower, input: &str, verbose: bool, format: &str) {
    match format {
        "json" => print_json_output(shower, input, verbose),
        "debug" => print_debug_output(shower, input),
        _ => print_text_output(shower, input, verbose),
    }
}

/// Print text format output
/// TODO: Update to use MeteorShower API (TICKET-007)
fn print_text_output(shower: &meteor::MeteorShower, input: &str, verbose: bool) {
    if verbose {
        println!("=== Meteor Token Parse Results ===");
        println!("Input: {}", input);
        println!("MeteorShower contains {} meteors", shower.len());
        println!();
    }

    // Temporary output until MeteorShower API integration complete
    println!("MeteorShower parsed successfully");
    println!("Contains {} meteors across {} contexts",
        shower.len(),
        shower.contexts().len());

    // TODO: Implement proper iteration over meteors
    // for meteor in shower.meteors() {
    //     println!("{}", meteor);
    // }
}

/// Print JSON format output
/// TODO: Update to use MeteorShower API (TICKET-007)
fn print_json_output(shower: &meteor::MeteorShower, _input: &str, _verbose: bool) {
    // Temporary JSON output until MeteorShower API integration complete
    println!("{{");
    println!("  \"meteors\": {},", shower.len());
    println!("  \"contexts\": {:?}", shower.contexts());
    println!("}}");
}

/// Handle the validate command using RSB patterns
fn validate_command(args: Args) -> i32 {
    // Get input from positional args, skipping flags
    let mut input = String::new();

    // Find first non-flag argument as input
    for i in 1..=args.len() {
        let arg = args.get(i);
        if !arg.is_empty() && !arg.starts_with('-') && arg != "validate" {
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
        Ok(_shower) => {
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
/// TODO: Update to use MeteorShower API (TICKET-007)
fn print_debug_output(shower: &meteor::MeteorShower, input: &str) {
    println!("=== DEBUG: Meteor Token Analysis ===");
    println!("Raw input: {:?}", input);
    println!("MeteorShower structure: {:#?}", shower);
    println!("Total meteors: {}", shower.len());
    println!("Contexts: {:?}", shower.contexts());
}