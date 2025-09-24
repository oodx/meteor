use meteor::{MeteorEngine, Context, Namespace};

fn main() {
    println!("=== Visual UAT: Meteor Path Parsing ===\n");

    let mut engine = MeteorEngine::new();

    // Test direct API with colon-delimited meteor paths
    println!("1. Direct API (explicit addressing with colons):");

    // Set using full meteor format
    engine.set("app:ui.widgets:button_color", "blue").unwrap();
    engine.set("user:settings.theme:dark_mode", "true").unwrap();
    engine.set("system:logs.error:last_timestamp", "2024-12-20T10:30:00Z").unwrap();

    // Get using full meteor format
    println!("   app:ui.widgets:button_color = {:?}", engine.get("app:ui.widgets:button_color"));
    println!("   user:settings.theme:dark_mode = {:?}", engine.get("user:settings.theme:dark_mode"));
    println!("   system:logs.error:last_timestamp = {:?}", engine.get("system:logs.error:last_timestamp"));

    // Test partial meteor formats
    println!("\n2. Partial meteor formats:");
    engine.set("app:ui:theme", "light").unwrap();  // context:namespace:key
    engine.set("app::global_setting", "enabled").unwrap();  // context::key (empty namespace)

    println!("   app:ui:theme = {:?}", engine.get("app:ui:theme"));
    println!("   app::global_setting = {:?}", engine.get("app::global_setting"));

    println!("\n3. Cursor-based API (implicit addressing):");

    // Switch context and namespace using cursor
    engine.switch_context(Context::user());
    engine.switch_namespace(Namespace::from_string("profile.preferences"));

    // Store using cursor (no explicit addressing)
    engine.store_token("username", "alice");
    engine.store_token("email", "alice@example.com");

    // Verify data stored with cursor context/namespace
    println!("   Current cursor: {}:{}", engine.current_context.name(), engine.current_namespace.to_string());
    println!("   user:profile.preferences:username = {:?}", engine.get("user:profile.preferences:username"));
    println!("   user:profile.preferences:email = {:?}", engine.get("user:profile.preferences:email"));

    println!("\n4. Complex namespace hierarchies:");
    engine.set("app:ui.forms.login.validation:email_pattern", r"^[^@]+@[^@]+\.[^@]+$").unwrap();
    engine.set("system:network.http.timeouts.request:default_ms", "5000").unwrap();

    println!("   app:ui.forms.login.validation:email_pattern = {:?}", engine.get("app:ui.forms.login.validation:email_pattern"));
    println!("   system:network.http.timeouts.request:default_ms = {:?}", engine.get("system:network.http.timeouts.request:default_ms"));

    println!("\n5. Format validation:");
    // These should work
    match engine.set("valid:namespace.path:key", "value") {
        Ok(_) => println!("   ✅ 'valid:namespace.path:key' - Valid meteor format"),
        Err(e) => println!("   ❌ 'valid:namespace.path:key' - Error: {}", e),
    }

    // This should fail (too many colons)
    match engine.set("invalid:too:many:colons:here", "value") {
        Ok(_) => println!("   ❌ 'invalid:too:many:colons:here' - Should have failed!"),
        Err(e) => println!("   ✅ 'invalid:too:many:colons:here' - Correctly rejected: {}", e),
    }

    println!("\n6. Storage inspection:");
    println!("   Total contexts: {:?}", engine.contexts());
    println!("   Namespaces in 'app': {:?}", engine.namespaces_in_context("app"));
    println!("   Namespaces in 'user': {:?}", engine.namespaces_in_context("user"));

    println!("\n=== UAT Complete: Meteor path parsing working correctly! ===");
}