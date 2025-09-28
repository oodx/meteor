use rsb::prelude::*;

fn main() {
    let args = bootstrap!();
    options!(&args);
    dispatch!(&args, {
        "parse" => parse_command, desc: "Parse meteor token streams",
        "validate" => validate_command, desc: "Validate meteor token stream",
        "token" => token_command, desc: "Parse token stream without context",
        "get" => get_command, desc: "Get value by path",
        "list" => list_command, desc: "List keys and values",
        "contexts" => contexts_command, desc: "List all contexts",
        "namespaces" => namespaces_command, desc: "List namespaces in context",
        "set" => set_command, desc: "Set key-value pair",
        "delete" => delete_command, desc: "Delete key by path",
        "history" => history_command, desc: "Show command audit trail",
        "reset" => reset_command, desc: "Reset cursor or clear data"
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

    // Use meteor view APIs instead of direct storage access (CLI-05)
    let contexts = engine.contexts();

    if contexts.is_empty() {
        println!("No meteors parsed");
        return;
    }

    println!();
    println!("=== Parsed Data ===");

    let mut meteor_count = 0;
    let total_contexts = contexts.len();

    // Use meteor view APIs for structured access with workspace ordering
    for context in &contexts {
        let namespaces = engine.namespaces_in_context(context);
        for namespace in namespaces {
            if let Some(view) = engine.namespace_view(context, &namespace) {
                // Use NamespaceView for ordered iteration and metadata
                for (key, value) in view.entries() {
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
    }

    println!(
        "Total: {} meteors across {} contexts",
        meteor_count,
        total_contexts
    );
}

fn print_json_engine_output(engine: &meteor::MeteorEngine, _input: &str, _verbose: bool) {
    // Use meteor view APIs instead of direct storage access (CLI-05)
    let contexts = engine.contexts();

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
    // Use meteor view APIs for structured access with workspace ordering
    for context in &contexts {
        let namespaces = engine.namespaces_in_context(context);
        for namespace in namespaces {
            if let Some(view) = engine.namespace_view(context, &namespace) {
                // Use NamespaceView for ordered iteration
                for (key, value) in view.entries() {
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

fn get_command(args: Args) -> i32 {
    let format_key = get_var("opt_format");
    let format = resolve_format(&format_key);
    let input = collect_input(&args);

    if input.is_empty() {
        eprintln!("Error: No path provided");
        eprintln!("Usage: meteor get [--format=FORMAT] <context:namespace:key>");
        eprintln!("Example: meteor get app:ui:button");
        return 1;
    }

    let engine = meteor::MeteorEngine::new();

    // Need to populate engine first - load from file or require parse first
    // For now, this command only works on empty engine (will return not found)
    // In future, could load from persistent storage

    match engine.get(&input) {
        Some(value) => {
            match format {
                "json" => {
                    println!("{{");
                    println!("  \"path\": \"{}\",", input);
                    println!("  \"value\": \"{}\"", value);
                    println!("}}");
                }
                _ => {
                    println!("{} = {}", input, value);
                }
            }
            0
        }
        None => {
            match format {
                "json" => {
                    println!("{{");
                    println!("  \"path\": \"{}\",", input);
                    println!("  \"found\": false");
                    println!("}}");
                }
                _ => {
                    eprintln!("{} not found", input);
                }
            }
            1
        }
    }
}

fn contexts_command(args: Args) -> i32 {
    let format_key = get_var("opt_format");
    let format = resolve_format(&format_key);
    let _input = collect_input(&args);

    let engine = meteor::MeteorEngine::new();
    let contexts = engine.contexts();

    if contexts.is_empty() {
        match format {
            "json" => println!("[]"),
            _ => println!("No contexts"),
        }
        return 0;
    }

    match format {
        "json" => {
            println!("[");
            for (index, ctx) in contexts.iter().enumerate() {
                if index + 1 < contexts.len() {
                    println!("  \"{}\",", ctx);
                } else {
                    println!("  \"{}\"", ctx);
                }
            }
            println!("]");
        }
        _ => {
            println!("Contexts:");
            for ctx in contexts {
                println!("  {}", ctx);
            }
        }
    }
    0
}

fn namespaces_command(args: Args) -> i32 {
    let format_key = get_var("opt_format");
    let format = resolve_format(&format_key);
    let input = collect_input(&args);

    if input.is_empty() {
        eprintln!("Error: No context provided");
        eprintln!("Usage: meteor namespaces [--format=FORMAT] <context>");
        eprintln!("Example: meteor namespaces app");
        return 1;
    }

    let engine = meteor::MeteorEngine::new();
    let namespaces = engine.namespaces_in_context(&input);

    if namespaces.is_empty() {
        match format {
            "json" => {
                println!("{{");
                println!("  \"context\": \"{}\",", input);
                println!("  \"namespaces\": []");
                println!("}}");
            }
            _ => {
                println!("No namespaces in context '{}'", input);
            }
        }
        return 0;
    }

    match format {
        "json" => {
            println!("{{");
            println!("  \"context\": \"{}\",", input);
            println!("  \"namespaces\": [");
            for (index, ns) in namespaces.iter().enumerate() {
                let display = if ns.is_empty() { "(root)" } else { ns };
                if index + 1 < namespaces.len() {
                    println!("    \"{}\",", display);
                } else {
                    println!("    \"{}\"", display);
                }
            }
            println!("  ]");
            println!("}}");
        }
        _ => {
            println!("Namespaces in '{}':", input);
            for ns in namespaces {
                println!("  {}", if ns.is_empty() { "(root)" } else { &ns });
            }
        }
    }
    0
}

fn list_command(args: Args) -> i32 {
    let format_key = get_var("opt_format");
    let format = resolve_format(&format_key);
    let input = collect_input(&args);

    if input.is_empty() {
        eprintln!("Error: No context provided");
        eprintln!("Usage: meteor list [--format=FORMAT] <context> [namespace]");
        eprintln!("Example: meteor list app");
        eprintln!("Example: meteor list app ui");
        return 1;
    }

    let parts: Vec<&str> = input.split_whitespace().collect();
    let context = parts[0];
    let namespace = if parts.len() > 1 { parts[1] } else { "" };

    let engine = meteor::MeteorEngine::new();
    let storage = engine.storage();
    let entries = storage.get_all_keys_in_namespace(context, namespace);

    if entries.is_empty() {
        match format {
            "json" => {
                println!("{{");
                println!("  \"context\": \"{}\",", context);
                if !namespace.is_empty() {
                    println!("  \"namespace\": \"{}\",", namespace);
                }
                println!("  \"entries\": []");
                println!("}}");
            }
            _ => {
                if namespace.is_empty() {
                    println!("No entries in context '{}'", context);
                } else {
                    println!("No entries in '{}:{}'", context, namespace);
                }
            }
        }
        return 0;
    }

    match format {
        "json" => {
            println!("{{");
            println!("  \"context\": \"{}\",", context);
            if !namespace.is_empty() {
                println!("  \"namespace\": \"{}\",", namespace);
            }
            println!("  \"entries\": [");
            for (index, (key, value)) in entries.iter().enumerate() {
                println!("    {{");
                println!("      \"key\": \"{}\",", key);
                println!("      \"value\": \"{}\"", value);
                if index + 1 < entries.len() {
                    println!("    }},");
                } else {
                    println!("    }}");
                }
            }
            println!("  ]");
            println!("}}");
        }
        _ => {
            if namespace.is_empty() {
                println!("Entries in '{}':", context);
            } else {
                println!("Entries in '{}:{}':", context, namespace);
            }
            for (key, value) in entries {
                println!("  {} = {}", key, value);
            }
        }
    }
    0
}

fn set_command(args: Args) -> i32 {
    let format_key = get_var("opt_format");
    let format = resolve_format(&format_key);
    let dry_run = get_var("opt_dry-run") == "true" || get_var("opt_n") == "true";
    let input = collect_input(&args);

    let parts: Vec<&str> = input.splitn(2, ' ').collect();
    if parts.len() < 2 {
        eprintln!("Error: Missing path or value");
        eprintln!("Usage: meteor set [--dry-run] [--format=FORMAT] <context:namespace:key> <value>");
        eprintln!("Example: meteor set app:ui:button click");
        eprintln!("Example: meteor set --dry-run app:ui:button click");
        return 1;
    }

    let path = parts[0];
    let value = parts[1];

    if dry_run {
        match format {
            "json" => {
                println!("{{");
                println!("  \"dry_run\": true,");
                println!("  \"action\": \"set\",");
                println!("  \"path\": \"{}\",", path);
                println!("  \"value\": \"{}\"", value);
                println!("}}");
            }
            _ => {
                println!("[DRY RUN] Would set: {} = {}", path, value);
            }
        }
        return 0;
    }

    let mut engine = meteor::MeteorEngine::new();
    match engine.set(path, value) {
        Ok(()) => {
            match format {
                "json" => {
                    println!("{{");
                    println!("  \"success\": true,");
                    println!("  \"path\": \"{}\",", path);
                    println!("  \"value\": \"{}\"", value);
                    println!("}}");
                }
                _ => {
                    println!("Set: {} = {}", path, value);
                }
            }
            0
        }
        Err(err) => {
            match format {
                "json" => {
                    println!("{{");
                    println!("  \"success\": false,");
                    println!("  \"error\": \"{}\"", err);
                    println!("}}");
                }
                _ => {
                    eprintln!("Set error: {}", err);
                }
            }
            1
        }
    }
}

fn delete_command(args: Args) -> i32 {
    let format_key = get_var("opt_format");
    let format = resolve_format(&format_key);
    let dry_run = get_var("opt_dry-run") == "true" || get_var("opt_n") == "true";
    let input = collect_input(&args);

    if input.is_empty() {
        eprintln!("Error: No path provided");
        eprintln!("Usage: meteor delete [--dry-run] [--format=FORMAT] <context:namespace:key>");
        eprintln!("Example: meteor delete app:ui:button");
        eprintln!("Example: meteor delete --dry-run app:ui:button");
        return 1;
    }

    if dry_run {
        match format {
            "json" => {
                println!("{{");
                println!("  \"dry_run\": true,");
                println!("  \"action\": \"delete\",");
                println!("  \"path\": \"{}\"", input);
                println!("}}");
            }
            _ => {
                println!("[DRY RUN] Would delete: {}", input);
            }
        }
        return 0;
    }

    let mut engine = meteor::MeteorEngine::new();
    match engine.delete(&input) {
        Ok(true) => {
            match format {
                "json" => {
                    println!("{{");
                    println!("  \"success\": true,");
                    println!("  \"deleted\": true,");
                    println!("  \"path\": \"{}\"", input);
                    println!("}}");
                }
                _ => {
                    println!("Deleted: {}", input);
                }
            }
            0
        }
        Ok(false) => {
            match format {
                "json" => {
                    println!("{{");
                    println!("  \"success\": true,");
                    println!("  \"deleted\": false,");
                    println!("  \"path\": \"{}\"", input);
                    println!("}}");
                }
                _ => {
                    println!("Not found: {}", input);
                }
            }
            1
        }
        Err(err) => {
            match format {
                "json" => {
                    println!("{{");
                    println!("  \"success\": false,");
                    println!("  \"error\": \"{}\"", err);
                    println!("}}");
                }
                _ => {
                    eprintln!("Delete error: {}", err);
                }
            }
            1
        }
    }
}

fn history_command(_args: Args) -> i32 {
    let format_key = get_var("opt_format");
    let format = resolve_format(&format_key);
    let limit_str = get_var("opt_limit");
    let limit: Option<usize> = if limit_str.is_empty() {
        None
    } else {
        limit_str.parse().ok()
    };

    let engine = meteor::MeteorEngine::new();
    let history = engine.command_history();

    let commands_to_show: &[_] = if let Some(n) = limit {
        if n < history.len() {
            &history[history.len() - n..]
        } else {
            history
        }
    } else {
        history
    };

    if commands_to_show.is_empty() {
        match format {
            "json" => println!("[]"),
            _ => println!("No command history"),
        }
        return 0;
    }

    match format {
        "json" => {
            println!("[");
            for (index, cmd) in commands_to_show.iter().enumerate() {
                println!("  {{");
                println!("    \"timestamp\": {},", cmd.timestamp);
                println!("    \"command_type\": \"{}\",", cmd.command_type);
                println!("    \"target\": \"{}\",", cmd.target);
                println!("    \"success\": {}", cmd.success);
                if let Some(ref err) = cmd.error_message {
                    println!("    ,\"error\": \"{}\"", err);
                }
                if index + 1 < commands_to_show.len() {
                    println!("  }},");
                } else {
                    println!("  }}");
                }
            }
            println!("]");
        }
        _ => {
            for cmd in commands_to_show {
                let status = if cmd.success { "✓" } else { "✗" };
                print!("{} [{}] {} {}", status, cmd.timestamp, cmd.command_type, cmd.target);
                if let Some(ref err) = cmd.error_message {
                    print!(" - Error: {}", err);
                }
                println!();
            }
        }
    }

    0
}

fn reset_command(args: Args) -> i32 {
    let format_key = get_var("opt_format");
    let format = resolve_format(&format_key);
    let input = collect_input(&args);

    if input.is_empty() {
        eprintln!("Error: Missing reset target");
        eprintln!("Usage: meteor reset <target>");
        eprintln!("Targets: cursor, storage, all, <context_name>");
        return 1;
    }

    let target = input.trim();
    let mut engine = meteor::MeteorEngine::new();

    let result = match target {
        "cursor" | "storage" | "all" => engine.execute_control_command("reset", target),
        _ => {
            // Treat as context name to delete
            engine.delete(target).map(|deleted| {
                if !deleted {
                    Err(format!("Context '{}' not found", target))
                } else {
                    Ok(())
                }
            }).unwrap_or_else(|e| Err(e))
        }
    };

    match result {
        Ok(()) => {
            match format {
                "json" => {
                    println!("{{");
                    println!("  \"success\": true,");
                    println!("  \"target\": \"{}\"", target);
                    println!("}}");
                }
                _ => {
                    match target {
                        "cursor" => println!("Cursor reset to default (app:main)"),
                        "storage" => println!("All storage cleared"),
                        "all" => println!("Cursor and storage reset"),
                        _ => println!("Context '{}' deleted", target),
                    }
                }
            }
            0
        }
        Err(err) => {
            match format {
                "json" => {
                    println!("{{");
                    println!("  \"success\": false,");
                    println!("  \"error\": \"{}\"", err);
                    println!("}}");
                }
                _ => {
                    eprintln!("Reset error: {}", err);
                }
            }
            1
        }
    }
}
