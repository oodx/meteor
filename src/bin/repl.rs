use rsb::prelude::*;
use std::io::{self, Write};

const MEM_CONTEXT: &str = "_mem";
const MEM_NAMESPACE: &str = "scratch";

fn main() {
    let args = bootstrap!();
    options!(&args);

    println!("🌠 Meteor REPL – type 'help' for commands, 'exit' to quit");

    let mut engine = meteor::MeteorEngine::new();
    run_repl(&mut engine);
}

fn run_repl(engine: &mut meteor::MeteorEngine) {
    let mut buffer = String::new();

    loop {
        buffer.clear();
        print!("meteor> ");
        if io::stdout().flush().is_err() {
            println!("(io error)");
            continue;
        }

        match io::stdin().read_line(&mut buffer) {
            Ok(0) => {
                println!("bye");
                break;
            }
            Ok(_) => {
                let line = buffer.trim();
                if line.is_empty() {
                    continue;
                }

                if matches!(line, "exit" | "quit") {
                    println!("bye");
                    break;
                }

                if line == "help" {
                    print_help();
                    continue;
                }

                let mut parts = line.splitn(2, ' ');
                // SAFETY: splitn(2, ' ') on non-empty string always yields at least one element
                let command = parts.next().expect("splitn always yields at least one element");
                let rest = parts.next().unwrap_or("").trim();

                match command {
                    "parse" => handle_parse(engine, rest),
                    "validate" => handle_validate(rest),
                    "token" => handle_token(rest),
                    "set" => handle_set(engine, rest),
                    "get" => handle_get(engine, rest),
                    "delete" => handle_delete(engine, rest),
                    "contexts" => handle_contexts(engine),
                    "namespaces" => handle_namespaces(engine, rest),
                    "list" => handle_list(engine, rest),
                    "mem" => handle_mem(engine, rest),
                    "load" => handle_load(engine, rest),
                    "dump" => render_engine(engine),
                    "help" => print_help(),
                    "show" => render_engine(engine),
                    other => println!("Unknown command '{}'. Try 'help'.", other),
                }
            }
            Err(err) => {
                println!("Read error: {}", err);
            }
        }
    }
}

fn handle_parse(engine: &mut meteor::MeteorEngine, input: &str) {
    if input.is_empty() {
        println!("Usage: parse <meteor_stream>");
        return;
    }

    match meteor::MeteorStreamParser::process(engine, input) {
        Ok(()) => {
            let segments = meteor::MeteorStreamParser::smart_split(input);
            println!("Parsed {} segment(s)", segments.len());
            render_engine(engine);
        }
        Err(err) => println!("Parse error: {}", err),
    }
}

fn handle_validate(input: &str) {
    if input.is_empty() {
        println!("Usage: validate <meteor_stream>");
        return;
    }

    match meteor::MeteorStreamParser::validate(input) {
        Ok(()) => {
            let segments = meteor::MeteorStreamParser::smart_split(input);
            println!(
                "✅ Valid meteor stream ({} segment{})",
                segments.len(),
                if segments.len() == 1 { "" } else { "s" }
            );
        }
        Err(err) => println!("❌ Invalid stream: {}", err),
    }
}

fn handle_token(input: &str) {
    if input.is_empty() {
        println!("Usage: token <token_stream>");
        return;
    }

    match meteor::Token::parse(input) {
        Ok(tokens) => {
            if tokens.is_empty() {
                println!("No tokens parsed");
                return;
            }
            println!("Parsed {} token(s):", tokens.len());
            for (index, token) in tokens.iter().enumerate() {
                println!("Token {}:", index + 1);
                if let Some(namespace) = token.namespace() {
                    println!("  Namespace: {}", namespace.to_string());
                }
                println!("  Key: {}", token.key().to_string());
                println!("  Value: {}", token.value());
            }
        }
        Err(err) => println!("Parse error: {}", err),
    }
}

fn handle_set(engine: &mut meteor::MeteorEngine, input: &str) {
    let mut parts = input.splitn(2, ' ');
    let path = parts.next().unwrap_or("");
    let value = parts.next().unwrap_or("");

    if path.is_empty() || value.is_empty() {
        println!("Usage: set <context:namespace:key> <value>");
        return;
    }

    match engine.set(path, value) {
        Ok(()) => println!("Stored '{}'.", path),
        Err(err) => println!("Set error: {}", err),
    }
}

fn handle_get(engine: &meteor::MeteorEngine, path: &str) {
    if path.is_empty() {
        println!("Usage: get <context:namespace:key>");
        return;
    }

    match engine.get(path) {
        Some(value) => println!("{} = {}", path, value),
        None => println!("{} not found", path),
    }
}

fn handle_delete(engine: &mut meteor::MeteorEngine, path: &str) {
    if path.is_empty() {
        println!("Usage: delete <context:namespace:key>");
        return;
    }

    match engine.delete(path) {
        Ok(true) => println!("Removed {}", path),
        Ok(false) => println!("Nothing removed"),
        Err(err) => println!("Delete error: {}", err),
    }
}

