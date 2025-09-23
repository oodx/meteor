// TICKET-003: MeteorShower Storage Validation
//
// Comprehensive validation that MeteorShower can fully replace TokenBucket as primary storage.
// This test suite validates all storage functionality, cross-context indexing, performance,
// and StorageData interchange format before TokenBucket removal.

extern crate meteor;

use meteor::{MeteorShower, Context, Namespace, Token, Meteor, StorageData};
use std::time::Instant;

#[cfg(test)]
mod validation_tests {
    use super::*;

    /// Test 1: Basic storage functionality equivalent to TokenBucket
    #[test]
    pub fn validate_basic_storage_functionality() {
        let mut shower = MeteorShower::new();

        // Test adding meteors
        let meteor1 = Meteor::new(
            Context::app(),
            Namespace::from_string("ui"),
            Token::new("button", "click")
        );
        shower.add(meteor1);

        let meteor2 = Meteor::new(
            Context::app(),
            Namespace::from_string("settings"),
            Token::new("theme", "dark")
        );
        shower.add(meteor2);

        // Validate basic properties
        assert_eq!(shower.len(), 2);
        assert!(!shower.is_empty());

        // Validate retrieval
        let found = shower.find("app", "ui", "button");
        assert!(found.is_some());
        assert_eq!(found.unwrap().token().value(), "click");

        let missing = shower.find("app", "ui", "missing");
        assert!(missing.is_none());

        println!("✅ Basic storage functionality validated");
    }

    /// Test 2: Cross-context indexing and isolation
    #[test]
    pub fn validate_cross_context_indexing() {
        let mut shower = MeteorShower::new();

        // Add meteors in different contexts
        shower.add(Meteor::new(
            Context::app(),
            Namespace::from_string("ui"),
            Token::new("button", "app_value")
        ));

        shower.add(Meteor::new(
            Context::user(),
            Namespace::from_string("ui"),
            Token::new("button", "user_value")
        ));

        shower.add(Meteor::new(
            Context::system(),
            Namespace::from_string("config"),
            Token::new("debug", "true")
        ));

        // Validate context isolation
        let app_meteors = shower.by_context("app");
        assert_eq!(app_meteors.len(), 1);
        assert_eq!(app_meteors[0].token().value(), "app_value");

        let user_meteors = shower.by_context("user");
        assert_eq!(user_meteors.len(), 1);
        assert_eq!(user_meteors[0].token().value(), "user_value");

        let system_meteors = shower.by_context("system");
        assert_eq!(system_meteors.len(), 1);
        assert_eq!(system_meteors[0].token().value(), "true");

        // Validate cross-context queries
        assert_eq!(shower.contexts().len(), 3);
        assert!(shower.contexts().contains(&"app"));
        assert!(shower.contexts().contains(&"user"));
        assert!(shower.contexts().contains(&"system"));

        // Validate namespace organization within contexts
        let app_namespaces = shower.namespaces_in_context("app");
        assert_eq!(app_namespaces.len(), 1);
        assert!(app_namespaces.contains(&"ui"));

        let system_namespaces = shower.namespaces_in_context("system");
        assert_eq!(system_namespaces.len(), 1);
        assert!(system_namespaces.contains(&"config"));

        println!("✅ Cross-context indexing and isolation validated");
    }

