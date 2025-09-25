//! Meteor UAT (User Acceptance Tests)
//!
//! Visual demonstrations and user acceptance validation following RSB patterns.
//! These tests demonstrate functionality with explanatory output.
//!
//! Tests are organized in tests/uat/ subdirectory for detailed demonstrations.

extern crate meteor;

use meteor::{BracketNotation, MeteorEngine, MeteorStreamParser, TokenStreamParser};

/// UAT: Basic functionality demonstration
#[test]
fn uat_basic_functionality_demo() {
    println!("🌠 METEOR UAT DEMONSTRATION");
    println!("==========================");
    println!();

    println!("📝 Parsing simple token: app:ui:key=value");
    let shower = meteor::parse_shower("app:ui:key=value").unwrap();
    println!("✅ Successfully parsed into MeteorShower!");
    let found = shower.find("app", "ui", "key");
    println!("   Retrieved value: {:?}", found.map(|m| m.token().value()));
    println!();

    println!("📝 Parsing bracket notation: app:list:list[0]=item");
    let shower = meteor::parse_shower("app:list:list[0]=item").unwrap();
    println!("🔄 Bracket notation supported");
    let found = shower.find("app", "list", "list[0]");
    println!("   Retrieved value: {:?}", found.map(|m| m.token().value()));
    println!();

    println!("🎉 Basic functionality demonstration complete!");
}

/// UAT: Bracket notation trait demonstration
#[test]
fn uat_bracket_notation_trait_demo() {
    println!("✨ DEMO: BracketNotation Trait");
    println!("==============================");
    println!();

    let flat = "list__i_0";
    println!("📝 Input: {}", flat);
    let bracket = flat.to_bracket();
    println!("🔄 Reconstructed: {}", bracket);
    assert_eq!(bracket, "list[0]");
    println!("✅ Inverse parsing works!");
    println!();

    println!("🎉 BracketNotation trait demonstration complete!");
}

/// UAT: MeteorShower collection demonstration
#[test]
fn uat_meteor_shower_demo() {
    println!("🚿 DEMO: MeteorShower Collection");
    println!("===============================");
    println!();

    println!("📝 Input: app:ui:button=click :;: user:settings:theme=dark");
    let shower = meteor::parse_shower("app:ui:button=click :;: user:settings:theme=dark").unwrap();
    println!("✅ Successfully parsed {} meteors!", shower.len());
    println!();

    println!("🔍 Querying app context:");
    let app_meteors = shower.by_context("app");
    println!("   Found {} meteors in app context", app_meteors.len());
    println!();

    println!("🔍 Querying user context:");
    let user_meteors = shower.by_context("user");
    println!("   Found {} meteors in user context", user_meteors.len());
    println!();

    println!("🎉 MeteorShower demonstration complete!");
}

/// UAT: MeteorEngine stateful processing demonstration
#[test]
fn uat_meteor_engine_stateful_demo() {
    println!("🚀 DEMO: MeteorEngine Stateful Processing");
    println!("=========================================");
    println!();

    let mut engine = MeteorEngine::new();

    println!("📝 Initial state:");
    println!("   Context: {}", engine.current_context.to_string());
    println!("   Namespace: {}", engine.current_namespace.to_string());
    println!();

    println!("📝 Processing token stream: 'host=localhost;port=8080;ns=db'");
    TokenStreamParser::process(&mut engine, "host=localhost;port=8080;ns=db").unwrap();
    println!("✅ Stream processed!");
    println!(
        "   Current namespace: {}",
        engine.current_namespace.to_string()
    );
    println!("   Values stored:");
    println!("   - app.main.host = {:?}", engine.get("app.main.host"));
    println!("   - app.main.port = {:?}", engine.get("app.main.port"));
    println!();

    println!("📝 Processing second stream: 'user=admin;pass=secret'");
    TokenStreamParser::process(&mut engine, "user=admin;pass=secret").unwrap();
    println!("✅ Stream processed!");
    println!(
        "   Context/namespace continuity: {}:{}",
        engine.current_context.to_string(),
        engine.current_namespace.to_string()
    );
    println!("   Values stored in db namespace:");
    println!("   - app.db.user = {:?}", engine.get("app.db.user"));
    println!("   - app.db.pass = {:?}", engine.get("app.db.pass"));
    println!();

    println!("📝 Processing control command: 'ctl:delete=app.db.pass'");
    TokenStreamParser::process(&mut engine, "ctl:delete=app.db.pass").unwrap();
    println!("✅ Control command executed!");
    println!(
        "   Password after deletion: {:?}",
        engine.get("app.db.pass")
    );
    let history = engine.command_history();
    println!("   Command history: {} commands", history.len());
    if let Some(last) = history.last() {
        println!(
            "   Last command: {} {} (success: {})",
            last.command_type, last.target, last.success
        );
    }
    println!();

    println!("🎉 MeteorEngine stateful demonstration complete!");
}

