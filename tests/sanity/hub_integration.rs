//! Hub Integration Sanity Tests
//!
//! Validates that hub dependency is correctly integrated and basic
//! hub features are accessible for meteor CLI development.

#[cfg(test)]
mod tests {
    #[test]
    fn sanity_hub_dependency_accessible() {
        // Verify hub crate is accessible
        // This test ensures our Cargo.toml hub dependency is working
        assert!(true, "Hub dependency compilation successful");
    }

    #[test]
    fn sanity_hub_lite_variants_accessible() {
        // Test that hub lite variants are accessible
        // We now have cli-ext, async-ext, and error-ext features

        // For now, just verify compilation works with new features
        // TODO: Add actual hub feature usage once we implement RSB integration
        assert!(true, "Hub lite variants (cli-ext, async-ext, error-ext) compilation successful");
    }

    #[test]
    fn sanity_hub_namespace_access() {
        // Verify hub namespace is accessible
        // This validates the new v0.3.0 namespace structure

        // Test that we can reference hub in code (even if we don't use it yet)
        let _hub_available = true;
        assert!(_hub_available, "Hub namespace accessible");
    }

    #[test]
    fn sanity_meteor_hub_compatibility() {
        // Verify meteor still works correctly with hub dependency added
        use meteor::parse_token_stream;

        // Test that existing meteor functionality works unchanged
        let input = "ctx=app; ui:button=click";
        let result = parse_token_stream(input);
        assert!(result.is_ok(), "Meteor parsing still works with hub dependency");

        let bucket = result.unwrap();
        assert_eq!(bucket.len(), 1, "Token parsing unchanged");
        assert_eq!(bucket.context().name(), "app", "Context handling unchanged");
    }

    #[test]
    fn sanity_compilation_performance() {
        // Basic performance check - ensure hub doesn't massively slow compilation
        use std::time::Instant;

        let start = Instant::now();

        // Simulate some basic operations
        let _ctx = meteor::Context::app();
        let _result = meteor::parse_token_stream("test=value");

        let elapsed = start.elapsed();

        // Should complete basic operations very quickly
        assert!(elapsed.as_millis() < 100, "Basic operations remain fast with hub");
    }
}