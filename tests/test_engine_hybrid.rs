//! Test MeteorEngine hybrid storage integration

use meteor::MeteorEngine;

#[test]
fn test_meteor_engine_hybrid_storage_methods() {
    let mut engine = MeteorEngine::new();

    // Test basic set/get
    engine.set("app:ui:button", "click").unwrap();
    assert_eq!(engine.get("app:ui:button"), Some("click"));

    // Test hierarchical structure
    engine.set("user:settings:theme", "dark").unwrap();
    engine.set("user:settings:lang", "en").unwrap();

    // These methods should exist on MeteorEngine
    assert!(engine.is_directory("user:settings"));
    assert!(engine.is_file("user:settings:theme"));
    assert!(!engine.is_file("user:settings")); // directory, not file
    assert!(!engine.is_directory("user:settings:theme")); // file, not directory

    // Test default values
    engine.set("user:index", "default_user").unwrap();
    assert!(engine.has_default("user"));
    assert_eq!(engine.get_default("user"), Some("default_user"));

    // Test that non-existent defaults return None
    assert!(!engine.has_default("nonexistent"));
    assert_eq!(engine.get_default("nonexistent"), None);
}

#[test]
fn test_meteor_engine_context_operations() {
    let mut engine = MeteorEngine::new();

    // Add data in different contexts
    engine.set("app:main:key1", "value1").unwrap();
    engine.set("user:main:key2", "value2").unwrap();
    engine.set("system:main:key3", "value3").unwrap();

    // Test context listing
    let contexts = engine.contexts();
    assert!(contexts.contains(&"app".to_string()));
    assert!(contexts.contains(&"user".to_string()));
    assert!(contexts.contains(&"system".to_string()));

    // Test namespace listing within context
    engine.set("app:ui:button", "click").unwrap();
    engine.set("app:db:connection", "postgres").unwrap();

    let namespaces = engine.namespaces_in_context("app");
    assert!(namespaces.contains(&"main".to_string()));
    assert!(namespaces.contains(&"ui".to_string()));
    assert!(namespaces.contains(&"db".to_string()));
}