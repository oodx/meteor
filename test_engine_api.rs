// Quick test to verify MeteorEngine hybrid storage API
use meteor::{MeteorEngine};

fn main() {
    let mut engine = MeteorEngine::new();

    println!("🧪 Testing MeteorEngine Hybrid Storage API");
    println!("===========================================");

    // Test basic set/get
    println!("\n1. Testing basic set/get operations:");
    match engine.set("app:ui:button", "click") {
        Ok(_) => println!("✅ Set app:ui:button = click"),
        Err(e) => println!("❌ Set failed: {}", e),
    }

    match engine.get("app:ui:button") {
        Some(value) => println!("✅ Get app:ui:button = {}", value),
        None => println!("❌ Get failed: value not found"),
    }

    // Test hierarchical operations
    println!("\n2. Testing hierarchical operations:");
    let _ = engine.set("user:settings:theme", "dark");
    let _ = engine.set("user:settings:lang", "en");

    if engine.is_directory("user:settings") {
        println!("✅ user:settings is recognized as directory");
    } else {
        println!("❌ user:settings should be directory");
    }

    if engine.is_file("user:settings:theme") {
        println!("✅ user:settings:theme is recognized as file");
    } else {
        println!("❌ user:settings:theme should be file");
    }

    // Test default values
    println!("\n3. Testing default value operations:");
    let _ = engine.set("user:index", "default_user_value");

    if engine.has_default("user") {
        println!("✅ user has default value");
        if let Some(default) = engine.get_default("user") {
            println!("✅ user default = {}", default);
        }
    } else {
        println!("❌ user should have default value");
    }

    println!("\n🎉 MeteorEngine API test complete!");
}