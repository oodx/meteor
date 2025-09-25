use rsb::prelude::*;

fn main() {
    let args = bootstrap!();
    options!(&args);
    dispatch!(&args, {
        "parse" => parse_command, desc: "Parse meteor token streams",
        "validate" => validate_command, desc: "Validate meteor token stream",
        "token" => token_command, desc: "Parse token stream without context"
    });
}

fn parse_command(args: Args) -> i32 {
    let verbose = verbose_enabled();
    let format_key = get_var("opt_format");
    let format = resolve_format(&format_key);

    let input = collect_input(&args);
    if input.is_empty() {
        eprintln!("Error: No input provided");
        eprintln!("Usage: meteor parse [--verbose] [--format=FORMAT] <meteor_stream>");
        eprintln!("Example: meteor parse \"app:ui:button=click :;: user:settings:theme=dark\"");
        return 1;
    }

    if verbose {
        eprintln!("Parsing input: {}", input);
        eprintln!("Output format: {}", format);
    }

    let mut engine = meteor::MeteorEngine::new();
    if let Err(err) = meteor::MeteorStreamParser::process(&mut engine, &input) {
        eprintln!("Parse error: {}", err);
        return 1;
    }

    print_engine_output(&engine, &input, verbose, format);
    0
}

fn validate_command(args: Args) -> i32 {
    let verbose = verbose_enabled();
    let input = collect_input(&args);

    if input.is_empty() {
        eprintln!("Error: No input provided");
        eprintln!("Usage: meteor validate [--verbose] <meteor_stream>");
        eprintln!("Example: meteor validate \"app:ui:button=click :;: user:settings:theme=dark\"");
        return 1;
    }

    if verbose {
        eprintln!("Validating input: {}", input);
    }

    match meteor::MeteorStreamParser::validate(&input) {
        Ok(()) => {
            if verbose {
                let segments = meteor::MeteorStreamParser::smart_split(&input);
                println!("✅ Valid meteor format");
                println!("Segments parsed: {}", segments.len());
            } else {
                println!("✅ Valid meteor format");
            }
            0
        }
        Err(err) => {
            if verbose {
                println!("❌ Invalid meteor format");
                println!("Error: {}", err);
                println!();
                show_validation_help();
            } else {
                println!("❌ Invalid meteor format");
            }
            1
        }
    }
}

fn token_command(args: Args) -> i32 {
    let verbose = verbose_enabled();
    let format_key = get_var("opt_format");
    let format = resolve_format(&format_key);

    let input = collect_input(&args);
    if input.is_empty() {
        eprintln!("Error: No input provided");
        eprintln!("Usage: meteor token [--verbose] [--format=FORMAT] <token_stream>");
        eprintln!("Example: meteor token \"profile=name;role=admin\"");
        return 1;
    }

    if verbose {
        eprintln!("Parsing token stream: {}", input);
        eprintln!("Output format: {}", format);
    }

    match meteor::Token::parse(&input) {
        Ok(tokens) => {
            print_tokens_output(&tokens, format, verbose);
            0
        }
        Err(err) => {
            eprintln!("Parse error: {}", err);
            1
        }
    }
}

fn verbose_enabled() -> bool {
    if get_var("opt_verbose") == "true" {
        true
    } else {
        get_var("opt_v") == "true"
    }
}

fn resolve_format(raw: &str) -> &'static str {
    if raw.eq_ignore_ascii_case("json") {
        "json"
    } else if raw.eq_ignore_ascii_case("debug") {
        "debug"
    } else {
        "text"
    }
}

fn collect_input(args: &Args) -> String {
    args.remaining()
        .into_iter()
        .filter(|arg| !arg.starts_with("--") && !arg.starts_with('-'))
        .collect::<Vec<_>>()
        .join(" ")
        .trim()
        .to_string()
}

fn print_engine_output(engine: &meteor::MeteorEngine, input: &str, verbose: bool, format: &str) {
    match format {
        "json" => print_json_engine_output(engine, input, verbose),
        "debug" => print_debug_engine_output(engine, input),
        _ => print_text_engine_output(engine, input, verbose),
    }
}