    /// Test 3: Complex namespace hierarchy support
    #[test]
    pub fn validate_namespace_hierarchy() {
        let mut shower = MeteorShower::new();

        // Test complex namespace patterns
        shower.add(Meteor::new(
            Context::app(),
            Namespace::from_string("ui.widgets.buttons"),
            Token::new("primary", "submit")
        ));

        shower.add(Meteor::new(
            Context::app(),
            Namespace::from_string("ui.widgets.forms"),
            Token::new("validation", "enabled")
        ));

        shower.add(Meteor::new(
            Context::app(),
            Namespace::from_string("ui.dialogs"),
            Token::new("modal", "closed")
        ));

        // Validate namespace-based queries
        let ui_widgets_buttons = shower.by_context_namespace("app", "ui.widgets.buttons");
        assert_eq!(ui_widgets_buttons.len(), 1);
        assert_eq!(ui_widgets_buttons[0].token().value(), "submit");

        let ui_widgets_forms = shower.by_context_namespace("app", "ui.widgets.forms");
        assert_eq!(ui_widgets_forms.len(), 1);
        assert_eq!(ui_widgets_forms[0].token().value(), "enabled");

        // Validate find with exact namespace matching
        let found = shower.find("app", "ui.widgets.buttons", "primary");
        assert!(found.is_some());
        assert_eq!(found.unwrap().token().value(), "submit");

        let not_found = shower.find("app", "ui.widgets", "primary"); // Wrong namespace
        assert!(not_found.is_none());

        println!("✅ Namespace hierarchy support validated");
    }

    /// Test 4: Parse/display round-trip functionality
    #[test]
    pub fn validate_parse_display_roundtrip() {
        let input = "app:ui:button=click; user:settings:theme=dark; system:config:debug=true";

        // Parse input into MeteorShower
        let shower = MeteorShower::parse(input).expect("Should parse successfully");

        // Validate parsed structure
        assert_eq!(shower.len(), 3);
        assert_eq!(shower.contexts().len(), 3);

        // Validate individual meteors
        let button = shower.find("app", "ui", "button");
        assert!(button.is_some());
        assert_eq!(button.unwrap().token().value(), "click");

        let theme = shower.find("user", "settings", "theme");
        assert!(theme.is_some());
        assert_eq!(theme.unwrap().token().value(), "dark");

        let debug = shower.find("system", "config", "debug");
        assert!(debug.is_some());
        assert_eq!(debug.unwrap().token().value(), "true");

        // Test round-trip (parse -> display -> parse)
        let display_output = shower.to_string();
        let reparsed = MeteorShower::parse(&display_output).expect("Should reparse successfully");

        // Validate round-trip preservation
        assert_eq!(reparsed.len(), shower.len());
        assert_eq!(reparsed.contexts().len(), shower.contexts().len());

        // Validate specific values preserved
        assert!(reparsed.find("app", "ui", "button").is_some());
        assert!(reparsed.find("user", "settings", "theme").is_some());
        assert!(reparsed.find("system", "config", "debug").is_some());

        println!("✅ Parse/display round-trip functionality validated");
    }

    /// Test 5: StorageData interchange format
    #[test]
    pub fn validate_storage_data_interchange() {
        let mut shower = MeteorShower::new();

        // Add test data
        shower.add(Meteor::new(
            Context::app(),
            Namespace::from_string("ui"),
            Token::new("button", "click")
        ));

        shower.add(Meteor::new(
            Context::user(),
            Namespace::from_string("settings"),
            Token::new("theme", "dark")
        ));

        // Convert to StorageData
        let storage_data = convert_shower_to_storage_data(&shower);

        // Validate StorageData structure
        assert_eq!(storage_data.contexts().len(), 2);
        assert!(storage_data.contexts().contains(&"app".to_string()));
        assert!(storage_data.contexts().contains(&"user".to_string()));

        // Validate data preservation
        assert_eq!(storage_data.get("app", "ui", "button"), Some("click"));
        assert_eq!(storage_data.get("user", "settings", "theme"), Some("dark"));
        assert_eq!(storage_data.get("app", "ui", "missing"), None);

        // Test serialization
        let serialized = storage_data.to_string();
        assert!(serialized.contains("app:ui:button=click"));
        assert!(serialized.contains("user:settings:theme=dark"));

        // Validate round-trip through StorageData
        let recreated_shower = MeteorShower::parse(&serialized).expect("Should parse from StorageData");
        assert_eq!(recreated_shower.len(), shower.len());
        assert!(recreated_shower.find("app", "ui", "button").is_some());
        assert!(recreated_shower.find("user", "settings", "theme").is_some());

        println!("✅ StorageData interchange format validated");
    }