fn handle_contexts(engine: &meteor::MeteorEngine) {
    let contexts = engine.contexts();
    if contexts.is_empty() {
        println!("No contexts");
        return;
    }
    println!("Contexts:");
    for ctx in contexts {
        println!("  {}", ctx);
    }
}

fn handle_namespaces(engine: &meteor::MeteorEngine, input: &str) {
    let context = input.trim();
    if context.is_empty() {
        println!("Usage: namespaces <context>");
        return;
    }

    let namespaces = engine.namespaces_in_context(context);
    if namespaces.is_empty() {
        println!("No namespaces for {}", context);
        return;
    }

    println!("Namespaces in {}:", context);
    for ns in namespaces {
        println!("  {}", if ns.is_empty() { "(root)".into() } else { ns });
    }
}

fn handle_list(engine: &meteor::MeteorEngine, input: &str) {
    let mut parts = input.split_whitespace();
    let context = match parts.next() {
        Some(ctx) => ctx,
        None => {
            println!("Usage: list <context> [namespace]");
            return;
        }
    };

    let namespace = parts.next().unwrap_or("");
    let storage = engine.storage();

    let entries = storage.get_all_keys_in_namespace(context, namespace);
    if entries.is_empty() {
        println!(
            "No entries for {}:{}",
            context,
            if namespace.is_empty() {
                "(root)"
            } else {
                namespace
            }
        );
        return;
    }

    println!(
        "Entries for {}:{}:",
        context,
        if namespace.is_empty() {
            "(root)"
        } else {
            namespace
        }
    );
    for (key, value) in entries {
        println!("  {} = {}", key, value);
    }
}

fn render_engine(engine: &meteor::MeteorEngine) {
    let storage = engine.storage();
    let contexts = storage.contexts();

    if contexts.is_empty() {
        println!("(engine empty)");
        return;
    }

    println!("=== Meteor Engine State ===");
    println!(
        "Cursor: {}:{}",
        engine.current_context.name(),
        engine.current_namespace.to_string()
    );
    println!();

    for context in contexts {
        if context == MEM_CONTEXT {
            println!("Context: {} (scratch)", context);
        } else {
            println!("Context: {}", context);
        }
        let namespaces = storage.namespaces_in_context(&context);
        for namespace in namespaces {
            let entries = storage.get_all_keys_in_namespace(&context, &namespace);
            let display_ns = if namespace.is_empty() {
                "(root)"
            } else {
                &namespace
            };
            println!("  Namespace: {}", display_ns);
            for (key, value) in entries {
                println!("    {} = {}", key, value);
            }
        }
        println!();
    }
}

fn print_help() {
    println!("Available commands:");
    println!("  parse <stream>            - parse meteor stream into engine");
    println!("  validate <stream>         - validate meteor stream without storing");
    println!("  token <stream>            - parse token stream (no context)");
    println!("  set <path> <value>        - store value at context:namespace:key");
    println!("  get <path>                - read value at context:namespace:key");
    println!("  delete <path>             - remove value at context:namespace:key");
    println!("  contexts                  - list contexts");
    println!("  namespaces <context>      - list namespaces in context");
    println!("  list <context> [ns]       - list key/value pairs for context/namespace");
    println!("  mem <cmd> [...]           - scratch-pad helpers (see 'mem help')");
    println!("  load <name>               - edit scratch entry (alias for 'mem edit')");
    println!("  dump | show               - print complete engine state");
    println!("  help                      - this help message");
    println!("  exit | quit               - leave the REPL");
}

fn handle_mem(engine: &mut meteor::MeteorEngine, input: &str) {
    let trimmed = input.trim();
    if trimmed.is_empty() || trimmed == "help" {
        println!("mem commands:");
        println!("  mem help                 - show this help");
        println!("  mem list                 - list scratch entries");
        println!("  mem set <name> <value>   - store value in scratch pad");
        println!("  mem get <name>           - show value for scratch entry");
        println!("  mem edit <name>          - interactively edit entry");
        println!("  mem delete <name>        - remove scratch entry");
        println!("  load <name>              - alias for 'mem edit'");
        return;
    }

    if trimmed == "list" {
        mem_list(engine);
        return;
    }

    if let Some(rest) = trimmed.strip_prefix("set") {
        if let Some((name, value)) = split_name_value(rest) {
            mem_set(engine, name, value);
        } else {
            println!("Usage: mem set <name> <value>");
        }
        return;
    }

    if let Some(rest) = trimmed.strip_prefix("get") {
        let name = rest.trim();
        if name.is_empty() {
            println!("Usage: mem get <name>");
        } else {
            mem_get(engine, name);
        }
        return;
    }

    if let Some(rest) = trimmed.strip_prefix("edit") {
        let name = rest.trim();
        if name.is_empty() {
            println!("Usage: mem edit <name>");
        } else {
            mem_edit(engine, name);
        }
        return;
    }

    if let Some(rest) = trimmed.strip_prefix("delete") {
        let name = rest.trim();
        if name.is_empty() {
            println!("Usage: mem delete <name>");
        } else {
            mem_delete(engine, name);
        }
        return;
    }

    println!(
        "Unknown mem subcommand '{}'. Try 'mem help'.",
        trimmed.split_whitespace().next().unwrap_or(trimmed)
    );
}