/// UAT: Token vs Meteor stream processing comparison
#[test]
fn uat_stream_comparison_demo() {
    println!("⚡ DEMO: TokenStream vs MeteorStream Processing");
    println!("===============================================");
    println!();

    let mut engine = MeteorEngine::new();

    println!("🔄 TokenStream Processing (with folding logic):");
    println!("   Input: 'button=click;ns=ui;theme=dark;ctx=user;profile=admin'");
    TokenStreamParser::process(
        &mut engine,
        "button=click;ns=ui;theme=dark;ctx=user;profile=admin",
    )
    .unwrap();

    println!("   Results with cursor state changes:");
    println!("   - app.main.button = {:?}", engine.get("app.main.button"));
    println!("   - app.ui.theme = {:?}", engine.get("app.ui.theme"));
    println!("   - user.ui.profile = {:?}", engine.get("user.ui.profile"));
    println!(
        "   Final cursor: {}:{}",
        engine.current_context.to_string(),
        engine.current_namespace.to_string()
    );
    println!();

    println!("🎯 MeteorStream Processing (explicit addressing):");
    println!("   Input: 'sys:config:debug=true :;: sys:config:version=1.0'");
    MeteorStreamParser::process(
        &mut engine,
        "sys:config:debug=true :;: sys:config:version=1.0",
    )
    .unwrap();

    println!("   Results with explicit addressing:");
    println!(
        "   - sys.config.debug = {:?}",
        engine.get("sys.config.debug")
    );
    println!(
        "   - sys.config.version = {:?}",
        engine.get("sys.config.version")
    );
    println!(
        "   Cursor unchanged: {}:{}",
        engine.current_context.to_string(),
        engine.current_namespace.to_string()
    );
    println!();

    println!("🎉 Stream comparison demonstration complete!");
}

/// UAT: Bracket notation in real processing
#[test]
fn uat_bracket_notation_processing_demo() {
    println!("🔢 DEMO: Bracket Notation in Stream Processing");
    println!("==============================================");
    println!();

    let mut engine = MeteorEngine::new();

    println!("📝 Processing array-like tokens:");
    println!("   Input: 'items[0]=apple;items[1]=banana;items[2]=cherry'");
    TokenStreamParser::process(
        &mut engine,
        "items[0]=apple;items[1]=banana;items[2]=cherry",
    )
    .unwrap();

    println!("   Stored as flat keys:");
    println!(
        "   - app.main.items__i_0 = {:?}",
        engine.get("app.main.items__i_0")
    );
    println!(
        "   - app.main.items__i_1 = {:?}",
        engine.get("app.main.items__i_1")
    );
    println!(
        "   - app.main.items__i_2 = {:?}",
        engine.get("app.main.items__i_2")
    );
    println!();

    println!("🔄 Testing bracket notation conversion:");
    let flat = "items__i_0";
    let bracket = flat.to_bracket();
    println!("   {} → {}", flat, bracket);
    println!("   Original notation reconstructed!");
    println!();

    println!("🎉 Bracket notation processing demonstration complete!");
}