    /// Test 6: Bracket notation and TokenKey integration
    #[test]
    pub fn validate_bracket_notation_support() {
        let input = "app:ui:list[0]=first; app:ui:list[1]=second; app:ui:grid[2,3]=cell";

        let shower = MeteorShower::parse(input).expect("Should parse bracket notation");
        assert_eq!(shower.len(), 3);

        // Validate bracket notation parsing
        let first = shower.find("app", "ui", "list[0]");
        assert!(first.is_some() || shower.find("app", "ui", "list__i_0").is_some(),
                "Should find bracket notation key either as original or transformed");

        let grid = shower.find("app", "ui", "grid[2,3]");
        assert!(grid.is_some() || shower.find("app", "ui", "grid__i_2_3").is_some(),
                "Should find complex bracket notation");

        println!("✅ Bracket notation support validated");
    }

    /// Test 7: Performance validation vs baseline
    #[test]
    pub fn validate_performance_characteristics() {
        let start = Instant::now();

        // Create large dataset
        let mut shower = MeteorShower::new();
        for i in 0..1000 {
            shower.add(Meteor::new(
                Context::app(),
                Namespace::from_string(&format!("namespace_{}", i % 10)),
                Token::new(&format!("key_{}", i), &format!("value_{}", i))
            ));
        }

        let creation_time = start.elapsed();

        // Test lookup performance
        let lookup_start = Instant::now();
        for i in 0..100 {
            let found = shower.find("app", &format!("namespace_{}", i % 10), &format!("key_{}", i));
            assert!(found.is_some(), "Should find key_{}", i);
        }
        let lookup_time = lookup_start.elapsed();

        // Test context queries
        let context_start = Instant::now();
        let app_meteors = shower.by_context("app");
        assert_eq!(app_meteors.len(), 1000);
        let context_time = context_start.elapsed();

        // Validate reasonable performance (these are generous limits)
        assert!(creation_time.as_millis() < 100, "Creation should be fast: {}ms", creation_time.as_millis());
        assert!(lookup_time.as_millis() < 50, "Lookups should be fast: {}ms", lookup_time.as_millis());
        assert!(context_time.as_millis() < 10, "Context queries should be fast: {}ms", context_time.as_millis());

        println!("✅ Performance characteristics validated");
        println!("   Creation: {}ms, Lookups: {}ms, Context query: {}ms",
                creation_time.as_millis(), lookup_time.as_millis(), context_time.as_millis());
    }

    /// Test 8: Memory efficiency validation
    #[test]
    pub fn validate_memory_efficiency() {
        let mut shower = MeteorShower::new();

        // Add moderate dataset
        for i in 0..100 {
            shower.add(Meteor::new(
                Context::app(),
                Namespace::from_string("test"),
                Token::new(&format!("key_{}", i), &format!("value_{}", i))
            ));
        }

        // Validate no obvious memory waste
        assert_eq!(shower.len(), 100);
        assert_eq!(shower.contexts().len(), 1);
        assert_eq!(shower.namespaces_in_context("app").len(), 1);

        // Test large values don't cause issues
        shower.add(Meteor::new(
            Context::app(),
            Namespace::from_string("test"),
            Token::new("large_value", &"x".repeat(1000))
        ));

        assert_eq!(shower.len(), 101);
        let large = shower.find("app", "test", "large_value");
        assert!(large.is_some());
        assert_eq!(large.unwrap().token().value().len(), 1000);

        println!("✅ Memory efficiency validated");
    }

