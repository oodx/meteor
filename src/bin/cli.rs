use rsb::prelude::*;

fn main() {
    // Bootstrap host + CLI environment, returns Args
    let args = bootstrap!();

    // Parse command-line options into global context
    options!(&args);

    // Dispatch to command handlers with built-in help/inspect/stack
    dispatch!(&args, {
        "parse" => parse_command, desc: "Parse meteor token streams",
        "validate" => validate_command, desc: "Validate meteor token format"
    });
}

/// Handle the parse command using RSB patterns
fn parse_command(args: Args) -> i32 {
    // Get flags from global context (set by options!)
    let verbose = get_var("opt_verbose") == "true" || get_var("opt_v") == "true";
    let format = get_var("opt_format");
    let format = if format.is_empty() { "text" } else { &format };

    // Get remaining non-flag arguments and join them with spaces
    let remaining = args.remaining();
    let input = remaining.join(" ");

    if input.is_empty() {
        eprintln!("Error: No input provided");
        eprintln!("Usage: meteor parse [--verbose] [--format=FORMAT] <token_string>");
        eprintln!("Example: meteor parse app:ui:button=click");
        return 1;
    }

    if verbose {
        eprintln!("Parsing input: {}", input);
        eprintln!("Output format: {}", format);
    }

    // Parse the meteor token stream using MeteorShower::parse
    match meteor::MeteorShower::parse(&input) {
        Ok(shower) => {
            print_output(&shower, &input, verbose, format);
            0
        }
        Err(e) => {
            eprintln!("Parse error: {}", e);
            1
        }
    }
}

/// Handle the validate command using RSB patterns
fn validate_command(args: Args) -> i32 {
    // Get flags from global context (set by options!)
    let verbose = get_var("opt_verbose") == "true" || get_var("opt_v") == "true";

    // Get remaining non-flag arguments and join them with spaces
    let remaining = args.remaining();
    let input = remaining.join(" ");

    if input.is_empty() {
        eprintln!("Error: No input provided");
        eprintln!("Usage: meteor validate [--verbose] <token_string>");
        eprintln!("Example: meteor validate app:ui:button=click");
        return 1;
    }

    if verbose {
        eprintln!("Validating input: {}", input);
    }

    // Validate using is_valid_meteor_shower helper
    if meteor::is_valid_meteor_shower(&input) {
        if verbose {
            println!("✅ Valid meteor token format");
            println!("Input: {}", input);
            println!("Status: All tokens parsed successfully");
        } else {
            println!("✅ Valid meteor format");
        }
        0
    } else {
        if verbose {
            // Try to get detailed error by parsing
            match meteor::MeteorShower::parse(&input) {
                Ok(_) => unreachable!(), // Should not happen if is_valid returned false
                Err(e) => {
                    println!("❌ Invalid meteor token format");
                    println!("Input: {}", input);
                    println!("Error: {}", e);
                    println!();
                    show_validation_help();
                }
            }
        } else {
            println!("❌ Invalid meteor format");
        }
        1
    }
}

/// Print output in the specified format
fn print_output(shower: &meteor::MeteorShower, input: &str, verbose: bool, format: &str) {
    match format {
        "json" => print_json_output(shower, input, verbose),
        "debug" => print_debug_output(shower, input),
        _ => print_text_output(shower, input, verbose),
    }
}

/// Print text format output
/// TODO: Update to use full MeteorShower API (TICKET-007)
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

    // TODO: Display actual meteor contents once MeteorShower API is complete
    if verbose {
        println!("Contexts: {:?}", shower.contexts());
    }
}

/// Print JSON format output
/// TODO: Update to use full MeteorShower API (TICKET-007)
fn print_json_output(shower: &meteor::MeteorShower, _input: &str, _verbose: bool) {
    println!("{{");
    println!("  \"meteors\": {},", shower.len());
    println!("  \"contexts\": {:?}", shower.contexts());
    println!("}}");
}

/// Print debug format output
/// TODO: Update to use full MeteorShower API (TICKET-007)
fn print_debug_output(shower: &meteor::MeteorShower, input: &str) {
    println!("=== DEBUG: Meteor Token Analysis ===");
    println!("Raw input: {:?}", input);
    println!("MeteorShower structure: {:#?}", shower);
    println!("Total meteors: {}", shower.len());
    println!("Contexts: {:?}", shower.contexts());
}

/// Show validation help
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
    println!("Use 'meteor help' for more information.");
}