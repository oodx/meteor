use meteor::{Namespace, TokenKey, Meteor, parse_shower};

fn main() {
    println!("=== Testing Namespace Flexibility ===");

    // Test different namespace scenarios
    println!("1. Root namespace (zero parts):");
    match Namespace::try_from_string("") {
        Ok(ns) => println!("   ✅ '{}' -> depth: {}", ns.to_string(), ns.depth()),
        Err(e) => println!("   ❌ Error: {}", e),
    }

    println!("2. Single part namespace:");
    match Namespace::try_from_string("ui") {
        Ok(ns) => println!("   ✅ '{}' -> depth: {}", ns.to_string(), ns.depth()),
        Err(e) => println!("   ❌ Error: {}", e),
    }

    println!("3. Multi-part namespace:");
    match Namespace::try_from_string("ui.widgets") {
        Ok(ns) => println!("   ✅ '{}' -> depth: {}", ns.to_string(), ns.depth()),
        Err(e) => println!("   ❌ Error: {}", e),
    }

    println!("4. Deep namespace (4 parts - should be clear):");
    match Namespace::try_from_string("ui.widgets.forms.inputs") {
        Ok(ns) => println!("   ✅ '{}' -> depth: {}, warn: {}", ns.to_string(), ns.depth(), ns.should_warn()),
        Err(e) => println!("   ❌ Error: {}", e),
    }

    println!("5. Warning depth (5 parts - should warn):");
    match Namespace::try_from_string("ui.widgets.forms.inputs.text") {
        Ok(ns) => println!("   ✅ '{}' -> depth: {}, warn: {}", ns.to_string(), ns.depth(), ns.should_warn()),
        Err(e) => println!("   ❌ Error: {}", e),
    }

    println!("6. Error depth (6 parts - should error):");
    match Namespace::try_from_string("ui.widgets.forms.inputs.text.field") {
        Ok(ns) => println!("   ✅ '{}' -> depth: {}, warn: {}", ns.to_string(), ns.depth(), ns.should_warn()),
        Err(e) => println!("   ❌ Error: {}", e),
    }

    // Test in meteor parsing context
    println!("\n=== Testing in Meteor Context ===");

    // Root namespace meteor
    match parse_shower("key=value") {
        Ok(shower) => println!("✅ 'key=value' parsed successfully"),
        Err(e) => println!("❌ 'key=value' failed: {}", e),
    }

    // Single namespace meteor
    match parse_shower("app:ui:key=value") {
        Ok(shower) => println!("✅ 'app:ui:key=value' parsed successfully"),
        Err(e) => println!("❌ 'app:ui:key=value' failed: {}", e),
    }

    // Multi namespace meteor
    match parse_shower("app:ui.widgets:key=value") {
        Ok(shower) => println!("✅ 'app:ui.widgets:key=value' parsed successfully"),
        Err(e) => println!("❌ 'app:ui.widgets:key=value' failed: {}", e),
    }
}