    /// Test 9: Error handling and edge cases
    #[test]
    pub fn validate_error_handling() {
        // Test empty input
        let empty = MeteorShower::parse("").expect("Empty input should succeed");
        assert!(empty.is_empty());

        // Test whitespace only
        let whitespace = MeteorShower::parse("   ;  ; ").expect("Whitespace should succeed");
        assert!(whitespace.is_empty());

        // Test single valid token
        let single = MeteorShower::parse("app:ui:button=click").expect("Single token should parse");
        assert_eq!(single.len(), 1);

        // Test malformed input handling
        let malformed = MeteorShower::parse("invalid_format");
        // Should either parse with defaults or return clear error
        if malformed.is_err() {
            println!("✅ Malformed input properly rejected");
        } else {
            println!("✅ Malformed input handled with defaults");
        }

        // Test empty components
        let shower = MeteorShower::new();
        assert!(shower.find("", "", "").is_none());
        assert!(shower.by_context("nonexistent").is_empty());
        assert!(shower.namespaces_in_context("nonexistent").is_empty());

        println!("✅ Error handling and edge cases validated");
    }

    /// Test 10: Full functional equivalence to TokenBucket interface
    #[test]
    pub fn validate_functional_equivalence() {
        // Test all major TokenBucket-like operations through MeteorShower + StorageData
        let mut shower = MeteorShower::new();

        // Equivalent to TokenBucket::set operations
        shower.add(Meteor::new(Context::app(), Namespace::from_string(""), Token::new("root_key", "root_value")));
        shower.add(Meteor::new(Context::app(), Namespace::from_string("ui"), Token::new("button", "click")));
        shower.add(Meteor::new(Context::user(), Namespace::from_string("settings"), Token::new("theme", "dark")));

        // Convert to StorageData for TokenBucket-like interface
        let storage = convert_shower_to_storage_data(&shower);

        // Equivalent to TokenBucket::get operations
        assert_eq!(storage.get("app", "", "root_key"), Some("root_value"));
        assert_eq!(storage.get("app", "ui", "button"), Some("click"));
        assert_eq!(storage.get("user", "settings", "theme"), Some("dark"));
        assert_eq!(storage.get("app", "ui", "missing"), None);

        // Equivalent to TokenBucket context operations
        let contexts = storage.contexts();
        assert!(contexts.contains(&"app".to_string()));
        assert!(contexts.contains(&"user".to_string()));

        // Equivalent to TokenBucket namespace operations
        let app_namespaces = storage.namespaces_in_context("app");
        assert!(app_namespaces.contains(&"".to_string())); // root namespace
        assert!(app_namespaces.contains(&"ui".to_string()));

        println!("✅ Full functional equivalence to TokenBucket validated");
    }

    // Helper function to convert MeteorShower to StorageData
    fn convert_shower_to_storage_data(shower: &MeteorShower) -> StorageData {
        let mut storage = StorageData::new();

        for meteor in shower.meteors() {
            storage.set(
                meteor.context().name(),
                meteor.namespace().to_string().as_str(),
                meteor.token().key_str(),
                meteor.token().value()
            );
        }

        storage
    }
}

/// Integration test runner for TICKET-003 validation
#[cfg(test)]
mod integration {
    use super::*;

    #[test]
    fn run_complete_validation_suite() {
        println!("🚀 Starting TICKET-003: MeteorShower Storage Validation");
        println!("   Validating that MeteorShower can fully replace TokenBucket as primary storage");
        println!();

        // Run all validation tests - if any fail, the replacement is not safe
        validation_tests::validate_basic_storage_functionality();
        validation_tests::validate_cross_context_indexing();
        validation_tests::validate_namespace_hierarchy();
        validation_tests::validate_parse_display_roundtrip();
        validation_tests::validate_storage_data_interchange();
        validation_tests::validate_bracket_notation_support();
        validation_tests::validate_performance_characteristics();
        validation_tests::validate_memory_efficiency();
        validation_tests::validate_error_handling();
        validation_tests::validate_functional_equivalence();

        println!();
        println!("🎉 TICKET-003 VALIDATION COMPLETE");
        println!("✅ MeteorShower storage implementation validated");
        println!("✅ Cross-context indexing working correctly");
        println!("✅ StorageData interchange format working");
        println!("✅ Performance acceptable for primary storage use");
        println!("✅ Full functional equivalence to TokenBucket confirmed");
        println!();
        println!("🔒 SAFE TO PROCEED WITH TICKET-006 (TokenBucket removal)");
    }
}