fn print_text_engine_output(engine: &meteor::MeteorEngine, input: &str, verbose: bool) {
    if verbose {
        println!("=== Meteor Parse Results ===");
        println!("Input: {}", input);
        println!();
    }

    println!(
        "Current cursor: {}:{}",
        engine.current_context.name(),
        engine.current_namespace.to_string()
    );

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
        for namespace in namespaces {
            for (key, value) in storage.get_all_keys_in_namespace(context, &namespace) {
                meteor_count += 1;
                println!("Meteor {}:", meteor_count);
                println!("  Context: {}", context);
                if namespace.is_empty() {
                    println!("  Namespace: (root)");
                } else {
                    println!("  Namespace: {}", namespace);
                }
                println!("  Key: {}", key);
                println!("  Value: {}", value);
                println!();
            }
        }
    }

    println!(
        "Total: {} meteors across {} contexts",
        meteor_count,
        contexts.len()
    );
}

fn print_json_engine_output(engine: &meteor::MeteorEngine, _input: &str, _verbose: bool) {
    let storage = engine.storage();
    let contexts = storage.contexts();

    println!("{{");
    println!("  \"cursor\": {{");
    println!("    \"context\": \"{}\",", engine.current_context.name());
    println!(
        "    \"namespace\": \"{}\"",
        engine.current_namespace.to_string()
    );
    println!("  }},");
    println!("  \"contexts\": {},", contexts.len());
    println!("  \"meteors\": [");

    let mut first = true;
    for context in &contexts {
        let namespaces = storage.namespaces_in_context(context);
        for namespace in namespaces {
            for (key, value) in storage.get_all_keys_in_namespace(context, &namespace) {
                if !first {
                    println!(",");
                }
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

    if !first {
        println!();
    }
    println!("  ]");
    println!("}}");
}

fn print_debug_engine_output(engine: &meteor::MeteorEngine, input: &str) {
    println!("=== DEBUG: Meteor Engine Analysis ===");
    println!("Raw input: {:?}", input);
    println!(
        "Engine cursor: {}:{}",
        engine.current_context.name(),
        engine.current_namespace.to_string()
    );
    println!("Storage data: {:#?}", engine.storage());
    println!("Command history: {:#?}", engine.command_history());
}

fn print_tokens_output(tokens: &[meteor::Token], format: &str, verbose: bool) {
    match format {
        "json" => print_json_tokens_output(tokens),
        "debug" => print_debug_tokens_output(tokens),
        _ => print_text_tokens_output(tokens, verbose),
    }
}

fn print_text_tokens_output(tokens: &[meteor::Token], verbose: bool) {
    if tokens.is_empty() {
        println!("No tokens parsed");
        return;
    }

    if verbose {
        println!("Parsed {} token(s)", tokens.len());
        println!();
    }

    for (index, token) in tokens.iter().enumerate() {
        if tokens.len() > 1 {
            println!("Token {}:", index + 1);
        } else {
            println!("Parsed Token:");
        }

        match token.namespace() {
            Some(namespace) => println!("  Namespace: {}", namespace.to_string()),
            None => println!("  Namespace: (none)"),
        }
        println!("  Key: {}", token.key().to_string());
        println!("  Value: {}", token.value());

        if verbose {
            println!("  Key (transformed): {}", token.key().transformed());
            println!("  Full format: {}", token);
        }

        if index + 1 < tokens.len() {
            println!();
        }
    }
}

fn print_json_tokens_output(tokens: &[meteor::Token]) {
    println!("[");
    for (index, token) in tokens.iter().enumerate() {
        println!("  {{");
        match token.namespace() {
            Some(namespace) => println!("    \"namespace\": \"{}\",", namespace.to_string()),
            None => println!("    \"namespace\": null,"),
        }
        println!("    \"key\": \"{}\",", token.key().to_string());
        println!(
            "    \"key_transformed\": \"{}\",",
            token.key().transformed()
        );
        println!("    \"value\": \"{}\"", token.value());
        if index + 1 < tokens.len() {
            println!("  }},");
        } else {
            println!("  }}");
        }
    }
    println!("]");
}

fn print_debug_tokens_output(tokens: &[meteor::Token]) {
    println!("{:#?}", tokens);
}

fn show_validation_help() {
    println!("Valid meteor format patterns:");
    println!("  key=value                    Basic key-value pair");
    println!("  ns:key=value                 Namespaced key-value");
    println!("  ctx:ns:key=value             Full context addressing");
    println!("  list[0]=item                 Bracket notation");
    println!("  key1=val1;key2=val2         Multiple tokens");
    println!("  ctx=app;ui:button=click      Context switch with data");
    println!("  Use ':;:' to separate meteors in a stream");
    println!();
    println!("Example: meteor parse \"app:ui:button=click :;: user:settings:theme=dark\"");
}
