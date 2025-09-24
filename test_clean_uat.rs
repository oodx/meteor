use meteor::{MeteorEngine, Context, Namespace};

fn main() {
    println!("=== Clean Visual UAT: Meteor Path Parsing ===\n");

    let mut engine = MeteorEngine::new();

    // Test direct API with colon-delimited meteor paths
    println!("1. Direct API (explicit addressing with colons):");

    // Set using full meteor format
    engine.set("app:ui.widgets:button_color", "blue").unwrap();
    engine.set("user:settings.theme:dark_mode", "true").unwrap();
    engine.set("system:logs.error:last_timestamp", "2024-12-20T10:30:00Z").unwrap();

    // Get using full meteor format (clean output)
    println!("   app:ui.widgets:button_color = {}", engine.get("app:ui.widgets:button_color").unwrap_or("NOT FOUND"));
    println!("   user:settings.theme:dark_mode = {}", engine.get("user:settings.theme:dark_mode").unwrap_or("NOT FOUND"));
    println!("   system:logs.error:last_timestamp = {}", engine.get("system:logs.error:last_timestamp").unwrap_or("NOT FOUND"));

    // Test partial meteor formats
    println!("\n2. Partial meteor formats:");
    engine.set("app:ui:theme", "light").unwrap();
    engine.set("app::global_setting", "enabled").unwrap();

    println!("   app:ui:theme = {}", engine.get("app:ui:theme").unwrap_or("NOT FOUND"));
    println!("   app::global_setting = {}", engine.get("app::global_setting").unwrap_or("NOT FOUND"));

    println!("\n3. Cursor-based API (implicit addressing):");

    // Switch context and namespace using cursor
    engine.switch_context(Context::user());
    engine.switch_namespace(Namespace::from_string("profile.preferences"));

    // Store using cursor (no explicit addressing)
    engine.store_token("username", "alice");
    engine.store_token("email", "alice@example.com");

    // Verify data stored with cursor context/namespace
    println!("   Current cursor: {}:{}", engine.current_context.name(), engine.current_namespace.to_string());
    println!("   user:profile.preferences:username = {}", engine.get("user:profile.preferences:username").unwrap_or("NOT FOUND"));
    println!("   user:profile.preferences:email = {}", engine.get("user:profile.preferences:email").unwrap_or("NOT FOUND"));

    println!("\n4. Complex namespace hierarchies:");
    engine.set("app:ui.forms.login.validation:email_pattern", r"^[^@]+@[^@]+\.[^@]+$").unwrap();
    engine.set("system:network.http.timeouts.request:default_ms", "5000").unwrap();

    println!("   app:ui.forms.login.validation:email_pattern = {}",
        engine.get("app:ui.forms.login.validation:email_pattern").unwrap_or("NOT FOUND"));
    println!("   system:network.http.timeouts.request:default_ms = {}",
        engine.get("system:network.http.timeouts.request:default_ms").unwrap_or("NOT FOUND"));

    println!("\n5. Testing missing values:");
    println!("   nonexistent:path:key = {}", engine.get("nonexistent:path:key").unwrap_or("NOT FOUND"));

    println!("\n6. Format validation:");
    match engine.set("invalid:too:many:colons:here", "value") {
        Ok(_) => println!("   ❌ 'invalid:too:many:colons:here' - Should have failed!"),
        Err(e) => println!("   ✅ 'invalid:too:many:colons:here' - Correctly rejected: {}", e),
    }

    println!("\n=== Clean UAT Complete: Values displayed without Option wrapper! ===");
}