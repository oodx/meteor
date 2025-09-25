use rsb::prelude::*;

fn main() {
    // Bootstrap host + CLI environment, returns Args
    let args = bootstrap!();

    // Parse command-line options into global context
    options!(&args);

    // Dispatch to command handlers with built-in help/inspect/stack
    dispatch!(&args, {
        "parse" => parse_command, desc: "Parse meteor token streams",
        "validate" => validate_command, desc: "Validate meteor token format",
        "token" => token_command, desc: "Parse individual token"
    });
}

/// Handle the parse command using RSB patterns
fn parse_command(args: Args) -> i32 {
    // Get flags from global context (set by options!)
    let verbose = get_var("opt_verbose") == "true" || get_var("opt_v") == "true";
    let format = get_var("opt_format");
    let format = if format.is_empty() { "text" } else { &format };

    // Get remaining non-flag arguments (flags are processed by options! but not removed)
    let remaining = args.remaining();
    let input = remaining
        .into_iter()
        .filter(|arg| !arg.starts_with("--") && !arg.starts_with("-"))
        .collect::<Vec<_>>()
        .join(" ");

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

    // Parse meteor format: context:namespace:token(s) where token(s) can be semicolon-separated
    match meteor::Meteor::parse(&input) {
        Ok(meteors) => {
            print_meteors_output(&meteors, &input, verbose, format);
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

    // Get remaining non-flag arguments (flags are processed by options! but not removed)
    let remaining = args.remaining();
    let input = remaining
        .into_iter()
        .filter(|arg| !arg.starts_with("--") && !arg.starts_with("-"))
        .collect::<Vec<_>>()
        .join(" ");

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
fn print_engine_output(engine: &meteor::MeteorEngine, input: &str, verbose: bool, format: &str) {
    match format {
        "json" => print_json_engine_output(engine, input, verbose),
        "debug" => print_debug_engine_output(engine, input),
        _ => print_text_engine_output(engine, input, verbose),
    }
}

/// Handle the token command using RSB patterns
fn token_command(args: Args) -> i32 {
    // Get flags from global context (set by options!)
    let verbose = get_var("opt_verbose") == "true" || get_var("opt_v") == "true";
    let format = get_var("opt_format");
    let format = if format.is_empty() { "text" } else { &format };

    // Get remaining non-flag arguments (flags are processed by options! but not removed)
    let remaining = args.remaining();
    let input = remaining
        .into_iter()
        .filter(|arg| !arg.starts_with("--") && !arg.starts_with("-"))
        .collect::<Vec<_>>()
        .join(" ");

    if input.is_empty() {
        eprintln!("Error: No input provided");
        eprintln!("Usage: meteor token [--verbose] [--format=FORMAT] <token_string>");
        eprintln!("Example: meteor token button=click");
        return 1;
    }

    if verbose {
        eprintln!("Parsing token: {}", input);
        eprintln!("Output format: {}", format);
    }

    // Parse using Token::parse
    match meteor::Token::parse(&input) {
        Ok(tokens) => {
            print_tokens_output(&tokens, &input, verbose, format);
            0
        }
        Err(e) => {
            eprintln!("Parse error: {}", e);
            1
        }
    }
}

/// Print output for a single meteor in the specified format
fn print_meteor_output(meteor: &meteor::Meteor, input: &str, verbose: bool, format: &str) {
    match format {
        "json" => print_json_meteor_output(meteor, input, verbose),
        "debug" => print_debug_meteor_output(meteor, input),
        _ => print_text_meteor_output(meteor, input, verbose),
    }
}

/// Print output for a meteor shower in the specified format
fn print_shower_output(shower: &meteor::MeteorShower, input: &str, verbose: bool, format: &str) {
    match format {
        "json" => print_json_shower_output(shower, input, verbose),
        "debug" => print_debug_shower_output(shower, input),
        _ => print_text_shower_output(shower, input, verbose),
    }
}

/// Print output for a single token in the specified format
fn print_tokens_output(tokens: &[meteor::Token], input: &str, verbose: bool, format: &str) {
    match format {
        "json" => print_json_tokens_output(tokens, input, verbose),
        "debug" => print_debug_tokens_output(tokens, input),
        _ => print_text_tokens_output(tokens, input, verbose),
    }
}

/// Print text format output showing actual meteor components
fn print_text_engine_output(engine: &meteor::MeteorEngine, input: &str, verbose: bool) {
    if verbose {
        println!("=== Meteor Parse Results ===");
        println!("Input: {}", input);
        println!();
    }

    // Show current engine state
    println!("Current cursor: {}:{}",
        engine.current_context.name(),
        engine.current_namespace.to_string());

    // Get storage data and display meteors
    let storage = engine.storage();
    let contexts = storage.contexts();

    if contexts.is_empty() {
        println!("No meteors parsed");
        return;
    }

    println!();
    println!("=== Parsed Data ===");
    let mut meteor_count = 0;

    for context in &contexts {
        let namespaces = storage.namespaces_in_context(context);
        for namespace in &namespaces {
            let key_value_pairs = storage.get_all_keys_in_namespace(context, namespace);
            for (key, value) in key_value_pairs {
                meteor_count += 1;
                println!("Meteor {}:", meteor_count);
                println!("  Context: {}", context);
                println!("  Namespace: {}", if namespace.is_empty() { "(root)" } else { namespace });
                println!("  Key: {}", key);
                println!("  Value: {}", value);
                println!();
            }
        }
    }

    println!("Total: {} meteors across {} contexts", meteor_count, contexts.len());
}

/// Print JSON format output
fn print_json_engine_output(engine: &meteor::MeteorEngine, _input: &str, _verbose: bool) {
    let storage = engine.storage();
    let contexts = storage.contexts();

    println!("{{");
    println!("  \"cursor\": {{");
    println!("    \"context\": \"{}\",", engine.current_context.name());
    println!("    \"namespace\": \"{}\"", engine.current_namespace.to_string());
    println!("  }},");
    println!("  \"contexts\": {},", contexts.len());
    println!("  \"meteors\": [");

    let mut first = true;
    for context in &contexts {
        let namespaces = storage.namespaces_in_context(context);
        for namespace in &namespaces {
            let key_value_pairs = storage.get_all_keys_in_namespace(context, namespace);
            for (key, value) in key_value_pairs {
                if !first { println!(","); }
                first = false;
                println!("    {{");
                println!("      \"context\": \"{}\",", context);
                println!("      \"namespace\": \"{}\",", namespace);
                println!("      \"key\": \"{}\",", key);
                println!("      \"value\": \"{}\"", value);
                print!("    }}");
            }
        }
    }
    println!();
    println!("  ]");
    println!("}}");
}

/// Print debug format output
fn print_debug_engine_output(engine: &meteor::MeteorEngine, input: &str) {
    println!("=== DEBUG: Meteor Engine Analysis ===");
    println!("Raw input: {:?}", input);
    println!("Engine cursor: {}:{}", engine.current_context.name(), engine.current_namespace.to_string());
    println!("Storage data: {:#?}", engine.storage());
    println!("Command history: {:#?}", engine.command_history());
}

/// Print text format output showing single meteor components
fn print_text_meteor_output(meteor: &meteor::Meteor, input: &str, verbose: bool) {
    if verbose {
        println!("=== Meteor Parse Results ===");
        println!("Input: {}", input);
        println!();
    }

    println!("=== Parsed Meteor ===");
    println!("Context: {}", meteor.context().name());
    let namespace_str = meteor.namespace().to_string();
    println!("Namespace: {}", namespace_str);

    let tokens = meteor.tokens();
    println!("Tokens: {}", tokens.len());

    for (i, token) in tokens.iter().enumerate() {
        println!("  Token {}:", i + 1);
        println!("    Key: {}", token.key().to_string());
        println!("    Value: {}", token.value());
        if verbose {
            println!("    Key (transformed): {}", token.key().transformed());
        }
    }

    if verbose {
        println!();
        println!("Full format: {}", meteor);
    }
}

/// Print JSON format output for single meteor
fn print_json_meteor_output(meteor: &meteor::Meteor, _input: &str, _verbose: bool) {
    println!("{{");
    println!("  \"context\": \"{}\",", meteor.context().name());
    println!("  \"namespace\": \"{}\",", meteor.namespace().to_string());
    println!("  \"tokens\": [");

    let tokens = meteor.tokens();
    for (i, token) in tokens.iter().enumerate() {
        if i > 0 { println!(","); }
        print!("    {{");
        print!("\"key\": \"{}\", ", token.key().to_string());
        print!("\"key_transformed\": \"{}\", ", token.key().transformed());
        print!("\"value\": \"{}\"", token.value());
        print!("}}");
    }
    println!();
    println!("  ]");
    println!("}}");
}

/// Print debug format output for single meteor
fn print_debug_meteor_output(meteor: &meteor::Meteor, input: &str) {
    println!("=== DEBUG: Single Meteor Analysis ===");
    println!("Raw input: {:?}", input);
    println!("Parsed format: {}", meteor);
    println!("Components:");
    println!("  Context: {:?}", meteor.context());
    println!("  Namespace: {:?}", meteor.namespace());
    println!("  Tokens: {:?}", meteor.tokens());
}

/// Print text format output showing meteor shower components
fn print_text_shower_output(shower: &meteor::MeteorShower, input: &str, verbose: bool) {
    if verbose {
        println!("=== Meteor Shower Parse Results ===");
        println!("Input: {}", input);
        println!();
    }

    let meteors = shower.meteors();
    if meteors.is_empty() {
        println!("No meteors parsed");
        return;
    }

    println!("=== Parsed Meteors ===");
    for (i, meteor) in meteors.iter().enumerate() {
        println!("Meteor {}:", i + 1);
        println!("  Context: {}", meteor.context().name());
        println!("  Namespace: {}", meteor.namespace().to_string());
        println!("  Key: {}", meteor.token().key().to_string());
        println!("  Value: {}", meteor.token().value());
        if verbose {
            println!("  Key (transformed): {}", meteor.token().key().transformed());
            println!("  Full format: {}", meteor);
        }
        println!();
    }

    println!("Total: {} meteors", meteors.len());
}

/// Print JSON format output for meteor shower
fn print_json_shower_output(shower: &meteor::MeteorShower, _input: &str, _verbose: bool) {
    let meteors = shower.meteors();

    println!("{{");
    println!("  \"meteors\": [");

    for (i, meteor) in meteors.iter().enumerate() {
        if i > 0 { println!(","); }
        print!("    {{");
        print!("\"context\": \"{}\", ", meteor.context().name());
        print!("\"namespace\": \"{}\", ", meteor.namespace().to_string());
        print!("\"key\": \"{}\", ", meteor.token().key().to_string());
        print!("\"key_transformed\": \"{}\", ", meteor.token().key().transformed());
        print!("\"value\": \"{}\"", meteor.token().value());
        print!("}}");
    }
    println!();
    println!("  ],");
    println!("  \"total\": {}", meteors.len());
    println!("}}");
}

/// Print debug format output for meteor shower
fn print_debug_shower_output(shower: &meteor::MeteorShower, input: &str) {
    println!("=== DEBUG: Meteor Shower Analysis ===");
    println!("Raw input: {:?}", input);
    println!("Total meteors: {}", shower.meteors().len());
    println!("Meteors: {:#?}", shower.meteors());
}

/// Parse meteor format with potential multiple tokens: context:namespace:token1=val1;token2=val2
fn parse_meteor_with_multiple_tokens(input: &str) -> Result<Vec<meteor::Meteor>, String> {
    // Count colons to determine format
    let colon_count = input.chars().filter(|&c| c == ':').count();

    match colon_count {
        0 => {
            // Simple token format: key=value or key1=val1;key2=val2
            // Split by semicolon and create meteors with default context/namespace
            let tokens = input.split(';').map(|s| s.trim()).filter(|s| !s.is_empty());
            let mut meteors = Vec::new();

            for token_str in tokens {
                let meteor = meteor::Meteor::first(token_str)?;
                meteors.push(meteor);
            }
            Ok(meteors)
        }
        1 => {
            // Format: namespace:key=value or namespace:key1=val1;key2=val2
            let parts: Vec<&str> = input.splitn(2, ':').collect();
            let namespace_part = parts[0];
            let tokens_part = parts[1];

            let tokens = tokens_part.split(';').map(|s| s.trim()).filter(|s| !s.is_empty());
            let mut meteors = Vec::new();

            for token_str in tokens {
                let full_meteor = format!("{}:{}", namespace_part, token_str);
                let meteor = meteor::Meteor::first(&full_meteor)?;
                meteors.push(meteor);
            }
            Ok(meteors)
        }
        2 => {
            // Full format: context:namespace:key=value or context:namespace:key1=val1;key2=val2
            let parts: Vec<&str> = input.splitn(3, ':').collect();
            let context_part = parts[0];
            let namespace_part = parts[1];
            let tokens_part = parts[2];

            let tokens = tokens_part.split(';').map(|s| s.trim()).filter(|s| !s.is_empty());
            let mut meteors = Vec::new();

            for token_str in tokens {
                let full_meteor = format!("{}:{}:{}", context_part, namespace_part, token_str);
                let meteor = meteor::Meteor::first(&full_meteor)?;
                meteors.push(meteor);
            }
            Ok(meteors)
        }
        _ => {
            Err(format!("Too many colons in meteor format: {}", input))
        }
    }
}

/// Print output for multiple meteors
fn print_meteors_output(meteors: &[meteor::Meteor], input: &str, verbose: bool, format: &str) {
    if meteors.len() == 1 {
        // Single meteor - use single meteor output
        print_meteor_output(&meteors[0], input, verbose, format);
    } else {
        // Multiple meteors - use multi-meteor output
        match format {
            "json" => print_json_meteors_output(meteors, input, verbose),
            "debug" => print_debug_meteors_output(meteors, input),
            _ => print_text_meteors_output(meteors, input, verbose),
        }
    }
}

/// Print text format output for multiple meteors
fn print_text_meteors_output(meteors: &[meteor::Meteor], input: &str, verbose: bool) {
    if verbose {
        println!("=== Meteor Parse Results ===");
        println!("Input: {}", input);
        println!();
    }

    println!("=== Parsed Meteors ===");
    for (i, meteor) in meteors.iter().enumerate() {
        println!("Meteor {}:", i + 1);
        println!("  Context: {}", meteor.context().name());
        println!("  Namespace: {}", meteor.namespace().to_string());
        println!("  Key: {}", meteor.token().key().to_string());
        println!("  Value: {}", meteor.token().value());
        if verbose {
            println!("  Key (transformed): {}", meteor.token().key().transformed());
            println!("  Full format: {}", meteor);
        }
        println!();
    }

    println!("Total: {} meteors", meteors.len());
}

/// Print JSON format output for multiple meteors
fn print_json_meteors_output(meteors: &[meteor::Meteor], _input: &str, _verbose: bool) {
    println!("{{");
    println!("  \"meteors\": [");

    for (i, meteor) in meteors.iter().enumerate() {
        if i > 0 { println!(","); }
        print!("    {{");
        print!("\"context\": \"{}\", ", meteor.context().name());
        print!("\"namespace\": \"{}\", ", meteor.namespace().to_string());
        print!("\"key\": \"{}\", ", meteor.token().key().to_string());
        print!("\"key_transformed\": \"{}\", ", meteor.token().key().transformed());
        print!("\"value\": \"{}\"", meteor.token().value());
        print!("}}");
    }
    println!();
    println!("  ],");
    println!("  \"total\": {}", meteors.len());
    println!("}}");
}

/// Print debug format output for multiple meteors
fn print_debug_meteors_output(meteors: &[meteor::Meteor], input: &str) {
    println!("=== DEBUG: Multiple Meteors Analysis ===");
    println!("Raw input: {:?}", input);
    println!("Total meteors: {}", meteors.len());
    for (i, meteor) in meteors.iter().enumerate() {
        println!("Meteor {}: {:?}", i + 1, meteor);
    }
}

/// Print text format output for single token
fn print_text_tokens_output(tokens: &[meteor::Token], input: &str, verbose: bool) {
    if verbose {
        println!("=== Token Parse Results ===");
        println!("Input: {}", input);
        println!("Found {} token(s)", tokens.len());
        println!();
    }

    for (i, token) in tokens.iter().enumerate() {
        if tokens.len() > 1 {
            println!("=== Token {} ===", i + 1);
        } else {
            println!("=== Parsed Token ===");
        }

        match token.namespace() {
            Some(namespace) => println!("Namespace: {}", namespace.to_string()),
            None => println!("Namespace: (none)"),
        }
        println!("Key: {}", token.key().to_string());
        println!("Value: {}", token.value());

        if verbose {
            println!("Key (transformed): {}", token.key().transformed());
            println!("Full format: {}", token);
        }

        if i < tokens.len() - 1 {
            println!();
        }
    }
}

/// Print JSON format output for tokens
fn print_json_tokens_output(tokens: &[meteor::Token], _input: &str, _verbose: bool) {
    println!("[");
    for (i, token) in tokens.iter().enumerate() {
        println!("  {{");
        match token.namespace() {
            Some(namespace) => println!("    \"namespace\": \"{}\",", namespace.to_string()),
            None => println!("    \"namespace\": null,"),
        }
        println!("    \"key\": \"{}\",", token.key().to_string());
        println!("    \"key_transformed\": \"{}\",", token.key().transformed());
        println!("    \"value\": \"{}\"", token.value());
        if i < tokens.len() - 1 {
            println!("  }},");
        } else {
            println!("  }}");
        }
    }
    println!("]");
}

/// Print debug format output for tokens
fn print_debug_tokens_output(tokens: &[meteor::Token], input: &str) {
    println!("=== DEBUG: Tokens Analysis ===");
    println!("Raw input: {:?}", input);
    println!("Found {} token(s)", tokens.len());
    println!();

    for (i, token) in tokens.iter().enumerate() {
        println!("--- Token {} ---", i + 1);
        println!("Parsed format: {}", token);
        println!("Components:");
        println!("  Namespace: {:?}", token.namespace());
        println!("  Key: {:?}", token.key());
        println!("  Value: {:?}", token.value());
        if i < tokens.len() - 1 {
            println!();
        }
    }
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