fn handle_load(engine: &mut meteor::MeteorEngine, name: &str) {
    if name.trim().is_empty() {
        println!("Usage: load <name>");
        return;
    }
    mem_edit(engine, name);
}

fn mem_slot_name(raw: &str) -> Option<String> {
    let trimmed = raw.trim();
    if trimmed.is_empty() {
        return None;
    }
    let without_prefix = trimmed.trim_start_matches('$');
    let cleaned = without_prefix
        .chars()
        .map(|c| if c.is_whitespace() { '_' } else { c })
        .collect::<String>();
    if cleaned.is_empty() {
        None
    } else {
        Some(cleaned)
    }
}

fn mem_path(slot: &str) -> String {
    format!("{}:{}:{}", MEM_CONTEXT, MEM_NAMESPACE, slot)
}

fn mem_set(engine: &mut meteor::MeteorEngine, raw_name: &str, value: &str) {
    let Some(slot) = mem_slot_name(raw_name) else {
        println!("Invalid name");
        return;
    };
    let path = mem_path(&slot);
    if let Err(err) = engine.set(&path, value) {
        println!("Set error: {}", err);
    } else {
        println!("Stored ${}", slot);
    }
}

fn mem_get(engine: &meteor::MeteorEngine, raw_name: &str) {
    let Some(slot) = mem_slot_name(raw_name) else {
        println!("Invalid name");
        return;
    };
    let path = mem_path(&slot);
    match engine.get(&path) {
        Some(value) => println!("${} = {}", slot, value),
        None => println!("${} is empty", slot),
    }
}

fn mem_edit(engine: &mut meteor::MeteorEngine, raw_name: &str) {
    let Some(slot) = mem_slot_name(raw_name) else {
        println!("Invalid name");
        return;
    };
    let path = mem_path(&slot);
    let current = engine.get(&path).unwrap_or("");
    println!("Editing ${} (current value: {})", slot, current);
    print!("Enter new value (leave blank to keep): ");
    if io::stdout().flush().is_err() {
        println!("(io error)");
        return;
    }
    let mut input = String::new();
    match io::stdin().read_line(&mut input) {
        Ok(_) => {
            let new_value = input.trim_end();
            if new_value.is_empty() {
                println!("No change");
                return;
            }
            print!("Save to ${}? [y/N]: ", slot);
            if io::stdout().flush().is_err() {
                println!("(io error)");
                return;
            }
            let mut confirm = String::new();
            if io::stdin().read_line(&mut confirm).is_ok() {
                let reply = confirm.trim().to_lowercase();
                if reply == "y" || reply == "yes" {
                    if let Err(err) = engine.set(&path, new_value) {
                        println!("Save error: {}", err);
                    } else {
                        println!("Saved ${}", slot);
                    }
                } else {
                    println!("Discarded");
                }
            }
        }
        Err(err) => println!("Read error: {}", err),
    }
}

fn mem_delete(engine: &mut meteor::MeteorEngine, raw_name: &str) {
    let Some(slot) = mem_slot_name(raw_name) else {
        println!("Invalid name");
        return;
    };
    let path = mem_path(&slot);
    match engine.delete(&path) {
        Ok(true) => println!("Removed ${}", slot),
        Ok(false) => println!("${} was empty", slot),
        Err(err) => println!("Delete error: {}", err),
    }
}

fn mem_list(engine: &meteor::MeteorEngine) {
    let storage = engine.storage();
    let entries = storage.get_all_keys_in_namespace(MEM_CONTEXT, MEM_NAMESPACE);
    if entries.is_empty() {
        println!("Scratch pad empty");
        return;
    }
    println!("Scratch entries:");
    for (key, value) in entries {
        println!("  ${} = {}", key, value);
    }
}

fn split_name_value(rest: &str) -> Option<(&str, &str)> {
    let trimmed = rest.trim_start();
    if trimmed.is_empty() {
        return None;
    }
    let mut chars = trimmed.char_indices();
    let mut split_at = None;
    while let Some((idx, ch)) = chars.next() {
        if ch.is_whitespace() {
            split_at = Some(idx);
            break;
        }
    }
    let idx = split_at?;
    let name = &trimmed[..idx];
    let value = trimmed[idx..].trim_start();
    if name.is_empty() || value.is_empty() {
        None
    } else {
        Some((name, value))
    }
}
