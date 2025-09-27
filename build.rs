//! Build script to read meteor.toml configuration at compile time
//!
//! This script reads the meteor.toml configuration file and sets environment
//! variables that can be used by the config module to set compile-time constants.

use std::env;
use std::fs;
use std::path::Path;

fn main() {
    // Tell cargo to rerun if meteor.toml changes
    println!("cargo:rerun-if-changed=meteor.toml");

    // Declare custom cfg attributes to avoid warnings
    println!("cargo:rustc-check-cfg=cfg(meteor_default)");
    println!("cargo:rustc-check-cfg=cfg(meteor_enterprise)");
    println!("cargo:rustc-check-cfg=cfg(meteor_embedded)");
    println!("cargo:rustc-check-cfg=cfg(meteor_strict)");

    // Read meteor.toml configuration
    let toml_path = Path::new("meteor.toml");
    let config = if toml_path.exists() {
        match fs::read_to_string(toml_path) {
            Ok(content) => content,
            Err(_) => {
                eprintln!("Warning: Could not read meteor.toml, using default configuration");
                return;
            }
        }
    } else {
        eprintln!("Warning: meteor.toml not found, using default configuration");
        return;
    };

    // Parse TOML (simple manual parsing to avoid dependencies)
    let active_profile = extract_active_profile(&config);
    let profile_name = env::var("METEOR_PROFILE").unwrap_or(active_profile);

    println!("cargo:rustc-env=METEOR_ACTIVE_PROFILE={}", profile_name);

    // Set cfg attribute based on active profile
    match profile_name.as_str() {
        "enterprise" => println!("cargo:rustc-cfg=meteor_enterprise"),
        "embedded" => println!("cargo:rustc-cfg=meteor_embedded"),
        "strict" => println!("cargo:rustc-cfg=meteor_strict"),
        "default" => println!("cargo:rustc-cfg=meteor_default"),
        _ => println!("cargo:rustc-cfg=meteor_default"), // Unknown profiles default to default
    }

    // Extract security settings
    let security_settings = extract_security_settings(&config);
    for (key, value) in security_settings {
        let env_var = format!("METEOR_SECURITY_{}", key.to_uppercase());
        println!("cargo:rustc-env={}={}", env_var, value);
    }
}

fn extract_active_profile(config: &str) -> String {
    for line in config.lines() {
        let line = line.trim();
        if line.starts_with("active = ") {
            if let Some(value) = line.split('=').nth(1) {
                return value.trim().trim_matches('"').to_string();
            }
        }
    }
    "default".to_string()
}

fn extract_security_settings(config: &str) -> Vec<(String, String)> {
    let mut in_section = false;
    let mut settings = Vec::new();

    for line in config.lines() {
        let line = line.trim();

        if line == "[security]" {
            in_section = true;
            continue;
        }

        if line.starts_with('[') && line.ends_with(']') && in_section {
            // Entering a new section
            break;
        }

        if in_section && line.contains('=') && !line.starts_with('#') {
            if let Some((key, value)) = line.split_once('=') {
                let key = key.trim().to_string();
                let value = value.trim().to_string();
                settings.push((key, value));
            }
        }
    }

    settings
}
