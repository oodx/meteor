//! Demonstrates how meteor configuration works in a consuming project

use meteor::{config, MeteorEngine};

fn main() {
    println!("=== Meteor Configuration Demo ===");
    println!();

    // These values are COMPILED IN - cannot be changed at runtime
    println!("Build-time Configuration (Fixed):");
    println!("- Profile: {}", config::config_profile());
    println!(
        "- Max meteors per shower: {}",
        config::MAX_METEORS_PER_SHOWER
    );
    println!("- Max command history: {}", config::MAX_COMMAND_HISTORY);
    println!("- Max contexts: {}", config::MAX_CONTEXTS);
    println!("- Max token key length: {}", config::MAX_TOKEN_KEY_LENGTH);
    println!(
        "- Max token value length: {}",
        config::MAX_TOKEN_VALUE_LENGTH
    );
    println!();

    println!("Security Features:");
    println!(
        "- Runtime tampering prevention: {}",
        config::prevent_runtime_tampering()
    );
    println!(
        "- Namespace character validation: {}",
        config::validate_namespace_characters()
    );
    println!(
        "- Reserved word enforcement: {}",
        config::enforce_reserved_words()
    );
    println!(
        "- Command audit trail: {}",
        config::enable_command_audit_trail()
    );
    println!();

    // Create engine with these fixed limits
    let _engine = MeteorEngine::new();
    println!("Created MeteorEngine with compiled configuration limits.");

    // Try to demonstrate the limits are enforced
    println!();
    println!("=== Security Demonstration ===");
    println!("These limits are enforced by the compiled binary:");
    println!(
        "- Cannot exceed {} meteors per shower",
        config::MAX_METEORS_PER_SHOWER
    );
    println!(
        "- Cannot exceed {} command history entries",
        config::MAX_COMMAND_HISTORY
    );
    println!(
        "- Cannot create more than {} contexts",
        config::MAX_CONTEXTS
    );
    println!(
        "- Token keys limited to {} characters",
        config::MAX_TOKEN_KEY_LENGTH
    );
    println!(
        "- Token values limited to {} characters",
        config::MAX_TOKEN_VALUE_LENGTH
    );
    println!();
    println!("⚠️  IMPORTANT: These limits cannot be changed at runtime!");
    println!("   They are baked into the compiled binary for security.");
}
